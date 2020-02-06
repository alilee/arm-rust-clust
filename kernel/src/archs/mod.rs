#[cfg(test)]
pub mod test;

#[cfg(all(not(test), target_arch = "arm"))]
pub mod arm;

#[cfg(all(not(test), target_arch = "aarch64"))]
pub mod aarch64;

#[cfg(all(not(test), target_arch = "x86_64"))]
pub mod x86_64;
