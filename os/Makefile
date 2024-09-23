clean:
	cargo clean
	cd ../sbi-qemu/ && cargo clean
	cd ../uart/ && cargo clean

build:
	cargo build --release
	cd ../sbi-qemu/ && cargo build --release

dump:build
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin
	cd ../sbi-qemu/ && rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/sbi-qemu -O binary target/riscv64gc-unknown-none-elf/release/sbi-qemu.bin

run:dump
	qemu-system-riscv64 \
		-machine virt \
        -nographic \
        -bios ../sbi-qemu/target/riscv64gc-unknown-none-elf/release/sbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
debug:
	cargo build
	cd ../sbi-qemu/ && cargo build
	rust-objcopy --strip-debug target/riscv64gc-unknown-none-elf/debug/os -O binary target/riscv64gc-unknown-none-elf/debug/os.bin
	cd ../sbi-qemu/ && rust-objcopy --strip-debug target/riscv64gc-unknown-none-elf/debug/sbi-qemu -O binary target/riscv64gc-unknown-none-elf/debug/sbi-qemu.bin

gdbserver:debug
	qemu-system-riscv64 \
    		-machine virt \
            -nographic \
            -bios ../sbi-qemu/target/riscv64gc-unknown-none-elf/debug/sbi-qemu.bin \
            -device loader,file=target/riscv64gc-unknown-none-elf/debug/os.bin,addr=0x80200000 \
            -s -S
gdbclient:
	riscv64-unknown-elf-gdb \
		-ex 'set confirm off' \
		-ex 'file ../sbi-qemu/target/riscv64gc-unknown-none-elf/debug/sbi-qemu' \
        -ex 'add-symbol-file target/riscv64gc-unknown-none-elf/debug/os 0x80200000' \
        -ex 'set arch riscv:rv64' \
        -ex 'target remote localhost:1234' \
        -ex 'directory ../sbi-qemu/src' \
        -ex 'directory $(PWD)/src'