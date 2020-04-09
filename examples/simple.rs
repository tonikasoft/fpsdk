use fpsdk::{create_plugin, DispatcherResult, Host, HostMessage, Info, InfoBuilder, Plugin};
use std::cell::RefCell;
use std::panic::AssertUnwindSafe;

struct Test {
    host: Host,
    tag: i32,
    data: AssertUnwindSafe<RefCell<i32>>,
}

impl Plugin for Test {
    fn new(host: Host, tag: i32) -> Self {
        Self {
            host,
            tag,
            data: AssertUnwindSafe(RefCell::new(10)),
        }
    }

    fn info(&self) -> Info {
        InfoBuilder::new_effect("Simple", "Simple", 1).build()
    }

    fn on_message(&mut self, message: HostMessage) -> Box<dyn DispatcherResult> {
        Box::new(0)
    }
}

create_plugin!(Test);
