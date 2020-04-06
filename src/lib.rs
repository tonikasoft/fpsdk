//! The FL Plugin sdk helps you to make plugins for FL Studio. For more information about FL
//! Studio, visit the website (www.flstudio.com).
//!
//! Note that this sdk is not meant to make hosts for FL plugins.
//!
//! ## Types of plugins
//!
//! There are two kinds of Fruity plugins: effects and generators. Effects are plugins that receive
//! some audio data from FL Studio and do something to it (apply an effect). Generators on the
//! other hand create sounds that they send to FL Studio. Generators are seen as channels by the
//! user (like the SimSynth and Sytrus). The main reason to make something a generator is that it
//! needs input from the FL Studio pianoroll (although there are other reasons possible).
//!
//! ## Installation
//!
//! Plugins are installed in FL Studio in subfolders of the FL Studio\Plugins\Fruity folder on
//! Windows and FL\ Studio.app/Contents/Resources/FL/Plugins/Fruity for macOS. Effects go in the
//! Effects subfolder, generators are installed in the Generators subfolder. Each plugin has its
//! own folder. The name of the folder has to be same as the name of the plugin. On macOS the
//! plugin (.dylib) also has to have `_x64` suffix.
//!
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts
)]
#![warn(
    deprecated_in_future,
    missing_docs,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unreachable_pub
)]

use std::os::raw::c_int;

#[cxx::bridge]
mod ffi {

    extern "C" {
        include!("wrapper.h");

        type TFruityPlug;
        type TFruityPlugHost;

        fn create_plug_instance_c(
            host: &'static mut TFruityPlugHost,
            tag: i64,
        ) -> &'static mut TFruityPlug;
    }

    extern "Rust" {}
}

/// This structure holds some information about the plugin that is used by the host. It is the same
/// for all instances of the same plugin.
#[derive(Debug)]
pub struct Info {
    /// This has to be the version of the SDK used to create the plugin. This value is available in
    /// the constant CurrentSDKVersion
    pub sdk_version: i32,
    /// The name of the plugin dll, without the extension (.dll)
    pub long_name: String,
    pub short_name: String,
    flags: i32,
    pub num_parms: i32,
    pub def_poly: i32,
    pub num_out_ctrls: i32,
    pub num_out_voices: i32,
}

#[allow(non_snake_case)]
#[no_mangle]
/// # Safety
/// Unsafe
pub unsafe extern "C" fn CreatePlugInstance(
    host: *mut ffi::TFruityPlugHost,
    tag: c_int,
) -> *mut ffi::TFruityPlug {
    ffi::create_plug_instance_c(&mut *host, tag as i64)
}
