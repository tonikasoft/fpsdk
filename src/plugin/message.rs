//! Plugin messages.
use std::mem;
use std::os::raw::{c_int, c_void};

use crate::host::{GetName, Host};
use crate::plugin;
use crate::{
    intptr_t, AsRawPtr, FlMessage, MessageBoxFlags, MessageBoxResult, NameColor, Note, Notes,
    ParamMenuEntry, SongTime, TNameColor, TParamMenuEntry, Tag, Time, TimeFormat, ValuePtr,
};

/// Messsage which you can send to the host using
/// [`Host::on_message`](../../host/struct.Host.html#method.on_message).
pub trait Message {
    /// The result returned after sending the message.
    type Return;

    /// Send the message.
    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return;
}

macro_rules! impl_message {
    ($message: ident) => {
        impl Message for $message {
            type Return = ();

            fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
                unsafe {
                    host_on_message(*host.host_ptr.get_mut(), tag.0, self.into());
                }
            }
        }
    };
}

macro_rules! impl_message_ty {
    ($message: ident, $res: ty) => {
        impl Message for $message {
            type Return = $res;

            fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
                ValuePtr(unsafe { host_on_message(*host.host_ptr.get_mut(), tag.0, self.into()) })
                    .get::<$res>()
            }
        }
    };
}

extern "C" {
    fn host_on_message(host: *mut c_void, plugin_tag: Tag, message: FlMessage) -> intptr_t;
}

/// Tells the host that the user has clicked an item of the control popup menu.
///
/// The first value holds the parameter index.
///
/// The second value holds the popup item index.
#[derive(Debug)]
pub struct ParamMenu(pub usize, pub usize);

impl_message!(ParamMenu);

impl From<ParamMenu> for FlMessage {
    fn from(message: ParamMenu) -> Self {
        FlMessage {
            id: 0,
            index: message.0.as_raw_ptr(),
            value: message.1.as_raw_ptr(),
        }
    }
}

/// Notify the host that the editor has been resized.
#[derive(Debug)]
pub struct EditorResized;

impl_message!(EditorResized);

impl From<EditorResized> for FlMessage {
    fn from(_message: EditorResized) -> Self {
        FlMessage {
            id: 2,
            index: 0,
            value: 0,
        }
    }
}

/// Notify the host that names ([`Plugin::name_of`](../trait.Plugin.html#tymethod.name_of)) have
/// changed, with the type of names in value (see [`GetName`](../../host/enum.GetName.html)).
#[derive(Debug)]
pub struct NamesChanged(pub GetName);

impl_message!(NamesChanged);

impl From<NamesChanged> for FlMessage {
    fn from(message: NamesChanged) -> Self {
        FlMessage {
            id: 3,
            index: 0,
            value: Option::<FlMessage>::from(message.0)
                .map(|msg| msg.id)
                .unwrap_or_default(),
        }
    }
}

/// This makes the host enable its MIDI output. This is useful when a MIDI out plugin is
/// created (a plugin which will send midi messages to external midi hardware, most likely).
#[derive(Debug)]
pub struct ActivateMidi;

impl_message!(ActivateMidi);

impl From<ActivateMidi> for FlMessage {
    fn from(_: ActivateMidi) -> Self {
        FlMessage {
            id: 4,
            index: 0,
            value: 0,
        }
    }
}

/// The plugin either wants to be notified about MIDI messages (for processing or filtering), or
/// wants to stop being notified about them.
///
/// Value tells the host whether the plugin want to be notified or not (`true` to be added to the
/// list of plugins that are notified, `false` to be removed from the list).
#[derive(Debug)]
pub struct WantMidiInput(pub bool);

impl_message!(WantMidiInput);

