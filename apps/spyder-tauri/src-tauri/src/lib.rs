//! Spyder Tauri shell — spawn `spyder-gui` and open the web UI.

use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{Manager, RunEvent};

const API_PORT: u16 = 7700;

struct Backend(Mutex<Option<Child>>);

fn resolve_spyder_gui() -> PathBuf {
    if let Ok(path) = std::env::var("SPYDER_GUI_BIN") {
        return PathBuf::from(path);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "../../../target/debug/spyder-gui",
        "../../../target/release/spyder-gui",
    ] {
        let candidate = manifest.join(rel);
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("spyder-gui")
}

fn spawn_backend() -> Result<Child, String> {
    let bin = resolve_spyder_gui();
    Command::new(&bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {e}", bin.display()))
}

fn wait_for_port(port: u16, timeout: Duration) -> Result<(), String> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "spyder-gui did not open port {port} within {:?}",
        timeout
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut child = match spawn_backend() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            eprintln!("Build with: cargo build -p spyder-gui");
            std::process::exit(1);
        }
    };

    if let Err(e) = wait_for_port(API_PORT, Duration::from_secs(30)) {
        eprintln!("{e}");
        let _ = child.kill();
        std::process::exit(1);
    }

    let app = tauri::Builder::default()
        .manage(Backend(Mutex::new(Some(child))))
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("tauri build error: {e}");
            std::process::exit(1);
        });

    app.run(|app_handle, event| {
        if let RunEvent::Exit = event {
            if let Some(state) = app_handle.try_state::<Backend>() {
                if let Ok(mut guard) = state.0.lock() {
                    if let Some(mut child) = guard.take() {
                        let _ = child.kill();
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_path_does_not_panic() {
        let _ = resolve_spyder_gui();
    }
}
