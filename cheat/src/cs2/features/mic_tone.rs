use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::{config::Config, cs2::input::Input};

#[derive(Debug)]
pub struct MicTone {
    active: Arc<AtomicBool>,
    mode: Arc<AtomicU8>,
    freq_bits: Arc<AtomicU32>,
    gain_bits: Arc<AtomicU32>,
    initialized: bool,
    last_hw_boost: f32,
}

impl Default for MicTone {
    fn default() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            mode: Arc::new(AtomicU8::new(1)),
            freq_bits: Arc::new(AtomicU32::new(4000.0f32.to_bits())),
            gain_bits: Arc::new(AtomicU32::new(10.0f32.to_bits())),
            initialized: false,
            last_hw_boost: 500.0,
        }
    }
}

impl MicTone {
    fn init_virtual_mic() {
        let sinks = Command::new("pactl")
            .args(["list", "sinks", "short"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        if let Ok(out) = sinks {
            let str_out = String::from_utf8_lossy(&out.stdout);
            if !str_out.contains("MicMixSink") {
                let _ = Command::new("pactl")
                    .args([
                        "load-module",
                        "module-null-sink",
                        "sink_name=MicMixSink",
                        "sink_properties=device.description=Mic_Mix_Sink",
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                let _ = Command::new("pactl")
                    .args([
                        "load-module",
                        "module-remap-source",
                        "source_name=VirtualMic",
                        "master=MicMixSink.monitor",
                        "source_properties=device.description=Virtual_Microphone",
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                if let Ok(sources_out) = Command::new("pactl")
                    .args(["list", "sources", "short"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                {
                    let s_str = String::from_utf8_lossy(&sources_out.stdout);
                    for line in s_str.lines() {
                        if line.contains("input") && !line.contains("VirtualMic") && !line.contains("monitor") {
                            if let Some(default_mic) = line.split_whitespace().nth(1) {
                                let _ = Command::new("pactl")
                                    .args([
                                        "load-module",
                                        "module-loopback",
                                        &format!("source={default_mic}"),
                                        "sink=MicMixSink",
                                        "latency_msec=1",
                                    ])
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .status();
                                break;
                            }
                        }
                    }
                }

                let _ = Command::new("pactl")
                    .args(["set-default-source", "VirtualMic"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
        let _ = Command::new("pactl")
            .args(["set-sink-volume", "MicMixSink", "100%"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = Command::new("pactl")
            .args(["set-source-volume", "VirtualMic", "100%"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    pub fn run(&mut self, input: &Input, config: &Config) {
        if !config.misc.mic_tone {
            if self.active.load(Ordering::Relaxed) {
                self.active.store(false, Ordering::Relaxed);
            }
            if (self.last_hw_boost - 100.0).abs() > 0.1 {
                self.last_hw_boost = 100.0;
                let _ = Command::new("pactl")
                    .args(["set-sink-volume", "MicMixSink", "100%"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                let _ = Command::new("pactl")
                    .args(["set-source-volume", "VirtualMic", "100%"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            return;
        }

        let is_pressed = input.is_key_pressed(config.misc.mic_tone_hotkey);
        self.active.store(is_pressed, Ordering::Relaxed);

        let target_hw_boost = if is_pressed {
            config.misc.mic_hw_boost
        } else {
            100.0
        };

        if (self.last_hw_boost - target_hw_boost).abs() > 0.1 {
            self.last_hw_boost = target_hw_boost;
            let val_str = format!("{}%", target_hw_boost as u32);
            let _ = Command::new("pactl")
                .args(["set-sink-volume", "MicMixSink", &val_str])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("pactl")
                .args(["set-source-volume", "VirtualMic", &val_str])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        self.mode.store(config.misc.mic_tone_mode as u8, Ordering::Relaxed);
        self.freq_bits.store(config.misc.mic_tone_frequency.to_bits(), Ordering::Relaxed);
        self.gain_bits.store(config.misc.mic_tone_volume.to_bits(), Ordering::Relaxed);

        if !self.initialized {
            Self::init_virtual_mic();
            self.initialized = true;

            let active_flag = self.active.clone();
            let mode_flag = self.mode.clone();
            let freq_bits = self.freq_bits.clone();
            let gain_bits = self.gain_bits.clone();

            thread::spawn(move || {
                let sample_rate = 48000;
                let mut reader_proc: Option<Child> = None;
                let mut writer_proc: Option<Child> = None;

                loop {
                    if active_flag.load(Ordering::Relaxed) {
                        let current_mode = mode_flag.load(Ordering::Relaxed);
                        let gain = f32::from_bits(gain_bits.load(Ordering::Relaxed));
                        let freq = f32::from_bits(freq_bits.load(Ordering::Relaxed));
                        if current_mode == 1 {
                            // Desktop Audio Ear-Rape Mode (Captures what user is hearing at 10x volume)
                            if reader_proc.is_none() {
                                reader_proc = Command::new("parec")
                                    .args([
                                        "--device=@DEFAULT_SINK@.monitor",
                                        "--raw",
                                        "--format=s16le",
                                        "--rate=48000",
                                        "--channels=1",
                                        "--latency-msec=5",
                                    ])
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .ok();
                            }
                            if writer_proc.is_none() {
                                writer_proc = Command::new("paplay")
                                    .args([
                                        "--device=MicMixSink",
                                        "--raw",
                                        "--format=s16le",
                                        "--rate=48000",
                                        "--channels=1",
                                        "--latency-msec=5",
                                        "--volume=65536",
                                    ])
                                    .stdin(Stdio::piped())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .ok();
                            }

                            if let (Some(r), Some(w)) = (reader_proc.as_mut(), writer_proc.as_mut()) {
                                if let (Some(r_out), Some(w_in)) = (r.stdout.as_mut(), w.stdin.as_mut()) {
                                    let mut buf = [0u8; 1920];
                                    match r_out.read(&mut buf) {
                                        Ok(n) if n > 0 => {
                                            let mut boosted = Vec::with_capacity(n);
                                            for i in (0..n).step_by(2) {
                                                if i + 1 < n {
                                                    let sample = i16::from_le_bytes([buf[i], buf[i + 1]]);
                                                    let val = (sample as f32 * gain).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                                                    boosted.extend_from_slice(&val.to_le_bytes());
                                                }
                                            }
                                            if w_in.write_all(&boosted).is_err() {
                                                let _ = w.kill();
                                                writer_proc = None;
                                            }
                                        }
                                        _ => {
                                            thread::sleep(Duration::from_millis(2));
                                        }
                                    }
                                }
                            }
                        } else {
                            // Sine Tone Mode
                            if let Some(mut r) = reader_proc.take() {
                                let _ = r.kill();
                                let _ = r.wait();
                            }

                            let chunk_samples = (sample_rate as f32 * 0.05) as usize;
                            let mut chunk_bytes = Vec::with_capacity(chunk_samples * 2);
                            let amplitude = 32767.0f32 * gain;

                            for i in 0..chunk_samples {
                                let t = i as f32 / sample_rate as f32;
                                let raw_sample = amplitude * (2.0 * std::f32::consts::PI * freq * t).sin();
                                let val = raw_sample.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                                chunk_bytes.extend_from_slice(&val.to_le_bytes());
                            }

                            if writer_proc.is_none() {
                                writer_proc = Command::new("paplay")
                                    .args([
                                        "--device=MicMixSink",
                                        "--raw",
                                        "--format=s16le",
                                        "--rate=48000",
                                        "--channels=1",
                                        "--volume=65536",
                                    ])
                                    .stdin(Stdio::piped())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .ok();
                            }

                            if let Some(ref mut proc) = writer_proc {
                                if let Some(ref mut stdin) = proc.stdin {
                                    if stdin.write_all(&chunk_bytes).is_err() {
                                        let _ = proc.kill();
                                        writer_proc = None;
                                    }
                                }
                            }
                        }
                    } else {
                        if let Some(mut proc) = reader_proc.take() {
                            let _ = proc.kill();
                            let _ = proc.wait();
                        }
                        if let Some(mut proc) = writer_proc.take() {
                            let _ = proc.kill();
                            let _ = proc.wait();
                        }
                        thread::sleep(Duration::from_millis(20));
                    }
                }
            });
        }
    }
}

impl Drop for MicTone {
    fn drop(&mut self) {
        if self.initialized {
            let _ = Command::new("pactl")
                .args(["set-sink-volume", "MicMixSink", "100%"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("pactl")
                .args(["set-source-volume", "VirtualMic", "100%"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            if let Ok(out) = Command::new("pactl")
                .args(["list", "modules", "short"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                let s_str = String::from_utf8_lossy(&out.stdout);
                for line in s_str.lines() {
                    if line.contains("MicMixSink") || line.contains("VirtualMic") {
                        if let Some(mod_id) = line.split_whitespace().next() {
                            let _ = Command::new("pactl")
                                .args(["unload-module", mod_id])
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .status();
                        }
                    }
                }
            }
        }
    }
}
