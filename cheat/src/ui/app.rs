use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use shared::{
    data::{Data, SoundType},
    weapon::Weapon,
};
use utils::{Channel, Mutex};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, StartCause, WindowEvent},
    keyboard::NamedKey,
};

use crate::{
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME,
        application::{ApplicationConfig, read_app_config},
        available_configs, parse_config, write_config,
    },
    message::{GameMessage, GameStatus, UiMessage},
    ui::{
        grenades::{Grenade, GrenadeList, read_grenades},
        gui::{Tab, aimbot::AimbotTab},
        trail::Trail,
        window_context::WindowContext,
    },
};

pub struct App {
    pub gui: Option<WindowContext>,
    pub overlay: Option<WindowContext>,
    next_frame_time: Instant,
    pub show_about: bool,

    pub channel: Channel<GameMessage, UiMessage>,
    pub data: Arc<Mutex<Data>>,

    pub game_status: GameStatus,
    pub display_scale: f32,
    pub trails: HashMap<u64, Trail>,
    pub player_sounds: HashMap<u64, (Instant, SoundType)>,
    pub frame_times: VecDeque<Duration>,

    pub grenades: GrenadeList,
    pub new_grenade: Grenade,
    pub current_grenade: Option<(String, usize)>,

    #[allow(dead_code)]
    pub app_config: ApplicationConfig,
    pub config: Config,
    pub current_config: PathBuf,
    pub available_configs: Vec<PathBuf>,
    pub new_config_name: String,

    pub current_tab: Tab,
    pub aimbot_tab: AimbotTab,
    pub aimbot_weapon: Weapon,
    
    pub hit_marker_time: Instant,
    pub last_total_damage: u32,
    pub last_pie_chart_update: Instant,
    pub smoothed_telemetry: Option<crate::game::TelemetryData>,
    pub max_monitor_hz: u32,

    pub thread_history_bhop: VecDeque<f32>,
    pub thread_history_aimbot: VecDeque<f32>,
    pub thread_history_trigger: VecDeque<f32>,
    pub thread_history_input: VecDeque<f32>,
    pub thread_history_bvh: VecDeque<f32>,
    pub thread_history_cache: VecDeque<f32>,
    pub thread_history_gui: VecDeque<f32>,
    pub thread_history_other: VecDeque<f32>,
    pub thread_history_loop: VecDeque<f32>,

    pub last_overlay_pos: Option<(i32, i32)>,
    pub last_overlay_size: Option<(u32, u32)>,

    pub demo_mode: bool,
    pub demo_last_step: Instant,
    pub demo_tab_idx: usize,
}

impl App {
    pub fn new(channel: Channel<GameMessage, UiMessage>, data: Arc<Mutex<Data>>) -> Self {
        // read config
        let config = parse_config(&CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        // override config if invalid
        write_config(&config, &CONFIG_PATH.join(DEFAULT_CONFIG_NAME));
        let grenades = read_grenades();

        let app_config = read_app_config();

        let ret = Self {
            gui: None,
            overlay: None,

            next_frame_time: Instant::now() + Duration::from_millis(16),
            show_about: false,

            channel,
            data,

            app_config,
            config,
            current_config: CONFIG_PATH.join(DEFAULT_CONFIG_NAME),
            available_configs: available_configs(),
            new_config_name: String::new(),

            game_status: GameStatus::NotStarted,
            display_scale: 1.0,
            trails: HashMap::new(),
            player_sounds: HashMap::new(),
            frame_times: VecDeque::with_capacity(500),

            grenades,
            new_grenade: Grenade::new(),
            current_grenade: None,

            current_tab: Tab::default(),
            aimbot_tab: AimbotTab::default(),
            aimbot_weapon: Weapon::default(),
            
            hit_marker_time: Instant::now() - Duration::from_secs(10),
            last_total_damage: 0,
            last_pie_chart_update: Instant::now(),
            smoothed_telemetry: None,
            max_monitor_hz: 240,

            thread_history_bhop: VecDeque::with_capacity(60),
            thread_history_aimbot: VecDeque::with_capacity(60),
            thread_history_trigger: VecDeque::with_capacity(60),
            thread_history_input: VecDeque::with_capacity(60),
            thread_history_bvh: VecDeque::with_capacity(60),
            thread_history_cache: VecDeque::with_capacity(60),
            thread_history_gui: VecDeque::with_capacity(60),
            thread_history_other: VecDeque::with_capacity(60),
            thread_history_loop: VecDeque::with_capacity(60),

            last_overlay_pos: None,
            last_overlay_size: None,

            demo_mode: false,
            demo_last_step: Instant::now(),
            demo_tab_idx: 0,
        };
        ret
    }

    fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let gui = WindowContext::new(event_loop, false, self.config.accent_color);
        let overlay = WindowContext::new(event_loop, true, self.config.accent_color);

        self.config.font.set(gui.egui());
        self.config.font.set(overlay.egui());

        if self.demo_mode {
            let _ = gui.window().request_inner_size(winit::dpi::LogicalSize::new(1280, 720));
            self.display_scale = 1.25;
        } else {
            self.display_scale = gui.window().scale_factor() as f32;
        }
        utils::info!("detected display scale: {}", self.display_scale);

        self.gui = Some(gui);
        self.overlay = Some(overlay);
    }

