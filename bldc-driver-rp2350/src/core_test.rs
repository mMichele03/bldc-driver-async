use crate::{
    encoder::{ENCODER_BITS, SpiEncoder},
    flash::RpFlash,
};
use bldc_driver_core::telemetry::telemetry_run;
use bldc_driver_hal::{BldcMotor, Encoder, EncoderReceiver, EncoderSender, EncoderWatch};
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::{Duration, Ticker};

const BITS: usize = ENCODER_BITS;
type EncoderImpl = SpiEncoder;
type FlashImpl = RpFlash;

pub static WATCH: EncoderWatch<BITS> = Watch::new();
pub static TELEMETRY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub struct TelemetryEndSignal(&'static Signal<CriticalSectionRawMutex, ()>);

impl TelemetryEndSignal {
    pub async fn wait(&self) {
        self.0.wait().await
    }
}

#[embassy_executor::task]
async fn encoder_task(mut encoder: EncoderImpl, sender: EncoderSender<BITS>) {
    let mut ticker = Ticker::every(Duration::from_micros(EncoderImpl::ENCODER_PERIOD_US));

    loop {
        let data = encoder.read_value_blocking();
        sender.send(data);

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn pll_observer_task(mut rec: EncoderReceiver<BITS>) {
    loop {
        let _val = rec.changed().await;
    }
}

#[embassy_executor::task]
pub async fn telemetry_task(flash: FlashImpl, frequency: u32, duration_us: u64) {
    telemetry_run::<{ ENCODER_BITS }, { RpFlash::BUFFER_LEN }>(
        frequency,
        duration_us,
        WATCH.receiver().unwrap(),
        flash,
    )
    .await;

    TELEMETRY_SIGNAL.signal(());
}

pub fn run_telemetry(
    spawner: Spawner,
    flash: FlashImpl,
    frequency: u32,
    duration_us: u64,
) -> TelemetryEndSignal {
    spawner.spawn(
        telemetry_task(flash, frequency, duration_us).expect("Failed to create telemetry task"),
    );

    TelemetryEndSignal(&TELEMETRY_SIGNAL)
}

pub fn run_bldc_driver_loop(spawner: Spawner, _motor: impl BldcMotor<BITS>, encoder: EncoderImpl) {
    spawner.spawn(encoder_task(encoder, WATCH.sender()).expect("Failed to allocate encoder task"));

    let rec = WATCH
        .receiver()
        .expect("Encoder watch run out of receivers");

    spawner.spawn(pll_observer_task(rec).expect("Failed to allocate pll observer task"));
}
