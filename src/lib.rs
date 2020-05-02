//! The FL Plugin SDK helps you to make plugins for FL Studio. For more information about FL
//! Studio, visit the [website](https://www.image-line.com/flstudio/).
//!
//! Note that this SDK is not meant to make hosts for FL plugins.
//!
//! ## How to use this library
//!
//! You should implement [`Plugin`](plugin/trait.Plugin.html) and export it with
//! [`create_plugin!`](macro.create_plugin.html).
//!
//! To talk to host use [`Host`](host/struct.Host.html).
//!
//! `examples/simple.rs` provides you with more details.
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
//! Plugins are installed in FL Studio in subfolders of the `FL Studio\Plugins\Fruity` folder on
//! Windows and `FL\ Studio.app/Contents/Resources/FL/Plugins/Fruity` for macOS.
//!
//! Effects go in the **Effects** subfolder, generators are installed in the **Generators**
//! subfolder. Each plugin has its own folder.
//!
//! The name of the folder has to be same as the name of the plugin. On macOS the plugin (.dylib)
//! also has to have `_x64` suffix.
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

/// Used internally for C++ <-> Rust interoperability. Shouldn't be used directly.
#[doc(hidden)]
#[cxx::bridge]
pub mod ffi {
    /// Time signature.
    pub struct TimeSignature {
        /// Steps per bar.
        pub steps_per_bar: u32,
        /// Steps per beat.
        pub steps_per_beat: u32,
        /// Pulses per quarter note.
        pub ppq: u32,
    }

    /// MIDI message.
    pub struct MidiMessage {
        pub status: u8,
        pub data1: u8,
        pub data2: u8,
        /// -1 if not applicable
        pub port: i32,
    }

    pub struct Message {
        pub id: isize,
        pub index: isize,
        pub value: isize,
    }

    extern "C" {
        include!("wrapper.h");

        pub fn time_sig_from_raw(raw_time_sig: isize) -> TimeSignature;
    }

    extern "Rust" {
        type PluginAdapter;

        fn fplog(message: &str);
        // Used for debugging
        fn print_adapter(adapter: &PluginAdapter);
    }
}

pub mod host;
pub mod plugin;
pub mod voice;

use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_void};

use bitflags::bitflags;
use log::{debug, error};

pub use ffi::{MidiMessage, TimeSignature};
use plugin::PluginAdapter;

/// An identefier the host uses to identify plugin and voice instances.
pub type Tag = i32;

/// Current FL SDK version.
pub const CURRENT_SDK_VERSION: u32 = 1;

/// Size of wavetable used by FL.
pub const WAVETABLE_SIZE: usize = 16384;

/// intptr_t alias
#[allow(non_camel_case_types)]
type intptr_t = isize;

fn fplog(message: &str) {
    debug!("{}", message);
}

fn print_adapter(adapter: &PluginAdapter) {
    debug!("{:?}", adapter);
}

/// FFI to free rust's Box::into_raw pointer.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn free_rbox_raw(raw_ptr: *mut c_void) {
    let _ = Box::from_raw(raw_ptr);
}

/// FFI to free rust's CString pointer.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn free_rstring(raw_str: *mut c_char) {
    let _ = CString::from_raw(raw_str);
}

/// FFI to make C string (`char *`) managed by C side. Because `char *` produced by
/// `CString::into_raw` leads to memory leak:
///
/// > The pointer which this function returns must be returned to Rust and reconstituted using
/// > from_raw to be properly deallocated. Specifically, one should not use the standard C free()
/// > function to deallocate this string.
#[no_mangle]
extern "C" {
    fn alloc_real_cstr(raw_str: *mut c_char) -> *mut c_char;
}

/// Raw pointer to value.
#[derive(Debug)]
pub struct ValuePtr(intptr_t);

impl ValuePtr {
    /// Get value.
    ///
    /// See [`FromRawPtr`](trait.FromRawPtr.html) for implemented types.
    pub fn get<T: FromRawPtr>(&self) -> T {
        T::from_raw_ptr(self.0)
    }
}

/// For types, which can be represented as `intptr_t`.
pub trait AsRawPtr {
    /// Conversion method.
    fn as_raw_ptr(&self) -> intptr_t;
}

macro_rules! primitive_as_raw_ptr {
    ($type:ty) => {
        impl AsRawPtr for $type {
            fn as_raw_ptr(&self) -> intptr_t {
                (*self) as intptr_t
            }
        }
    };
}

primitive_as_raw_ptr!(i8);
primitive_as_raw_ptr!(u8);
primitive_as_raw_ptr!(i16);
primitive_as_raw_ptr!(u16);
primitive_as_raw_ptr!(i32);
primitive_as_raw_ptr!(u32);
primitive_as_raw_ptr!(i64);
primitive_as_raw_ptr!(u64);
primitive_as_raw_ptr!(usize);
primitive_as_raw_ptr!(*mut c_void);
primitive_as_raw_ptr!(*const c_void);

