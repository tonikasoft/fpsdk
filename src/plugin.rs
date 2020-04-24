//! Plugin related stuff.
use std::panic::RefUnwindSafe;

use crate::host::{Event, GetName, Host, HostMessage};
use crate::{AsRawPtr, Info, ProcessParamFlags, ValuePtr};

/// Plugin indentifier
pub type PluginTag = i32;

/// This trait must be implemented for your plugin.
pub trait Plugin: std::fmt::Debug + RefUnwindSafe + Send + Sync + 'static {
    /// Initializer
    fn new(host: Host, tag: PluginTag) -> Self
    where
        Self: Sized;
    /// Get plugin [`Info`](../struct.Info.html).
    fn info(&self) -> Info;
    /// Get plugin tag. You should store it when [`Plugin::new`](trait.Plugin.html#tymethod.new) is
    /// called.
    fn tag(&self) -> PluginTag;
    /// The host calls this function to request something that isn't done in a specialized
    /// function.
    ///
    /// See [`HostMessage`](../host/enum.HostMessage.html) for possible messages.
    ///
    /// Can be called from GUI or mixer threads.
    fn on_message(&mut self, message: HostMessage<'_>) -> Box<dyn AsRawPtr>;
    /// This is called when the host wants to know a text representation of some value.
    ///
    /// Can be called from GUI or mixer threads.
    fn name_of(&self, value: GetName) -> String;
    /// Process an event sent by the host.
    ///
    /// Can be called from GUI or mixer threads.
    fn process_event(&mut self, _event: Event) {}
    /// Gets called before a new tick is mixed (not played), if the plugin added
    /// [`PluginBuilder::want_new_tick`](../struct.InfoBuilder.html#method.want_new_tick) into
    /// [`Info`](../struct.Info.html).
    ///
    /// Internal controller plugins should call
    /// [`host::Host::on_control_change`](../host/struct.Host.html#method.on_control_change) from
    /// here.
    ///
    /// Called from mixer thread.
    fn tick(&mut self) {}
    /// **NOT USED YET, OMIT THIS METHOD**
    ///
    /// This is called before a new midi tick is played (not mixed).
    ///
    /// Can be called from GUI or mixer threads.
    fn midi_tick(&mut self) {}
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
    /// The processing function. The input buffer is empty for generator plugins.
    ///
    /// The buffers are in interlaced 32Bit float stereo format.
    ///
    /// Called from mixer thread.
    fn render(&mut self, _input: &[[f32; 2]], _output: &mut [[f32; 2]]) {}
}
