use crate::core::device::{DeviceEvent, DeviceKind};
use ipc_channel::ErrorKind;
use ipc_channel::ipc::IpcSender;
use rdev::{Event, EventType, listen};
use serde_json::json;
use std::cell::Cell;
use std::env;

pub fn run_child() {
    let args: Vec<String> = env::args().collect();
    let server_name = args.get(2).expect("no server name").to_string();
    let tx: IpcSender<DeviceEvent> = IpcSender::connect(server_name).unwrap();
    let fail_count = Cell::new(0);
    let max_fail = 5;

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
        match tx.send(device) {
            Ok(_) => fail_count.set(0),
            Err(e) => {
                let kind = e.as_ref();
                let is_disconnected = match kind {
                    ErrorKind::Io(io_err) => {
                        use std::io::ErrorKind as Ek;
                        matches!(io_err.kind(), Ek::BrokenPipe | Ek::ConnectionReset)
                    }
                    _ => false,
                };
                if is_disconnected {
                    eprintln!("检测到主进程IPC关闭(BrokenPipe/ConnectionReset)，子进程自杀");
                    std::process::exit(1);
                }
                fail_count.set(fail_count.get() + 1);
                eprintln!("IPC发送失败{}/{}： {:?}", fail_count.get(), max_fail, e);
                if fail_count.get() > max_fail {
                    eprintln!("发送失败次数到达上限，子进程自杀退出");
                    std::process::exit(1);
                }
            }
        }
    };

    if let Err(e) = listen(callback) {
        eprintln!("Device listening error: {:?}", e);
        std::process::exit(2);
    }
}
