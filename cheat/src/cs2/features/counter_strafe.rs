use crate::{
    config::Config,
    cs2::{entity::player::Player, input::Input, offsets::Offsets},
    os::{keyboard::Keyboard, process::Process},
};

use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct CounterStrafe {
    was_w: bool,
    was_s: bool,
    was_a: bool,
    was_d: bool,
    release_w: Option<Instant>,
    release_s: Option<Instant>,
    release_a: Option<Instant>,
    release_d: Option<Instant>,
    last_fake_release_w: Option<Instant>,
    last_fake_release_s: Option<Instant>,
    last_fake_release_a: Option<Instant>,
    last_fake_release_d: Option<Instant>,
    debug_checked: bool,
    verbosity: u8,
}

impl CounterStrafe {
    pub fn is_w_active(&self) -> bool { self.release_w.is_some() }
    pub fn is_s_active(&self) -> bool { self.release_s.is_some() }
    pub fn is_a_active(&self) -> bool { self.release_a.is_some() }
    pub fn is_d_active(&self) -> bool { self.release_d.is_some() }

    pub fn reset(&mut self) {
        self.was_w = false;
        self.was_s = false;
        self.was_a = false;
        self.was_d = false;
        self.release_w = None;
        self.release_s = None;
        self.release_a = None;
        self.release_d = None;
        self.last_fake_release_w = None;
        self.last_fake_release_s = None;
        self.last_fake_release_a = None;
        self.last_fake_release_d = None;
    }