impl AsRawPtr for bool {
    fn as_raw_ptr(&self) -> intptr_t {
        (self.to_owned() as u8).into()
    }
}

impl AsRawPtr for String {
    fn as_raw_ptr(&self) -> intptr_t {
        let value = CString::new(self.clone()).unwrap_or_else(|e| {
            error!("{}", e);
            panic!();
        });
        // alloc_real_cstr prevents memory leak caused by CString::into_raw
        unsafe { alloc_real_cstr(value.into_raw()) as intptr_t }
    }
}

/// For conversion from `intptr_t`.
pub trait FromRawPtr {
    /// Conversion method.
    fn from_raw_ptr(value: intptr_t) -> Self
    where
        Self: Sized;
}

macro_rules! primitive_from_raw_ptr {
    ($type:ty) => {
        impl FromRawPtr for $type {
            fn from_raw_ptr(value: intptr_t) -> Self {
                value as Self
            }
        }
    };
}

primitive_from_raw_ptr!(i8);
primitive_from_raw_ptr!(u8);
primitive_from_raw_ptr!(i16);
primitive_from_raw_ptr!(u16);
primitive_from_raw_ptr!(i32);
primitive_from_raw_ptr!(u32);
primitive_from_raw_ptr!(i64);
primitive_from_raw_ptr!(u64);
primitive_from_raw_ptr!(usize);
primitive_from_raw_ptr!(*mut c_void);
primitive_from_raw_ptr!(*const c_void);

impl FromRawPtr for f32 {
    fn from_raw_ptr(value: intptr_t) -> Self {
        f32::from_bits(value as i32 as u32)
    }
}

impl FromRawPtr for f64 {
    fn from_raw_ptr(value: intptr_t) -> Self {
        f64::from_bits(value as i64 as u64)
    }
}

impl FromRawPtr for String {
    fn from_raw_ptr(value: intptr_t) -> Self {
        let cstr = unsafe { CStr::from_ptr(value as *const c_char) };
        cstr.to_string_lossy().to_string()
    }
}

impl FromRawPtr for bool {
    fn from_raw_ptr(value: intptr_t) -> Self {
        value != 0
    }
}

bitflags! {
    /// Parameter flags.
    pub struct ParameterFlags: isize {
        /// Makes no sense to interpolate parameter values (when values are not levels).
        const CANT_INTERPOLATE = 1;
        /// Parameter is a normalized (0..1) single float. (Integer otherwise)
        const FLOAT = 2;
        /// Parameter appears centered in event editors.
        const CENTERED = 4;
    }
}

impl AsRawPtr for ParameterFlags {
    fn as_raw_ptr(&self) -> intptr_t {
        self.bits()
    }
}

bitflags! {
    /// Processing mode flags.
    pub struct ProcessModeFlags: isize {
        /// Realtime rendering.
        const NORMAL = 0;
        /// Realtime rendering with a higher quality.
        const HQ_REALTIME = 1;
        /// Non realtime processing (CPU does not matter, quality does) (normally set when
        /// rendering only).
        const HQ_NON_REALTIME = 2;
        /// FL is rendering to file if this flag is set.
        const IS_RENDERING = 16;
        /// (changed in FL 7.0) 3 bits value for interpolation quality.
        ///
        /// - 0=none (obsolete)
        /// - 1=linear
        /// - 2=6 point hermite (default)
        /// - 3=32 points sinc
        /// - 4=64 points sinc
        /// - 5=128 points sinc
        /// - 6=256 points sinc
        const IP_MASK = 0xFFFF << 8;
    }
}

bitflags! {
    /// Processing parameters flags
    pub struct ProcessParamFlags: isize {
        /// Update the value of the parameter.
        const UPDATE_VALUE = 1;
        /// Return the value of the parameter as the result of the function.
        const GET_VALUE = 2;
        /// Update the hint if there is one.
        const SHOW_HINT = 4;
        /// Update the parameter control (wheel, slider, ...).
        const UPDATE_CONTROL = 16;
        /// A value between 0 and 65536 has to be translated to the range of the parameter control.
        ///
        /// Note that you should also return the translated value, even if
        /// [ProcessParamFlags::GET_VALUE](
        /// struct.ProcessParamFlags.html#associatedconstant.GET_VALUE) isn't included.
        const FROM_MIDI = 32;
        /// (internal) Don't check if wheels are linked.
        const NO_LINK = 1024;
        /// Sent by an internal controller. Internal controllers should pay attention to these,
        /// to avoid Feedback of controller changes.
        const INTERNAL_CTRL = 2048;
        /// This flag is free to be used by the plugin as it wishes.
        const PLUG_RESERVED = 4096;
    }
}

