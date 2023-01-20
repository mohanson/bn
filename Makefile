# RISCV_RUNNER could be one of https://github.com/mohanson/ckb-vm-runner
ifndef RISCV_RUNNER
	RISCV_RUNNER=echo
endif

# RISCV toolkits
ifndef RISCV
	$(error RISCV is not set)
endif

alt_bn128:
	cd alt_bn128 && cargo clean
	cd alt_bn128 && cargo run --release --example ut
	cd alt_bn128 && cargo run --release --example bench_pairing

alt_bn128_rv:
	cd alt_bn128_rv && cargo clean
	cd alt_bn128_rv && cargo build --release --target riscv64imac-unknown-none-elf --example ut
	cd alt_bn128_rv && $(RISCV_RUNNER) target/riscv64imac-unknown-none-elf/release/examples/ut
	cd alt_bn128_rv && cargo build --release --target riscv64imac-unknown-none-elf --example bench_pairing
	cd alt_bn128_rv && $(RISCV_RUNNER) target/riscv64imac-unknown-none-elf/release/examples/bench_pairing

alt_bn128_rv_bench_pairing_pprof:
	cd alt_bn128_rv && cargo clean
	cd alt_bn128_rv && cargo build --target riscv64imac-unknown-none-elf --example bench_pairing
	/src/ckb-vm-pprof/target/release/ckb-vm-pprof --bin ./alt_bn128_rv/target/riscv64imac-unknown-none-elf/debug/examples/bench_pairing | py /src/ckb-vm-pprof/scripts/folder.py | inferno-flamegraph > /tmp/out.svg

alt_bn128_staticlib:
	cd alt_bn128_staticlib && cargo clean
	cd alt_bn128_staticlib && cargo build --release --target riscv64imac-unknown-none-elf
	cd alt_bn128_staticlib && $(RISCV)/bin/riscv64-unknown-elf-gcc -o target/ut examples/ut.c target/riscv64imac-unknown-none-elf/release/libalt_bn128.a
	cd alt_bn128_staticlib && $(RISCV_RUNNER) target/ut

.PHONY: alt_bn128 alt_bn128_rv alt_bn128_rv_bench_pairing alt_bn128_rv_bench_pairing_pprof alt_bn128_staticlib
