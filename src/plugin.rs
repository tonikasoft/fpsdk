use std::panic::RefUnwindSafe;

use crate::host::{Host, HostMessage};
use crate::{DispatcherResult, Info};

/// This trait must be implemented for your plugin.
pub trait Plugin: std::fmt::Debug + RefUnwindSafe {
    /// Initializer
    fn new(host: Host, tag: i32) -> Self
    where
        Self: Sized;
    /// Get plugin [`Info`](struct.Info.html)
    fn info(&self) -> Info;
    /// The host calls this function to request something that isn't done in a specialized
    /// function.
    ///
    /// See [`HostMessage`](enum.HostMessage.html) for possible messages.
    fn on_message(&mut self, message: HostMessage<'_>) -> Box<dyn DispatcherResult>;
}
