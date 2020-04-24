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
    /// This structure holds some information about the plugin that is used by the host. It is the
    /// same for all instances of the same plugin.
    ///
    /// Instantiate it using [`InfoBuilder`](struct.InfoBuilder.html).
    pub struct Info {
        /// This has to be the version of the SDK used to create the plugin. This value is
        /// available in the constant CurrentSDKVersion
        pub sdk_version: u32,
        /// The name of the plugin dll, without the extension (.dll)
        pub long_name: String,
        /// Short plugin name, to be used in labels to tell the user which plugin he is working
        /// with
        pub short_name: String,
        flags: u32,
        /// The number of parameters for this plugin
        pub num_params: u32,
        /// Preferred (default) maximum polyphony (FL Studio manages the polyphony) (0=infinite)
        pub def_poly: u32,
        /// Number of internal output controllers
        pub num_out_ctrls: u32,
        /// Number of internal output voices
        pub num_out_voices: u32,
    }

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

        pub type TFruityPlug;
        pub type TFruityPlugHost;

        pub fn create_plug_instance_c(
            host: &'static mut TFruityPlugHost,
            tag: i32,
            adapter: Box<PluginAdapter>,
        ) -> &'static mut TFruityPlug;

        pub fn time_sig_from_raw(raw_time_sig: isize) -> TimeSignature;
    }

    extern "Rust" {
        type PluginAdapter;

        fn plugin_info(adapter: &PluginAdapter) -> Info;
        // Used for debugging
        fn print_adapter(adapter: &PluginAdapter);
        fn plugin_name_of(adapter: &PluginAdapter, message: Message) -> String;
    }
}

pub mod host;
pub mod plugin;

use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_void};

use bitflags::bitflags;
use log::debug;

pub use ffi::{Info, MidiMessage, TimeSignature};
use plugin::Plugin;

/// Current FL SDK version.
pub const CURRENT_SDK_VERSION: u32 = 1;

/// Size of wavetable used by FL.
pub const WAVETABLE_SIZE: usize = 16384;

/// intptr_t alias
#[allow(non_camel_case_types)]
type intptr_t = isize;

/// As far as we can't use trait objects to share them with C++, we need a concrete type. This type
/// wraps user's plugin as a delegate and calls its methods.
///
/// This is for internal usage only and shouldn't be used directly.
#[doc(hidden)]
#[derive(Debug)]
pub struct PluginAdapter(pub Box<dyn Plugin>);

fn plugin_info(adapter: &PluginAdapter) -> Info {
    adapter.0.info()
}

fn print_adapter(adapter: &PluginAdapter) {
    debug!("{:?}", adapter);
}

/// [`Plugin::on_message`](plugin/trait.Plugin.html#tymethod.on_message) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn plugin_dispatcher(
    adapter: *mut PluginAdapter,
    message: ffi::Message,
) -> intptr_t {
    (*adapter).0.on_message(message.into()).as_raw_ptr()
}

fn plugin_name_of(adapter: &PluginAdapter, message: ffi::Message) -> String {
    adapter.0.name_of(message.into())
}

/// [`Plugin::process_event`](plugin/trait.Plugin.html#tymethod.process_event) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn plugin_process_event(
    adapter: *mut PluginAdapter,
    event: ffi::Message,
) -> intptr_t {
    (*adapter).0.process_event(event.into());
    0
}

/// [`Plugin::process_param`](plugin/trait.Plugin.html#tymethod.process_param) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn plugin_process_param(
    adapter: *mut PluginAdapter,
    message: ffi::Message,
) -> intptr_t {
    (*adapter)
        .0
        .process_param(
            message.id as usize,
            ValuePtr(message.index),
            ProcessParamFlags::from_bits_truncate(message.value),
        )
        .as_raw_ptr()
}

/// [`Plugin::tick`](plugin/trait.Plugin.html#tymethod.tick) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn plugin_tick(adapter: *mut PluginAdapter) {
    (*adapter).0.tick();
}

/// [`Plugin::midi_tick`](plugin/trait.Plugin.html#tymethod.midi_tick) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn plugin_midi_tick(adapter: *mut PluginAdapter) {
    (*adapter).0.midi_tick();
}

