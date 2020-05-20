//! Plugin's host (FL Studio).
use std::collections::HashMap;
use std::ffi::c_void;
use std::os::raw::{c_char, c_int, c_uchar};
use std::sync::atomic::AtomicPtr;
use std::sync::{Arc, Mutex};

use log::trace;

use crate::plugin::{self, message};
use crate::voice::{self, SendVoiceHandler, Voice};
use crate::{
    ffi, intptr_t, AsRawPtr, FromRawPtr, MidiMessage, ProcessModeFlags, TimeSignature, Transport,
    ValuePtr, WAVETABLE_SIZE,
};

/// [`Host::in_buf`](struct.Host.html#method.in_buf) flag, which is added before adding to the
/// buffer.
pub const IO_LOCK: i32 = 0;

/// [`Host::in_buf`](struct.Host.html#method.in_buf) flag, which is added after adding to the
/// buffer.
pub const IO_UNLOCK: i32 = 1;

/// [`Host::out_buf`](struct.Host.html#method.out_buf) flag, which tells if the buffer is filled.
pub const IO_FILLED: i32 = 1;

/// Plugin host.
#[derive(Debug)]
pub struct Host {
    voicer: Arc<Mutex<Voicer>>,
    out_voicer: Arc<Mutex<OutVoicer>>,
    pub(crate) host_ptr: AtomicPtr<c_void>,
}

impl Host {
    /// Initializer.
    pub fn new(host_ptr: *mut c_void) -> Self {
        let voicer = Arc::new(Mutex::new(Voicer::new(AtomicPtr::new(host_ptr))));
        let out_voicer = Arc::new(Mutex::new(OutVoicer::new(AtomicPtr::new(host_ptr))));
        Self {
            voicer,
            out_voicer,
            host_ptr: AtomicPtr::new(host_ptr),
        }
    }

    /// Get the version of FL Studio. It is stored in one integer. If the version of FL Studio
    /// would be 1.2.3 for example, `version` would be 1002003
    pub fn version(&self) -> i32 {
        todo!()
    }

    /// Send message to host.
    ///
    /// See [`plugin::message`](../plugin/message/index.html).
    pub fn on_message<T: message::Message>(&mut self, tag: plugin::Tag, message: T) -> T::Return {
        message.send(tag, self)
    }

    /// Notify the host that a parameter value has changed.
    ///
    /// In order to make your parameters recordable in FL Studio, you have to call this function
    /// whenever a parameter is changed from within your plugin (probably because the user turned a
    /// wheel or something).
    ///
    /// - tag - plugin's tag.
    /// - index - the parameter index
    /// - value - the new parameter value.
    pub fn on_parameter(&mut self, tag: plugin::Tag, index: usize, value: ValuePtr) {
        unsafe {
            host_on_parameter(
                *self.host_ptr.get_mut(),
                tag.0,
                index as c_int,
                value.0 as c_int,
            )
        };
    }

    /// Let the host show a hint, as specified by the parameters.
    ///
    /// - tag - the plugin's tag
    /// - text - the text to show as a hint
    ///
    /// There is one extra feature of parameter hints. It is possible to tell FL Studio to show
    /// little icons next to the hint, that have a special meaning. For the moment there are three
    /// of those. Note that these have to be inserted at the BEGINNING of the string.
    ///
    /// - "^a" - shows a little icon that informs the user that the parameter that the hint is
    /// about can be linked to a MIDI controller.
    /// - "^b" - informs the user that the parameter is recordable.
    /// - "^c" - shows a little smiley. No real use, just for fun.
    /// - "^d" - shows a mouse with the right button clicked, to denote a control that has a popup
    /// menu.
    /// - "^e" - shows an unhappy smiley, to use when something went wrong.
    /// - "^f" - shows a left-pointing arrow
    /// - "^g" - shows a double right-pointing arrow, for fast forward
    /// - "^h" - is an exclamation mark, for a warning to the user
    /// - "^i" - is an hourglass
    /// - "^j" - shows a double left-pointing arrow, for fast reverse
    pub fn on_hint(&mut self, tag: plugin::Tag, text: String) {
        unsafe {
            host_on_hint(
                *self.host_ptr.get_mut(),
                tag.0,
                text.as_raw_ptr() as *mut c_char,
            )
        };
    }

