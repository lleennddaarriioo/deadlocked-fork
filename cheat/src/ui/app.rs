use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use shared::{Data, SoundType, Weapon};
use utils::{Channel, Mutex};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, StartCause, WindowEvent},
    keyboard::NamedKey,
};

use crate::{
    config::{
        CONFIG_PATH, Config, DEFAULT_CONFIG_NAME,
        application::{ApplicationConfig, read_app_config, write_app_config},
        available_configs, parse_config, write_config,
    },
    message::{GameMessage, GameStatus, RadarMessage, RadarStatus, UiMessage},
    ui::{
        grenades::{Grenade, GrenadeList, read_grenades},
        gui::{Tab, aimbot::AimbotTab},
        overlay::model::ModelRenderer,
        trail::Trail,
        window_context::WindowContext,
    },
    update::UpdateStatus,
};

pub struct AppState {
    pub channel_game: Channel<GameMessage, UiMessage>,
    pub channel_radar: Channel<RadarMessage, RadarStatus>,
    pub data: Arc<Mutex<Data>>,

    pub game_status: GameStatus,
    pub display_scale: f32,
    pub trails: HashMap<usize, Trail>,
    pub player_sounds: HashMap<u64, (Instant, SoundType)>,
    pub frame_times: VecDeque<Duration>,
    pub game_framecount: i32,
    pub game_tickcount: i32,
    pub last_rendered_framecount: i32,

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
    pub aimbot_show_curve: bool,
    
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

    pub update_status: UpdateStatus,
    pub text_popup: Option<String>,
    pub update_popup: bool,
    pub overlay_egui: Option<egui::Context>,
    pub model_renderer: Option<Arc<ModelRenderer>>,
    pub radar_status: RadarStatus,
}

pub struct App {
    pub gui: Option<WindowContext>,
    pub overlay: Option<WindowContext>,
    next_frame_time: Instant,
    pub state: AppState,
}

impl Deref for App {
    type Target = AppState;
    fn deref(&self) -> &AppState {
        &self.state
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

impl AppState {
    pub fn new(
        channel_game: Channel<GameMessage, UiMessage>,
        channel_radar: Channel<RadarMessage, RadarStatus>,
        data: Arc<Mutex<Data>>,
    ) -> Self {
        let mut app_config = read_app_config();
        let config_name = Path::new(&app_config.config_name)
            .file_name()
            .filter(|name| name.to_string_lossy() == app_config.config_name)
            .filter(|name| name.to_string_lossy().ends_with(".toml"))
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| DEFAULT_CONFIG_NAME.to_owned());
        if app_config.config_name != config_name {
            app_config.config_name = config_name.clone();
            write_app_config(&app_config);
        }

        let current_config = CONFIG_PATH.join(&config_name);
        let config = parse_config(&current_config);
        write_config(&config, &current_config);
        let grenades = read_grenades();
        write_app_config(&app_config);

        let update_status = crate::update::check();
        let update_popup = matches!(update_status, crate::update::UpdateStatus::Available { .. });

        Self {
            channel_game,
            channel_radar,
            data,
            app_config,
            config,
            current_config,
            available_configs: available_configs(),
            new_config_name: String::new(),
            game_status: GameStatus::NotStarted,
            display_scale: 1.0,
            trails: HashMap::new(),
            player_sounds: HashMap::new(),
            frame_times: VecDeque::with_capacity(500),
            game_framecount: 0,
            game_tickcount: 0,
            last_rendered_framecount: 0,
            grenades,
            new_grenade: Grenade::new(),
            current_grenade: None,
            current_tab: Tab::default(),
            aimbot_tab: AimbotTab::default(),
            aimbot_weapon: Weapon::default(),
            aimbot_show_curve: false,
            
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

            update_status,
            text_popup: None,
            update_popup,
            overlay_egui: None,
            model_renderer: None,
            radar_status: RadarStatus::Disabled,
        }
    }
}

impl App {
    pub fn new(
        channel_game: Channel<GameMessage, UiMessage>,
        channel_radar: Channel<RadarMessage, RadarStatus>,
        data: Arc<Mutex<Data>>,
    ) -> Self {
        let state = AppState::new(channel_game, channel_radar, data);
        let ret = Self {
            gui: None,
            overlay: None,
            next_frame_time: Instant::now() + Duration::from_millis(16),
            state,
        };
        ret.send_config_game();
        ret.send_config_radar();
        ret
    }

    pub fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let gui = WindowContext::new(event_loop, false, self.state.config.accent_color);
        let overlay = WindowContext::new(event_loop, true, self.state.config.accent_color);

        self.state.config.font.set(gui.egui());
        self.state.config.font.set(overlay.egui());

