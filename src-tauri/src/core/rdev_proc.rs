use crate::core::device::{DeviceEvent, DeviceKind};
use ipc_channel::ipc::IpcSender;
use rdev::{Event, EventType, listen};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::{
    Arc,
    mpsc::{Sender, channel},
};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub fn run_child() {
    let cat = Arc::new(CatSender::connect());
    cat.send_ipc_event(IpcEvent::default());
    cat.send_ipc_event(IpcEvent::default());
    cat.send_ipc_event(IpcEvent::default());
    cat.send_ipc_event(IpcEvent::default());

    let cat_clone = cat.clone();
    thread::spawn(move || {
        sleep(Duration::from_secs(60));
        cat_clone.send_ipc_event(IpcEvent {
            action: "exit".to_string(),
            device_event: None,
        });
    });

    let callback = move |event: Event| {
        let device = match event.event_type {
            EventType::ButtonPress(button) => DeviceEvent {
                kind: DeviceKind::MousePress,
                value: format!("{:?}", button),
            },
            EventType::ButtonRelease(button) => DeviceEvent {
                kind: DeviceKind::MouseRelease,
                value: format!("{:?}", button),
            },
            EventType::MouseMove { x, y } => DeviceEvent {
                kind: DeviceKind::MouseMove,
                value: json!({ "x": x, "y": y }).to_string(),
            },
            EventType::KeyPress(key) => DeviceEvent {
                kind: DeviceKind::KeyboardPress,
                value: format!("{:?}", key),
            },
            EventType::KeyRelease(key) => DeviceEvent {
                kind: DeviceKind::KeyboardRelease,
                value: format!("{:?}", key),
            },
            _ => return,
        };
        cat.send_ipc_event(IpcEvent {
            action: "rdev".to_string(),
            device_event: Some(device),
        });
    };

    if let Err(e) = listen(callback) {
        eprintln!("Device listening error: {:?}", e);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IpcEvent {
    pub action: String,
    pub device_event: Option<DeviceEvent>,
}

struct CatSender {
    sender: Arc<Sender<IpcEvent>>,
}

impl CatSender {
    fn connect() -> Self {
        let args: Vec<String> = env::args().collect();
        let server_name = args.get(2).expect("no server name").to_string();
        let ipc_sender: IpcSender<IpcEvent> = IpcSender::connect(server_name).unwrap();

        let (tx, rx) = channel::<IpcEvent>();
        thread::spawn(move || {
            for evt in rx {
                if let Err(e) = ipc_sender.send(evt) {
                    eprintln!("ipc_sender to server failed: {:?}", e);
                }
            }
        });
        Self {
            sender: Arc::new(tx),
        }
    }

    fn send_ipc_event(&self, event: IpcEvent) {
        let sender = self.sender.clone();
        if let Err(e) = sender.send(event) {
            eprintln!("failed to sync ipc_event: {:?}", e);
        }
    }
}
