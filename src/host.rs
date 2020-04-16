use std::ffi::{c_void, CStr};
use std::os::raw::c_char;

use crate::{ffi, MidiMessage, ProcessModeFlags, TimeSignature, Transport, WAVETABLE_SIZE};

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
    /// - return `1u8` if the plugin supports the default per-voice level value (filter cutoff (0) or
    ///   filter resonance (1))
    /// - return `2u8` if the plugin supports the per-voice level value, but for another function
    ///   (then check FPN_VoiceLevel to provide your own names)
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
    /// The first value is playing status. The second value is position.
    ///
    /// **Warning: can be called from the mixing thread**
    SetPlaying(bool, u64),
    /// The song position has jumped from one position to another non-consecutive position
    ///
    /// **Warning: can be called from the mixing thread**
    SongPosChanged,
    /// The time signature has changed.
    ///
    /// The value is [`TimeSignature`](struct.TimeSignature.html).
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
    SetFitTime(f64),
    /// Sets the number of samples in each tick. This value changes when the tempo, ppq or sample
    /// rate have changed.
    ///
    /// **Warning: can be called from the mixing thread**
    SetSamplesPerTick(f64),
    /// Sets the frequency at which Idle is called.
    ///
    /// The value holds the new time (milliseconds)
    SetIdleTime(u64),
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

impl From<ffi::Message> for HostMessage<'_> {
    fn from(message: ffi::Message) -> Self {
        match message.id {
            0 => HostMessage::from_show_editor(message),
            1 => HostMessage::from_process_mode(message),
            2 => HostMessage::Flush,
            3 => HostMessage::SetBlockSize(message.value as u32),
            4 => HostMessage::SetSampleRate(message.value as u32),
            5 => HostMessage::WindowMinMax(
                message.index as *mut c_void,
                message.value as *mut c_void,
            ),
            6 => HostMessage::KillVoice,
            7 => HostMessage::UseVoiceLevels(message.index as u8),
            9 => HostMessage::SetPreset(message.index as u64),
            10 => HostMessage::from_chan_sample_changed(message),
            11 => HostMessage::SetEnabled(message.value != 0),
            12 => HostMessage::SetPlaying(message.value != 0, message.value as u64),
            13 => HostMessage::SongPosChanged,
            14 => HostMessage::SetTimeSig(ffi::time_sig_from_raw(message.value)),
            15 => HostMessage::CollectFile(message.index as usize),
            16 => HostMessage::SetInternalParam,
            17 => HostMessage::SetNumSends(message.value as u64),
            18 => HostMessage::from_load_file(message),
            19 => HostMessage::SetFitTime(unsafe { *(message.value as *mut f64) }),
            20 => HostMessage::SetSamplesPerTick(unsafe { *(message.value as *mut f64) }),
            21 => HostMessage::SetIdleTime(message.value as u64),
            22 => HostMessage::SetFocus(message.value != 0),
            23 => HostMessage::Transport(message.into()),
            24 => HostMessage::MidiIn((message.value as u64).into()),
            25 => HostMessage::RoutingChanged,
            26 => HostMessage::GetParamInfo(message.index as usize),
            27 => HostMessage::ProjLoaded,
            28 => HostMessage::WrapperLoadState,
            29 => HostMessage::ShowSettings(message.value != 0),
            30 => HostMessage::SetIoLatency(message.index as u32, message.value as u32),
            32 => HostMessage::PreferredNumIo(message.index as u8),
            _ => HostMessage::Unknown,
        }
    }
}

impl HostMessage<'_> {
    fn from_show_editor(message: ffi::Message) -> Self {
        if message.value == 1 {
            HostMessage::ShowEditor(None)
        } else {
            HostMessage::ShowEditor(Some(message.value as *mut c_void))
        }
    }

    fn from_process_mode(message: ffi::Message) -> Self {
        let flags = ProcessModeFlags::from_bits_truncate(message.value);
        HostMessage::ProcessMode(flags)
    }

    fn from_chan_sample_changed(message: ffi::Message) -> Self {
        let slice =
            unsafe { std::slice::from_raw_parts_mut(message.value as *mut f32, WAVETABLE_SIZE) };
        HostMessage::ChanSampleChanged(slice)
    }

    fn from_load_file(message: ffi::Message) -> Self {
        let cstr = unsafe { CStr::from_ptr(message.value as *const c_char) };
        let value = cstr.to_string_lossy().to_string();
        HostMessage::LoadFile(value)
    }
}