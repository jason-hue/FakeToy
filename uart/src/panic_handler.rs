#[cfg(feature = "panic_handler")]
use core::panic::PanicInfo;

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}