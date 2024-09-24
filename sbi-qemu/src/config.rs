pub const FW_JUMP_ADDR: usize = 0x80200000;
pub const logo: &str = r#"
          .-------------------.
         /   _____ ____ ___    \         RenSBI
        /   / ___// __ )/ /     \
       /   (__  )/ __  / /       \
      /   ____// /_/ / /___    O  \
     /   /_____\____/_____/  _/|\  \
    /       |  Welcome  |     / \   \
   /        |    to     |            \
  /    O    | the future|     O       \
 |    /|\   |    of     |    /|\      |
 |    / \   |  RISC-V  |    / \      |
 |         / \         / \           |
  \       /   \       /   \         /
   \     /     \     /     \       /
    \___/       \___/       \_____/
        |                  |
        |                  |
      __/ \__          ___/ \___
     (       )        (        )
"#;
// QEMU-specific MMIO address for system poweroff
pub const QEMU_VIRT_poweroff: *mut u32 = 0x10000_0000 as *mut u32;