    fn frame_duration(&self) -> Duration {
        let ui_fps = self.config.fps.min(self.max_monitor_hz).max(1);
        Duration::from_secs_f32(1.0 / ui_fps as f32)
    }

    pub fn detect_highest_monitor_hz(event_loop: &winit::event_loop::ActiveEventLoop) -> u32 {
        let mut max_hz = 60u32;
        for monitor in event_loop.available_monitors() {
            if let Some(mhz) = monitor.refresh_rate_millihertz() {
                let hz = (mhz as f32 / 1000.0).round() as u32;
                if hz > max_hz {
                    max_hz = hz;
                }
            }
            for mode in monitor.video_modes() {
                let mhz = mode.refresh_rate_millihertz();
                let hz = (mhz as f32 / 1000.0).round() as u32;
                if hz > max_hz {
                    max_hz = hz;
                }
            }
        }
        utils::info!("Detected highest monitor VSync refresh rate: {} Hz", max_hz);
        max_hz
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.next_frame_time += self.frame_duration();

            let now = Instant::now();
            if self.next_frame_time < now {
                self.next_frame_time = now + self.frame_duration();
            }

            self.render();

            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                self.next_frame_time,
            ));
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_window(event_loop);
        self.max_monitor_hz = Self::detect_highest_monitor_hz(event_loop);
        self.send_config();

        self.next_frame_time = Instant::now() + self.frame_duration();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.next_frame_time,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        window_event: WindowEvent,
    ) {
        while let Ok(message) = self.channel.try_receive() {
            match message {
                UiMessage::Status(status) => self.game_status = status,
                UiMessage::FrameTime(time) => {
                    if self.frame_times.len() >= 500 {
                        self.frame_times.pop_front();
                    }
                    self.frame_times.push_back(time);
                }
            }
        }

        let Some(gui) = &self.gui else {
            return;
        };
        let Some(overlay) = &self.overlay else {
            return;
        };

        let window = if gui.window().id() == window_id {
            gui
        } else if overlay.window().id() == window_id {
            overlay
        } else {
            return;
        };

        match &window_event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                window.resize(*new_size);
            }
            WindowEvent::RedrawRequested => {
                if !self
                    .gui
                    .as_ref()
                    .map(|window| window.window().id() == window_id)
                    .unwrap_or_default()
                {
                    return;
                }
                self.render();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                if let winit::keyboard::Key::Named(key) = event.logical_key {
                    let modifiers = match key {
                        NamedKey::Control => Some(egui::Modifiers::CTRL),
                        NamedKey::Shift => Some(egui::Modifiers::SHIFT),
                        NamedKey::Alt => Some(egui::Modifiers::ALT),
                        _ => None,
                    };

                    if let Some(modifiers) = modifiers {
                        self.gui.as_mut().unwrap().process_modifier(
                            modifiers,
                            event.state == ElementState::Pressed,
                            event.repeat,
                        );
                    }
                }
                let _ = self
                    .gui
                    .as_mut()
                    .map(|gui| gui.process_event(&window_event));
            }
            _ => {
                let _ = self
                    .gui
                    .as_mut()
                    .map(|gui| gui.process_event(&window_event));
            }
        }
    }
}
