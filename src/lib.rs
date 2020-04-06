//! The FL Plugin sdk helps you to make plugins for FL Studio. For more information about FL
//! Studio, visit the [website](www.flstudio.com).
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

    extern "C" {
        include!("wrapper.h");

        pub type TFruityPlug;
        pub type TFruityPlugHost;

        pub fn create_plug_instance_c(
            host: &'static mut TFruityPlugHost,
            tag: i32,
        ) -> &'static mut TFruityPlug;
    }

    extern "Rust" {}
}

/// Current FL SDK version
pub const CURRENT_SDK_VERSION: i32 = 1;

/// This trait should be implemented for your plugin
pub trait Plugin {
    /// Initializer
    // We can't just use Default inheritance, because we need to specify Sized marker for Self
    fn new() -> Self where Self: Sized;
    /// Get plugin [`Info`](struct.Info.html)
    fn info(&self) -> Info;
    /// Called when a new instance of the plugin is created.
    fn create_instance(&mut self, host: Host, tag: i32);
}

/// Plugin host.
#[derive(Debug)]
pub struct Host {
    version: i32,
    flags: i32,
}

/// This structure holds some information about the plugin that is used by the host. It is the same
/// for all instances of the same plugin.
///
/// Instantiate it using [`InfoBuilder`](struct.InfoBuilder.html).
#[derive(Clone, Debug)]
pub struct Info {
    /// This has to be the version of the SDK used to create the plugin. This value is available in
    /// the constant CurrentSDKVersion
    pub sdk_version: i32,
    /// The name of the plugin dll, without the extension (.dll)
    pub long_name: String,
    /// Short plugin name, to be used in labels to tell the user which plugin he is working with
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

    /// The plugin will send delayed messages to itself (will require the internal sync clock to be enabled).
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

/// Exposes your plugin from DLL
#[macro_export]
macro_rules! create_plugin {
    ($pl:ty) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn CreatePlugInstance(
            host: *mut $crate::ffi::TFruityPlugHost,
            tag: i32,
        ) -> *mut $crate::ffi::TFruityPlug {
            let pl: Box<dyn $crate::Plugin> = Box::new(<$pl>::default());
            $crate::ffi::create_plug_instance_c(&mut *host, tag)
        }
    };
}

#[cfg(test)]
mod tests {}
