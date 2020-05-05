//! Voices used by generators to track events like their instantiation, release, freeing and
//! processing some events.
use std::os::raw::c_void;

use crate::plugin::PluginAdapter;
use crate::{ffi, intptr_t, AsRawPtr};

crate::implement_tag!();

/// Implement this trait for your type if you make a generator plugin.
///
/// All methods can be called either from GUI or mixer thread.
pub trait VoiceHandler: Send + Sync {
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
    /// Getter for [`OutVoiceHandler`](trait.OutVoiceHandler.html).
    fn out_handler(&mut self) -> Option<&mut dyn OutVoiceHandler> {
        None
    }
}

/// You should add this marker to your voice type.
pub trait Voice: Send + Sync {
    /// Get ID of the voice.
    fn tag(&self) -> Tag;
}

/// This is the type for the parameters for a voice. Normally, you'll only use `final_levels`. The
/// final levels are the initial (voice) levels altered by the channel levels. But the initial
/// levels are also available for, for example, note layering. In any case the initial levels are
/// made to be checked once the voice is triggered, while the other ones are to be checked every
/// time.
#[derive(Debug)]
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
#[derive(Debug)]
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
    /// Unknown event.
    Unknown,
}

impl From<ffi::Message> for Event {
    fn from(message: ffi::Message) -> Self {
        match message.id {
            0 => Event::Retrigger,
            _ => Event::Unknown,
        }
    }
}

/// Additional methods used by [`VoiceHandler`](trait.VoiceHandler.html) in VFX plugins for the
/// output voices.
pub trait OutVoiceHandler: Send + Sync {
    /// Called when the voice has to be discarded.
    fn kill(&mut self, tag: Tag);
    /// Process a voice event.
    fn on_event(&mut self, _tag: Tag, _event: OutEvent) -> Box<dyn AsRawPtr> {
        Box::new(0)
    }
}

/// Output voice events.
#[derive(Debug)]
pub enum OutEvent {
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
    /// Function result holds the velocity (0.0..1.0).
    GetVelocity,
    /// (FL 7.0) Retrieve release velocity (0.0..1.0) in result. Use this if some release velocity
    /// mapping is involved. This should be called from `release` method.
    ///
    /// Function result holds the velocity (0.0..1.0) (to be called from `release` method).
    GetRelVelocity,
    /// (FL 7.0) Retrieve release time multiplicator. Use this for direct release multiplicator.
    /// This should be called from `release` method.
    ///
    /// Function result holds the value (0.0..2.0).
    GetRelTime,
    /// (FL 7.0) Call this to set if velocity is linked to volume or not. The default is on.
    SetLinkVelocity(bool),
    /// Unknown event.
    Unknown,
}

impl From<ffi::Message> for OutEvent {
    fn from(message: ffi::Message) -> Self {
        match message.id {
            1 => OutEvent::GetLength,
            2 => OutEvent::GetColor,
            3 => OutEvent::GetVelocity,
            4 => OutEvent::GetRelVelocity,
            5 => OutEvent::GetRelTime,
            6 => OutEvent::SetLinkVelocity(message.index != 0),
            _ => OutEvent::Unknown,
        }
    }
}

/// [`VoiceHandler::trigger`](trait.VoiceHandler.html#tymethod.trigger) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn voice_handler_trigger(
    adapter: *mut PluginAdapter,
    params: Params,
    tag: i32,
) -> intptr_t {
    let handler = (*adapter).0.voice_handler();
    let voice_ptr: *mut &mut dyn Voice = Box::leak(Box::new(handler.trigger(params, Tag(tag))));
    voice_ptr as *mut c_void as intptr_t
}

/// [`VoiceHandler::release`](trait.VoiceHandler.html#tymethod.release) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn voice_handler_release(
    adapter: *mut PluginAdapter,
    voice: *mut &mut dyn Voice,
) {
    // We don't call Box::from_raw because:
    // 1. Host calls this then voice_handler_kill — this way we'll get double deallocation
    // 2. Given FL SDK documentation, we shouldn't deallocate voices here
    (*adapter).0.voice_handler().release((*voice).tag());
}

/// [`VoiceHandler::kill`](trait.VoiceHandler.html#tymethod.kill) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn voice_handler_kill(
    adapter: *mut PluginAdapter,
    voice: *mut &mut dyn Voice,
) {
    let r_voice = Box::from_raw(voice);
    (*adapter).0.voice_handler().kill(r_voice.tag());
}

/// [`VoiceHandler::kill_out`](trait.VoiceHandler.html#tymethod.kill_out) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn out_voice_handler_kill(
    adapter: *mut PluginAdapter,
    voice: *mut &mut dyn Voice,
) {
    let r_voice = Box::from_raw(voice);
    let handler = (*adapter).0.voice_handler();
    if let Some(out_handler) = handler.out_handler() {
        out_handler.kill(r_voice.tag());
    }
}

/// [`VoiceHandler::on_event`](trait.VoiceHandler.html#tymethod.on_event) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn voice_handler_on_event(
    adapter: *mut PluginAdapter,
    voice: *mut &mut dyn Voice,
    message: ffi::Message,
) -> intptr_t {
    (*adapter)
        .0
        .voice_handler()
        .on_event((*voice).tag(), message.into())
        .as_raw_ptr()
}

/// [`OutVoiceHandler::on_event`](trait.OutVoiceHandler.html#method.on_event) FFI.
///
/// It supposed to be used internally. Don't use it.
///
/// # Safety
///
/// Unsafe
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn out_voice_handler_on_event(
    adapter: *mut PluginAdapter,
    voice: *mut &mut dyn Voice,
    message: ffi::Message,
) -> intptr_t {
    let handler = (*adapter).0.voice_handler();
    match handler.out_handler() {
        Some(out_handler) => out_handler
            .on_event((*voice).tag(), message.into())
            .as_raw_ptr(),
        None => Box::new(0).as_raw_ptr(),
    }
}
