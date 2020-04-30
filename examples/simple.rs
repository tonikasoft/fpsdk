#[cfg(unix)]
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

use bincode;
use log::{error, info, trace, LevelFilter};
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use simple_logging;
#[cfg(unix)]
use simplelog::{ConfigBuilder, WriteLogger};

use fpsdk::host::{Event, GetName, Host, HostMessage};
use fpsdk::plugin::{Info, InfoBuilder, Plugin, PluginTag, StateReader, StateWriter};
use fpsdk::{create_plugin, AsRawPtr, MidiMessage, ProcessParamFlags, ValuePtr};

static ONCE: Once = Once::new();
const LOG_PATH: &str = "simple.log";

#[derive(Debug)]
struct Test {
    host: Host,
    tag: PluginTag,
    param_names: Vec<String>,
    state: State,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct State {
    _time: u64,
    _param_1: f64,
    _param_2: i64,
}

impl Plugin for Test {
    fn new(host: Host, tag: i32) -> Self {
        init_log();

        info!("init plugin with tag {}", tag);

        Self {
            host,
            tag,
            param_names: vec![
                "Parameter 1".into(),
                "Parameter 2".into(),
                "Parameter 3".into(),
            ],
            state: State::default(),
        }
    }

    fn info(&self) -> Info {
        info!("plugin {} will return info", self.tag);

        InfoBuilder::new_effect("Simple", "Simple", self.param_names.len() as u32)
            .want_new_tick()
            .build()
    }

    fn tag(&self) -> PluginTag {
        self.tag
    }

    fn save_state(&mut self, writer: StateWriter) {
        let now = SystemTime::now();
        let time = now.duration_since(UNIX_EPOCH).expect("").as_secs();
        self.state._time = time;
        self.state._param_1 = time as f64 * 0.001;
        self.state._param_2 = time as i64 / 2;
        match bincode::serialize_into(writer, &self.state) {
            Ok(_) => info!("state {:?} saved", self.state),
            Err(e) => error!("error serializing state {}", e),
        }
    }

    fn load_state(&mut self, mut reader: StateReader) {
        let mut buf = [0; std::mem::size_of::<State>()];
        reader
            .read(&mut buf)
            .and_then(|_| {
                bincode::deserialize::<State>(&buf).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("error deserializing value {}", e),
                    )
                })
            })
            .and_then(|value| {
                self.state = value;
                Ok(info!("read state {:?}", self.state))
            })
            .unwrap_or_else(|e| error!("error reading value from state {}", e));
    }

    fn on_message(&mut self, message: HostMessage) -> Box<dyn AsRawPtr> {
        info!("{} got message from host: {:?}", self.tag, message);

        Box::new(0)
    }

    fn name_of(&self, message: GetName) -> String {
        info!("{} host asks name of {:?}", self.tag, message);

        match message {
            GetName::Param(index) => self.param_names[index].clone(),
            _ => "What?".into(),
        }
    }

    fn process_event(&mut self, event: Event) {
        info!("{} host sends event {:?}", self.tag, event);
    }

    fn tick(&mut self) {
        trace!("{} receive new tick", self.tag);
    }

    fn idle(&mut self) {
        trace!("{} idle", self.tag);
    }

    fn process_param(
        &mut self,
        index: usize,
        value: ValuePtr,
        flags: ProcessParamFlags,
    ) -> Box<dyn AsRawPtr> {
        info!(
            "{} process param: index {}, value {}, flags {:?}",
            self.tag,
            index,
            value.get::<i32>(),
            flags
        );
        Box::new(0)
    }

    fn midi_in(&mut self, message: MidiMessage) {
        trace!("receive MIDI message {:?}", message);
    }

    fn render(&mut self, input: &[[f32; 2]], output: &mut [[f32; 2]]) {
        input.iter().zip(output).for_each(|(inp, outp)| {
            outp[0] = inp[0] * 0.25;
            outp[1] = inp[1] * 0.25;
        });
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