        self.state.display_scale = if self.demo_mode {
            let _ = gui.window().request_inner_size(winit::dpi::LogicalSize::new(1280, 720));
            1.25
        } else {
            gui.window().scale_factor() as f32
        };
        self.state.overlay_egui = Some(overlay.egui().clone());
        utils::info!("detected display scale: {}", self.state.display_scale);

        self.gui = Some(gui);
        self.overlay = Some(overlay);
    }

    fn frame_duration(&self) -> Duration {
        let ui_fps = self.config.fps.max(1);
        Duration::from_secs_f64(1.0 / ui_fps as f64)
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

    fn render_gui(&mut self) {
        let state = &mut self.state;
        let gui = self.gui.as_mut().unwrap();

        gui.make_current().unwrap();
        gui.run(|ui| state.gui(ui));
        gui.clear();
        gui.paint();
        gui.swap_buffers().unwrap();

        if gui.egui().has_requested_repaint() {
            gui.window().request_redraw();
        }
    }

    fn render_overlay(&mut self) {
        let state = &mut self.state;
        let overlay = self.overlay.as_mut().unwrap();

        overlay.window().set_cursor_hittest(false).unwrap();
        {
            let data = state.data.lock();
            Self::update_overlay_window(overlay, &data);
        }
        overlay.make_current().unwrap();
        let glow = overlay.glow();
        overlay.run(|ui| state.overlay(ui, &glow));
        overlay.clear();
        overlay.paint();
        overlay.swap_buffers().unwrap();
    }

    fn receive_events(&mut self) {
        while let Ok(message) = self.state.channel_game.try_receive() {
            match message {
                UiMessage::Status(status) => self.state.game_status = status,
                UiMessage::FrameTime(time) => {
                    if self.state.frame_times.len() >= 500 {
                        self.state.frame_times.pop_front();
                    }
                    self.state.frame_times.push_back(time);
                }
            }
        }
        while let Ok(message) = self.state.channel_radar.try_receive() {
            self.state.radar_status = message;
        }
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } | StartCause::Poll = cause {
            // Check for new messages from the game loop
            while let Ok(message) = self.state.channel_game.try_receive() {
                match message {
                    UiMessage::Status(status) => self.state.game_status = status,
                    UiMessage::FrameTime(time, framecount, tickcount) => {
                        if self.state.frame_times.len() >= 500 {
                            self.state.frame_times.pop_front();
                        }
                        self.state.frame_times.push_back(time);
                        self.state.game_framecount = framecount;
                        self.state.game_tickcount = tickcount;
                    }
                }
            }
            while let Ok(message) = self.state.channel_radar.try_receive() {
                self.state.radar_status = message;
            }

            self.receive_events();
            self.render_overlay();

            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                now + Duration::from_millis(1),
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
        self.gui.as_ref().unwrap().window().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.receive_events();

        let gui_window_id = self.gui.as_ref().map(|gui| gui.window().id());
        let overlay_window_id = self.overlay.as_ref().map(|overlay| overlay.window().id());
        let is_gui = gui_window_id == Some(window_id);
        let is_overlay = overlay_window_id == Some(window_id);

        if !is_gui && !is_overlay {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if is_gui {
                    let Some(gui) = self.gui.as_mut() else { return };
                    gui.resize(new_size);
                    let response = gui.process_event(&WindowEvent::Resized(new_size));
                    if response.repaint {
                        gui.window().request_redraw();
                    }
                } else if is_overlay {
                    let Some(overlay) = self.overlay.as_mut() else {
                        return;
                    };
                    overlay.resize(new_size);
                    let response = overlay.process_event(&WindowEvent::Resized(new_size));
                    if response.repaint {
                        overlay.window().request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested if is_gui => self.render_gui(),
            _ if is_gui => {
                let Some(gui) = self.gui.as_mut() else { return };
                if let WindowEvent::KeyboardInput {
                    event,
                    is_synthetic: false,
                    ..
                } = &event
                    && let winit::keyboard::Key::Named(key) = event.logical_key
                {
                    let modifiers = match key {
                        NamedKey::Control => Some(egui::Modifiers::CTRL),
                        NamedKey::Shift => Some(egui::Modifiers::SHIFT),
                        NamedKey::Alt => Some(egui::Modifiers::ALT),
                        _ => None,
                    };
                    if let Some(modifiers) = modifiers {
                        gui.process_modifier(
                            modifiers,
                            event.state == ElementState::Pressed,
                            event.repeat,
                        );
                    }
                }
                let response = gui.process_event(&event);
                if response.repaint {
                    gui.window().request_redraw();
                }
            }
            _ => {}
        }
    }
}