    pub fn run(
        &mut self,
        process: &Process,
        offsets: &Offsets,
        input: &Input,
        config: &Config,
        keyboard: &mut Keyboard,
        local_player: &Player,
    ) {
        if !self.debug_checked {
            let mut verbosity = 0;
            for arg in std::env::args() {
                if arg == "debug" || arg == "--debug" {
                    verbosity = verbosity.max(1);
                } else if arg.starts_with("-v") {
                    let v_count = arg.chars().filter(|&c| c == 'v').count() as u8;
                    verbosity = verbosity.max(v_count);
                }
            }
            self.verbosity = verbosity;
            self.debug_checked = true;
        }

        // Query pure physical hardware key state directly from linux evdev drivers
        let (is_w, is_a, is_s, is_d) = keyboard.get_physical_wasd();
        let real_hardware_pressed = is_w || is_a || is_s || is_d;

        // If the user is physically pressing ANY movement key on their real hardware keyboard, immediately abort/release synthetic taps and void the tick
        if real_hardware_pressed {
            let mut aborted = false;
            if self.release_w.is_some() {
                keyboard.w_release();
                self.release_w = None;
                aborted = true;
            }
            if self.release_s.is_some() {
                keyboard.s_release();
                self.release_s = None;
                aborted = true;
            }
            if self.release_a.is_some() {
                keyboard.a_release();
                self.release_a = None;
                aborted = true;
            }
            if self.release_d.is_some() {
                keyboard.d_release();
                self.release_d = None;
                aborted = true;
            }

            if aborted && self.verbosity >= 4 {
                ::utils::info!("[COUNTER-STRAFE] Physical key pressed (W:{} A:{} S:{} D:{}) -> Aborted active synthetic pulse!", is_w, is_a, is_s, is_d);
            }

            if (self.was_w != is_w || self.was_a != is_a || self.was_s != is_s || self.was_d != is_d) && self.verbosity >= 4 {
                ::utils::info!("[COUNTER-STRAFE] Physical key state change: W:{} A:{} S:{} D:{}", is_w, is_a, is_s, is_d);
            }

            self.was_w = is_w;
            self.was_s = is_s;
            self.was_a = is_a;
            self.was_d = is_d;
            return;
        }

        let velocity = process.read::<glam::Vec3>(local_player.pawn + offsets.pawn.velocity);
        let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        let view_angles = local_player.view_angles_direct(process, offsets);
        
        // Project velocity onto forward and right vectors
        let yaw_rad = view_angles.y.to_radians();
        let forward_x = yaw_rad.cos();
        let forward_y = yaw_rad.sin();
        let right_x = (yaw_rad - std::f32::consts::FRAC_PI_2).cos();
        let right_y = (yaw_rad - std::f32::consts::FRAC_PI_2).sin();

        let vel_forward = velocity.x * forward_x + velocity.y * forward_y;
        let vel_right = velocity.x * right_x + velocity.y * right_y;

        // Handle synthetic tap releases: hold counter-tap until directional velocity hits 0 (<= 2.0 u/s) or 200ms timeout
        if let Some(t) = self.release_w {
            let elapsed_ms = t.elapsed().as_millis();
            if vel_forward >= -2.0 || t.elapsed() > Duration::from_millis(200) {
                keyboard.w_release();
                self.release_w = None;
                self.last_fake_release_w = Some(Instant::now());
                if self.verbosity >= 4 {
                    ::utils::info!("[COUNTER-STRAFE] Cut off W tap at near-zero speed (vel_forward: {:.1}, speed: {:.1}, elapsed: {}ms)", vel_forward, speed, elapsed_ms);
                }
            }
        }
        if let Some(t) = self.release_s {
            let elapsed_ms = t.elapsed().as_millis();
            if vel_forward <= 2.0 || t.elapsed() > Duration::from_millis(200) {
                keyboard.s_release();
                self.release_s = None;
                self.last_fake_release_s = Some(Instant::now());
                if self.verbosity >= 4 {
                    ::utils::info!("[COUNTER-STRAFE] Cut off S tap at near-zero speed (vel_forward: {:.1}, speed: {:.1}, elapsed: {}ms)", vel_forward, speed, elapsed_ms);
                }
            }
        }
        if let Some(t) = self.release_a {
            let elapsed_ms = t.elapsed().as_millis();
            if vel_right <= 2.0 || t.elapsed() > Duration::from_millis(200) {
                keyboard.a_release();
                self.release_a = None;
                self.last_fake_release_a = Some(Instant::now());
                if self.verbosity >= 4 {
                    ::utils::info!("[COUNTER-STRAFE] Cut off A tap at near-zero speed (vel_right: {:.1}, speed: {:.1}, elapsed: {}ms)", vel_right, speed, elapsed_ms);
                }
            }
        }
        if let Some(t) = self.release_d {
            let elapsed_ms = t.elapsed().as_millis();
            if vel_right >= -2.0 || t.elapsed() > Duration::from_millis(200) {
                keyboard.d_release();
                self.release_d = None;
                self.last_fake_release_d = Some(Instant::now());
                if self.verbosity >= 4 {
                    ::utils::info!("[COUNTER-STRAFE] Cut off D tap at near-zero speed (vel_right: {:.1}, speed: {:.1}, elapsed: {}ms)", vel_right, speed, elapsed_ms);
                }
            }
        }

        if !config.misc.counter_strafe {
            return;
        }

        // Disable counter-strafe while bunnyhopping or holding jump hotkey to prevent speed drops
        let is_space_hotkey = config.misc.bhop_hotkey == crate::cs2::key_codes::KeyCode::Space;
        let is_bhop_active = config.misc.bunnyhop
            && (keyboard.is_physical_space_pressed()
                || input.is_key_pressed(config.misc.bhop_hotkey)
                || (is_space_hotkey && input.is_key_pressed(crate::cs2::key_codes::KeyCode::Space)));

        if is_bhop_active {
            return;
        }

        let health = process.read::<i32>(local_player.pawn + offsets.pawn.health);
        let life_state = process.read::<u8>(local_player.pawn + offsets.pawn.life_state);
        if health <= 0 || life_state != 0 {
            return;
        }

        let is_in_air = local_player.is_in_air_direct(process, offsets);
        if is_in_air {
            self.was_w = false;
            self.was_s = false;
            self.was_a = false;
            self.was_d = false;
            return;
        }

        let user_released_w = self.was_w && !is_w && self.last_fake_release_w.map(|t| t.elapsed() > Duration::from_millis(200)).unwrap_or(true);
        let user_released_s = self.was_s && !is_s && self.last_fake_release_s.map(|t| t.elapsed() > Duration::from_millis(200)).unwrap_or(true);
        let user_released_a = self.was_a && !is_a && self.last_fake_release_a.map(|t| t.elapsed() > Duration::from_millis(200)).unwrap_or(true);
        let user_released_d = self.was_d && !is_d && self.last_fake_release_d.map(|t| t.elapsed() > Duration::from_millis(200)).unwrap_or(true);

        let no_active_pulse = self.release_w.is_none() && self.release_s.is_none() && self.release_a.is_none() && self.release_d.is_none();

        // Trigger counter-strafe only when user explicitly releases a key and speed > 45 u/s
        if speed > 45.0 && no_active_pulse {
            if user_released_w && vel_forward > 45.0 {
                keyboard.s_press();
                self.release_s = Some(Instant::now());
                self.last_fake_release_w = Some(Instant::now());
                self.was_w = false; self.was_s = false; self.was_a = false; self.was_d = false;
                if self.verbosity >= 1 {
                    ::utils::info!("[COUNTER-STRAFE] Triggered: W release -> S press (speed: {:.1}, vel_forward: {:.1})", speed, vel_forward);
                }
            } else if user_released_s && vel_forward < -45.0 {
                keyboard.w_press();
                self.release_w = Some(Instant::now());
                self.last_fake_release_s = Some(Instant::now());
                self.was_w = false; self.was_s = false; self.was_a = false; self.was_d = false;
                if self.verbosity >= 1 {
                    ::utils::info!("[COUNTER-STRAFE] Triggered: S release -> W press (speed: {:.1}, vel_forward: {:.1})", speed, vel_forward);
                }
            } else if user_released_a && vel_right < -45.0 {
                keyboard.d_press();
                self.release_d = Some(Instant::now());
                self.last_fake_release_a = Some(Instant::now());
                self.was_w = false; self.was_s = false; self.was_a = false; self.was_d = false;
                if self.verbosity >= 1 {
                    ::utils::info!("[COUNTER-STRAFE] Triggered: A release -> D press (speed: {:.1}, vel_right: {:.1})", speed, vel_right);
                }
            } else if user_released_d && vel_right > 45.0 {
                keyboard.a_press();
                self.release_a = Some(Instant::now());
                self.last_fake_release_d = Some(Instant::now());
                self.was_w = false; self.was_s = false; self.was_a = false; self.was_d = false;
                if self.verbosity >= 1 {
                    ::utils::info!("[COUNTER-STRAFE] Triggered: D release -> A press (speed: {:.1}, vel_right: {:.1})", speed, vel_right);
                }
            }
        }

        // Only record input history when no synthetic counter-tap pulse is active
        if no_active_pulse {
            if (self.was_w != is_w || self.was_a != is_a || self.was_s != is_s || self.was_d != is_d) && self.verbosity >= 4 {
                ::utils::info!("[COUNTER-STRAFE] Physical state updated: W:{} A:{} S:{} D:{}", is_w, is_a, is_s, is_d);
            }
            self.was_w = is_w;
            self.was_s = is_s;
            self.was_a = is_a;
            self.was_d = is_d;
        }
    }
}
