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
 |    / \   |  finance  |    / \      |
 |         / \         / \           |
  \       /   \       /   \         /
   \     /     \     /     \       /
    \___/       \___/       \_____/
        |                  |
        |                  |
      __/ \__          ___/ \___
     (       )        (        )
"#;
pub const UART16550_CLOCK: usize = 1843200;
pub const UART_DEFAULT_BAUD: usize = 115200;
pub const VIRT_CLINT_ADDR: usize = 0x2000000;
pub const VIRT_CLINT_TIMER_CMP: usize = VIRT_CLINT_ADDR + 0x4000;
pub const VIRT_CLINT_TIMER_VAL: usize = VIRT_CLINT_ADDR + 0xbff8;