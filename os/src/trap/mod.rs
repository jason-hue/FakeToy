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
    extern "C"{
        fn stacks_end();
    }
    let kernel_stack_top = unsafe { stacks_end as usize };
    write_csr!("sscratch",kernel_stack_top);
    write_csr!("sie",-1);
}