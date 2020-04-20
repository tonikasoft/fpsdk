use std::cell::RefCell;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::panic::AssertUnwindSafe;
use std::sync::Once;

use log::{info, LevelFilter};
use simple_logging;
#[cfg(unix)]
use simplelog::{ConfigBuilder, WriteLogger};

use fpsdk::{create_plugin, DispatcherResult, Host, HostMessage, Info, InfoBuilder, Plugin};

static ONCE: Once = Once::new();
const LOG_PATH: &str = "simple.log";

#[derive(Debug)]
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
        info!("plugin {} will return info", self.tag);

        InfoBuilder::new_effect("Simple", "Simple", 1).build()
    }

    fn on_message(&mut self, message: HostMessage) -> Box<dyn DispatcherResult> {
        info!("{} get message from host: {:?}", self.tag, message);

        Box::new(0)
    }
}

fn init_log() {
    ONCE.call_once(|| {
        _init_log();
        info!("init log");
    });
}

#[cfg(windows)]
fn _init_log() {
    simple_logging::log_to_file(LOG_PATH, LevelFilter::Debug).unwrap();
}

#[cfg(unix)]
fn _init_log() {
    // the file is created at FL's resources root directory
    // for macOS it's /Applications/FL Studio 20.app/Contents/Resources/FL
    // for Windows it's <Drive>:\Program Files\Image-Line\FL Studio 20
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(LOG_PATH)
        .unwrap();
    let config = ConfigBuilder::new().set_time_to_local(true).build();
    let _ = WriteLogger::init(LevelFilter::Debug, config, file).unwrap();
}

create_plugin!(Test);
