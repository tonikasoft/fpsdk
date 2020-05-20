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
        pub port: u8,
    }

    #[derive(Clone)]
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
use std::mem;
use std::os::raw::{c_char, c_int, c_void};

use bitflags::bitflags;
use log::{debug, error};

pub use ffi::{MidiMessage, TimeSignature};
use plugin::PluginAdapter;

/// Current FL SDK version.
pub const CURRENT_SDK_VERSION: u32 = 1;

/// Size of wavetable used by FL.
pub const WAVETABLE_SIZE: usize = 16384;

/// intptr_t alias
#[allow(non_camel_case_types)]
#[doc(hidden)]
pub type intptr_t = isize;

/// An identefier the host uses to identify plugin and voice instances.
///
/// To make it more type safe, `plugin` and `voice` modules provide their own `Tag` type.
pub(crate) type Tag = intptr_t;

/// This macro is used internally to implement `Tag` type in a type-safe manner.
#[doc(hidden)]
#[macro_export]
macro_rules! implement_tag {
    () => {
        use std::fmt;

        /// Identifier.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct Tag(pub crate::Tag);

        impl fmt::Display for Tag {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

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

impl AsRawPtr for f32 {
    fn as_raw_ptr(&self) -> intptr_t {
        self.to_bits() as intptr_t
    }
}

impl AsRawPtr for f64 {
    fn as_raw_ptr(&self) -> intptr_t {
        self.to_bits() as intptr_t
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

impl FromRawPtr for ValuePtr {
    fn from_raw_ptr(value: intptr_t) -> Self {
        ValuePtr(value)
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

/// Song time in **bar:step:tick** format.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct SongTime {
    pub bar: i32,
    pub step: i32,
    pub tick: i32,
}

impl FromRawPtr for SongTime {
    fn from_raw_ptr(value: intptr_t) -> Self {
        unsafe { *Box::from_raw(value as *mut c_void as *mut Self) }
    }
}

/// Collection of notes, which you can add to the piano roll using
/// [`Host::on_message`](host/struct.Host.html#on_message.new) with message
/// [`plugin::Message::AddToPianoRoll`](../plugin/enum.Message.html#variant.AddToPianoRoll).
#[derive(Debug)]
pub struct Notes {
    // 0=step seq (not supported yet), 1=piano roll
    //target: i32,
    /// Notes.
    pub notes: Vec<Note>,
    /// See [`NotesFlags`](struct.NotesFlags.html).
    pub flags: NotesFlags,
    /// Pattern number. `None` for current.
    pub pattern: Option<u32>,
    /// Channel number. `None` for plugin's channel, or selected channel if plugin is an effect.
    pub channel: Option<u32>,
}

/// This type represents a note in [`Notes`](struct.Notes.html).
#[derive(Debug)]
#[repr(C)]
pub struct Note {
    /// Position in PPQ.
    pub position: i32,
    /// Length in PPQ.
    pub length: i32,
    /// Pan in range -100..100.
    pub pan: i32,
    /// Volume.
    pub vol: i32,
    /// Note number.
    pub note: i16,
    /// Color or MIDI channel in range of 0..15.
    pub color: i16,
    /// Fine pitch in range -1200..1200.
    pub pitch: i32,
    /// Mod X or filter cutoff frequency.
    pub mod_x: f32,
    /// Mod Y or filter resonance (Q).
    pub mod_y: f32,
}

bitflags! {
    /// Notes parameters flags
    pub struct NotesFlags: isize {
        /// Delete everything currently on the piano roll before adding the notes.
        const EMPTY_FIRST = 1;
        /// Put the new notes in the piano roll selection, if there is one.
        const USE_SELECTION = 2;
    }
}

// This type in FL SDK is what we represent as Notes. Here we use it for FFI, to send it to C++.
#[repr(C)]
struct TNotesParams {
    target: c_int,
    flags: c_int,
    pat_num: c_int,
    chan_num: c_int,
    count: c_int,
    notes: *mut Note,
}

impl From<Notes> for TNotesParams {
    fn from(mut notes: Notes) -> Self {
        notes.notes.shrink_to_fit();
        let notes_ptr = notes.notes.as_mut_ptr();
        let len = notes.notes.len();
        mem::forget(notes.notes);

        Self {
            target: 1,
            flags: notes.flags.bits() as c_int,
            pat_num: notes.pattern.map(|v| v as c_int).unwrap_or(-1),
            chan_num: notes.channel.map(|v| v as c_int).unwrap_or(-1),
            count: len as c_int,
            notes: notes_ptr,
        }
    }
}

/// Describes an item that should be added to a control's right-click popup menu.
#[derive(Debug)]
pub struct ParamMenuEntry {
    /// Name.
    pub name: String,
    /// Flags.
    pub flags: ParamMenuItemFlags,
}

bitflags! {
    /// Parameter popup menu item flags
    pub struct ParamMenuItemFlags: i32 {
        /// The item is disabled
        const DISABLED = 1;
        /// The item is checked
        const CHECKED = 2;
    }
}

impl ParamMenuEntry {
    fn from_ffi(ffi_t: *mut TParamMenuEntry) -> Self {
        Self {
            name: unsafe { CString::from_raw((*ffi_t).name) }
                .to_string_lossy()
                .to_string(),
            flags: ParamMenuItemFlags::from_bits(unsafe { (*ffi_t).flags })
                .unwrap_or_else(ParamMenuItemFlags::empty),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct TParamMenuEntry {
    name: *mut c_char,
    flags: c_int,
}

bitflags! {
    /// Message box flags (see
    /// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox).
    pub struct MessageBoxFlags: isize {
        // To indicate the buttons displayed in the message box, specify one of the following
        // values.

        /// The message box contains three push buttons: Abort, Retry, and Ignore.
        const ABORTRETRYIGNORE = 0x0000_0002;
        /// The message box contains three push buttons: Cancel, Try Again, Continue. Use this
        /// message box type instead of ABORTRETRYIGNORE.
        const CANCELTRYCONTINUE = 0x0000_0006;
        /// Adds a Help button to the message box. When the user clicks the Help button or presses
        /// F1, the system sends a WM_HELP message to the owner.
        const HELP = 0x0000_4000;
        /// The message box contains one push button: OK. This is the default.
        const OK = 0x0000_0000;
        /// The message box contains two push buttons: OK and Cancel.
        const OKCANCEL = 0x0000_0001;
        /// The message box contains two push buttons: Retry and Cancel.
        const RETRYCANCEL = 0x0000_0005;
        /// The message box contains two push buttons: Yes and No.
        const YESNO = 0x0000_0004;
        /// The message box contains three push buttons: Yes, No, and Cancel.
        const YESNOCANCEL = 0x0000_0003;

        // To display an icon in the message box, specify one of the following values.

        /// An exclamation-point icon appears in the message box.
        const ICONEXCLAMATION = 0x0000_0030;
        /// An exclamation-point icon appears in the message box.
        const ICONWARNING = 0x0000_0030;
        /// An icon consisting of a lowercase letter i in a circle appears in the message box.
        const ICONINFORMATION = 0x0000_0040;
        /// An icon consisting of a lowercase letter i in a circle appears in the message box.
        const ICONASTERISK = 0x0000_0040;
        /// A question-mark icon appears in the message box. The question-mark message icon is no
        /// longer recommended because it does not clearly represent a specific type of message and
        /// because the phrasing of a message as a question could apply to any message type. In
        /// addition, users can confuse the message symbol question mark with Help information.
        /// Therefore, do not use this question mark message symbol in your message boxes. The
        /// system continues to support its inclusion only for backward compatibility.
        const ICONQUESTION = 0x0000_0020;
        /// A stop-sign icon appears in the message box.
        const ICONSTOP = 0x0000_0010;
        /// A stop-sign icon appears in the message box.
        const ICONERROR = 0x0000_0010;
        /// A stop-sign icon appears in the message box.
        const ICONHAND = 0x0000_0010;

        // To indicate the default button, specify one of the following values.

        /// The first button is the default button.
        ///
        /// DEFBUTTON1 is the default unless DEFBUTTON2, DEFBUTTON3, or DEFBUTTON4 is specified.
        const DEFBUTTON1 = 0x0000_0000;

        /// The second button is the default button.
        const DEFBUTTON2 = 0x0000_0100;
        /// The third button is the default button.
        const DEFBUTTON3 = 0x0000_0200;
        /// The fourth button is the default button.
        const DEFBUTTON4 = 0x0000_0300;

        // To indicate the modality of the dialog box, specify one of the following values.

        /// The user must respond to the message box before continuing work in the window
        /// identified by the hWnd parameter. However, the user can move to the windows of other
        /// threads and work in those windows.
        ///
        /// Depending on the hierarchy of windows in the application, the user may be able to move
        /// to other windows within the thread. All child windows of the parent of the message box
        /// are automatically disabled, but pop-up windows are not.
        ///
        /// APPLMODAL is the default if neither SYSTEMMODAL nor TASKMODAL is specified.
        const APPLMODAL = 0x0000_0000;


        /// Same as APPLMODAL except that the message box has the WS_EX_TOPMOST style. Use
        /// system-modal message boxes to notify the user of serious, potentially damaging errors
        /// that require immediate attention (for example, running out of memory). This flag has no
        /// effect on the user's ability to interact with windows other than those associated with
        /// hWnd.
        const SYSTEMMODAL = 0x0000_1000;
        /// Same as APPLMODAL except that all the top-level windows belonging to the current thread
        /// are disabled if the hWnd parameter is NULL. Use this flag when the calling application
        /// or library does not have a window handle available but still needs to prevent input to
        /// other windows in the calling thread without suspending other threads.
        const TASKMODAL = 0x0000_2000;

        // To specify other options, use one or more of the following values.

        /// Same as desktop of the interactive window station. For more information, see Window
        /// Stations.
        ///
        /// If the current input desktop is not the default desktop, MessageBox does not return
        /// until the user switches to the default desktop.
        const DEFAULT_DESKTOP_ONLY = 0x0002_0000;

        /// The text is right-justified.
        const RIGHT = 0x0008_0000;
        /// Displays message and caption text using right-to-left reading order on Hebrew and
        /// Arabic systems.
        const RTLREADING = 0x0010_0000;
        /// The message box becomes the foreground window. Internally, the system calls the
        /// SetForegroundWindow function for the message box.
        const SETFOREGROUND = 0x0001_0000;
        /// The message box is created with the WS_EX_TOPMOST window style.
        const TOPMOST = 0x0004_0000;
        /// The caller is a service notifying the user of an event. The function displays a message
        /// box on the current active desktop, even if there is no user logged on to the computer.
        ///
        /// Terminal Services: If the calling thread has an impersonation token, the function
        /// directs the message box to the session specified in the impersonation token.
        ///
        /// If this flag is set, the hWnd parameter must be NULL. This is so that the message box
        /// can appear on a desktop other than the desktop corresponding to the hWnd.
        ///
        /// For information on security considerations in regard to using this flag, see
        /// Interactive Services. In particular, be aware that this flag can produce interactive
        /// content on a locked desktop and should therefore be used for only a very limited set of
        /// scenarios, such as resource exhaustion.
        const SERVICE_NOTIFICATION = 0x0020_0000;
    }
}

impl AsRawPtr for MessageBoxFlags {
    fn as_raw_ptr(&self) -> intptr_t {
        self.bits()
    }
}

/// The result returned by a message box.
///
/// See
/// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox#return-value
/// for more info.
#[derive(Debug)]
pub enum MessageBoxResult {
    /// The OK button was selected.
    Ok,
    /// The Cancel button was selected.
    Cancel,
    /// The Abort button was selected.
    Abort,
    /// The Retry button was selected.
    Retry,
    /// The Ignore button was selected.
    Ignore,
    /// The Yes button was selected.
    Yes,
    /// The No button was selected.
    No,
    /// The Try Again button was selected.
    TryAgain,
    /// The Continue button was selected.
    Continue,
    /// Unknown.
    Unknown,
}

impl FromRawPtr for MessageBoxResult {
    fn from_raw_ptr(value: intptr_t) -> Self {
        match value {
            1 => MessageBoxResult::Ok,
            2 => MessageBoxResult::Cancel,
            3 => MessageBoxResult::Abort,
            4 => MessageBoxResult::Retry,
            5 => MessageBoxResult::Ignore,
            6 => MessageBoxResult::Yes,
            7 => MessageBoxResult::No,
            10 => MessageBoxResult::TryAgain,
            11 => MessageBoxResult::Continue,
            _ => MessageBoxResult::Unknown,
        }
    }
}

/// Time format.
#[derive(Debug)]
pub enum TimeFormat {
    /// Beats.
    Beats,
    /// Absolute ms.
    AbsoluteMs,
    /// Running ms.
    RunningMs,
    /// Time since sound card restart (in ms).
    RestartMs,
}

impl From<TimeFormat> for u8 {
    fn from(format: TimeFormat) -> Self {
        match format {
            TimeFormat::Beats => 0,
            TimeFormat::AbsoluteMs => 1,
            TimeFormat::RunningMs => 2,
            TimeFormat::RestartMs => 3,
        }
    }
}

/// Time
///
/// The first value is mixing time.
///
/// The second value is offset in samples.
#[derive(Debug, Default)]
#[repr(C)]
pub struct Time(pub f64, pub f64);

impl FromRawPtr for Time {
    fn from_raw_ptr(value: intptr_t) -> Self {
        unsafe { *Box::from_raw(value as *mut c_void as *mut Time) }
    }
}

/// Name of the color (or MIDI channel) in Piano Roll.
#[derive(Debug)]
pub struct NameColor {
    /// User-defined name (can be empty).
    pub name: String,
    /// Visible name (can be guessed).
    pub vis_name: String,
    /// Color/MIDI channel index.
    pub color: u8,
    /// Real index of the item (can be used to translate plugin's own in/out into real mixer track
    /// number).
    pub index: usize,
}

/// Type used in FFI for [`NameColor`](struct.NameColor.html).
#[repr(C)]
pub struct TNameColor {
    name: [u8; 256],
    vis_name: [u8; 256],
    color: c_int,
    index: c_int,
}

impl FromRawPtr for TNameColor {
    fn from_raw_ptr(value: intptr_t) -> Self {
        unsafe { *Box::from_raw(value as *mut Self) }
    }
}

impl From<TNameColor> for NameColor {
    fn from(name_color: TNameColor) -> Self {
        Self {
            name: String::from_utf8_lossy(&name_color.name[..]).to_string(),
            vis_name: String::from_utf8_lossy(&name_color.vis_name[..]).to_string(),
            color: name_color.color as u8,
            index: name_color.index as usize,
        }
    }
}

impl From<NameColor> for TNameColor {
    fn from(name_color: NameColor) -> Self {
        let mut name = [0_u8; 256];
        name.copy_from_slice(name_color.name.as_bytes());
        let mut vis_name = [0_u8; 256];
        vis_name.copy_from_slice(name_color.vis_name.as_bytes());
        Self {
            name,
            vis_name,
            color: name_color.color as c_int,
            index: name_color.index as c_int,
        }
    }
}

impl From<u64> for MidiMessage {
    fn from(value: u64) -> Self {
        MidiMessage {
            status: (value & 0xff) as u8,
            data1: ((value >> 8) & 0xff) as u8,
            data2: ((value >> 16) & 0xff) as u8,
            port: 0,
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
