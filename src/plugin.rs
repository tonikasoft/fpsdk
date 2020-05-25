//! Plugin related stuff.

pub mod message;

use std::ffi::CString;
use std::io::{self, Read, Write};
use std::os::raw::{c_char, c_int, c_void};
use std::panic::RefUnwindSafe;

use hresult::HRESULT;
use log::{debug, error};

use crate::host::{self, Event, GetName, Host};
use crate::voice::ReceiveVoiceHandler;
use crate::{
    alloc_real_cstr, intptr_t, AsRawPtr, FlMessage, MidiMessage, ProcessParamFlags, ValuePtr,
    CURRENT_SDK_VERSION,
};

crate::implement_tag!();

/// Exposes your plugin from DLL. Accepts type name as input. The type should implement
/// [`Plugin`](plugin/trait.Plugin.html) trait.
#[macro_export]
macro_rules! create_plugin {
    ($pl:ty) => {
        use std::os::raw::c_void;

        extern "C" {
            fn create_plug_instance_c(
                host: *mut c_void,
                tag: $crate::intptr_t,
                adapter: *mut c_void,
            ) -> *mut c_void;
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn CreatePlugInstance(
            host: *mut c_void,
            tag: $crate::intptr_t,
        ) -> *mut c_void {
            let ho = $crate::host::Host::new(host);
            let plugin = <$pl as $crate::plugin::Plugin>::new(
                ho,
                $crate::plugin::Tag(tag as $crate::intptr_t),
            );
            let adapter = $crate::plugin::PluginAdapter(Box::new(plugin));
            create_plug_instance_c(host, tag, Box::into_raw(Box::new(adapter)) as *mut c_void)
        }
    };
}

/// This trait must be implemented for your plugin.
pub trait Plugin: std::fmt::Debug + RefUnwindSafe + Send + Sync + 'static {
    /// Initializer.
    fn new(host: Host, tag: Tag) -> Self
    where
        Self: Sized;
    /// Get plugin [`Info`](struct.Info.html).
    fn info(&self) -> Info;
    /// Save plugin's state.
    fn save_state(&mut self, writer: StateWriter);
    /// Load plugin's state.
    fn load_state(&mut self, reader: StateReader);
    /// The host calls this function to request something that isn't done in a specialized
    /// function.
    ///
    /// See [`host::Message`](../host/enum.Message.html) for possible messages.
    ///
    /// Can be called from GUI or mixer threads.
    fn on_message(&mut self, message: host::Message<'_>) -> Box<dyn AsRawPtr>;
    /// This is called when the host wants to know a text representation of some value.
    ///
    /// Can be called from GUI or mixer threads.
    fn name_of(&self, value: GetName) -> String;
    /// Process an event sent by the host.
    ///
    /// Can be called from GUI or mixer threads.
    fn process_event(&mut self, _event: Event) {}
    /// Something has to be done concerning a parameter. What exactly has to be done is explained
    /// by the `flags` parameter (see [`ProcessParamFlags`](../struct.ProcessParamFlags.html)).
    ///
    /// - `index` - the index of the parameter.
    /// - `value` - the (new) value of the parameter.
    /// - `flags` - describes what needs to be done to the parameter. It can be a combination of
    ///   several flags.
    ///
    /// If
    /// [`ProcessParamFlags::GET_VALUE`](
    /// ../struct.ProcessParamFlags.html#associatedconstant.GET_VALUE) is specified in `flags`, the
    /// result has to be the value of the parameter.
    ///
    /// Can be called from GUI or mixer threads.
    fn process_param(
        &mut self,
        _index: usize,
        _value: ValuePtr,
        _flags: ProcessParamFlags,
    ) -> Box<dyn AsRawPtr> {
        Box::new(0)
    }
    /// This function is called continuously. It allows the plugin to perform certain tasks that
    /// are not time-critical and which do not take up a lot of time either. For example, in this
    /// function you may show a hint message when the mouse moves over a control in the editor.
    ///
    /// Called from GUI thread.
    fn idle(&mut self) {}
    /// Gets called before a new tick is mixed (not played), if the plugin added
    /// [`InfoBuilder::want_new_tick`](../plugin/struct.InfoBuilder.html#method.want_new_tick) into
    /// [`Info`](../struct.Info.html).
    ///
    /// Internal controller plugins should call
    /// [`host::Host::on_controller`](../host/struct.Host.html#method.on_controller) from
    /// here.
    ///
    /// Called from mixer thread.
    fn tick(&mut self) {}
    /// This is called before a new midi tick is played (not mixed).
    ///
    /// Can be called from GUI or mixer threads.
    fn midi_tick(&mut self) {}
    /// The processing function. The input buffer is empty for generator plugins.
    ///
    /// The buffers are in interlaced 32Bit float stereo format.
    ///
    /// Called from mixer thread.
    fn render(&mut self, _input: &[[f32; 2]], _output: &mut [[f32; 2]]) {}
    /// Get [`ReceiveVoiceHandler`](../voice/trait.ReceiveVoiceHandler.html).
    ///
    /// Implement this method if you make a generator plugin.
    fn voice_handler(&mut self) -> Option<&mut dyn ReceiveVoiceHandler> {
        None
    }
    /// The host will call this when there's new MIDI data available. This function is only called
    /// when the plugin has called the
    /// [`host::Host::on_message`](../host/struct.Host.html#method.on_message) with
    /// [`plugin::message::WantMidiInput`](../plugin/message/struct.WantMidiInput.html) and
    /// value set to `true`.
    ///
    /// Can be called from GUI or mixer threads.
    fn midi_in(&mut self, _message: MidiMessage) {}
    /// **MAY NOT WORK**
    ///
    /// This gets called with a new buffered message to the plugin itself.
    fn loop_in(&mut self, _message: ValuePtr) {}
}

/// This structure holds some information about the plugin that is used by the host. It is the
/// same for all instances of the same plugin.
///
/// It's not supposed to be used directly, instantiate it using
/// [`InfoBuilder`](struct.InfoBuilder.html).
#[repr(C)]
#[derive(Debug)]
pub struct Info {
    /// This has to be the version of the SDK used to create the plugin. This value is
    /// available in the constant CurrentSDKVersion.
    pub sdk_version: u32,
    /// The name of the plugin dll, without the extension (.dll).
    pub long_name: *mut c_char,
    /// Short plugin name, to be used in labels to tell the user which plugin he is working
    /// with.
    pub short_name: *mut c_char,
    flags: u32,
    /// The number of parameters for this plugin.
    pub num_params: u32,
    /// Preferred (default) maximum polyphony (FL Studio manages the polyphony) (0=infinite).
    pub def_poly: u32,
    /// Number of internal output controllers.
    pub num_out_ctrls: u32,
    /// Number of internal output voices.
    pub num_out_voices: u32,
}

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

    /// The plugin will use a sample that the user loads into the plugin's channel.
    pub fn get_chan_custom_shape(mut self) -> Self {
        self.flags |= 1 << 3;
        self
    }

    /// The plugin reacts to note events.
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

    /// This plugin as a generator will use the sample loaded in its parent channel (see
    /// [`host::Message::ChanSampleChanged`](
    /// ../host/enum.Message.html#variant.ChanSampleChanged)).
    pub fn get_chan_sample(mut self) -> Self {
        self.flags |= 1 << 19;
        self
    }

    /// Fit to time selector will appear in channel settings window (see
    /// [`host::Message::SetFitTime`](../host/enum.Message.html#variant.SetFitTime)).
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

    /// Plugin can't be smart disabled.
    pub fn cant_smart_disable(mut self) -> Self {
        self.flags |= 1 << 23;
        self
    }

    /// Plugin wants a settings button on the titlebar (mainly for the wrapper).
    pub fn want_settings_button(mut self) -> Self {
        self.flags |= 1 << 24;
        self
    }

    /// Finish builder and init [`Info`](struct.Info.html)
    pub fn build(self) -> Info {
        let log_err = |e| {
            error!("{}", e);
            panic!();
        };
        let long_name = CString::new(self.long_name)
            .unwrap_or_else(log_err)
            .into_raw();
        let short_name = CString::new(self.short_name)
            .unwrap_or_else(log_err)
            .into_raw();

        Info {
            sdk_version: self.sdk_version,
            long_name: unsafe { alloc_real_cstr(long_name) },
            short_name: unsafe { alloc_real_cstr(short_name) },
            flags: self.flags,
            num_params: self.num_params,
            def_poly: self.def_poly,
            num_out_ctrls: self.num_out_ctrls,
            num_out_voices: self.num_out_voices,
        }
    }
}

