use core::arch::asm;
use crate::{println, write_csr};
use crate::csr::read_stvec;

mod trap_context;
mod trap_handler;
extern "C"{
    fn do_exception_vectors();
}
pub fn trap_init(){
    write_csr!("stvec",do_exception_vectors);
    write_csr!("sie",-1);
}