bitflags! {
    /// Parameter popup menu item flags
    pub struct ParamMenuItemFlags: isize {
        /// The item is disabled
        const DISABLED = 1;
        /// The item is checked
        const CHECKED = 2;
    }
}

bitflags! {
    /// Sample loading flags
    pub struct SampleLoadFlags: isize {
        ///This tells the sample loader to show an open box, for the user to select a sample
        const SHOW_DIALOG = 1;
        /// Force the sample to be reloaded, even if the filename is the same.
        ///
        /// This is handy in case you modified the sample, for example
        const FORCE_RELOAD = 2;
        /// Don't load the sample, instead get its filename & make sure that the format is correct
        ///
        /// (useful after [host::HostMessage::ChanSampleChanged](
        /// enum.HostMessage.html#variant.ChanSampleChanged))
        const GET_NAME = 4;
        /// Don't resample to the host sample rate
        const NO_RESAMPLING = 5;
    }
}

bitflags! {
    /// Notes parameters flags
    pub struct NotesParamsFlags: isize {
        /// Delete everything currently on the piano roll before adding the notes
        const EMPTY_FIRST = 1;
        /// Put the new notes in the piano roll selection, if there is one
        const USE_SELECTION = 2;
    }
}

/// if `Jog`, `StripJog`, `MarkerJumpJog`, `MarkerSelJog`, `Previous` or `Next` don't answer,
/// `PreviousNext` will be tried. So it's best to implement at least `PreviousNext`.
///
/// if `PunchIn` or `PunchOut` don't answer, `Punch` will be tried
///
/// if `UndoUp` doesn't answer, `UndoJog` will be tried
///
/// if `AddAltMarker` doesn't answer, `AddMarker` will be tried
///
/// if `Cut`, `Copy`, `Paste`, `Insert`, `Delete`, `NextWindow`, `Enter`, `Escape`, `Yes`, `No`,
/// `Fx` don't answer, standard keystrokes will be simulated
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Transport {
    /// Generic jog (can be used to select stuff).
    Jog(Jog),
    /// Alternate generic jog (can be used to relocate stuff).
    Jog2(Jog),
    /// Touch-sensitive jog strip, value will be in -65536..65536 for leftmost..rightmost.
    Strip(Jog),
    /// Touch-sensitive jog in jog mode.
    StripJog(Jog),
    /// Value will be `0` for release, 1,2 for 1,2 fingers centered mode, -1,-2 for 1,2 fingers jog
    /// mode (will then send `StripJog`).
    StripHold(Jog),
    Previous(Button),
    Next(Button),
    /// Generic track selection.
    PreviousNext(Jog),
    /// Used to relocate items.
    MoveJog(Jog),
    /// Play/pause.
    Play(Button),
    Stop(Button),
    Record(Button),
    Rewind(Hold),
    FastForward(Hold),
    Loop(Button),
    Mute(Button),
    /// Generic or record mode.
    Mode(Button),
    /// Undo/redo last, or undo down in history.
    Undo(Button),
    /// Undo up in history (no need to implement if no undo history).
    UndoUp(Button),
    /// Undo in history (no need to implement if no undo history).
    UndoJog(Jog),
    /// Live selection.
    Punch(Hold),
    PunchIn(Button),
    PunchOut(Button),
    AddMarker(Button),
    /// Add alternate marker.
    AddAltMarker(Button),
    /// Marker jump.
    MarkerJumpJog(Jog),
    /// Marker selection.
    MarkerSelJog(Jog),
    Up(Button),
    Down(Button),
    Left(Button),
    Right(Button),
    HZoomJog(Jog),
    VZoomJog(Jog),
    /// Snap on/off.
    Snap(Button),
    SnapMode(Jog),
    Cut(Button),
    Copy(Button),
    Paste(Button),
    Insert(Button),
    Delete(Button),
    /// TAB.
    NextWindow(Button),
    /// Window selection.
    WindowJog(Jog),
    F1(Button),
    F2(Button),
    F3(Button),
    F4(Button),
    F5(Button),
    F6(Button),
    F7(Button),
    F8(Button),
    F9(Button),
    F10(Button),
    /// Enter/accept.
    Enter(Button),
    /// Escape/cancel.
    Escape(Button),
    Yes(Button),
    No(Button),
    /// Generic menu.
    Menu(Button),
    /// Item edit/tool/contextual menu.
    ItemMenu(Button),
    Save(Button),
    SaveNew(Button),
    Unknown,
}

