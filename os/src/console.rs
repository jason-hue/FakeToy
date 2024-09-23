use core::fmt;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};
use uart::{uart_init, uart_send};

static UART_INITIALIZED: AtomicBool = AtomicBool::new(false);
struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !UART_INITIALIZED.load(Ordering::Acquire) {
            unsafe { uart_init(); }
            UART_INITIALIZED.store(true, Ordering::Release);
        }
        uart_send(s);
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}
