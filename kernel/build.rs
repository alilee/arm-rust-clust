extern crate gcc;

fn main() {
    #[cfg(target_arch = "aarch64")]
    gcc::compile_library("libstartup.a", &["src/archs/aarch64/startup.s"]);
}
