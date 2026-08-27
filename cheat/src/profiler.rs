use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub struct ScopeStats {
    pub count: u64,
    pub total_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub last_time: Duration,
    pub avg_time_us: f32,
    pub spike_count: u32,
}

impl Default for ScopeStats {
    fn default() -> Self {
        Self {
            count: 0,
            total_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            last_time: Duration::ZERO,
            avg_time_us: 0.0,
            spike_count: 0,
        }
    }
}

pub struct Profiler {
    stats: HashMap<&'static str, ScopeStats>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    pub fn record(&mut self, scope: &'static str, duration: Duration) {
        let entry = self.stats.entry(scope).or_default();
        entry.count += 1;
        entry.total_time += duration;
        entry.last_time = duration;
        if duration < entry.min_time {
            entry.min_time = duration;
        }
        if duration > entry.max_time {
            entry.max_time = duration;
        }
        if duration > Duration::from_millis(2) {
            entry.spike_count += 1;
        }
        entry.avg_time_us = (entry.total_time.as_secs_f64() * 1_000_000.0 / entry.count as f64) as f32;
    }

    pub fn get_all(&self) -> HashMap<&'static str, ScopeStats> {
        self.stats.clone()
    }

    pub fn reset(&mut self) {
        self.stats.clear();
    }
}

pub static GLOBAL_PROFILER: std::sync::LazyLock<Mutex<Profiler>> =
    std::sync::LazyLock::new(|| Mutex::new(Profiler::new()));

pub struct ProfileScope {
    name: &'static str,
    start: Instant,
}

impl ProfileScope {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        if let Ok(mut profiler) = GLOBAL_PROFILER.lock() {
            profiler.record(self.name, elapsed);
        }
    }
}

/// Macro helper for lightweight profiling scopes: `profile_scope!("aimbot");`
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _profile_guard = $crate::profiler::ProfileScope::new($name);
    };
}