impl From<WantMidiInput> for FlMessage {
    fn from(message: WantMidiInput) -> Self {
        FlMessage {
            id: 5,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Ask the host to kill the automation linked to the plugin. This can for example be used for a
/// demo version of the plugin. The host will kill all automation information in the range <first
/// value>..<second value>. So to kill the automation for all parameters, you'd call the method
/// with <first value> = 0 and <second value> = <num of params> - 1.
///
/// The first value is the first parameter index for which to kill the automation.
///
/// The second value is the last parameter index for which to kill the automation (inclusive).
#[derive(Debug)]
pub struct KillAutomation(pub usize, pub usize);

impl_message!(KillAutomation);

impl From<KillAutomation> for FlMessage {
    fn from(message: KillAutomation) -> Self {
        FlMessage {
            id: 8,
            index: message.0.as_raw_ptr(),
            value: message.1.as_raw_ptr(),
        }
    }
}

/// This tells the host how many presets are supported by the plugin (this is mainly used by the
/// wrapper plugin).
///
/// The value holds the number of presets.
#[derive(Debug)]
pub struct SetNumPresets(pub usize);

impl_message!(SetNumPresets);

impl From<SetNumPresets> for FlMessage {
    fn from(message: SetNumPresets) -> Self {
        FlMessage {
            id: 9,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Sets a new short name for the parent.
///
/// The value is the new name.
#[derive(Debug)]
pub struct SetNewName(pub String);

impl_message!(SetNewName);

impl From<SetNewName> for FlMessage {
    fn from(message: SetNewName) -> Self {
        FlMessage {
            id: 10,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Used by the VSTi wrapper, because the dumb VSTGUI needs idling for his knobs.
#[derive(Debug)]
pub struct VstiIdle;

impl_message!(VstiIdle);

impl From<VstiIdle> for FlMessage {
    fn from(_message: VstiIdle) -> Self {
        FlMessage {
            id: 11,
            index: 0,
            value: 0,
        }
    }
}

// Ask the host to open a selector for its channel sample (Also see FPF_UseChanSample).
//
// HYBRID GENERATORS ARE DEPRECATED.
// pub struct SelectChanSample;

/// Tell the host that the plugin wants to receive the idle message (or not). Idle messages are
/// received by default.
#[derive(Debug)]
pub enum WantIdle {
    /// Disabled.
    Disabled,
    /// Enabled when UI is visible (default).
    EnabledVisible,
    /// Always enabled.
    EnabledAlways,
}

impl_message!(WantIdle);

impl From<WantIdle> for FlMessage {
    fn from(message: WantIdle) -> Self {
        FlMessage {
            id: 13,
            index: 0,
            value: message.into(),
        }
    }
}

impl From<WantIdle> for intptr_t {
    fn from(message: WantIdle) -> Self {
        match message {
            WantIdle::Disabled => 0,
            WantIdle::EnabledVisible => 1,
            WantIdle::EnabledAlways => 2,
        }
    }
}

/// Ask the host to search for a file in its search paths.
///
/// Value should hold the simple filename.
///
/// The full path is returned as result of the function (`String`).
#[derive(Debug)]
pub struct LocateDataFile(pub String);

impl_message_ty!(LocateDataFile, String);

impl From<LocateDataFile> for FlMessage {
    fn from(message: LocateDataFile) -> Self {
        FlMessage {
            id: 14,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Translate tick time into Bar:Step:Tick (warning: it's *not* Bar:Beat:Tick).
///
/// The value should hold the tick time to translate.
///
/// The result is [`SongTime`](../struct.SongTime.html).
#[derive(Debug)]
pub struct TicksToTime(pub u32);

impl Message for TicksToTime {
    type Return = SongTime;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        let message = FlMessage::from(self);
        let time_ptr = message.index;
        unsafe { host_on_message(*host.host_ptr.get_mut(), tag.0, message) };
        ValuePtr(time_ptr).get::<Self::Return>()
    }
}

impl From<TicksToTime> for FlMessage {
    fn from(message: TicksToTime) -> Self {
        let time = SongTime::default();
        FlMessage {
            id: 16,
            index: (Box::into_raw(Box::new(time)) as *mut c_void).as_raw_ptr(),
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Ask the host to add one or more notes to the piano roll.
#[derive(Debug)]
pub struct AddToPianoRoll(pub Notes);

impl_message!(AddToPianoRoll);

impl From<AddToPianoRoll> for FlMessage {
    fn from(mut message: AddToPianoRoll) -> Self {
        message.0.notes.shrink_to_fit();
        let notes_ptr = message.0.notes.as_mut_ptr();
        let len = message.0.notes.len();

        let p_notes_params = unsafe {
            init_p_notes_params(
                1,
                message.0.flags.bits() as c_int,
                message.0.channel.map(|v| v as c_int).unwrap_or(-1),
                message.0.pattern.map(|v| v as c_int).unwrap_or(-1),
                notes_ptr,
                len as c_int,
            )
        };

        FlMessage {
            id: 17,
            index: 0,
            value: p_notes_params,
        }
    }
}

extern "C" {
    // target:
    // 0=step seq (not supported yet), 1=piano roll
    //
    // so we always use 1 for this
    fn init_p_notes_params(
        target: c_int,
        flags: c_int,
        ch_num: c_int,
        pat_num: c_int,
        notes: *mut Note,
        count: c_int,
    ) -> intptr_t;
}

/// Before the popup menu is shown, you must fill it with the entries set by the host. You use this
/// message to find out which those are.
///
/// First value is the parameter index.
///
/// Second value holds the popup item index.
///
/// The result is [`Option<ParamMenuEntry>`](../struct.ParamMenuEntry.html).
#[derive(Debug)]
pub struct GetParamMenuEntry(pub usize, pub usize);

impl Message for GetParamMenuEntry {
    type Return = Option<ParamMenuEntry>;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        let message = FlMessage::from(self);
        let result = unsafe { host_on_message(*host.host_ptr.get_mut(), tag.0, message) };

        if (result as *mut c_void).is_null() {
            return None;
        }

        Some(ParamMenuEntry::from_ffi(
            result as *mut c_void as *mut TParamMenuEntry,
        ))
    }
}

impl From<GetParamMenuEntry> for FlMessage {
    fn from(message: GetParamMenuEntry) -> Self {
        FlMessage {
            id: 18,
            index: message.0.as_raw_ptr(),
            value: message.1.as_raw_ptr(),
        }
    }
}

/// This will make FL to show a message box.
///
/// The first value is the message box title.
///
/// The second value is the message.
///
/// The third value is flags (see [`MessageBoxFlags`](../../struct.MessageBoxFlags.html)).
///
/// The result is [`MessageBoxResult`](../../enum.MessageBoxResult.html).
#[derive(Debug)]
pub struct MessageBox(pub String, pub String, pub MessageBoxFlags);

impl_message_ty!(MessageBox, MessageBoxResult);

impl From<MessageBox> for FlMessage {
    fn from(message: MessageBox) -> Self {
        FlMessage {
            id: 19,
            index: format!("{}|{}", message.0, message.1).as_raw_ptr(),
            value: message.2.as_raw_ptr(),
        }
    }
}

/// Turn on a preview note.
///
/// The first value is the note number.
///
/// The second value is the color (or MIDI channel).
///
/// The third value is the velocity.
#[derive(Debug)]
pub struct NoteOn(pub u8, pub u8, pub u8);

impl_message!(NoteOn);

impl From<NoteOn> for FlMessage {
    fn from(message: NoteOn) -> Self {
        FlMessage {
            id: 20,
            index: dword_from_note_and_ch(message.0, message.1).as_raw_ptr(),
            value: message.2.as_raw_ptr(),
        }
    }
}

/// Turn a preview note off.
///
/// The value is note number.
#[derive(Debug)]
pub struct NoteOff(pub u8);

impl_message!(NoteOff);

impl From<NoteOff> for FlMessage {
    fn from(message: NoteOff) -> Self {
        FlMessage {
            id: 21,
            index: message.0.as_raw_ptr(),
            value: 0,
        }
    }
}

/// This shows a hint message in the FL hint area. It's the same as OnHint, but shows it
/// immediately (to show a progress while you're doing something).
///
/// The value is the message.
#[derive(Debug)]
pub struct OnHintDirect(pub String);

impl_message!(OnHintDirect);

impl From<OnHintDirect> for FlMessage {
    fn from(message: OnHintDirect) -> Self {
        FlMessage {
            id: 22,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Use this code to set a new color for the parent.
///
/// The value is the color.
///
/// (also see [`SetNewName`](../struct.SetNewName.html)).
#[derive(Debug)]
pub struct SetNewColor(pub u8);

impl_message!(SetNewColor);

impl From<SetNewColor> for FlMessage {
    fn from(message: SetNewColor) -> Self {
        FlMessage {
            id: 23,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

// This returns the module instance of the host in Windows. This could be an exe or a DLL, so it
// won't be the process itself.
// #[derive(Debug)]
// pub struct GetInstance;

// impl From<GetInstance> for FlMessage {
// fn from(_: GetInstance) -> Self {
// FlMessage {
// id: 24,
// index: 0,
// value: 0,
// }
// }
// }

/// Ask the host to kill anything linked to an internal controller. This is used when undeclaring
/// internal controllers.
///
/// The first value is the index of the first internal controller to kill.
///
/// The second value is the index of the last internal controller to kill.
#[derive(Debug)]
pub struct KillIntCtrl(pub usize, pub usize);

impl_message!(KillIntCtrl);

impl From<KillIntCtrl> for FlMessage {
    fn from(message: KillIntCtrl) -> Self {
        FlMessage {
            id: 25,
            index: message.0.as_raw_ptr(),
            value: message.1.as_raw_ptr(),
        }
    }
}

/// Call this to override the number of parameters that this plugin instance has. This is meant for
/// plugins that have a different set of parameters per instance.
///
/// The value holds the new number of parameters.
#[derive(Debug)]
pub struct SetNumParams(pub usize);

impl_message!(SetNumParams);

impl From<SetNumParams> for FlMessage {
    fn from(message: SetNumParams) -> Self {
        FlMessage {
            id: 27,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Ask the host to create a filename relative to the FL Studio data folder. This makes it much
/// faster to look for this file (samples, for example) when the song is loaded again.
///
/// The value is the full name.
///
/// The result is the packed filename `String`.
#[derive(Debug)]
pub struct PackDataFile(pub String);

impl_message_ty!(PackDataFile, String);

impl From<PackDataFile> for FlMessage {
    fn from(message: PackDataFile) -> Self {
        FlMessage {
            id: 28,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Ask the host where the FL Studio engine DLL is. This may be different from the location of the
/// executable. It can be used to discover the location of the FL Studio data path.
///
/// The result is the path `String`.
#[derive(Debug)]
pub struct GetProgPath;

impl_message_ty!(GetProgPath, String);

impl From<GetProgPath> for FlMessage {
    fn from(_: GetProgPath) -> Self {
        FlMessage {
            id: 29,
            index: 0,
            value: 0,
        }
    }
}

/// Set the plugin latency, if any.
///
/// The value is the latency in samples.
#[derive(Debug)]
pub struct SetLatency(pub u32);

impl_message!(SetLatency);

impl From<SetLatency> for FlMessage {
    fn from(message: SetLatency) -> Self {
        FlMessage {
            id: 30,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 6.0) Ask the host to show the preset downloader/selector for this plugin.
#[derive(Debug)]
pub struct CallDownloader;

impl_message!(CallDownloader);

impl From<CallDownloader> for FlMessage {
    fn from(_: CallDownloader) -> Self {
        FlMessage {
            id: 31,
            index: 0,
            value: 0,
        }
    }
}

/// (FL 7.0) Edits sample in Edison.
///
/// The first value holds the sample filename.
///
/// The second value is `true` if an existing instance of Edison can be re-used or `false`
/// otherwise.
#[derive(Debug)]
pub struct EditSample(pub String, pub bool);

impl_message!(EditSample);

impl From<EditSample> for FlMessage {
    fn from(message: EditSample) -> Self {
        FlMessage {
            id: 32,
            index: message.1.as_raw_ptr(),
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 7.0) Call this to let FL know that this plugin is thread-safe (or not). The default is not.
/// You should do your own thread-sync using
/// [`Host::lock_mix`](../../host/struct.Host.html#method.lock_mix).
///
/// The value is `false` for `not safe` and `true` for `safe`.
///
/// **Important: this should only be used from a generator plugin!**
#[derive(Debug)]
pub struct SetThreadSafe(pub bool);

impl_message!(SetThreadSafe);

impl From<SetThreadSafe> for FlMessage {
    fn from(message: SetThreadSafe) -> Self {
        FlMessage {
            id: 33,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 7.0) The plugin asks FL to enable or disable smart disabling. This is mainly for
/// generators, so they can get MIDI input (if applicable).
///
/// The value holds the switch.
#[derive(Debug)]
pub struct SmartDisable(pub bool);

impl_message!(SmartDisable);

impl From<SmartDisable> for FlMessage {
    fn from(message: SmartDisable) -> Self {
        FlMessage {
            id: 34,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 8.0) Sets the unique identifying string for this plugin. This will be used to save/restore
/// custom data related to this plugin. Handy for wrapper plugins.
///
/// The value is the identifying string.
#[derive(Debug)]
pub struct SetUid(pub String);

impl_message!(SetUid);

impl From<SetUid> for FlMessage {
    fn from(message: SetUid) -> Self {
        FlMessage {
            id: 35,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 8.0) Get the mixer time, relative to the current time.
///
/// The first value is the time format required.
///
/// The second value is offset in samples.
///
/// The result is [`Time`](../struct.Time.html).
#[derive(Debug)]
pub struct GetMixingTime(pub TimeFormat, pub u64);

impl Message for GetMixingTime {
    type Return = Time;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        get_time_send(self, tag, host)
    }
}

fn get_time_send<T: Into<FlMessage>>(msg: T, tag: plugin::Tag, host: &mut Host) -> Time {
    let message: FlMessage = msg.into();
    let time_ptr = message.value;
    unsafe { host_on_message(*host.host_ptr.get_mut(), tag.0, message) };
    ValuePtr(time_ptr).get::<Time>()
}

impl From<GetMixingTime> for FlMessage {
    fn from(message: GetMixingTime) -> Self {
        get_time_ffi(36, message.0, message.1)
    }
}

fn get_time_ffi(id: intptr_t, format: TimeFormat, offset: u64) -> FlMessage {
    let time = Time(offset as f64, offset as f64);
    FlMessage {
        id,
        index: u8::from(format).as_raw_ptr(),
        value: (Box::into_raw(Box::new(time)) as *mut c_void).as_raw_ptr(),
    }
}

/// (FL 8.0) Get playback time. See `GetMixingTime` for details.
#[derive(Debug)]
pub struct GetPlaybackTime(pub TimeFormat, pub u64);

impl Message for GetPlaybackTime {
    type Return = Time;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        get_time_send(self, tag, host)
    }
}

impl From<GetPlaybackTime> for FlMessage {
    fn from(message: GetPlaybackTime) -> Self {
        get_time_ffi(37, message.0, message.1)
    }
}

/// (FL 8.0) Get selection time.
///
/// The value is the time formad required.
///
/// The result is [`Time`](../struct.Time.html). If there's no selection, the `Time` will content
/// the full song range.
#[derive(Debug)]
pub struct GetSelTime(pub TimeFormat);

impl Message for GetSelTime {
    type Return = Time;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        get_time_send(self, tag, host)
    }
}

impl From<GetSelTime> for FlMessage {
    fn from(message: GetSelTime) -> Self {
        get_time_ffi(38, message.0, 0)
    }
}

/// (FL 8.0) Get the current tempo multiplicator. This is not part of the song but used for
/// fast-forward.
///
/// The result is `f32`.
#[derive(Debug)]
pub struct GetTimeMul;

impl_message_ty!(GetTimeMul, f32);

impl From<GetTimeMul> for FlMessage {
    fn from(_: GetTimeMul) -> Self {
        FlMessage {
            id: 39,
            index: 0,
            value: 0,
        }
    }
}

/// (FL 8.0) Captionize the plugin. This can be useful when dragging.
///
/// The value is `true` for captionized or `false` otherwise.
#[derive(Debug)]
pub struct Captionize(pub bool);

impl_message!(Captionize);

impl From<Captionize> for FlMessage {
    fn from(message: Captionize) -> Self {
        FlMessage {
            id: 40,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 8.0) Send a SysEx bytes, without delay. Do not abuse this!
///
/// The first value is the port to send to.
///
/// The second value is the data to send.
#[derive(Debug)]
pub struct SendSysEx<'a>(pub usize, pub &'a [u8]);

impl Message for SendSysEx<'_> {
    type Return = ();

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        unsafe {
            host_on_message(*host.host_ptr.get_mut(), tag.0, self.into());
        }
    }
}

impl From<SendSysEx<'_>> for FlMessage {
    fn from(message: SendSysEx<'_>) -> Self {
        let len = message.1.len() as i32;
        let len_bytes: [u8; mem::size_of::<i32>()] = unsafe { mem::transmute(len) };
        let mut final_data = [&len_bytes, message.1].concat();
        let data_ptr = final_data.as_mut_ptr();
        mem::forget(final_data);

        FlMessage {
            id: 41,
            index: message.0.as_raw_ptr(),
            value: (data_ptr as *mut c_void).as_raw_ptr(),
        }
    }
}

/// (FL 8.0) Send an audio file to the playlist as an audio clip, starting at the playlist
/// selection (mainly for Edison).
///
/// The value is the file name.
#[derive(Debug)]
pub struct LoadAudioClip(pub String);

impl_message!(LoadAudioClip);

impl From<LoadAudioClip> for FlMessage {
    fn from(message: LoadAudioClip) -> Self {
        FlMessage {
            id: 42,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 8.0) Send a file to the selected channel(s) (mainly for Edison).
///
/// The value is the file name.
#[derive(Debug)]
pub struct LoadInChannel(pub String);

impl_message!(LoadInChannel);

impl From<LoadInChannel> for FlMessage {
    fn from(message: LoadInChannel) -> Self {
        FlMessage {
            id: 43,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// (FL 8.0) Locates the specified file in the browser and jumps to it. This also adds the file's
/// folder to the browser search paths if necessary.
///
/// The value is the file name.
#[derive(Debug)]
pub struct ShowInBrowser(pub String);

impl_message!(ShowInBrowser);

impl From<ShowInBrowser> for FlMessage {
    fn from(message: ShowInBrowser) -> Self {
        FlMessage {
            id: 44,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Adds message to the debug log.
///
/// The value is the message.
#[derive(Debug)]
pub struct DebugLogMsg(pub String);

impl_message!(DebugLogMsg);

impl From<DebugLogMsg> for FlMessage {
    fn from(message: DebugLogMsg) -> Self {
        FlMessage {
            id: 45,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Gets the handle of the main form.
///
/// The result is `Option<*mut c_void>` (`Option<HWND>`).
#[derive(Debug)]
pub struct GetMainFormHandle;

impl Message for GetMainFormHandle {
    type Return = Option<*mut c_void>;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        let message = FlMessage::from(self);
        let result_ptr = message.value;
        unsafe { host_on_message(*host.host_ptr.get_mut(), tag.0, message) };

        if result_ptr == 0 {
            None
        } else {
            Some(result_ptr as *mut c_void)
        }
    }
}

impl From<GetMainFormHandle> for FlMessage {
    fn from(_: GetMainFormHandle) -> Self {
        FlMessage {
            id: 46,
            index: 0,
            value: 0,
        }
    }
}

/// Ask the host where the project data is, to store project data.
///
/// The result is `String`.
#[derive(Debug)]
pub struct GetProjDataPath;

impl_message_ty!(GetProjDataPath, String);

impl From<GetProjDataPath> for FlMessage {
    fn from(_message: GetProjDataPath) -> Self {
        FlMessage {
            id: 47,
            index: 0,
            value: 0,
        }
    }
}

/// Mark project as dirty (not required for automatable parameters, only for tweaks the host can't
/// be aware of).
#[derive(Debug)]
pub struct SetDirty;

impl_message!(SetDirty);

impl From<SetDirty> for FlMessage {
    fn from(_message: SetDirty) -> Self {
        FlMessage {
            id: 48,
            index: 0,
            value: 0,
        }
    }
}

/// Add file to recent files.
///
/// The value is file name.
#[derive(Debug)]
pub struct AddToRecent(pub String);

impl_message!(AddToRecent);

impl From<AddToRecent> for FlMessage {
    fn from(message: AddToRecent) -> Self {
        FlMessage {
            id: 49,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Ask the host how many inputs are routed to this effect, or how many outputs this effect is
/// routed to.
///
/// The result is `usize`.
#[derive(Debug)]
pub enum GetNumInOut {
    /// To get inputs number.
    Inputs,
    /// To get outputs number.
    Outputs,
}

impl_message_ty!(GetNumInOut, usize);

impl From<GetNumInOut> for FlMessage {
    fn from(message: GetNumInOut) -> Self {
        FlMessage {
            id: 50,
            index: message.into(),
            value: 0,
        }
    }
}

impl From<GetNumInOut> for intptr_t {
    fn from(message: GetNumInOut) -> Self {
        match message {
            GetNumInOut::Inputs => 0,
            GetNumInOut::Outputs => 1,
        }
    }
}

/// Ask the host the name of the input.
///
/// The value is the input index starting from 1.
///
/// The result is [`Option<NameColor>`](../struct.NameColor.html).
///
/// **Important: the first input index is `1`.**
#[derive(Debug)]
pub struct GetInName(pub usize);

impl Message for GetInName {
    type Return = Option<NameColor>;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        get_name_dispatcher(self, tag, host)
    }
}

fn get_name_dispatcher<T: Into<FlMessage>>(
    msg: T,
    tag: plugin::Tag,
    host: &mut Host,
) -> Option<NameColor> {
    let message: FlMessage = msg.into();
    let result_ptr = message.value;
    let result = unsafe { host_on_message(*host.host_ptr.get_mut(), tag.0, message) };

    if result == 0 || (result_ptr as *mut c_void).is_null() {
        return None;
    }

    Some(ValuePtr(result_ptr).get::<TNameColor>().into())
}

impl From<GetInName> for FlMessage {
    fn from(message: GetInName) -> Self {
        get_name_ffi(51, message.0)
    }
}

fn get_name_ffi(id: intptr_t, index: usize) -> FlMessage {
    let name_color = TNameColor {
        name: [0; 256],
        vis_name: [0; 256],
        color: 0,
        index: index as c_int,
    };
    FlMessage {
        id,
        index: index.as_raw_ptr(),
        value: (Box::into_raw(Box::new(name_color)) as *mut c_void).as_raw_ptr(),
    }
}

/// Ask the host the name of the output.
///
/// The value is the output index starting from 1.
///
/// The result is [`Option<NameColor>`](../struct.NameColor.html).
///
/// **Important: the first output index is `1`.**
#[derive(Debug)]
pub struct GetOutName(pub usize);

impl Message for GetOutName {
    type Return = Option<NameColor>;

    fn send(self, tag: plugin::Tag, host: &mut Host) -> Self::Return {
        get_name_dispatcher(self, tag, host)
    }
}

impl From<GetOutName> for FlMessage {
    fn from(message: GetOutName) -> Self {
        get_name_ffi(52, message.0)
    }
}

/// Make the host bring plugin's editor.
#[derive(Debug)]
pub enum ShowEditor {
    /// Show.
    Show,
    /// Hide.
    Hide,
    /// Toggle.
    Toggle,
}

impl_message!(ShowEditor);

impl From<ShowEditor> for FlMessage {
    fn from(message: ShowEditor) -> Self {
        FlMessage {
            id: 53,
            index: 0,
            value: message.into(),
        }
    }
}

impl From<ShowEditor> for intptr_t {
    fn from(message: ShowEditor) -> Self {
        match message {
            ShowEditor::Show => 1,
            ShowEditor::Hide => 0,
            ShowEditor::Toggle => -1,
        }
    }
}

/// (for the plugin wrapper only) Ask the host to turn 0..65536 automation into 0..1 float, for
/// params number between the first and last value (included).
#[derive(Debug)]
pub struct FloatAutomation(pub usize, pub usize);

impl_message!(FloatAutomation);

impl From<FloatAutomation> for FlMessage {
    fn from(message: FloatAutomation) -> Self {
        FlMessage {
            id: 54,
            index: message.0.as_raw_ptr(),
            value: message.1.as_raw_ptr(),
        }
    }
}

/// Called when the settings button on the titlebar should be switched.
///
/// The value is `true` to show and `false` to hide.
///
/// See
/// [`InfoBuilder::want_settings_button`](
/// ../plugin/struct.InfoBuilder.html#method.want_settings_button).
#[derive(Debug)]
pub struct ShowSettings(pub bool);

impl_message!(ShowSettings);

impl From<ShowSettings> for FlMessage {
    fn from(message: ShowSettings) -> Self {
        FlMessage {
            id: 55,
            index: 0,
            value: message.0.as_raw_ptr(),
        }
    }
}

/// Note on/off.
///
/// The first value is note nummber.
///
/// The second value is the color/MIDI channel.
///
/// The third value is velocity. Note off send for velocity `0`, note on otherwise.
#[derive(Debug)]
pub struct NoteOnOff(pub u8, pub u8, pub u8);

impl_message!(NoteOnOff);

impl From<NoteOnOff> for FlMessage {
    fn from(message: NoteOnOff) -> Self {
        FlMessage {
            id: 56,
            index: dword_from_note_and_ch(message.0, message.1).as_raw_ptr(),
            value: message.2.as_raw_ptr(),
        }
    }
}

/// Show picker.
#[derive(Debug)]
pub enum ShowPicker {
    /// Plugins.
    Plugins(PickerFilter),
    /// Project.
    Project(PickerFilter),
}

impl_message!(ShowPicker);

/// What kind of items the picker should show.
#[derive(Debug)]
pub enum PickerFilter {
    /// Generators.
    Generators,
    /// Effects.
    Effects,
    /// Generators and effects.
    GeneratorsEffects,
    /// Patcher (includes VFX).
    Patcher,
}

impl From<ShowPicker> for FlMessage {
    fn from(message: ShowPicker) -> Self {
        let (index, value): (intptr_t, intptr_t) = message.into();
        FlMessage {
            id: 57,
            index,
            value,
        }
    }
}

impl From<ShowPicker> for (intptr_t, intptr_t) {
    fn from(message: ShowPicker) -> Self {
        match message {
            ShowPicker::Plugins(filter) => (0, filter.into()),
            ShowPicker::Project(filter) => (1, filter.into()),
        }
    }
}

impl From<PickerFilter> for intptr_t {
    fn from(filter: PickerFilter) -> Self {
        match filter {
            PickerFilter::Generators => 0,
            PickerFilter::Effects => 1,
            PickerFilter::GeneratorsEffects => -1,
            PickerFilter::Patcher => -2,
        }
    }
}

/// Ask the host for the number of extra frames `Plugin::idle` should process, generally 0 if no
/// overflow/frameskip occured.
#[derive(Debug)]
pub struct GetIdleOverflow;

impl_message!(GetIdleOverflow);

impl From<GetIdleOverflow> for FlMessage {
    fn from(_: GetIdleOverflow) -> Self {
        FlMessage {
            id: 58,
            index: 0,
            value: 0,
        }
    }
}

/// Used by FL plugins, when idling from a modal window, mainly for the smoothness hack.
#[derive(Debug)]
pub struct ModalIdle;

impl_message!(ModalIdle);

impl From<ModalIdle> for FlMessage {
    fn from(_: ModalIdle) -> Self {
        FlMessage {
            id: 59,
            index: 0,
            value: 0,
        }
    }
}

/// Prompt the rendering dialog in song mode.
#[derive(Debug)]
pub struct RenderProject;

impl_message!(RenderProject);

impl From<RenderProject> for FlMessage {
    fn from(_: RenderProject) -> Self {
        FlMessage {
            id: 60,
            index: 0,
            value: 0,
        }
    }
}

/// Get project title, author, comments or URL.
///
/// The result is `String`.
#[derive(Debug)]
pub enum GetProjectInfo {
    /// Title.
    Title,
    /// Author.
    Author,
    /// Comments.
    Comments,
    /// URL.
    Url,
}

impl_message_ty!(GetProjectInfo, String);

impl From<GetProjectInfo> for FlMessage {
    fn from(message: GetProjectInfo) -> Self {
        FlMessage {
            id: 61,
            index: message.into(),
            value: 0,
        }
    }
}

impl From<GetProjectInfo> for intptr_t {
    fn from(value: GetProjectInfo) -> Self {
        match value {
            GetProjectInfo::Title => 0,
            GetProjectInfo::Author => 1,
            GetProjectInfo::Comments => 2,
            GetProjectInfo::Url => 3,
        }
    }
}

fn dword_from_note_and_ch(note: u8, channel: u8) -> u32 {
    (note as u32) | ((channel as u32) << 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dword() {
        let value = dword_from_note_and_ch(60, 15);
        assert_eq!(60, value & 0xff);
        assert_eq!(15, (value >> 16) & 0xff);
    }
}
