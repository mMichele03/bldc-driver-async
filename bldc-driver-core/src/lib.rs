#![no_std]

pub mod encoder;
pub mod pll;
pub mod telemetry;

#[macro_export]
macro_rules! generate_bldc_driver_tasks {
    ( $encoder:ty, $motor:ty, $flash:ty, $bits:expr, $buffer_len:expr $(,)? ) => {
        pub static WATCH: bldc_driver_hal::EncoderWatch<{ $bits }> =
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
        async fn encoder_task(
            encoder: $encoder,
            sender: bldc_driver_hal::EncoderSender<{ $bits }>,
        ) {
            $crate::encoder::encoder_run::<{ $bits }, $encoder>(encoder, sender).await;
        }

        #[embassy_executor::task]
        async fn pll_observer_task(mut rec: bldc_driver_hal::EncoderReceiver<{ $bits }>) {
            loop {
                let _val = rec.changed().await;
            }
        }

        #[embassy_executor::task]
        pub async fn telemetry_task(flash: $flash, frequency: u32, duration_us: u64) {
            $crate::telemetry::telemetry_run::<{ $bits }, { $buffer_len }>(
                frequency,
                duration_us,
                WATCH.receiver().unwrap(),
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
        ) {
            spawner.spawn(
                encoder_task(encoder, WATCH.sender()).expect("Failed to allocate encoder task"),
            );

            let rec = WATCH
                .receiver()
                .expect("Encoder watch run out of receivers");

            spawner.spawn(pll_observer_task(rec).expect("Failed to allocate pll observer task"));
        }
    };
}
