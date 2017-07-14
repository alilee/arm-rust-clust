extern crate gcc;

fn main() {
    gcc::compile_library(
        "libaarch64.a",
        &[
            "src/archs/aarch64/startup.s",
            "src/archs/aarch64/handler/handler.s",
        ],
    );
}
