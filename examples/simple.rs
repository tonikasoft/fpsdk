use std::collections::HashMap;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::sync::{Arc, Mutex, Once};
use std::time::{SystemTime, UNIX_EPOCH};

use bincode;
use log::{error, info, trace, LevelFilter};
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use simple_logging;
#[cfg(unix)]
use simplelog::{ConfigBuilder, WriteLogger};

use fpsdk::host::{self, Event, GetName, Host, OutVoicer, Voicer};
use fpsdk::plugin::message;
use fpsdk::plugin::{self, Info, InfoBuilder, Plugin, StateReader, StateWriter};
use fpsdk::voice::{self, ReceiveVoiceHandler, SendVoiceHandler, Voice};
use fpsdk::{
    create_plugin, AsRawPtr, MessageBoxFlags, MidiMessage, Note, Notes, NotesFlags,
    ProcessParamFlags, TimeFormat, ValuePtr,
};

static ONCE: Once = Once::new();
const LOG_PATH: &str = "simple.log";

#[derive(Debug)]
struct Simple {
    host: Host,
    tag: plugin::Tag,
    param_names: Vec<String>,
    state: State,
    voice_handler: SimpleVoiceHandler,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct State {
    _time: u64,
    _param_1: f64,
    _param_2: i64,
}

impl Simple {
    fn add_notes(&mut self) {
        let notes = Notes {
            notes: vec![
                Note {
                    position: 0,
                    length: 384,
                    pan: 0,
                    vol: 100,
                    note: 60,
                    color: 0,
                    pitch: 0,
                    mod_x: 1.0,
                    mod_y: 1.0,
                },
                Note {
                    position: 384,
                    length: 384,
                    pan: 0,
                    vol: 100,
                    note: 62,
                    color: 0,
                    pitch: 0,
                    mod_x: 1.0,
                    mod_y: 1.0,
                },
                Note {
                    position: 768,
                    length: 384,
                    pan: 0,
                    vol: 100,
                    note: 64,
                    color: 0,
                    pitch: 0,
                    mod_x: 1.0,
                    mod_y: 1.0,
                },
            ],
            flags: NotesFlags::EMPTY_FIRST,
            pattern: None,
            channel: None,
        };
        self.host
            .on_message(self.tag, message::AddToPianoRoll(notes));
    }

    fn show_annoying_message(&mut self) {
        self.host.on_message(
            self.tag,
            message::MessageBox(
                "Message".to_string(),
                "This message is shown when plugin is enabled. \
                Feel free to comment this out if it annoys you."
                    .to_string(),
                MessageBoxFlags::OK | MessageBoxFlags::ICONINFORMATION,
            ),
        );
    }

    fn log_selection(&mut self) {
        let selection = self
            .host
            .on_message(self.tag, message::GetSelTime(TimeFormat::Beats));
        self.host.on_message(
            self.tag,
            message::DebugLogMsg(format!(
                "current selection or full song range is: {:?}",
                selection
            )),
        );
    }

    fn say_hello_hint(&mut self) {
        self.host.on_hint(self.tag, "^c Hello".to_string());
    }
}

impl Plugin for Simple {
    fn new(host: Host, tag: plugin::Tag) -> Self {
        init_log();

        info!("init plugin with tag {}", tag);

        let voice_h = host.voice_handler();
        let out_voice_h = host.out_voice_handler();

        Self {
            voice_handler: SimpleVoiceHandler::new(voice_h, out_voice_h),
            host,
            tag,
            param_names: vec![
                "Parameter 1".into(),
                "Parameter 2".into(),
                "Parameter 3".into(),
            ],
            state: Default::default(),
        }
    }

