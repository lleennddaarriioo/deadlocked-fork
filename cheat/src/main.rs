use std::sync::Arc;

use shared::Data;
use utils::{
    Channel, Mutex,
    log::{Level, LoggerOptions},
};
use winit::platform::x11::EventLoopBuilderExtX11;

use crate::{config::BASE_PATH, os::mouse::check_uinput, ui::app::App};

mod config;
mod constants;
mod cs2;
mod font;
mod game;
mod math;
mod message;
mod os;
mod parser;
mod profiler;
mod radar;
mod ui;
mod update;

#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported.");

fn main() {
    let debug = std::env::args().any(|arg| arg == "--debug");
    utils::log::init(
        LoggerOptions::default()
            .level(if debug { Level::Debug } else { Level::Info })
            .file(BASE_PATH.join("deadlocked.log"))
            .truncate(true),
        |w, rec| {
            writeln!(
                w,
                "[{}] [{}:{}] {}",
                rec.level, rec.location.file, rec.location.line, rec.args
            )
        },
    )
    .expect("failed to initialize logger");

    let args: Vec<String> = std::env::args().collect();
    let demo_mode = args.iter().any(|a| a == "--demo-screenshots");
    let test_bvh_mode = args.iter().any(|a| a == "--test-bvh");

    if test_bvh_mode {
        println!("[test-bvh] Initializing CS2 memory reader...");
        let mut cs2 = cs2::CS2::new();
        cs2.setup();
        if cs2.is_valid() {
            println!("[test-bvh] CS2 process found! PID: {}", cs2.process.pid);
            println!("[test-bvh] vphys_world offset: {:#x}", cs2.offsets.direct.vphys_world);
            println!("[test-bvh] Attempting read_map()...");
            if let Some(bvh) = parser::read_map(&cs2) {
                println!("[test-bvh] SUCCESS! Loaded BVH map geometry with {} triangles!", bvh.all_triangles().len());
                std::process::exit(0);
            } else {
                println!("[test-bvh] FAIL: read_map() returned None.");
                std::process::exit(1);
            }
        } else {
            println!("[test-bvh] CS2 process not found or invalid!");
            std::process::exit(1);
        }
    }

    if !demo_mode && !check_uinput() {
        utils::warn!("uinput device is missing or unreadable; starting deadlocked in Visuals Only / ESP Mode!");
    }

    let (channel_gui_game, channel_game) = Channel::new();
    let (channel_gui_radar, channel_radar) = Channel::new();
    let data = Arc::new(Mutex::new(Data::default()));
    let data_game = data.clone();
    let data_radar = data.clone();

    std::thread::spawn(move || {
        game::GameManager::new(channel_game, data_game).run();
    });

    std::thread::spawn(move || {
        radar::Radar::new(channel_radar, data_radar).run();
    });

    let event_loop = match winit::event_loop::EventLoop::builder().with_x11().build() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            utils::error!("failed to create event loop: {err}");
            return;
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    
    let mut app = App::new(channel_gui_game, channel_gui_radar, data);
    app.demo_mode = demo_mode;
    event_loop.run_app(&mut app).unwrap();
}
