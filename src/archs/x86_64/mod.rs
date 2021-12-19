// SPDX-License-Identifier: Unlicense

pub struct Arch {}

impl super::ArchTrait for Arch {}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    unimplemented!()
}
