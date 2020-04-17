use std::cell::RefCell;
use std::fs::OpenOptions;
use std::panic::AssertUnwindSafe;

use log::info;
use simplelog::{Config, LevelFilter, WriteLogger};

use fpsdk::{create_plugin, DispatcherResult, Host, HostMessage, Info, InfoBuilder, Plugin};

const LOG_PATH: &str = "simple.log";

struct Test {
    host: Host,
    tag: i32,
    data: AssertUnwindSafe<RefCell<i32>>,
}

impl Plugin for Test {
    fn new(host: Host, tag: i32) -> Self {
        init_log();

        info!("init plugin with tag {}", tag);

        Self {
            host,
            tag,
            data: AssertUnwindSafe(RefCell::new(10)),
        }
    }

    fn info(&self) -> Info {
        info!("plugin will return info");

        InfoBuilder::new_effect("Simple", "Simple", 1).build()
    }

    fn on_message(&mut self, message: HostMessage) -> Box<dyn DispatcherResult> {
        info!("get message from host: {:?}", message);

        Box::new(0)
    }
}

fn init_log() {
    // the file is created at FL's resources root directory
    // for macOS it's /Applications/FL Studio 20.app/Contents/Resources/FL
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(LOG_PATH)
        .unwrap();
    let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), file);
}

create_plugin!(Test);
