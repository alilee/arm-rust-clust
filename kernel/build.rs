extern crate gcc;

fn main() {
    gcc::compile_library("libstartup.a", &["src/archs/aarch64/startup.s"]);
}
