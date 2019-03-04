#[cfg(test)]
pub mod test;

#[cfg(all(not(test), target_arch = "arm"))]
pub mod arm;

#[cfg(all(not(test), target_arch = "aarch64"))]
pub mod aarch64;
