RUNNER=/src/ckb-vm-run/target/release/asm
RISCV=/root/app/riscv

test:
	cd alt_bn128 && \
	cargo run --example ut

rv:
	cd alt_bn128_rv && \
	cargo build --release --target riscv64imac-unknown-none-elf --example ut && \
	$(RUNNER) target/riscv64imac-unknown-none-elf/release/examples/ut

bench_pairing:
	cd alt_bn128_rv && \
	cargo build --release --target riscv64imac-unknown-none-elf --example bench_pairing && \
	$(RUNNER) target/riscv64imac-unknown-none-elf/release/examples/bench_pairing

staticlib:
	cd alt_bn128_staticlib && \
	cargo build --release --target riscv64imac-unknown-none-elf && \
	$(RISCV)/bin/riscv64-unknown-elf-gcc -o target/ut examples/ut.c target/riscv64imac-unknown-none-elf/release/libalt_bn128.a && \
	$(RUNNER) target/ut
