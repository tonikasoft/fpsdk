//! The FL Plugin SDK helps you to make plugins for FL Studio. For more information about FL
//! Studio, visit the [website](https://www.image-line.com/flstudio/).
//!
//! Note that this SDK is not meant to make hosts for FL plugins.
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
        pub sdk_version: i32,
        /// The name of the plugin dll, without the extension (.dll)
        pub long_name: String,
        /// Short plugin name, to be used in labels to tell the user which plugin he is working
        /// with
        pub short_name: String,
        flags: i32,
        /// The number of parameters for this plugin
        pub num_params: i32,
        /// Preferred (default) maximum polyphony (FL Studio manages the polyphony) (0=infinite)
        pub def_poly: i32,
        /// Number of internal output controllers
        pub num_out_ctrls: i32,
        /// Number of internal output voices
        pub num_out_voices: i32,
    }

    pub struct TimeSignature {
        pub steps_per_bar: i32,
        pub steps_per_beat: i32,
        pub ppq: i32,
    }

    pub struct MidiMessage {
        pub status: u8,
        pub data1: u8,
        pub data2: u8,
        pub port: u8,
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
    }

    extern "Rust" {
        type PluginAdapter;

        fn plugin_info(adapter: &PluginAdapter) -> Info;
    }
}

use std::ffi::c_void;
use std::panic::RefUnwindSafe;

use bitflags::bitflags;

pub use ffi::{Info, MidiMessage, TimeSignature};

/// Current FL SDK version.
pub const CURRENT_SDK_VERSION: i32 = 1;

/// As far as we can't use trait objects to share them with C++, we need a concrete type. This type
/// wraps user's plugin as a delegate and calls its methods.
///
/// This is for internal usage only and shouldn't be used directly.
#[doc(hidden)]
pub struct PluginAdapter(pub Box<dyn Plugin>);

fn plugin_info(adapter: &PluginAdapter) -> Info {
    adapter.0.info()
}