    /// Send a MIDI out message immediately.
    ///
    /// To be able to use this method, you should enable MIDI out for the plugin (see
    /// [`InfoBuilder::midi_out`](../plugin/struct.InfoBuilder.html#method.midi_out)) and send
    /// [`plugin::message::ActivateMidi`](../plugin/message/struct.ActivateMidi.html) to the host.
    pub fn midi_out(&mut self, tag: plugin::Tag, message: MidiMessage) {
        // We could use MidiMessage directly with Box::into_raw, but we can't because of the Rust
        // memory layer. We couldn't free the allocated memory properly, because it's managed by
        // the host. So we just send parameters and instantiate FL's TMIDIOutMsg on the C side.
        //
        // The same is inside midi_out_del.
        unsafe {
            host_midi_out(
                *self.host_ptr.get_mut(),
                tag.0,
                message.status,
                message.data1,
                message.data2,
                message.port,
            )
        };
    }

    /// Send a delayed MIDI out message. This message will actually be sent when the MIDI tick has
    /// reached the current mixer tick.
    ///
    /// To be able to use this method, you should enable MIDI out for the plugin (see
    /// [`InfoBuilder::midi_out`](../plugin/struct.InfoBuilder.html#method.midi_out)).
    pub fn midi_out_del(&mut self, tag: plugin::Tag, message: MidiMessage) {
        unsafe {
            host_midi_out_del(
                *self.host_ptr.get_mut(),
                tag.0,
                message.status,
                message.data1,
                message.data2,
                message.port,
            )
        };
    }

    /// **MAY NOT WORK**
    /// 
    /// Ask for a message to be dispatched to itself when the current mixing tick will be played
    /// (to synchronize stuff) 
    ///
    /// (see [`Plugin::loop_in`](../plugin/trait.Plugin.html#method.loop_in)). 
    ///
    /// The message is guaranteed to be dispatched, however it could be sent immediately if it
    /// couldn't be buffered (it's only buffered when playing).
    pub fn loop_out(&mut self, tag: plugin::Tag, message: ValuePtr) {
        unsafe { host_loop_out(*self.host_ptr.get_mut(), tag.0, message.0) };
    }

    /// Remove the buffered message scheduled by
    /// [`Host::loop_out`](struct.Host.html#method.loop_out), so that it will never be dispatched.
    pub fn loop_kill(&mut self, tag: plugin::Tag, message: ValuePtr) {
        unsafe { host_loop_kill(*self.host_ptr.get_mut(), tag.0, message.0) };
    }

    /// This is a function for thread synchronization. When this is called, no more voices shall be
    /// created and there will be no more rendering until
    /// [`Host::unlock_mix`](struct.Host.html#method.unlock_mix) has been called.
    pub fn lock_mix(&mut self) {
        unsafe { host_lock_mix(*self.host_ptr.get_mut()) };
    }

    /// Unlocks the mix thread if it was previously locked with.
    /// [`Host::lock_mix`](struct.Host.html#method.lock_mix)
    pub fn unlock_mix(&mut self) {
        unsafe { host_unlock_mix(*self.host_ptr.get_mut()) };
    } 

    /// **Warning: this function is not very performant, so avoid using it if possible.**
    /// 
    /// This is an alternative to [`Host::lock_mix`](struct.Host.html#method.lock_mix). 
    /// It won't freeze the audio. This function can only be called from the GUI thread! 
    pub fn lock_plugin(&mut self, tag: plugin::Tag) {
        unsafe { host_lock_plugin(*self.host_ptr.get_mut(), tag.0) };
    } 