impl From<ffi::Message> for Transport {
    fn from(message: ffi::Message) -> Self {
        match message.index {
            0 => Transport::Jog(Jog(message.value as i64)),
            1 => Transport::Jog2(Jog(message.value as i64)),
            2 => Transport::Strip(Jog(message.value as i64)),
            3 => Transport::StripJog(Jog(message.value as i64)),
            4 => Transport::StripHold(Jog(message.value as i64)),
            5 => Transport::Previous(Button(message.value as u8)),
            6 => Transport::Next(Button(message.value as u8)),
            7 => Transport::PreviousNext(Jog(message.value as i64)),
            8 => Transport::MoveJog(Jog(message.value as i64)),
            10 => Transport::Play(Button(message.value as u8)),
            11 => Transport::Stop(Button(message.value as u8)),
            12 => Transport::Record(Button(message.value as u8)),
            13 => Transport::Rewind(Hold(message.value != 0)),
            14 => Transport::FastForward(Hold(message.value != 0)),
            15 => Transport::Loop(Button(message.value as u8)),
            16 => Transport::Mute(Button(message.value as u8)),
            17 => Transport::Mode(Button(message.value as u8)),
            20 => Transport::Undo(Button(message.value as u8)),
            21 => Transport::UndoUp(Button(message.value as u8)),
            22 => Transport::UndoJog(Jog(message.value as i64)),
            30 => Transport::Punch(Hold(message.value != 0)),
            31 => Transport::PunchIn(Button(message.value as u8)),
            32 => Transport::PunchOut(Button(message.value as u8)),
            33 => Transport::AddMarker(Button(message.value as u8)),
            34 => Transport::AddAltMarker(Button(message.value as u8)),
            35 => Transport::MarkerJumpJog(Jog(message.value as i64)),
            36 => Transport::MarkerSelJog(Jog(message.value as i64)),
            40 => Transport::Up(Button(message.value as u8)),
            41 => Transport::Down(Button(message.value as u8)),
            42 => Transport::Left(Button(message.value as u8)),
            43 => Transport::Right(Button(message.value as u8)),
            44 => Transport::HZoomJog(Jog(message.value as i64)),
            45 => Transport::VZoomJog(Jog(message.value as i64)),
            48 => Transport::Snap(Button(message.value as u8)),
            49 => Transport::SnapMode(Jog(message.value as i64)),
            50 => Transport::Cut(Button(message.value as u8)),
            51 => Transport::Copy(Button(message.value as u8)),
            52 => Transport::Paste(Button(message.value as u8)),
            53 => Transport::Insert(Button(message.value as u8)),
            54 => Transport::Delete(Button(message.value as u8)),
            58 => Transport::NextWindow(Button(message.value as u8)),
            59 => Transport::WindowJog(Jog(message.value as i64)),
            60 => Transport::F1(Button(message.value as u8)),
            61 => Transport::F2(Button(message.value as u8)),
            62 => Transport::F3(Button(message.value as u8)),
            63 => Transport::F4(Button(message.value as u8)),
            64 => Transport::F5(Button(message.value as u8)),
            65 => Transport::F6(Button(message.value as u8)),
            66 => Transport::F7(Button(message.value as u8)),
            67 => Transport::F8(Button(message.value as u8)),
            68 => Transport::F9(Button(message.value as u8)),
            69 => Transport::F10(Button(message.value as u8)),
            80 => Transport::Enter(Button(message.value as u8)),
            81 => Transport::Escape(Button(message.value as u8)),
            82 => Transport::Yes(Button(message.value as u8)),
            83 => Transport::No(Button(message.value as u8)),
            90 => Transport::Menu(Button(message.value as u8)),
            91 => Transport::ItemMenu(Button(message.value as u8)),
            92 => Transport::Save(Button(message.value as u8)),
            93 => Transport::SaveNew(Button(message.value as u8)),
            _ => Transport::Unknown,
        }
    }
}

/// `0` for release, `1` for switch (if release is not supported), `2` for hold (if release should
/// be expected).
#[derive(Debug)]
pub struct Button(pub u8);

/// `false` for release, `true` for hold.
#[derive(Debug)]
pub struct Hold(pub bool);

/// Value is an integer increment.
#[derive(Debug)]
pub struct Jog(pub i64);

impl From<u64> for MidiMessage {
    fn from(value: u64) -> Self {
        MidiMessage {
            status: (value & 0xff) as u8,
            data1: ((value >> 8) & 0xff) as u8,
            data2: ((value >> 16) & 0xff) as u8,
            port: -1,
        }
    }
}

impl fmt::Debug for MidiMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidiMessage")
            .field("status", &self.status)
            .field("data1", &self.data1)
            .field("data2", &self.data2)
            .field("port", &self.port)
            .finish()
    }
}

impl fmt::Debug for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeSignature")
            .field("steps_per_bar", &self.steps_per_bar)
            .field("steps_per_beat", &self.steps_per_beat)
            .field("ppq", &self.ppq)
            .finish()
    }
}

impl fmt::Debug for ffi::Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message")
            .field("id", &self.id)
            .field("index", &self.index)
            .field("value", &self.value)
            .finish()
    }
}

#[cfg(test)]
mod tests {}
