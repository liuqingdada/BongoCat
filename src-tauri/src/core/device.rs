use ipc_channel::ipc::IpcOneShotServer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
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
        let (server, server_name) = IpcOneShotServer::<DeviceEvent>::new().unwrap();
        println!("Starting server on {}", server_name);
        let exe = env::current_exe().unwrap();
        let mut child = Command::new(&exe)
            .arg("--child")
            .arg(server_name)
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to spawn child");

        let (rx, _) = server.accept().unwrap();

        let (kill_tx, kill_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(60));
            let _ = kill_tx.send(());
        });

        loop {
            if kill_rx.try_recv().is_ok() {
                println!("Timeout, killing child...");
                let _ = child.kill();
                let _ = child.wait();
                println!("Child killed & collected");
                break;
            }

            match rx.try_recv_timeout(Duration::from_millis(2000)) {
                Ok(msg) => {
                    if let Err(e) = app_handle.emit("device-changed", msg) {
                        eprintln!("Failed to emit event: {:?}", e);
                    }
                }
                Err(e) => {
                    println!("Error receiving from child process: {}", e);
                    continue;
                }
            }
        }
    }
}