    /// **Warning: this function is not very performant, so avoid using it if possible.**
    /// 
    /// Unlocks the mix thread if it was previously locked with
    /// [`Host::lock_plugin`](struct.Host.html#method.lock_plugin). 
    pub fn unlock_plugin(&mut self, tag: plugin::Tag) {
        unsafe { host_unlock_plugin(*self.host_ptr.get_mut(), tag.0) };
    }

    /// Get [`Voicer`](struct.Voicer.html)
    pub fn voice_handler(&self) -> Arc<Mutex<Voicer>> {
        Arc::clone(&self.voicer)
    }

    /// Get [`OutVoicer`](struct.OutVoicer.html).
    pub fn out_voice_handler(&self) -> Arc<Mutex<OutVoicer>> {
        Arc::clone(&self.out_voicer)
    }
}

extern "C" {
    fn host_on_parameter(host: *mut c_void, tag: intptr_t, index: c_int, value: c_int);
    fn host_on_hint(host: *mut c_void, tag: intptr_t, text: *mut c_char);
    fn host_midi_out(
        host: *mut c_void,
        tag: intptr_t,
        status: c_uchar,
        data1: c_uchar,
        data2: c_uchar,
        port: c_uchar,
    );
    fn host_midi_out_del(
        host: *mut c_void,
        tag: intptr_t,
        status: c_uchar,
        data1: c_uchar,
        data2: c_uchar,
        port: c_uchar,
    );
    fn host_loop_out(host: *mut c_void, tag: intptr_t, message: intptr_t);
    fn host_loop_kill(host: *mut c_void, tag: intptr_t, message: intptr_t);
    fn host_lock_mix(host: *mut c_void);
    fn host_unlock_mix(host: *mut c_void);
    fn host_lock_plugin(host: *mut c_void, tag: intptr_t);
    fn host_unlock_plugin(host: *mut c_void, tag: intptr_t);
}

/// Use this to manually release, kill and notify voices about events.
#[derive(Debug)]
pub struct Voicer {
    host_ptr: AtomicPtr<c_void>,
}

impl Voicer {
    fn new(host_ptr: AtomicPtr<c_void>) -> Self {
        Self { host_ptr }
    }
}

impl SendVoiceHandler for Voicer {
    /// Tell the host the specified voice should be silent (Note Off).
    fn release(&mut self, tag: voice::Tag) {
        trace!("manully release voice {}", tag);
        unsafe { host_release_voice(*self.host_ptr.get_mut(), tag.0) };
    }

    /// Tell the host that the specified voice can be killed (freed from memory).
    ///
    /// This method forces FL Studio to ask the plugin to destroy its voice.
    fn kill(&mut self, tag: voice::Tag) {
        trace!("manully kill voice {}", tag);
        unsafe { host_kill_voice(*self.host_ptr.get_mut(), tag.0) };
    }

    /// Tell the host that some event has happened concerning the specified voice.
    fn on_event(&mut self, tag: voice::Tag, event: voice::Event) -> Option<ValuePtr> {
        Option::<ffi::Message>::from(event).map(|value| {
            ValuePtr(unsafe { host_on_voice_event(*self.host_ptr.get_mut(), tag.0, value) })
        })
    }
}

extern "C" {
    fn host_release_voice(host: *mut c_void, tag: intptr_t);
    fn host_kill_voice(host: *mut c_void, tag: intptr_t);
    fn host_on_voice_event(host: *mut c_void, tag: intptr_t, message: ffi::Message) -> intptr_t;
}

/// Use this for operations with output voices (i.e. for VFX inside [patcher](
/// https://www.image-line.com/support/flstudio_online_manual/html/plugins/Patcher.htm)).
#[derive(Debug)]
pub struct OutVoicer {
    voices: HashMap<voice::Tag, OutVoice>,
    host_ptr: AtomicPtr<c_void>,
}

