fn main() {
    cc::Build::new()
        .compiler(format!(
            "{}/bin/riscv64-unknown-elf-gcc",
            std::env::var("RISCV").unwrap()
        ))
        .target("elf64-littleriscv")
        .file("src/ll_u256_mont-riscv64.S")
        .compile("ll_u256_mont");
}
