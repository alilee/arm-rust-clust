#![feature(lang_items)]
// #![feature(core_intrinsics)]
// #![feature(core_str_ext)]
// #![feature(core_slice_ext)]
#![feature(type_macros)]
#![feature(const_fn)]
#![feature(associated_consts)]
#![no_std]

// #[macro_use]
// mod log;
//
// mod uart;
//
// pub mod vm;


#[no_mangle]
pub extern fn rust_main() {

    // info!("starting");
    //
    // vm::init();
    //
    // error!("test error");
    // warn!("test warn");
    // info!("test info");
    // debug!("test debug");
    // trace!("test trace");
    //
    // info!("done, looping.");
    loop {}
    
}


#[cfg(not(test))] 
#[lang = "eh_personality"] extern fn eh_personality() {}

#[cfg(not(test))] 
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop{} }
