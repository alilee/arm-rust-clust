extern crate gcc;

fn main() {
    gcc::compile_library("libstartup.a",
                         &["src/arch/aarch64-unknown-linux-gnu/startup.s"]);
}
