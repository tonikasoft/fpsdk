use fpsdk::{create_plugin, Host, Info, InfoBuilder, Plugin};

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
        InfoBuilder::new_effect("Simple Rs", "Simple Rs", 1).build()
    }

    fn create_instance(&mut self, host: Host, tag: i32) {
        self.host = Some(host);
        self.tag = Some(tag);
    }
}

create_plugin!(Test);