/// This trait must be implemented for your plugin.
pub trait Plugin: RefUnwindSafe {
    /// Initializer
    fn new(host: Host, tag: i32) -> Self
    where
        Self: Sized;
    /// Get plugin [`Info`](struct.Info.html)
    fn info(&self) -> Info;
    /// The host calls this function to request something that isn't done in a specialized
    /// function.
    ///
    /// See [`HostMessage`](enum.HostMessage.html) for possible messages.
    fn on_message(&mut self, message: HostMessage<'_>) -> Box<dyn DispatcherResult>;
}

/// Plugin host.
#[derive(Debug)]
pub struct Host {
    /// The version of FL Studio. It is stored in one integer. If the version of FL Studio would be
    /// 1.2.3 for example, `version` would be 1002003
    pub version: i32,
}

/// Message from the host to the plugin
pub enum HostMessage<'a> {
    /// Contains the handle of the parent window if the editor has to be shown.
    ShowEditor(Option<*mut c_void>),
    /// Change the processing mode flags. This can be ignored.
    ///
    /// The value is [ProcessModeFlags](struct.ProcessModeFlags.html).
    ProcessMode(ProcessModeFlags),
    /// The continuity of processing is broken. This means that the user has jumped ahead or back
    /// in the playlist, for example. When this happens, the plugin needs to clear all buffers and
    /// start like new
    ///
    /// **Warning: this can be called from the mixer thread!**
    Flush,
    /// This changes the maximum processing length, expressed in samples.
    ///
    /// The value is the new length.
    SetBlockSize(u32),
    /// This changes the sample rate.
    ///
    /// Value holds the new sample rate
    SetSampleRate(u32),
    /// This allows the plugin to define how the editor window should be resized.
    ///
    /// The first value will hold a pointer to a rectangle for the minimum (Left and Top) and
    /// maximum (Right and Bottom) width and height of the window
    ///
    /// The second value holds a pointer to a point structure that defines by how much the window
    /// size should change horizontally and vertically when the user drags the border.
    WindowMinMax(*mut c_void, *mut c_void),
    /// (not used yet) The host has noticed that too much processing power is used and asks the
    /// plugin to kill its weakest voice.
    ///
    /// The plugin has to return `true` if it did anything, `false` otherwise
    KillVoice,
    /// Only full generators have to respond to this message. It's meant to allow the cutoff and
    /// resonance parameters of a voice to be used for other purposes, if the generator doesn't use
    /// them as cutoff and resonance.
    ///
    /// - return `0u8` if the plugin doesn't support the default per-voice level value
    /// - return `1u8` if the plugin supports the default per-voice level value (filter cutoff (0) or
    ///   filter resonance (1))
    /// - return `2u8` if the plugin supports the per-voice level value, but for another function
    ///   (then check FPN_VoiceLevel to provide your own names)
    UseVoiceLevels(u8),
    /// Called when the user selects a preset.
    ///
    /// The value tells you which one to set.
    SetPreset(u32),
    /// A sample has been loaded into the parent channel. This is given to the plugin as a
    /// wavetable, in the same format as the WaveTables member of TFruityPlugin. Also see
    /// FPF_GetChanCustomShape.
    ///
    /// The value holds the new shape.
    ChanSampleChanged(&'a [f32]),
    /// The host has enabled/disabled the plugin.
    ///
    /// The value will contain the new state (`false` for disabled, `true` for enabled)
    ///
    /// **Warning: this can be called from the mixer thread!**
    SetEnabled(bool),
    /// The host is playing (song pos info is valid when playing) or stopped (state in the value)
    ///
    /// **Warning: can be called from the mixing thread**
    SetPlaying(bool, u32),
    /// The song position has jumped from one position to another non-consecutive position
    ///
    /// **Warning: can be called from the mixing thread**
    SongPosChanged,
    /// The time signature has changed.
    ///
    /// The value is [`TimeSignature`](struct.TimeSignature.html).
    SetTimeSig(TimeSignature),
    /// This is called to let the plugin tell the host which files need to be collected or put in
    /// zip files. The name of the file is passed to the host as a `String` in the result of the
    /// dispatcher function.
    ///
    /// The value holds the file #, which starts at 0
    CollectFile(u32),
    /// (private message to known plugins, ignore) tells the plugin to update a specific,
    /// non-automated param
    SetInternalParam,
    /// This tells the plugin how many send tracks there are (fixed to 4, but could be set by the
    /// user at any time in a future update)
    ///
    /// The value holds the number of send tracks
    SetNumSends(u32),
    /// Called when a file has been dropped onto the parent channel's button.
    ///
    /// The value holds filename.
    LoadFile(String),
    /// Set fit to time in beats
    ///
    /// The value holds the time.
    SetFitTime(f32),
    /// Sets the number of samples in each tick. This value changes when the tempo, ppq or sample
    /// rate have changed.
    ///
    /// **Warning: can be called from the mixing thread**
    SetSamplesPerTick(u32),
    /// Sets the frequency at which Idle is called.
    ///
    /// The value holds the new time (milliseconds)
    SetIdleTime(u32),
    /// (FL 7.0) The host has focused/unfocused the editor (focused in the value) (plugin can use
    /// this to steal keyboard focus)
    SetFocus(bool),
    /// (FL 8.0) This is sent by the host for special transport messages, from a controller.
    ///
    /// The value is the type of message (see [Transport](enum.Transport.html))
    ///
    /// Result should be `true` if handled, `false` otherwise
    Transport(Transport),
    /// (FL 8.0) Live MIDI input preview. This allows the plugin to steal messages (mostly for
    /// transport purposes).
    ///
    /// The value has the packed MIDI message. Only note on/off for now.
    ///
    /// Result should be `true` if handled, `false` otherwise
    MidiIn(MidiMessage),
    /// Mixer routing changed, must use
    /// [`PluginMessage::GetInOuts`](enum.PluginMessage.html#variant.GetInOuts) if necessary
    RoutingChanged,
    /// Retrieves info about a parameter.
    ///
    /// The value is the parameter number.
    ///
    /// see [ParameterFlags](struct.ParameterFlags.html) for the result
    GetParamInfo(usize),
    /// Called after a project has been loaded, to leave a chance to kill automation (that could be
    /// loaded after the plugin is created) if necessary.
    ProjLoaded,
    /// (private message to the plugin wrapper) Load a (VST, DX) plugin state,
    ///
    /// WrapperLoadState,
    ShowSettings,
    /// Input (the first value)/output (the second value) latency of the output, in samples (only
    /// for information)
    SetIoLatency(u32, u32),
    /// (message from Patcher) retrieves the preferred number of audio inputs (the value is `0`),
    /// audio outputs (the value is `1`) or voice outputs (the value is `2`)
    ///
    /// Result has to be:
    ///
    /// * `0i32` - default number
    /// * `-1i32` - none
    PreferredNumIo(u8),
}

bitflags! {
    /// Parameter flags.
    pub struct ParameterFlags: i32 {
        /// Makes no sense to interpolate parameter values (when values are not levels).
        const CANT_INTERPOLATE = 1;
        /// Parameter is a normalized (0..1) single float. (Integer otherwise)
        const FLOAT = 2;
        /// Parameter appears centered in event editors.
        const CENTERED = 4;
    }
}

bitflags! {
    /// Processing mode flags.
    pub struct ProcessModeFlags: i32 {
        /// Realtime rendering.
        const NORMAL = 0;
        /// Realtime rendering with a higher quality.
        const HQ_REALTIME = 1;
        /// Non realtime processing (CPU does not matter, quality does) (normally set when
        /// rendering only).
        const HQ_NON_REALTIME = 2;
        /// FL is rendering to file if this flag is set.
        const IS_RENDERING = 16;
        /// (changed in FL 7.0) 3 bits value for interpolation quality 
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

/// Dispatcher result marker
pub trait DispatcherResult {}

impl DispatcherResult for String {}
impl DispatcherResult for bool {}
impl DispatcherResult for i32 {}
impl DispatcherResult for u8 {}
impl DispatcherResult for ParameterFlags {}

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
}

/// `0` for release, `1` for switch (if release is not supported), `2` for hold (if release should
/// be expected).
pub struct Button(pub u8);
/// `false` for release, `true` for hold.
pub struct Hold(pub bool);
/// Value is an integer increment.
pub struct Jog(pub i32);

/// Use this to instantiate [`Info`](struct.Info.html)
#[derive(Clone, Debug)]
pub struct InfoBuilder {
    sdk_version: i32,
    long_name: String,
    short_name: String,
    flags: i32,
    num_params: i32,
    def_poly: i32,
    num_out_ctrls: i32,
    num_out_voices: i32,
}

impl InfoBuilder {
    /// Initializer for an effect.
    ///
    /// This is the most basic type.
    pub fn new_effect(long_name: &str, short_name: &str, num_params: i32) -> Self {
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
    pub fn new_full_gen(long_name: &str, short_name: &str, num_params: i32) -> Self {
        InfoBuilder::new_effect(long_name, short_name, num_params)
            .generator()
            .get_note_input()
    }

    /// Initializer for a hybrid generator.
    ///
    /// It's a full generator with [`use_sampler`](struct.InfoBuilder.html#method.use_sampler)
    /// option.
    pub fn new_hybrid_gen(long_name: &str, short_name: &str, num_params: i32) -> Self {
        InfoBuilder::new_full_gen(long_name, short_name, num_params).use_sampler()
    }

    /// Initializer for a purely visual plugin, that doesn't process any audio data.
    ///
    /// It's a basic plugin with [`no_process`](struct.InfoBuilder.html#method.no_process) enabled.
    pub fn new_visual(long_name: &str, short_name: &str, num_params: i32) -> Self {
        InfoBuilder::new_effect(long_name, short_name, num_params).no_process()
    }

    /// Set prefered (default) maximum polyphony.
    pub fn with_poly(mut self, poly: i32) -> Self {
        self.def_poly = poly;
        self
    }

    /// Set number of internal output controllers.
    pub fn with_out_ctrls(mut self, out_ctrls: i32) -> Self {
        self.num_out_ctrls = out_ctrls;
        self
    }

    /// Set number of internal output voices.
    pub fn with_out_voices(mut self, out_voices: i32) -> Self {
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
    pub fn msg_out(mut self) -> Self {
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
    /// [`PluginDispatcherId::ChanSampleChanged`](enum.PluginDispatcherId.html)).
    pub fn get_chan_sample(mut self) -> Self {
        self.flags |= 1 << 19;
        self
    }

    /// Fit to time selector will appear in channel settings window (see
    /// [`PluginDispatcherId::SetFitTime`](enum.PluginDispatcherId.html)).
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
/// [`Plugin`](trait.Plugin.html) trait.
#[macro_export]
macro_rules! create_plugin {
    ($pl:ty) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn CreatePlugInstance(
            host: *mut $crate::ffi::TFruityPlugHost,
            tag: i32,
        ) -> *mut $crate::ffi::TFruityPlug {
            let ho = $crate::Host { version: 0 };
            let plugin = <$pl as $crate::Plugin>::new(ho, tag);
            let adapter = $crate::PluginAdapter(Box::new(plugin));
            $crate::ffi::create_plug_instance_c(&mut *host, tag, Box::new(adapter))
        }
    };
}

#[cfg(test)]
mod tests {}
