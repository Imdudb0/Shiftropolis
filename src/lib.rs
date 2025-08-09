// src/lib.rs
mod main;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(_app: AndroidApp) {
    main::main();
}

#[cfg(target_os = "android")]
pub use main::main;