/// State reader.
pub struct StateReader(pub(crate) *mut c_void);

impl Read for StateReader {
    /// Read state into buffer.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read = 0u32;
        let buf_ptr = buf.as_mut_ptr();
        let res = unsafe { istream_read(self.0, buf_ptr, buf.len() as u32, &mut read) };
        debug!("StateReader read {} bytes", read);
        check_hresult(
            HRESULT::from(res),
            read as usize,
            "Error reading from IStream",
        )
    }
}

extern "C" {
    fn istream_read(istream: *mut c_void, data: *mut u8, size: u32, read: *mut u32) -> i32;
}

/// State writer.
pub struct StateWriter(pub(crate) *mut c_void);

impl Write for StateWriter {
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Write state from buffer.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut write = 0u32;
        let buf_ptr = buf.as_ptr();
        let res = unsafe { istream_write(self.0, buf_ptr, buf.len() as u32, &mut write) };
        check_hresult(
            HRESULT::from(res),
            write as usize,
            "Error writing to IStream",
        )
    }
}

extern "C" {
    fn istream_write(istream: *mut c_void, data: *const u8, size: u32, write: *mut u32) -> i32;
}

fn check_hresult(result: HRESULT, read: usize, error_msg: &str) -> io::Result<usize> {
    if !result.is_success() {
        return Err(io::Error::new(io::ErrorKind::Other, error_msg));
    }

    Ok(read)
}