    fn info(&self) -> Info {
        info!("plugin {} will return info", self.tag);

        InfoBuilder::new_full_gen("Simple", "Simple", self.param_names.len() as u32)
            // InfoBuilder::new_effect("Simple", "Simple", self.param_names.len() as u32)
            // .want_new_tick()
            // Looks like MIDI out doesn't work :(
            // https://forum.image-line.com/viewtopic.php?f=100&t=199371
            // https://forum.image-line.com/viewtopic.php?f=100&t=199258
            .with_out_voices(1)
            .midi_out()
            .build()
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

    fn on_message(&mut self, message: host::Message) -> Box<dyn AsRawPtr> {
        self.host.on_message(
            self.tag,
            message::DebugLogMsg(format!("{} got message from host: {:?}", self.tag, message)),
        );

        if let host::Message::SetEnabled(enabled) = message {
            self.add_notes();
            self.log_selection();
            self.say_hello_hint();

            if enabled {
                self.show_annoying_message();
                // self.host.on_message(self.tag, message::ActivateMidi);
            }

            self.host
                .on_parameter(self.tag, 0, ValuePtr::new(0.123456789_f32.as_raw_ptr()));
        }

        // self.host.midi_out(self.tag, MidiMessage {
            // status: 0x90,
            // data1: 60,
            // data2: 100,
            // port: 2,
        // });

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

    // looks like doesn't work
    fn loop_in(&mut self, message: ValuePtr) {
        trace!("{:?} loop_in", message);
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
            value.get::<f32>(),
            flags
        );
        Box::new(0)
    }

    fn midi_in(&mut self, message: MidiMessage) {
        trace!("receive MIDI message {:?}", message);
    }

    fn render(&mut self, input: &[[f32; 2]], output: &mut [[f32; 2]]) {
        if self.voice_handler.voices.len() < 1 {
            // consider it an effect
            input.iter().zip(output).for_each(|(inp, outp)| {
                outp[0] = inp[0] * 0.25;
                outp[1] = inp[1] * 0.25;
            });
        }
    }

    fn voice_handler(&mut self) -> Option<&mut dyn ReceiveVoiceHandler> {
        Some(&mut self.voice_handler)
    }
}

#[derive(Debug)]
struct SimpleVoiceHandler {
    voices: HashMap<voice::Tag, SimpleVoice>,
    out_handler: SimpleOutVoiceHandler,
    send_handler: Arc<Mutex<Voicer>>,
    send_out_handler: Arc<Mutex<OutVoicer>>,
}

impl SimpleVoiceHandler {
    fn new(send_handler: Arc<Mutex<Voicer>>, send_out_handler: Arc<Mutex<OutVoicer>>) -> Self {
        Self {
            voices: HashMap::new(),
            out_handler: SimpleOutVoiceHandler::default(),
            send_handler,
            send_out_handler,
        }
    }
}

impl SimpleVoiceHandler {
    fn log_velocity(&self, tag: voice::Tag) {
        let mut send_handler = self.send_handler.lock().unwrap();
        if let Some(velocity) = send_handler.on_event(tag, voice::Event::GetVelocity) {
            trace!("get velocity {} for voice {}", velocity.get::<f32>(), tag);
        }
    }

    fn log_color(&self, tag: voice::Tag) {
        let mut send_handler = self.send_handler.lock().unwrap();
        if let Some(color) = send_handler.on_event(tag, voice::Event::GetColor) {
            trace!("get color {} for voice {}", color.get::<u8>(), tag);
        }
    }
}

impl ReceiveVoiceHandler for SimpleVoiceHandler {
    fn trigger(&mut self, params: voice::Params, tag: voice::Tag) -> &mut dyn Voice {
        let voice = SimpleVoice::new(params.clone(), tag);
        trace!("trigger voice {:?}", voice);
        self.voices.insert(tag, voice);

        let mut send_out_handler = self.send_out_handler.lock().unwrap();

        send_out_handler.trigger(params, 0, tag);

        self.log_velocity(tag);
        self.log_color(tag);

        self.voices.get_mut(&tag).unwrap()
    }

    fn release(&mut self, tag: voice::Tag) {
        trace!("release voice {:?}", self.voices.get(&tag));
        self.send_out_handler.lock().unwrap().release(tag);
        trace!("send kill voice {}", tag);
        self.send_handler.lock().unwrap().kill(tag);
    }

    fn kill(&mut self, tag: voice::Tag) {
        trace!("host wants to kill voice with tag {}", tag);
        trace!("kill voice {:?}", self.voices.remove(&tag));
        trace!(
            "remaining voices count {}, {:?}",
            self.voices.len(),
            self.voices
        );
    }

    fn on_event(&mut self, tag: voice::Tag, event: voice::Event) -> Box<dyn AsRawPtr> {
        trace!("event {:?} for voice {:?}", event, self.voices.get(&tag));
        Box::new(0)
    }

    fn out_handler(&mut self) -> Option<&mut dyn SendVoiceHandler> {
        Some(&mut self.out_handler)
    }
}

#[derive(Debug)]
struct SimpleVoice {
    tag: voice::Tag,
    params: voice::Params,
}

impl SimpleVoice {
    pub fn new(params: voice::Params, tag: voice::Tag) -> Self {
        Self { tag, params }
    }
}

impl Voice for SimpleVoice {
    fn tag(&self) -> voice::Tag {
        self.tag
    }
}

#[derive(Debug, Default)]
struct SimpleOutVoiceHandler;

impl SendVoiceHandler for SimpleOutVoiceHandler {
    fn kill(&mut self, tag: voice::Tag) {
        trace!("kill out voice with tag {}", tag);
    }

    fn on_event(&mut self, tag: voice::Tag, event: voice::Event) -> Option<ValuePtr> {
        trace!("event {:?} on out voice {}", event, tag);
        None
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
    simple_logging::log_to_file(LOG_PATH, LevelFilter::Trace).unwrap();
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
    let _ = WriteLogger::init(LevelFilter::Trace, config, file).unwrap();
}

create_plugin!(Simple);
