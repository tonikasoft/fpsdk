use fpsdk::{create_plugin, Host, HostMessage, Info, InfoBuilder, Plugin, DispatcherResult};

struct Test {
    host: Host,
    tag: i32,
}

impl Plugin for Test {
    fn new(host: Host, tag: i32) -> Self {
        Self { host, tag }
    }

    fn info(&self) -> Info {
        InfoBuilder::new_effect("Simple", "Simple", 1).build()
    }

    fn on_message(&mut self, message: HostMessage) -> Box<dyn DispatcherResult> {
        Box::new(0)
    }
}

create_plugin!(Test);
