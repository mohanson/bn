RUNNER=/src/ckb-vm-run/target/release/int64

test:
	cargo run --example ut

risc:
	cargo build --target riscv64imac-unknown-none-elf --release --example ut_riscv
	$(RUNNER) target/riscv64imac-unknown-none-elf/release/examples/ut_riscv
