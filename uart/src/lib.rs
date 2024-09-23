#![no_std]

pub use crate::uart::uart_init;
use crate::uart::uart_send_string;

mod uart;
mod io;
mod config;
mod panic_handler;
pub fn uart_send(s: &str){
    uart_send_string(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
