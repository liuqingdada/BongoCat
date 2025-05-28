use std::env;
use ipc_channel::ipc::IpcSender;
use rdev::{listen, Event, EventType};
use serde_json::json;
use crate::core::device::{DeviceEvent, DeviceKind};

pub fn run_child() {
    let args: Vec<String> = env::args().collect();
    let server_name = args.get(2).expect("no server name").to_string();
    let tx: IpcSender<DeviceEvent> = IpcSender::connect(server_name).unwrap();

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
        if let Err(e) = tx.send(device) {
            eprintln!("failed to send device event: {:?}", e);
        }
    };

    if let Err(e) = listen(callback) {
        eprintln!("Device listening error: {:?}", e);
    }
}
