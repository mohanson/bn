set -ex

cargo build --target riscv64imac-unknown-none-elf
cargo build --target riscv64imac-unknown-none-elf --release
cargo build --target riscv64imac-unknown-none-elf --example mytest_riscv
cargo build --target riscv64imac-unknown-none-elf --example mytest_riscv --release

/src/ckb-vm-run/target/release/asm /mnt/sata/src/bn/target/riscv64imac-unknown-none-elf/release/examples/mytest_riscv
