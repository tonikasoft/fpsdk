use std::cell::RefCell;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::panic::AssertUnwindSafe;
use std::sync::Once;

use log::{info, LevelFilter};
#[cfg(windows)]
use simple_logging;
#[cfg(unix)]
use simplelog::{ConfigBuilder, WriteLogger};

use fpsdk::host::{GetName, Host, HostMessage};
use fpsdk::plugin::{Plugin, PluginTag};
use fpsdk::{create_plugin, DispatcherResult, Info, InfoBuilder};

static ONCE: Once = Once::new();
const LOG_PATH: &str = "simple.log";

#[derive(Debug)]
struct Test {
    host: Host,
    tag: PluginTag,
    data: AssertUnwindSafe<RefCell<i32>>,
    param_names: Vec<String>,
}

impl Plugin for Test {
    fn new(host: Host, tag: i32) -> Self {
        init_log();

        info!("init plugin with tag {}", tag);

        Self {
            host,
            tag,
            data: AssertUnwindSafe(RefCell::new(10)),
            param_names: vec![
                "Parameter 1".into(),
                "Parameter 2".into(),
                "Parameter 3".into(),
            ],
        }
    }

    fn info(&self) -> Info {
        info!("plugin {} will return info", self.tag);

        InfoBuilder::new_effect("Simple", "Simple", self.param_names.len() as u32).build()
    }

    fn tag(&self) -> PluginTag {
        self.tag
    }

    fn on_message(&mut self, message: HostMessage) -> Box<dyn DispatcherResult> {
        info!("{} get message from host: {:?}", self.tag, message);

        Box::new(0)
    }

    fn name_of(&self, message: GetName) -> String {
        info!("{} host asks name of {:?}", self.tag, message);

        match message {
            GetName::Param(index) => self.param_names[index].clone(),
            _ => "What?".into(),
        }
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
        .append(true)
        .create(true)
        .open(LOG_PATH)
        .unwrap();
    let config = ConfigBuilder::new().set_time_to_local(true).build();
    let _ = WriteLogger::init(LevelFilter::Debug, config, file).unwrap();
}

create_plugin!(Test);