/// Type wraps `Plugin` trait object to simplify sharing with C/C++.
///
/// This is for internal usage only and shouldn't be used directly.
#[doc(hidden)]
#[derive(Debug)]
pub struct PluginAdapter(pub Box<dyn Plugin>);

/// [`Plugin::info`](trait.Plugin.html#tymethod.info) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_info(adapter: *mut PluginAdapter) -> *mut Info {
    Box::into_raw(Box::new((*adapter).0.info()))
}

/// [`Plugin::on_message`](trait.Plugin.html#tymethod.on_message) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_dispatcher(
    adapter: *mut PluginAdapter,
    message: FlMessage,
) -> intptr_t {
    (*adapter).0.on_message(message.into()).as_raw_ptr()
}

/// [`Plugin::name_of`](trait.Plugin.html#tymethod.name_of) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_name_of(
    adapter: *const PluginAdapter,
    message: FlMessage,
) -> *mut c_char {
    let name = CString::new((*adapter).0.name_of(message.into())).unwrap_or_else(|e| {
        error!("{}", e);
        panic!();
    });
    name.into_raw()
}

/// [`Plugin::process_event`](trait.Plugin.html#tymethod.process_event) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_process_event(adapter: *mut PluginAdapter, event: FlMessage) -> c_int {
    (*adapter).0.process_event(event.into());
    0
}

/// [`Plugin::process_param`](trait.Plugin.html#tymethod.process_param) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_process_param(
    adapter: *mut PluginAdapter,
    message: FlMessage,
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

/// [`Plugin::idle`](trait.Plugin.html#method.idle) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_idle(adapter: *mut PluginAdapter) {
    (*adapter).0.idle();
}

/// [`Plugin::tick`](trait.Plugin.html#tymethod.tick) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_tick(adapter: *mut PluginAdapter) {
    (*adapter).0.tick();
}

/// [`Plugin::midi_tick`](trait.Plugin.html#tymethod.midi_tick) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_midi_tick(adapter: *mut PluginAdapter) {
    (*adapter).0.midi_tick();
}

/// [`Plugin::render`](trait.Plugin.html#tymethod.render) FFI for effects.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_eff_render(
    adapter: *mut PluginAdapter,
    source: *const [f32; 2],
    dest: *mut [f32; 2],
    length: i32,
) {
    let input = std::slice::from_raw_parts(source, length as usize);
    let mut output = std::slice::from_raw_parts_mut(dest, length as usize);
    (*adapter).0.render(input, &mut output);
}

/// [`Plugin::render`](trait.Plugin.html#tymethod.render) FFI for generators.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_gen_render(
    adapter: *mut PluginAdapter,
    dest: *mut [f32; 2],
    length: i32,
) {
    let mut output = std::slice::from_raw_parts_mut(dest, length as usize);
    (*adapter).0.render(&[[0.0, 0.0]], &mut output);
}

/// [`Plugin::midi_in`](trait.Plugin.html#tymethod.midi_in) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_midi_in(adapter: *mut PluginAdapter, message: &mut c_int) {
    (*adapter).0.midi_in(message.into());
}

/// [`Plugin::save_state`](trait.Plugin.html#tymethod.save_state) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_save_state(adapter: *mut PluginAdapter, stream: *mut c_void) {
    (*adapter).0.save_state(StateWriter(stream));
}

/// [`Plugin::load_state`](trait.Plugin.html#tymethod.load_state) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_load_state(adapter: *mut PluginAdapter, stream: *mut c_void) {
    (*adapter).0.load_state(StateReader(stream));
}

/// [`Plugin::loop_in`](Plugin.html#method.loop_in) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn plugin_loop_in(adapter: *mut PluginAdapter, message: intptr_t) {
    (*adapter).0.loop_in(ValuePtr(message));
}