impl OutVoicer {
    fn new(host_ptr: AtomicPtr<c_void>) -> Self {
        Self {
            voices: HashMap::new(),
            host_ptr,
        }
    }
}

impl SendVoiceHandler for OutVoicer {
    /// It returns `None` if the output has no destination.
    fn trigger(
        &mut self,
        params: voice::Params,
        index: usize,
        tag: voice::Tag,
    ) -> Option<&mut dyn Voice> {
        let params_ptr = Box::into_raw(Box::new(params));
        let inner_tag = unsafe {
            host_trig_out_voice(*self.host_ptr.get_mut(), params_ptr, index as i32, tag.0)
        };

        if inner_tag == -1 {
            // if FVH_Null
            unsafe { Box::from_raw(params_ptr) }; // free the memory
            trace!("send trigger voice is null");
            return None;
        }

        let voice = OutVoice::new(tag, AtomicPtr::new(params_ptr), voice::Tag(inner_tag));
        trace!("send trigger output voice {:?}", voice);
        self.voices.insert(tag, voice);
        Some(self.voices.get_mut(&tag).unwrap())
    }

    fn release(&mut self, tag: voice::Tag) {
        if let Some(voice) = self.voices.get_mut(&tag) {
            trace!("send release output voice {:?}", voice);
            unsafe { host_release_out_voice(*self.host_ptr.get_mut(), voice.inner_tag().0) }
        }
    }

    fn kill(&mut self, tag: voice::Tag) {
        if let Some(mut voice) = self.voices.remove(&tag) {
            trace!("send kill output voice {}", tag);
            unsafe {
                host_kill_out_voice(*self.host_ptr.get_mut(), voice.inner_tag().0);
                Box::from_raw(*voice.params_ptr.get_mut());
            };
        }
    }

    fn on_event(&mut self, tag: voice::Tag, event: voice::Event) -> Option<ValuePtr> {
        trace!("send event {:?} for out voice {:?}", event, tag);
        let host_ptr = *self.host_ptr.get_mut();
        self.voices.get_mut(&tag).and_then(|voice| {
            Option::<ffi::Message>::from(event).map(|message| {
                ValuePtr(unsafe { host_on_out_voice_event(host_ptr, voice.inner_tag().0, message) })
            })
        })
    }
}

/// Output voice.
#[derive(Debug)]
pub struct OutVoice {
    tag: voice::Tag,
    params_ptr: AtomicPtr<voice::Params>,
    inner_tag: voice::Tag,
}

impl OutVoice {
    fn new(tag: voice::Tag, params_ptr: AtomicPtr<voice::Params>, inner_tag: voice::Tag) -> Self {
        Self {
            tag,
            params_ptr,
            inner_tag,
        }
    }

    /// Get voice parameters.
    pub fn params(&mut self) -> voice::Params {
        let boxed_params = unsafe { Box::from_raw(*self.params_ptr.get_mut()) };
        let params = boxed_params.clone();
        self.params_ptr = AtomicPtr::new(Box::into_raw(boxed_params));
        *params
    }

    /// Get inner tag.
    pub fn inner_tag(&self) -> voice::Tag {
        self.inner_tag
    }
}

impl Voice for OutVoice {
    fn tag(&self) -> voice::Tag {
        self.tag
    }
}

extern "C" {
    fn host_trig_out_voice(
        host: *mut c_void,
        params: *mut voice::Params,
        index: i32,
        tag: intptr_t,
    ) -> intptr_t;
    fn host_release_out_voice(host: *mut c_void, tag: intptr_t);
    fn host_kill_out_voice(host: *mut c_void, tag: intptr_t);
    fn host_on_out_voice_event(host: *mut c_void, tag: intptr_t, message: ffi::Message)
        -> intptr_t;
}

