//! Voices used by generators to track events like their instantiation, release, freeing and
//! processing some events.
use std::os::raw::c_void;

use crate::plugin::PluginAdapter;
use crate::{intptr_t, AsRawPtr, FlMessage, ValuePtr};

crate::implement_tag!();

/// Implement this trait for your type if you make a generator plugin.
///
/// All methods can be called either from GUI or mixer thread.
pub trait ReceiveVoiceHandler: Send + Sync {
    /// The host calls this to let it create a voice.
    ///
    /// The `tag` parameter is an identifier the host uses to identify the voice.
    fn trigger(&mut self, params: Params, tag: Tag) -> &mut dyn Voice;
    /// This gets called by the host when the voice enters the envelope release state (note off).
    fn release(&mut self, tag: Tag);
    /// Called when the voice has to be discarded.
    fn kill(&mut self, tag: Tag);
    /// Process a voice event.
    fn on_event(&mut self, _tag: Tag, _event: Event) -> Box<dyn AsRawPtr> {
        Box::new(0)
    }
    /// Getter for [`SendVoiceHandler`](trait.SendVoiceHandler.html).
    fn out_handler(&mut self) -> Option<&mut dyn SendVoiceHandler> {
        None
    }
}

/// You should implement this trait to your voice type.
pub trait Voice: Send + Sync {
    /// Get ID of the voice.
    fn tag(&self) -> Tag;
}

/// This is the type for the parameters for a voice. Normally, you'll only use `final_levels`. The
/// final levels are the initial (voice) levels altered by the channel levels. But the initial
/// levels are also available for, for example, note layering. In any case the initial levels are
/// made to be checked once the voice is triggered, while the other ones are to be checked every
/// time.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Params {
    /// Made to be checked once the voice is triggered.
    pub init_levels: LevelParams,
    /// Made to be checked every time.
    pub final_levels: LevelParams,
}

/// This structure holds the parameters for a channel. They're used both for final voice levels
/// (voice levels+parent channel levels) and original voice levels. `LevelParams` is used in
/// [`Params`](struct.Params.html).
///
/// **All of these parameters can go outside their defined range!**
#[derive(Clone, Debug)]
#[repr(C)]
pub struct LevelParams {
    /// Panning (-1..1).
    pub pan: f32,
    /// Volume/velocity (0..1).
    pub vol: f32,
    /// Pitch (in cents) (semitone=pitch/100).
    pub pitch: f32,
    /// Modulation X or filter cutoff (-1..1).
    pub mod_x: f32,
    /// Modulation Y or filter resonance (-1..1).
    pub mod_y: f32,
}

/// Voice events.
#[derive(Debug)]
pub enum Event {
    /// Monophonic mode can retrigger releasing voices.
    Retrigger,
    /// Retrieve the note length in ticks. The result is not reliable.
    ///
    /// Function result holds the length of the note, or -1 if it's not defined.
    GetLength,
    /// Retrieve the color for a note. A note can currently have up to 16 colors in the pianoroll.
    /// This can be mapped to a MIDI channel.
    ///
    /// Functions result holds the note color (0..15).
    GetColor,
    /// (FL 7.0) Retrieve note on velocity. This is computed from
    /// [`Params.init_levels.vol`](struct.Params.html). This should be called from `trigger`
    /// method.
    ///
    /// Function result holds the `f32` velocity (0.0..1.0).
    GetVelocity,
    /// (FL 7.0) Retrieve release velocity (0.0..1.0) in result. Use this if some release velocity
    /// mapping is involved. This should be called from `release` method.
    ///
    /// Function result holds the `f32` velocity (0.0..1.0) (to be called from `release` method).
    GetRelVelocity,
    /// (FL 7.0) Retrieve release time multiplicator. Use this for direct release multiplicator.
    /// This should be called from `release` method.
    ///
    /// Function result holds the `f32` value (0.0..2.0).
    GetRelTime,
    /// (FL 7.0) Call this to set if velocity is linked to volume or not. The default is on.
    SetLinkVelocity(bool),
    /// Unknown event.
    Unknown,
}

impl From<FlMessage> for Event {
    fn from(message: FlMessage) -> Self {
        match message.id {
            0 => Event::Retrigger,
            1 => Event::GetLength,
            2 => Event::GetColor,
            3 => Event::GetVelocity,
            4 => Event::GetRelVelocity,
            5 => Event::GetRelTime,
            6 => Event::SetLinkVelocity(message.index != 0),
            _ => Event::Unknown,
        }
    }
}

