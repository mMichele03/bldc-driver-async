use bldc_driver_hal::{Encoder, EncoderSender};
use embassy_time::{Duration, Ticker};

/// Encoder task loop, intended to be run in an embassy task
/// This is the default implementation for an encoder that supports continuous polling
/// If this is not your use case simply implement the task differently and ignore this function
///
/// # Usage example
///
/// ```
/// #[embassy_executor::task]
/// async fn encoder_task() {
///     encoder_run<{BITS}, MyEncoder>(/* ... */)
/// }
///
///
/// impl Encoder<ENCODER_BITS> for MyEncoder {
///
///     fn read_stream(self, spawner: Spawner) -> EncoderReceiver<ENCODER_BITS> {
///         spawner.spawn(encoder_task(self, WATCH.sender()).expect("Failed to allocate encoder task"));
///
///         WATCH
///             .receiver()
///             .expect("Encoder watch run out of receivers")
///     }
///
///     /* full impl... */
/// }
/// ```
pub async fn encoder_run<const BITS: usize, E: Encoder<BITS>>(
    mut encoder: E,
    sender: EncoderSender<BITS>,
) -> ! {
    let mut ticker = Ticker::every(Duration::from_micros(E::ENCODER_PERIOD_US));

    loop {
        let data = encoder.read_value_blocking();
        sender.send(data);

        ticker.next().await;
    }
}
