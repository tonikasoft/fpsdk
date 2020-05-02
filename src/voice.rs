//! Voices used by generators to track events like their instantiation, release, freeing and
//! processing some events.

use crate::plugin::PluginAdapter;
use crate::{ffi, intptr_t, Tag};

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
    fn on_event(&mut self, _tag: Tag, _event: Event) {}
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
    Box::into_raw(Box::new(handler.trigger(params, tag))) as intptr_t
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
) {
    (*adapter)
        .0
        .voice_handler()
        .on_event((*voice).tag(), message.into());
}
