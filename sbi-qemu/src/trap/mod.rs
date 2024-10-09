use crate::write_csr;
use crate::asm;
mod trap_context;
mod trap_handler;
extern "C" {
    fn _sbi_exception_vector();
}
pub fn sbi_trap_init(){
    write_csr!("mtvec",_sbi_exception_vector);
    write_csr!("mie",0);
}