/// [`Plugin::eff_render`](plugin/trait.Plugin.html#tymethod.eff_render) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn plugin_eff_render(
    adapter: *mut PluginAdapter,
    source: *const [f32; 2],
    dest: *mut [f32; 2],
    length: i32,
) {
    let input = std::slice::from_raw_parts(source, length as usize);
    let mut output = std::slice::from_raw_parts_mut(dest, length as usize);
    (*adapter).0.render(input, &mut output);
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
        let value = CString::new(self.as_str()).expect("Unexpected CString::new failure");
        // will the value still live when we return the pointer?
        value.as_ptr().to_owned() as intptr_t
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

impl fmt::Debug for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Info")
            .field("sdk_version", &self.sdk_version)
            .field("long_name", &self.long_name)
            .field("short_name", &self.short_name)
            .field("flags", &self.flags)
            .field("num_params", &self.num_params)
            .field("def_poly", &self.def_poly)
            .field("num_out_ctrls", &self.num_out_ctrls)
            .field("num_out_voices", &self.num_out_voices)
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

/// Use this to instantiate [`Info`](struct.Info.html)
#[derive(Clone, Debug)]
pub struct InfoBuilder {
    sdk_version: u32,
    long_name: String,
    short_name: String,
    flags: u32,
    num_params: u32,
    def_poly: u32,
    num_out_ctrls: u32,
    num_out_voices: u32,
}

impl InfoBuilder {
    /// Initializer for an effect.
    ///
    /// This is the most basic type.
    pub fn new_effect(long_name: &str, short_name: &str, num_params: u32) -> Self {
        Self {
            sdk_version: CURRENT_SDK_VERSION,
            long_name: long_name.to_string(),
            short_name: short_name.to_string(),
            flags: 0,
            num_params,
            def_poly: 0,
            num_out_ctrls: 0,
            num_out_voices: 0,
        }
        .new_voice_params()
    }

    /// Initializer for a full standalone generator.
    ///
    /// This is a combination of [`generator`](struct.InfoBuilder.html#method.generator) and
    /// [`note_input`](struct.InfoBuilder.html#method.get_note_input).
    pub fn new_full_gen(long_name: &str, short_name: &str, num_params: u32) -> Self {
        InfoBuilder::new_effect(long_name, short_name, num_params)
            .generator()
            .get_note_input()
    }

    /// Initializer for a hybrid generator.
    ///
    /// It's a full generator with [`use_sampler`](struct.InfoBuilder.html#method.use_sampler)
    /// option.
    pub fn new_hybrid_gen(long_name: &str, short_name: &str, num_params: u32) -> Self {
        InfoBuilder::new_full_gen(long_name, short_name, num_params).use_sampler()
    }

    /// Initializer for a purely visual plugin, that doesn't process any audio data.
    ///
    /// It's a basic plugin with [`no_process`](struct.InfoBuilder.html#method.no_process) enabled.
    pub fn new_visual(long_name: &str, short_name: &str, num_params: u32) -> Self {
        InfoBuilder::new_effect(long_name, short_name, num_params).no_process()
    }

    /// Set prefered (default) maximum polyphony.
    pub fn with_poly(mut self, poly: u32) -> Self {
        self.def_poly = poly;
        self
    }

    /// Set number of internal output controllers.
    pub fn with_out_ctrls(mut self, out_ctrls: u32) -> Self {
        self.num_out_ctrls = out_ctrls;
        self
    }

    /// Set number of internal output voices.
    pub fn with_out_voices(mut self, out_voices: u32) -> Self {
        self.num_out_voices = out_voices;
        self
    }

    /// The plugin is a generator (as opposed to an effect).
    pub fn generator(mut self) -> Self {
        self.flags |= 1;
        self
    }

    /// The generator plugin will stream into the host sampler.
    pub fn use_sampler(mut self) -> Self {
        self.flags |= 1 << 2;
        self
    }

    /// The plugin will use a sample that the user loads into the plugin's channel.
    pub fn get_chan_custom_shape(mut self) -> Self {
        self.flags |= 1 << 3;
        self
    }

    /// (not used yet) The plugin reacts to note events.
    pub fn get_note_input(mut self) -> Self {
        self.flags |= 1 << 4;
        self
    }

    /// The plugin will be notified on each tick and be able to control params (like a built-in
    /// MIDI controller).
    pub fn want_new_tick(mut self) -> Self {
        self.flags |= 1 << 5;
        self
    }

    /// The plugin won't process buffers at all
    /// ([`want_new_tick`](struct.InfoBuilder.html#method.want_new_tick), or special visual plugins
    /// (Fruity NoteBook))
    pub fn no_process(mut self) -> Self {
        self.flags |= 1 << 6;
        self
    }

    /// The plugin's editor window should be shown inside the channel properties window.
    pub fn no_window(mut self) -> Self {
        self.flags |= 1 << 10;
        self
    }

    /// (not used yet) The plugin doesn't provide its own interface, but relies on the host to
    /// create one.
    pub fn interfaceless(mut self) -> Self {
        self.flags |= 1 << 11;
        self
    }

    /// (not used yet) The plugin supports timewarps, that is can be told to change the playing
    /// position in a voice (direct from disk music tracks, ...).
    pub fn time_warp(mut self) -> Self {
        self.flags |= 1 << 13;
        self
    }

    /// The plugin will send MIDI out messages. Only plugins specifying this option will be enabled
    /// when rendering to a midi file.
    pub fn midi_out(mut self) -> Self {
        self.flags |= 1 << 14;
        self
    }

    /// The plugin is a demo version. Practically this means the host won't save its automation.
    pub fn demo_version(mut self) -> Self {
        self.flags |= 1 << 15;
        self
    }

    /// The plugin has access to the send tracks, so it can't be dropped into a send track or into
    /// the master.
    pub fn can_send(mut self) -> Self {
        self.flags |= 1 << 16;
        self
    }

    /// The plugin will send delayed messages to itself (will require the internal sync clock to be
    /// enabled).
    pub fn loop_out(mut self) -> Self {
        self.flags |= 1 << 17;
        self
    }

    /// The plugin is a hybrid generator and can release its envelope by itself. If the host's
    /// volume envelope is disabled, then the sound will keep going when the voice is stopped,
    /// until the plugin has finished its own release.
    pub fn hybrid_can_release(mut self) -> Self {
        self.flags |= 1 << 18;
        self
    }

    /// This plugin as a generator will use the sample loaded in its parent channel (see
    /// [`HostMessage::ChanSampleChanged`](
    /// ../host/enum.HostMessage.html#variant.ChanSampleChanged)).
    pub fn get_chan_sample(mut self) -> Self {
        self.flags |= 1 << 19;
        self
    }

    /// Fit to time selector will appear in channel settings window (see
    /// [`HostMessage::SetFitTime`](../host/enum.HostMessage.html#variant.SetFitTime)).
    pub fn want_fit_time(mut self) -> Self {
        self.flags |= 1 << 20;
        self
    }

    /// This must be used (for new plugins). It tells the host to use floating point values for
    /// Pitch and Pan in [`VoiceParams`](struct.VoiceParams.html).
    fn new_voice_params(mut self) -> Self {
        self.flags |= 1 << 21;
        self
    }

    /// Finish builder and init [`Info`](struct.Info.html)
    pub fn build(self) -> Info {
        Info {
            sdk_version: self.sdk_version,
            long_name: self.long_name,
            short_name: self.short_name,
            flags: self.flags,
            num_params: self.num_params,
            def_poly: self.def_poly,
            num_out_ctrls: self.num_out_ctrls,
            num_out_voices: self.num_out_voices,
        }
    }
}

/// Exposes your plugin from DLL. Accepts type name as input. The type should implement
/// [`Plugin`](plugin/trait.Plugin.html) trait.
#[macro_export]
macro_rules! create_plugin {
    ($pl:ty) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn CreatePlugInstance(
            host: *mut $crate::ffi::TFruityPlugHost,
            tag: i32,
        ) -> *mut $crate::ffi::TFruityPlug {
            let ho = $crate::host::Host { version: 0 };
            let plugin = <$pl as $crate::plugin::Plugin>::new(ho, tag);
            let adapter = $crate::PluginAdapter(Box::new(plugin));
            $crate::ffi::create_plug_instance_c(&mut *host, tag, Box::new(adapter))
        }
    };
}

#[cfg(test)]
mod tests {}
