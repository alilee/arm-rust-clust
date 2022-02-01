// SPDX-License-Identifier: Unlicense

#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(libkernel::util::testing::test_runner)]
#![feature(format_args_nl)] // for debug macros
#![feature(box_syntax)] // for init on heap
#![feature(map_first_last)] // for scanning BlockDeviceMap

#[allow(unused_imports)]
#[macro_use]
extern crate libkernel;

extern crate alloc;

use libkernel::Result;
use libkernel::{device, device::Sector};
use libkernel::{
    pager,
    pager::{Page, Paging},
};

use alloc::boxed::Box;

use core::sync::{atomic, atomic::Ordering};

use test_macros::kernel_test;

#[no_mangle]
pub extern "C" fn collect_tests() -> () {
    test_main()
}

#[inline(never)]
fn read() -> Result<Box<[Page; 4]>> {
    let mut blank = box Page::new();
    for x in blank.slice().iter_mut() {
        *x = 10203040;
    }
    let mut sector0 = box [*blank; 4];

    assert_eq!(10203040, sector0[3].slice()[42]);

    let disk = {
        let block_devices = device::BLOCK_DEVICES.lock();
        let (_, disk) = block_devices
            .first_key_value()
            .expect("block_devices.first_key");
        disk.clone()
    };
    let mut disk = disk.lock();

    let id = disk
        .read(
            &[
                pager::Pager::maps_to((&sector0[0]).into())?,
                pager::Pager::maps_to((&sector0[1]).into())?,
                pager::Pager::maps_to((&sector0[2]).into())?,
                pager::Pager::maps_to((&sector0[3]).into())?,
            ],
            Sector(0),
        )
        .expect("read");

    for _ in 0..10 {
        match disk.status(id) {
            Err(e) => {
                info!("{:?}", e);
            }
            Ok(used_len) => {
                dbg!(used_len);
                break;
            }
        }
    }
    atomic::fence(Ordering::SeqCst);
    assert_eq!(0, sector0[3].slice()[42]);

    info!("read!");
    Ok(sector0)
}

#[kernel_test]
fn device_init() {
    major!("initialising device");
    device::init().expect("device::init");
    info!("done");

    _breakpoint();

    let mut p = read().expect("sandwich");
    dbg!(p[3].slice()[42]);
    dbg!(&p[3].slice()[42] as *const u64);
    dbg!(pager::Pager::maps_to((&p[3].slice()[42]).into()));

    debug!("returned");
}

use libkernel::debug::{Level, _breakpoint};

#[no_mangle]
fn _override_log_levels() -> (Level, &'static [(&'static str, Level)]) {
    const LOG_LEVEL_SETTINGS: &[(&str, Level)] = &[
        ("aarch64::pager", Level::Info),
        ("aarch64::pager::walk", Level::Major),
        ("pager", Level::Info),
        ("pager::layout", Level::Major),
        ("pager::frames", Level::Info),
        ("pager::frames::deque", Level::Major),
        ("pager::bump", Level::Major),
        ("pager::alloc", Level::Major),
    ];
    (Level::Trace, LOG_LEVEL_SETTINGS)
}
