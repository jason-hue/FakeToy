use core::panic::PanicInfo;

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>)->!{
    loop {

    }
}