impl From<Event> for Option<FlMessage> {
    fn from(event: Event) -> Self {
        match event {
            Event::Retrigger => Some(FlMessage {
                id: 0,
                index: 0,
                value: 0,
            }),
            Event::GetLength => Some(FlMessage {
                id: 1,
                index: 0,
                value: 0,
            }),
            Event::GetColor => Some(FlMessage {
                id: 2,
                index: 0,
                value: 0,
            }),
            Event::GetVelocity => Some(FlMessage {
                id: 3,
                index: 0,
                value: 0,
            }),
            Event::GetRelVelocity => Some(FlMessage {
                id: 4,
                index: 0,
                value: 0,
            }),
            Event::GetRelTime => Some(FlMessage {
                id: 5,
                index: 0,
                value: 0,
            }),
            Event::SetLinkVelocity(value) => Some(FlMessage {
                id: 6,
                index: value as isize,
                value: 0,
            }),
            Event::Unknown => None,
        }
    }
}

/// Additional methods used by [`ReceiveVoiceHandler`](trait.ReceiveVoiceHandler.html) in VFX
/// plugins for the output voices.
pub trait SendVoiceHandler: Send + Sync {
    /// The host calls this to let it create a voice.
    ///
    /// - `tag` is an identifier the host uses to identify the voice.
    /// - `index` is voice output index in patcher.
    ///
    fn trigger(&mut self, _params: Params, _index: usize, _tag: Tag) -> Option<&mut dyn Voice> {
        None
    }
    /// This gets called by the host when the voice enters the envelope release state (note off).
    fn release(&mut self, _tag: Tag) {}
    /// Called when the voice has to be discarded.
    fn kill(&mut self, tag: Tag);
    /// Process a voice event.
    ///
    /// See [`Event`](enum.Event.html) for result variants.
    fn on_event(&mut self, _tag: Tag, _event: Event) -> Option<ValuePtr> {
        None
    }
}

/// [`ReceiveVoiceHandler::trigger`](trait.ReceiveVoiceHandler.html#tymethod.trigger) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn voice_handler_trigger(
    adapter: *mut PluginAdapter,
    params: Params,
    tag: intptr_t,
) -> intptr_t {
    (*adapter)
        .0
        .voice_handler()
        .map(|handler| {
            let voice_ptr: *mut &mut dyn Voice =
                Box::leak(Box::new(handler.trigger(params, Tag(tag))));
            voice_ptr as *mut c_void as intptr_t
        })
        .unwrap_or(-1)
}

/// [`ReceiveVoiceHandler::release`](trait.ReceiveVoiceHandler.html#tymethod.release) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn voice_handler_release(
    adapter: *mut PluginAdapter,
    voice: *mut &mut dyn Voice,
) {
    // We don't call Box::from_raw because:
    // 1. Host calls this then voice_handler_kill — this way we'll get double deallocation
    // 2. Given FL SDK documentation, we shouldn't deallocate voices here
    if let Some(handler) = (*adapter).0.voice_handler() {
        handler.release((*voice).tag())
    }
}

/// [`ReceiveVoiceHandler::kill`](trait.ReceiveVoiceHandler.html#tymethod.kill) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn voice_handler_kill(adapter: *mut PluginAdapter, voice: *mut &mut dyn Voice) {
    let r_voice = Box::from_raw(voice);
    if let Some(handler) = (*adapter).0.voice_handler() {
        handler.kill(r_voice.tag())
    }
}

/// [`ReceiveVoiceHandler::kill_out`](trait.ReceiveVoiceHandler.html#tymethod.kill_out) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn out_voice_handler_kill(adapter: *mut PluginAdapter, tag: intptr_t) {
    (*adapter).0.voice_handler().and_then(|handler| {
        handler.out_handler().map(|out_handler| {
            out_handler.kill(Tag(tag));
        })
    });
}

/// [`ReceiveVoiceHandler::on_event`](trait.ReceiveVoiceHandler.html#tymethod.on_event) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn voice_handler_on_event(
    adapter: *mut PluginAdapter,
    voice: *mut &mut dyn Voice,
    message: FlMessage,
) -> intptr_t {
    (*adapter)
        .0
        .voice_handler()
        .map(|handler| {
            handler
                .on_event((*voice).tag(), message.into())
                .as_raw_ptr()
        })
        .unwrap_or(-1)
}

/// [`SendVoiceHandler::on_event`](trait.SendVoiceHandler.html#method.on_event) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn out_voice_handler_on_event(
    adapter: *mut PluginAdapter,
    tag: intptr_t,
    message: FlMessage,
) -> intptr_t {
    (*adapter)
        .0
        .voice_handler()
        .and_then(|handler| handler.out_handler())
        .and_then(|out_handler| out_handler.on_event(Tag(tag), message.into()))
        .map(|result| result.0)
        .unwrap_or(-1)
}

/// Translate FL voice volume to linear velocity (0.0..1.0).
pub fn vol_to_vel(vol: f32) -> f32 {
    inv_log_vol(vol * 10.0, 2610.0 / 127.0)
}

/// Translate FL voice volume to linear velocity (0.0..127.0).
pub fn vol_to_midi_vel(vol: f32) -> f32 {
    inv_log_vol(vol * 10.0, 2610.0 / 127.0) * 127.0
}

fn inv_log_vol(value: f32, max_value: f32) -> f32 {
    (value + 1.0).ln() / (max_value + 1.0).ln()
}
