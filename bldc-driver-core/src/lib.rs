#![no_std]

use bldc_driver_hal::EncoderData;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};

use crate::pll::KinematicEst;

pub mod encoder;
pub mod pll;
pub mod telemetry;

const REC_N: usize = 10;
pub type SimpleWatchReceiver<T> = Receiver<'static, CriticalSectionRawMutex, T, REC_N>;
pub type SimpleWatchSender<T> = Sender<'static, CriticalSectionRawMutex, T, REC_N>;
pub type SimpleWatch<T> = Watch<CriticalSectionRawMutex, T, REC_N>;

pub type EncoderReceiver<const BITS: usize> = SimpleWatchReceiver<EncoderData<BITS>>;
pub type EncoderSender<const BITS: usize> = SimpleWatchSender<EncoderData<BITS>>;
pub type EncoderWatch<const BITS: usize> = SimpleWatch<EncoderData<BITS>>;

pub type KinematicEstReceiver<const BITS: usize> = SimpleWatchReceiver<KinematicEst<BITS>>;
pub type KinematicEstSender<const BITS: usize> = SimpleWatchSender<KinematicEst<BITS>>;
pub type KinematicEstWatch<const BITS: usize> = SimpleWatch<KinematicEst<BITS>>;

#[macro_export]
macro_rules! generate_bldc_driver_tasks {
    ( $encoder:ty, $motor:ty, $flash:ty, $bits:expr, $buffer_len:expr, $max_speed_rpm:expr $(,)? ) => {
        pub static ENCODER_WATCH: $crate::EncoderWatch<{ $bits }> =
            embassy_sync::watch::Watch::new();

        pub static KINEMATIC_EST_WATCH: $crate::KinematicEstWatch<{ $bits }> =
            embassy_sync::watch::Watch::new();

        pub static TELEMETRY_SIGNAL: embassy_sync::signal::Signal<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            (),
        > = embassy_sync::signal::Signal::new();

        pub struct TelemetryEndSignal(
            &'static embassy_sync::signal::Signal<
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                (),
            >,
        );

        impl TelemetryEndSignal {
            pub async fn wait(&self) {
                self.0.wait().await
            }
        }

        #[embassy_executor::task]
        async fn encoder_task(encoder: $encoder, sender: $crate::EncoderSender<{ $bits }>) {
            $crate::encoder::encoder_run::<{ $bits }, $encoder>(encoder, sender).await;
        }

        #[embassy_executor::task]
        async fn pll_observer_task(
            mut receiver: $crate::EncoderReceiver<{ $bits }>,
            mut sender: $crate::KinematicEstSender<{ $bits }>,
            bandwidth_hz: i32,
        ) {
            $crate::pll::pll_observer_run::<{ $bits }, { $max_speed_rpm }>(
                receiver,
                sender,
                <$encoder as bldc_driver_hal::Encoder<{ $bits }>>::ENCODER_PERIOD_US as i32,
                bandwidth_hz,
            )
            .await;
        }

        #[embassy_executor::task]
        pub async fn telemetry_task(flash: $flash, frequency: u32, duration_us: u64) {
            $crate::telemetry::telemetry_run::<{ $bits }, { $buffer_len }>(
                frequency,
                duration_us,
                ENCODER_WATCH
                    .receiver()
                    .expect("Encoder watch run out of receivers"),
                KINEMATIC_EST_WATCH
                    .receiver()
                    .expect("Kinematic estimation watch run out of receivers"),
                flash,
            )
            .await;

            TELEMETRY_SIGNAL.signal(());
        }

        pub fn run_telemetry(
            spawner: embassy_executor::Spawner,
            flash: $flash,
            frequency: u32,
            duration_us: u64,
        ) -> TelemetryEndSignal {
            spawner.spawn(
                telemetry_task(flash, frequency, duration_us)
                    .expect("Failed to create telemetry task"),
            );

            TelemetryEndSignal(&TELEMETRY_SIGNAL)
        }

        pub fn run_bldc_driver_loop(
            spawner: embassy_executor::Spawner,
            _motor: $motor,
            encoder: $encoder,
            pll_bandwidth_hz: i32,
        ) {
            spawner.spawn(
                encoder_task(encoder, ENCODER_WATCH.sender())
                    .expect("Failed to allocate encoder task"),
            );

            spawner.spawn(
                pll_observer_task(
                    ENCODER_WATCH
                        .receiver()
                        .expect("Encoder watch run out of receivers"),
                    KINEMATIC_EST_WATCH.sender(),
                    pll_bandwidth_hz,
                )
                .expect("Failed to allocate pll observer task"),
            );
        }
    };
}
