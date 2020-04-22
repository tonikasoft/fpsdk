//! Plugin related stuff.
use std::panic::RefUnwindSafe;

use log::warn;

use crate::host::{Event, GetName, Host, HostMessage};
use crate::{DispatcherResult, Info};

/// Plugin indentifier
pub type PluginTag = i32;

/// This trait must be implemented for your plugin.
pub trait Plugin: std::fmt::Debug + RefUnwindSafe {
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
    fn on_message(&mut self, message: HostMessage<'_>) -> Box<dyn DispatcherResult>;
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
    fn midi_tick(&mut self) {
        warn!("Host doesn't use this method.");
    }
}
