use crate::core::rdev_proc::IpcEvent;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, TryRecvError};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{env, thread};
use tauri::{AppHandle, Emitter};

static IS_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceKind {
    MousePress,
    MouseRelease,
    MouseMove,
    KeyboardPress,
    KeyboardRelease,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    pub kind: DeviceKind,
    pub value: String,
}

pub fn start_listening(app_handle: AppHandle) {
    if IS_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    IS_RUNNING.store(true, Ordering::SeqCst);

    thread::spawn(move || {
        start_child_loop(app_handle);
    });
}

fn start_child_loop(app_handle: AppHandle) {
    loop {
        let (server, server_name) = IpcOneShotServer::<IpcEvent>::new().unwrap();
        println!("Starting server on {}", server_name);

        let exe = env::current_exe().unwrap();
        let child = Command::new(&exe)
            .arg("--rdev")
            .arg(server_name)
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to spawn child");

        println!("started rdev proc");
        let (rx, _) = server.accept().unwrap();
        println!("rdev proc has connected");
        wait_rdev_exit(rx, child, app_handle.clone());
    }
}

fn wait_rdev_exit(rx: IpcReceiver<IpcEvent>, mut child: Child, app_handle: AppHandle) {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                eprintln!("子进程已退出，状态码：{}", status);
                break;
            }
            Ok(None) => match rx.try_recv_timeout(Duration::from_millis(100)) {
                Ok(ipc_event) => match ipc_event.action.as_str() {
                    "exit" => {
                        eprintln!("rdev exiting");
                        child.kill().unwrap();
                        break;
                    }
                    "rdev" => match ipc_event.device_event {
                        None => {
                            eprintln!("no device event");
                        }
                        Some(device_event) => {
                            if let Err(e) = app_handle.emit("device-changed", device_event) {
                                eprintln!("Failed to emit event: {:?}", e);
                            }
                        }
                    },
                    _ => {
                        eprintln!("unknown action: {}", ipc_event.action);
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(e) => {
                    eprintln!("Error receiving from child process: {}", e);
                    match child.try_wait() {
                        Ok(Some(_)) => { /* 子进程已经退出，无需kill */ }
                        Ok(None) => {
                            // 子进程还活着，需要 kill
                            match child.kill() {
                                Ok(_) => eprintln!("Force killed child due to channel error"),
                                Err(err) => eprintln!("Fail to kill child (maybe gone): {:?}", err),
                            }
                        }
                        Err(err) => {
                            eprintln!("don't need kill, 检测出错: {}", err)
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("无需 kill, 检测出错: {}", e);
                break;
            }
        }
    }
}
