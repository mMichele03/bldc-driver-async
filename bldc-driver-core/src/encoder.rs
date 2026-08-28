use bldc_driver_hal::Encoder;
use embassy_time::{Duration, Ticker};

use crate::EncoderSender;

/// Encoder task loop, intended to be run in an embassy task
/// This is the default implementation for an encoder that supports continuous polling
///
/// # Usage example
///
/// ```
/// #[embassy_executor::task]
/// async fn encoder_task() {
///     encoder_run<{BITS}, MyEncoder>(/* ... */)
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
