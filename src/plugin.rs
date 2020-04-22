//! Plugin related stuff.
use std::panic::RefUnwindSafe;

use crate::host::{Host, HostMessage, GetName};
use crate::{DispatcherResult, Info};

/// Plugin indentifier
pub type PluginTag = i32;

/// This trait must be implemented for your plugin.
pub trait Plugin: std::fmt::Debug + RefUnwindSafe {
    /// Initializer
    fn new(host: Host, tag: PluginTag) -> Self
    where
        Self: Sized;
    /// Get plugin [`Info`](../struct.Info.html)
    fn info(&self) -> Info;
    /// Get plugin tag. You should store it when [`Plugin::new`](trait.Plugin.html#tymethod.new) is
    /// called.
    fn tag(&self) -> PluginTag;
    /// The host calls this function to request something that isn't done in a specialized
    /// function.
    ///
    /// See [`HostMessage`](../host/enum.HostMessage.html) for possible messages.
    fn on_message(&mut self, message: HostMessage<'_>) -> Box<dyn DispatcherResult>;
    /// This is called when the host wants to know a text representation of some value.
    fn name_of(&self, value: GetName) -> String;
}
