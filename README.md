# bn

This repo implements the following EIPs on CKB-VM:

- https://github.com/ethereum/EIPs/blob/master/EIPS/eip-196.md
- https://github.com/ethereum/EIPs/blob/master/EIPS/eip-197.md

```sh
export RISCV=/opt/riscv
export RISCV_RUNNER=~/ckb-vm-run/target/release/mop

# Build alt_bn128 to native and test it in rust
make alt_bn128

# Build alt_bn128 to risc-v and test it in ckb-vm
make alt_bn128_rv

# Build 2 point pairing example to risc-v and test it in ckb-vm
make alt_bn128_rv_bench_pairing

# Build alt_bn128 to risc-v staticlib
make alt_bn128_staticlib
```
