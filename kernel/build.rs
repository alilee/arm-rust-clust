extern crate gcc;

fn main() {
    // #[cfg(target_arch = "aarch64")]
    gcc::compile_library(
        "libarch.a",
        // &["src/archs/aarch64/startup.s"],
        &["src/archs/aarch64/startup.s", "src/archs/aarch64/handler.s"],
    );
}