/// Message from the host to the plugin
#[derive(Debug)]
pub enum Message<'a> {
    /// Contains the handle of the parent window if the editor has to be shown.
    ShowEditor(Option<*mut c_void>),
    /// Change the processing mode flags. This can be ignored.
    ///
    /// The value is [ProcessModeFlags](../struct.ProcessModeFlags.html).
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
    /// The first value will hold a pointer to a rectangle (PRect) for the minimum (Left and Top)
    /// and maximum (Right and Bottom) width and height of the window
    ///
    /// The second value holds a pointer (PPoint) to a point structure that defines by how much the
    /// window size should change horizontally and vertically when the user drags the border.
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
    /// - return `1u8` if the plugin supports the default per-voice level value (filter cutoff (0)
    ///   or filter resonance (1))
    /// - return `2u8` if the plugin supports the per-voice level value, but for another function
    ///   (then check [`GetName::VoiceLevel`](../host/enum.GetName.html#variant.VoiceLevel) to
    ///   provide your own names)
    UseVoiceLevels(u8),
    /// Called when the user selects a preset.
    ///
    /// The value tells you which one to set.
    SetPreset(u64),
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
    /// The host is playing or stopped.
    ///
    /// The value is playing status.
    ///
    /// **Warning: can be called from the mixing thread**
    SetPlaying(bool),
    /// The song position has jumped from one position to another non-consecutive position
    ///
    /// **Warning: can be called from the mixing thread**
    SongPosChanged,
    /// The time signature has changed.
    ///
    /// The value is [`TimeSignature`](../struct.TimeSignature.html).
    SetTimeSig(TimeSignature),
    /// This is called to let the plugin tell the host which files need to be collected or put in
    /// zip files.
    ///
    /// The value holds the file #, which starts at 0
    ///
    /// The name of the file is passed to the host as a `String` in the result of the
    /// dispatcher function. The host keeps calling this until the plugin returns zero.
    CollectFile(usize),
    /// (private message to known plugins, ignore) tells the plugin to update a specific,
    /// non-automated param
    SetInternalParam,
    /// This tells the plugin how many send tracks there are (fixed to 4, but could be set by the
    /// user at any time in a future update)
    ///
    /// The value holds the number of send tracks
    SetNumSends(u64),
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
    SetSamplesPerTick(f32),
    /// Sets the frequency at which Idle is called.
    ///
    /// The value holds the new time (milliseconds)
    SetIdleTime(u64),
    /// (FL 7.0) The host has focused/unfocused the editor (focused in the value) (plugin can use
    /// this to steal keyboard focus)
    SetFocus(bool),
    /// (FL 8.0) This is sent by the host for special transport messages, from a controller.
    ///
    /// The value is the type of message (see [`Transport`](../enum.Transport.html))
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
    /// [`PluginMessage::GetInOuts`](../plugin/enum.PluginMessage.html#variant.GetInOuts) if
    /// necessary
    RoutingChanged,
    /// Retrieves info about a parameter.
    ///
    /// The value is the parameter number.
    ///
    /// see [`ParameterFlags`](../struct.ParameterFlags.html) for the result
    GetParamInfo(usize),
    /// Called after a project has been loaded, to leave a chance to kill automation (that could be
    /// loaded after the plugin is created) if necessary.
    ProjLoaded,
    /// (private message to the plugin wrapper) Load a (VST, DX) plugin state,
    ///
    WrapperLoadState,
    /// Called when the settings button on the titlebar is switched.
    ///
    /// On/off in value.
    ShowSettings(bool),
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
    /// Unknown message.
    Unknown,
}

