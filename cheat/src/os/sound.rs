use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

use crate::config::{BASE_PATH, hud::HitsoundPreset};

pub fn play_hitsound(preset: HitsoundPreset, volume: f32, pitch: f32, custom_wav_name: &str) {
    let volume = volume.clamp(0.0, 1.0);
    let pitch = pitch.clamp(0.5, 2.0);

    if volume <= 0.01 {
        return;
    }

    let custom_name = custom_wav_name.to_string();

    thread::spawn(move || {
        let sound_path = match preset {
            HitsoundPreset::CustomWav => {
                let sounds_dir = BASE_PATH.join("sounds");
                let custom_path = sounds_dir.join(&custom_name);
                if custom_path.exists() {
                    custom_path
                } else {
                    generate_preset_wav(HitsoundPreset::CodHitmarker, volume, pitch)
                }
            }
            _ => generate_preset_wav(preset, volume, pitch),
        };

        if sound_path.exists() {
            let path_str = sound_path.to_string_lossy();
            let status = Command::new("pw-play")
                .arg(&*path_str)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if status.is_err() || !status.unwrap().success() {
                let status2 = Command::new("paplay")
                    .arg(&*path_str)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                if status2.is_err() || !status2.unwrap().success() {
                    let _ = Command::new("aplay")
                        .arg("-q")
                        .arg(&*path_str)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            }
        }
    });
}

fn generate_preset_wav(preset: HitsoundPreset, volume: f32, pitch: f32) -> PathBuf {
    let filename = match preset {
        HitsoundPreset::RustHeadshot => format!("hs_rust_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::CodHitmarker => format!("hs_cod_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::MetallicBell => format!("hs_bell_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::CsgoDing => format!("hs_ding_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::Bubble => format!("hs_bubble_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::Neverlose => format!("hs_nl_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::Skeet => format!("hs_skeet_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::Aimware => format!("hs_aw_v{:.2}_p{:.2}.wav", volume, pitch),
        HitsoundPreset::Primordial => format!("hs_primo_v{:.2}_p{:.2}.wav", volume, pitch),
        _ => format!("hs_cod_v{:.2}_p{:.2}.wav", volume, pitch),
    };

    let cache_dir = std::env::temp_dir().join("deadlocked_sounds");
    if !cache_dir.exists() {
        let _ = fs::create_dir_all(&cache_dir);
    }

    let target_path = cache_dir.join(filename);
    if target_path.exists() {
        return target_path;
    }

    let sample_rate = 44100;
    let samples = match preset {
        HitsoundPreset::RustHeadshot => generate_rust_headshot(sample_rate, volume, pitch),
        HitsoundPreset::CodHitmarker => generate_cod_hitmarker(sample_rate, volume, pitch),
        HitsoundPreset::MetallicBell => generate_metallic_bell(sample_rate, volume, pitch),
        HitsoundPreset::CsgoDing => generate_csgo_ding(sample_rate, volume, pitch),
        HitsoundPreset::Bubble => generate_bubble(sample_rate, volume, pitch),
        HitsoundPreset::Neverlose => generate_neverlose(sample_rate, volume, pitch),
        HitsoundPreset::Skeet => generate_skeet(sample_rate, volume, pitch),
        HitsoundPreset::Aimware => generate_aimware(sample_rate, volume, pitch),
        HitsoundPreset::Primordial => generate_primordial(sample_rate, volume, pitch),
        _ => generate_cod_hitmarker(sample_rate, volume, pitch),
    };

    if let Ok(mut file) = File::create(&target_path) {
        let _ = write_wav_header(&mut file, sample_rate, samples.len() as u32);
        for sample in samples {
            let _ = file.write_all(&sample.to_le_bytes());
        }
    }

    target_path
}

fn write_wav_header<W: Write>(writer: &mut W, sample_rate: u32, num_samples: u32) -> std::io::Result<()> {
    let data_chunk_size = num_samples * 2;
    let file_size = 36 + data_chunk_size;

    writer.write_all(b"RIFF")?;
    writer.write_all(&file_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&(sample_rate * 2).to_le_bytes())?;
    writer.write_all(&2u16.to_le_bytes())?;
    writer.write_all(&16u16.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_chunk_size.to_le_bytes())?;
    Ok(())
}

fn generate_rust_headshot(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.09;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).powf(2.0);
        let freq_punch = 1800.0 * pitch * (1.0 - t / duration * 0.7);
        let freq_thud = 180.0 * pitch;

        let tone = (t * freq_punch * std::f32::consts::TAU).sin() * 0.6
                 + (t * freq_thud * std::f32::consts::TAU).sin() * 0.4;
        let noise = (rand_sample() * 0.4) * (1.0 - t / 0.03).max(0.0);

        let val = (tone + noise) * env * volume * 28000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_cod_hitmarker(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.04;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).powf(3.0);
        let freq = 900.0 * pitch;
        let tone = (t * freq * std::f32::consts::TAU).sin();
        let noise = rand_sample() * 0.3;

        let val = (tone + noise) * env * volume * 26000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_metallic_bell(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.16;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-t * 22.0).exp();
        let f1 = 1400.0 * pitch;
        let f2 = 2800.0 * pitch;

        let tone = (t * f1 * std::f32::consts::TAU).sin() * 0.7
                 + (t * f2 * std::f32::consts::TAU).sin() * 0.3;

        let val = tone * env * volume * 28000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_csgo_ding(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.12;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-t * 28.0).exp();
        let f1 = 1200.0 * pitch;
        let f2 = 2400.0 * pitch;

        let tone = (t * f1 * std::f32::consts::TAU).sin() * 0.8
                 + (t * f2 * std::f32::consts::TAU).sin() * 0.2;

        let val = tone * env * volume * 30000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_bubble(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.07;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).powf(1.5);
        let freq = (400.0 + (t / duration) * 800.0) * pitch;

        let tone = (t * freq * std::f32::consts::TAU).sin();

        let val = tone * env * volume * 27000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_neverlose(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.06;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-t * 45.0).exp();
        let f1 = 2200.0 * pitch;
        let f2 = 4400.0 * pitch;

        let tone = (t * f1 * std::f32::consts::TAU).sin() * 0.6
                 + (t * f2 * std::f32::consts::TAU).sin() * 0.4;
        let noise = (rand_sample() * 0.2) * (1.0 - t / 0.015).max(0.0);

        let val = (tone + noise) * env * volume * 29000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_skeet(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.08;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-t * 35.0).exp();
        let f1 = 1600.0 * pitch;
        let f2 = 3200.0 * pitch;

        let tone = (t * f1 * std::f32::consts::TAU).sin() * 0.75
                 + (t * f2 * std::f32::consts::TAU).sin() * 0.25;

        let val = tone * env * volume * 31000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_aimware(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.10;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-t * 25.0).exp();
        let f_thud = 350.0 * pitch;
        let f_ring = 1500.0 * pitch;

        let tone = (t * f_thud * std::f32::consts::TAU).sin() * 0.5
                 + (t * f_ring * std::f32::consts::TAU).sin() * 0.5;

        let val = tone * env * volume * 28000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn generate_primordial(sample_rate: u32, volume: f32, pitch: f32) -> Vec<i16> {
    let duration = 0.05;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut out = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - t / duration).powf(2.0);
        let freq = (800.0 + (t / duration).sqrt() * 1200.0) * pitch;

        let tone = (t * freq * std::f32::consts::TAU).sin();

        let val = tone * env * volume * 28000.0;
        out.push(val.clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn rand_sample() -> f32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(0x12345678);
    let mut s = SEED.load(Ordering::Relaxed);
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    SEED.store(s, Ordering::Relaxed);
    (s as f32 / u32::MAX as f32) * 2.0 - 1.0
}
