use fpsdk::{create_plugin, Host, HostMessage, Info, InfoBuilder, Plugin, DispatcherResult};

#[derive(Default)]
struct Test {
    host: Option<Host>,
    tag: Option<i32>,
}

impl Plugin for Test {
    fn new() -> Self {
        Test::default()
    }

    fn info(&self) -> Info {
        InfoBuilder::new_effect("Simple", "Simple", 1).build()
    }

    fn create_instance(&mut self, host: Host, tag: i32) {
        self.host = Some(host);
        self.tag = Some(tag);
    }

    fn on_message(&mut self, message: HostMessage) -> Box<dyn DispatcherResult> {
        Box::new(0)
    }
}

create_plugin!(Test);