impl From<ffi::Message> for Message<'_> {
    fn from(message: ffi::Message) -> Self {
        trace!("host::Message::from {:?}", message);

        let result = match message.id {
            0 => Message::from_show_editor(message),
            1 => Message::from_process_mode(message),
            2 => Message::Flush,
            3 => Message::SetBlockSize(message.value as u32),
            4 => Message::SetSampleRate(message.value as u32),
            5 => Message::WindowMinMax(message.index as *mut c_void, message.value as *mut c_void),
            6 => Message::KillVoice,
            7 => Message::UseVoiceLevels(message.index as u8),
            9 => Message::SetPreset(message.index as u64),
            10 => Message::from_chan_sample_changed(message),
            11 => Message::SetEnabled(message.value != 0),
            12 => Message::SetPlaying(message.value != 0),
            13 => Message::SongPosChanged,
            14 => Message::SetTimeSig(ffi::time_sig_from_raw(message.value)),
            15 => Message::CollectFile(message.index as usize),
            16 => Message::SetInternalParam,
            17 => Message::SetNumSends(message.value as u64),
            18 => Message::LoadFile(String::from_raw_ptr(message.value)),
            19 => Message::SetFitTime(f32::from_bits(message.value as i32 as u32)),
            20 => Message::SetSamplesPerTick(f32::from_bits(message.value as i32 as u32)),
            21 => Message::SetIdleTime(message.value as u64),
            22 => Message::SetFocus(message.value != 0),
            23 => Message::Transport(message.into()),
            24 => Message::MidiIn((message.value as u64).into()),
            25 => Message::RoutingChanged,
            26 => Message::GetParamInfo(message.index as usize),
            27 => Message::ProjLoaded,
            28 => Message::WrapperLoadState,
            29 => Message::ShowSettings(message.value != 0),
            30 => Message::SetIoLatency(message.index as u32, message.value as u32),
            32 => Message::PreferredNumIo(message.index as u8),
            _ => Message::Unknown,
        };

        trace!("host::Message::{:?}", result);

        result
    }
}

impl Message<'_> {
    fn from_show_editor(message: ffi::Message) -> Self {
        if message.value == 1 {
            Message::ShowEditor(None)
        } else {
            Message::ShowEditor(Some(message.value as *mut c_void))
        }
    }

    fn from_process_mode(message: ffi::Message) -> Self {
        let flags = ProcessModeFlags::from_bits_truncate(message.value);
        Message::ProcessMode(flags)
    }

    fn from_chan_sample_changed(message: ffi::Message) -> Self {
        let slice =
            unsafe { std::slice::from_raw_parts_mut(message.value as *mut f32, WAVETABLE_SIZE) };
        Message::ChanSampleChanged(slice)
    }
}

/// The host sends this message when it wants to know a text representation of some value.
///
/// See [`Plugin::name_of`](../plugin/trait.Plugin.html#tymethod.name_of)
#[derive(Debug)]
pub enum GetName {
    /// Retrieve the name of a parameter.
    ///
    /// Value specifies parameter index.
    Param(usize),
    /// Retrieve the text representation of the value of a parameter for use in the event editor.
    ///
    /// The first value specifies parameter index.
    ///
    /// The second value specifies value.
    ParamValue(usize, isize),
    /// Retrieve the name of a note in piano roll.
    ///
    /// The first value specifies note index.
    ///
    /// The second one specifies the color (or MIDI channel).
    Semitone(u8, u8),
    /// (not used yet) Retrieve the name of a patch.
    ///
    /// The value specifies patch index.
    Patch(usize),
    /// (optional) Retrieve the name of a per-voice parameter, specified by the value.
    ///
    /// Default is filter cutoff (value=0) and resonance (value=1).
    VoiceLevel(usize),
    /// Longer description for per-voice parameter (works like
    /// [`VoiceLevel`](enum.GetName.html#variant.VoiceLevel))
    VoiceLevelHint(usize),
    /// This is called when the host wants to know the name of a preset, for plugins that support
    /// presets (see
    /// [`PluginMessage::SetNumPresets`](../plugin/enum.PluginMessage.html#variant.SetNumPresets)).
    ///
    /// The value specifies preset index.
    Preset(usize),
    /// For plugins that output controllers, retrieve the name of output controller.
    ///
    /// The value specifies controller index.
    OutCtrl(usize),
    /// Retrieve name of per-voice color (MIDI channel).
    ///
    /// The value specifies the color.
    VoiceColor(u8),
    /// For plugins that output voices, retrieve the name of output voice.
    ///
    /// The value specifies voice index.
    OutVoice(usize),
    /// Message ID is unknown
    Unknown,
}

