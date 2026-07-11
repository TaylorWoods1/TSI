//! Spyder Tauri shell — spawn `spyder-gui` sidecar and open the web UI.

use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, RunEvent,
};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

const API_PORT: u16 = 7700;

enum BackendChild {
    Std(Child),
    Sidecar {
        _rx: tauri::async_runtime::Receiver<CommandEvent>,
        child: CommandChild,
    },
}

impl BackendChild {
    fn kill(self) {
        match self {
            Self::Std(mut child) => {
                let _ = child.kill();
            }
            Self::Sidecar { child, .. } => {
                let _ = child.kill();
            }
        }
    }
}

struct Backend(Mutex<Option<BackendChild>>);

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

fn web_dist_for_sidecar(app: &AppHandle) -> Option<PathBuf> {
    let resource = app.path().resource_dir().ok()?;
    let bundled = resource.join("web/dist");
    if bundled.exists() {
        return Some(bundled);
    }
    let flat = resource.join("dist");
    if flat.exists() {
        return Some(flat);
    }
    None
}

fn spawn_backend(app: &AppHandle) -> Result<BackendChild, String> {
    if let Ok(cmd) = app.shell().sidecar("spyder-gui") {
        let mut command = cmd;
        if let Some(dist) = web_dist_for_sidecar(app) {
            command = command.env("SPYDER_WEB_DIST", dist);
        }
        let (rx, child) = command
            .spawn()
            .map_err(|e| format!("failed to spawn spyder-gui sidecar: {e}"))?;
        return Ok(BackendChild::Sidecar { _rx: rx, child });
    }

    let bin = resolve_spyder_gui();
    Command::new(&bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(BackendChild::Std)
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

fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn shutdown_backend(app: &AppHandle) {
    if let Some(state) = app.try_state::<Backend>() {
        if let Ok(mut guard) = state.0.lock() {
            if let Some(child) = guard.take() {
                child.kill();
            }
        }
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let Some(icon) = app.default_window_icon().cloned() else {
        return Ok(());
    };
    let quit = MenuItem::with_id(app, "quit", "Quit Spyder", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;
    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Spyder")
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                shutdown_backend(app);
                app.exit(0);
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_shell::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main_window(app);
        }));
    }

    let app = builder
        .setup(|app| {
            let child = spawn_backend(app.handle()).map_err(|e| {
                eprintln!("{e}");
                eprintln!("Run: bash apps/spyder-tauri/scripts/prepare-sidecar.sh release");
                e
            })?;

            if let Err(e) = wait_for_port(API_PORT, Duration::from_secs(30)) {
                child.kill();
                return Err(e.into());
            }

            app.manage(Backend(Mutex::new(Some(child))));
            setup_tray(app.handle())?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("tauri build error: {e}");
            std::process::exit(1);
        });

    app.run(|app_handle, event| {
        if let RunEvent::Exit = event {
            shutdown_backend(app_handle);
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
