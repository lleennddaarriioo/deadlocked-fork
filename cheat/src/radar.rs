use std::{
    path::PathBuf,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use shared::data::Data;
use tungstenite::{Message, connect};
use utils::Mutex;

pub static RADAR_LOCAL_LINK: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
pub static RADAR_PUBLIC_LINK: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
static RADAR_THREAD_STARTED: AtomicBool = AtomicBool::new(false);
static SERVER_SPAWN_ATTEMPTED: AtomicBool = AtomicBool::new(false);

fn find_radar_dir() -> Option<PathBuf> {
    if std::path::Path::new("radar/server").exists() {
        return Some(PathBuf::from("radar/server"));
    }
    if std::path::Path::new("../radar/server").exists() {
        return Some(PathBuf::from("../radar/server"));
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe;
        for _ in 0..4 {
            if cur.pop() {
                let candidate = cur.join("radar/server");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn extract_cloudflare_url(content: &str) -> Option<String> {
    for line in content.lines().rev() {
        if let Some(start_idx) = line.find("https://") {
            let rest = &line[start_idx..];
            if let Some(end_domain) = rest.find(".trycloudflare.com") {
                let end_idx = end_domain + ".trycloudflare.com".len();
                let url = &rest[..end_idx];
                if url.starts_with("https://") && url.ends_with(".trycloudflare.com") {
                    return Some(url.to_string());
                }
            }
        }
    }
    None
}

fn is_cloudflared_running() -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg("cloudflared")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn restart_cloudflared() {
    utils::info!("Restarting Cloudflare tunnel...");
    let _ = Command::new("killall").arg("cloudflared").status();
    let _ = std::fs::remove_file("/tmp/cloudflared.log");
    if let Ok(mut p) = RADAR_PUBLIC_LINK.lock() {
        *p = "Fetching...".to_owned();
    }
    ensure_cloudflared_running();
}

pub fn ensure_cloudflared_running() {
    let local_url = "http://127.0.0.1:6346/".to_string();
    if let Ok(mut l) = RADAR_LOCAL_LINK.lock() {
        *l = local_url.clone();
    }

    let is_running = is_cloudflared_running();

    if !is_running {
        utils::info!("Spawning cloudflared tunnel...");
        let _ = std::fs::remove_file("/tmp/cloudflared.log");
        if let Ok(mut p) = RADAR_PUBLIC_LINK.lock() {
            *p = "Fetching...".to_owned();
        }

        let cmd = "export PATH=\"$HOME/.local/bin:$PATH\"; killall cloudflared 2>/dev/null || true; > /tmp/cloudflared.log; (cloudflared tunnel --url http://127.0.0.1:6346 || ~/.local/bin/cloudflared tunnel --url http://127.0.0.1:6346) > /tmp/cloudflared.log 2>&1 &";
        let _ = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn();
    }

    let start_parsing = {
        if let Ok(mut p) = RADAR_PUBLIC_LINK.lock() {
            if p.is_empty() {
                *p = "Fetching...".to_owned();
                true
            } else if p.as_str() == "Fetching..." {
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if start_parsing {
        std::thread::spawn(move || {
            for _ in 0..45 {
                if let Ok(content) = std::fs::read_to_string("/tmp/cloudflared.log") {
                    if let Some(url) = extract_cloudflare_url(&content) {
                        let public_url = format!("{}/", url);
                        utils::info!("RADAR PUBLIC LINK: {}", public_url);
                        if let Ok(mut p) = RADAR_PUBLIC_LINK.lock() {
                            *p = public_url.clone();
                        }
                        let _ = std::fs::write(
                            "radar_link.txt",
                            format!("Local Link:\n{}\n\nPublic Link:\n{}\n", local_url, public_url),
                        );
                        return;
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            utils::info!("RADAR PUBLIC LINK: Cloudflare tunnel log timed out");
            if let Ok(mut p) = RADAR_PUBLIC_LINK.lock() {
                if p.as_str() == "Fetching..." {
                    *p = "Cloudflare tunnel not found (use Local link)".to_owned();
                }
            }
        });
    }
}

fn auto_spawn_server() {
    if SERVER_SPAWN_ATTEMPTED.swap(true, Ordering::SeqCst) {
        return;
    }

    let Some(radar_dir) = find_radar_dir() else {
        utils::error!("Could not find radar/server directory");
        return;
    };

    utils::info!("Attempting to auto-spawn radar server in {:?}", radar_dir);

    let mut spawned = false;
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let bin = parent.join("server");
            if bin.exists() {
                utils::info!("Launching radar server binary: {:?}", bin);
                if Command::new(&bin).current_dir(&radar_dir).spawn().is_ok() {
                    spawned = true;
                }
            }
        }
    }

    if !spawned {
        utils::info!("Launching radar server via cargo run in {:?}", radar_dir);
        let _ = Command::new("cargo")
            .args(["run", "--bin", "server", "--release"])
            .current_dir(&radar_dir)
            .spawn();
    }

    ensure_cloudflared_running();
}

pub fn start_radar_thread(data: Arc<Mutex<Data>>) {
    if RADAR_THREAD_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(move || {
        loop {
            let Ok((mut socket, _)) = connect("ws://127.0.0.1:6346/server") else {
                if let Ok(mut l) = RADAR_LOCAL_LINK.lock() {
                    if l.is_empty() {
                        *l = "Starting server...".to_owned();
                    }
                }
                auto_spawn_server();
                sleep(Duration::from_secs(1));
                continue;
            };

            SERVER_SPAWN_ATTEMPTED.store(false, Ordering::SeqCst);

            let local_url = "http://127.0.0.1:6346/".to_string();
            if let Ok(mut l) = RADAR_LOCAL_LINK.lock() {
                *l = local_url.clone();
            }

            ensure_cloudflared_running();

            let mut tick_counter = 0u32;
            loop {
                tick_counter = tick_counter.wrapping_add(1);
                if tick_counter % 500 == 0 {
                    ensure_cloudflared_running();
                }

                let data_clone = { data.lock().clone() };

                let Ok(json) = serde_json::to_string(&data_clone) else {
                    break;
                };

                if socket.send(Message::Text(json.into())).is_err() {
                    break;
                }

                sleep(Duration::from_millis(8));
            }
        }
    });
}