impl From<ffi::Message> for GetName {
    fn from(message: ffi::Message) -> Self {
        trace!("GetName::from {:?}", message);

        let result = match message.id {
            0 => GetName::Param(message.index as usize),
            1 => GetName::ParamValue(message.index as usize, message.value),
            2 => GetName::Semitone(message.index as u8, message.value as u8),
            3 => GetName::Patch(message.index as usize),
            4 => GetName::VoiceLevel(message.index as usize),
            5 => GetName::VoiceLevelHint(message.index as usize),
            6 => GetName::Preset(message.index as usize),
            7 => GetName::OutCtrl(message.index as usize),
            8 => GetName::VoiceColor(message.index as u8),
            9 => GetName::OutVoice(message.index as usize),
            _ => GetName::Unknown,
        };

        trace!("GetName::{:?}", result);

        result
    }
}

impl From<GetName> for Option<ffi::Message> {
    fn from(value: GetName) -> Self {
        match value {
            GetName::Param(index) => Some(ffi::Message {
                id: 0,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::ParamValue(index, value) => Some(ffi::Message {
                id: 1,
                index: index.as_raw_ptr(),
                value,
            }),
            GetName::Semitone(index, value) => Some(ffi::Message {
                id: 2,
                index: index.as_raw_ptr(),
                value: value.as_raw_ptr(),
            }),
            GetName::Patch(index) => Some(ffi::Message {
                id: 3,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::VoiceLevel(index) => Some(ffi::Message {
                id: 4,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::VoiceLevelHint(index) => Some(ffi::Message {
                id: 5,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::Preset(index) => Some(ffi::Message {
                id: 6,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::OutCtrl(index) => Some(ffi::Message {
                id: 7,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::VoiceColor(index) => Some(ffi::Message {
                id: 8,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::OutVoice(index) => Some(ffi::Message {
                id: 9,
                index: index.as_raw_ptr(),
                value: 0,
            }),
            GetName::Unknown => None,
        }
    }
}

/// Event IDs.
#[derive(Debug)]
pub enum Event {
    /// The tempo has changed.
    ///
    /// First value holds the tempo.
    ///
    /// Second value holds the average samples per tick.
    Tempo(f32, u32),
    /// The maximum polyphony has changed. This is only of intrest to standalone generators.
    ///
    /// Value will hold the new maximum polyphony. A value <= 0 will mean infinite polyphony.
    MaxPoly(i32),
    /// The MIDI channel panning has changed.
    ///
    /// First value holds the new pan (0..127).
    ///
    /// Second value holds pan in -64..64 range.
    MidiPan(u8, i8),
    /// The MIDI channel volume has changed.
    ///
    /// First value holds the new volume (0..127).
    ///
    /// Second value also holds the new volume. It's in the range 0..1.
    MidiVol(u8, f32),
    /// The MIDI channel pitch has changed.
    ///
    /// Value will hold the new value in *cents*.
    ///
    /// This has to be translated according to the current pitch bend range.
    MidiPitch(i32),
    /// Unknown event.
    Unknown,
}

impl From<ffi::Message> for Event {
    fn from(message: ffi::Message) -> Self {
        trace!("Event::from {:?}", message);

        let result = match message.id {
            0 => Event::Tempo(f32::from_raw_ptr(message.index), message.value as u32),
            1 => Event::MaxPoly(message.index as i32),
            2 => Event::MidiPan(message.index as u8, message.value as i8),
            3 => Event::MidiVol(message.index as u8, f32::from_raw_ptr(message.value)),
            4 => Event::MidiPitch(message.index as i32),
            _ => Event::Unknown,
        };

        trace!("Event::{:?}", result);

        result
    }
}
