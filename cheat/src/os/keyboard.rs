use std::{
    fs::File,
    io::Write,
    os::fd::AsRawFd,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr, libc::c_ulong};

#[derive(Debug, Clone, Copy)]
struct Timeval {
    seconds: u64,
    microseconds: u64,
}

#[derive(Debug, Clone, Copy)]
struct InputEvent {
    time: Timeval,
    event_type: u16,
    code: u16,
    value: i32,
}

impl InputEvent {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(24);

        bytes.extend(&self.time.seconds.to_le_bytes());
        bytes.extend(&self.time.microseconds.to_le_bytes());

        bytes.extend(&self.event_type.to_le_bytes());
        bytes.extend(&self.code.to_le_bytes());
        bytes.extend(&self.value.to_le_bytes());

        bytes
    }
}

#[repr(C)]
struct DeviceSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

const DEVICE_SETUP: DeviceSetup = DeviceSetup {
    id: InputId {
        bustype: 0x03,
        vendor: 0x0451,
        product: 0xe009,
        version: 1,
    },
    // "TI-84 Plus Silver Keyboard"
    name: [
        84, 73, 45, 56, 52, 32, 80, 108, 117, 115, 32, 83, 105, 108, 118, 101, 114, 32, 75, 101,
        121, 98, 111, 97, 114, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0,
    ],
    ff_effects_max: 0,
};

const UINPUT_IOCTL_BASE: c_ulong = b'U' as c_ulong;
ioctl_none!(ui_dev_create, UINPUT_IOCTL_BASE, 1);
ioctl_none!(ui_dev_destroy, UINPUT_IOCTL_BASE, 2);
ioctl_write_int!(ui_set_evbit, UINPUT_IOCTL_BASE, 100);
ioctl_write_int!(ui_set_keybit, UINPUT_IOCTL_BASE, 101);
ioctl_write_ptr!(ui_dev_setup, UINPUT_IOCTL_BASE, 3, DeviceSetup);

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const SYN_REPORT: u16 = 0x00;
const KEY_W: u16 = 17;
const KEY_P: u16 = 25;
const KEY_A: u16 = 30;
const KEY_S: u16 = 31;
const KEY_D: u16 = 32;
const KEY_O: u16 = 24;
const KEY_SPACE: u16 = 57;
const KEY_END: u16 = 107;

pub struct Keyboard {
    file: File,
    physical_fds: Vec<File>,
    w_pressed: bool,
    s_pressed: bool,
    a_pressed: bool,
    d_pressed: bool,
    p_pressed: bool,
    o_pressed: bool,
    space_pressed: bool,
    end_pressed: bool,
    jump_timestamps: std::collections::VecDeque<std::time::Instant>,
    total_timestamps: std::collections::VecDeque<std::time::Instant>,
}

static CREATED: AtomicBool = AtomicBool::new(false);

// EVIOCGID = (2 << 30) | (69 << 8) | (2 << 0) | (8 << 16)
const EVIOCGID: c_ulong = 0x80084502;
// EVIOCGKEY(64) = (2 << 30) | (69 << 8) | (24 << 0) | (64 << 16)
const EVIOCGKEY_64: c_ulong = 0x80404518;
// EVIOCGNAME(80) = (2 << 30) | (69 << 8) | (6 << 0) | (80 << 16)
const EVIOCGNAME_80: c_ulong = 0x80504506;
// EVIOCGBIT(EV_KEY, 64) = (2 << 30) | (69 << 8) | (33 << 0) | (64 << 16)
const EVIOCGBIT_KEY_64: c_ulong = 0x80404521;

impl Keyboard {
    pub fn open() -> Result<Self, String> {
        if CREATED.swap(true, Ordering::Relaxed) {
            return Err("keyboard already initialized".into());
        }
        let file = File::options()
            .write(true)
            .open("/dev/uinput")
            .map_err(|e| e.to_string())?;
        let fd = file.as_raw_fd();

        unsafe {
            ui_set_evbit(fd, EV_SYN as u64).map_err(|e| e.to_string())?;
            ui_set_evbit(fd, EV_KEY as u64).map_err(|e| e.to_string())?;

            ui_set_keybit(fd, KEY_W as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_S as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_A as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_D as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_P as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_O as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_SPACE as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_END as u64).map_err(|e| e.to_string())?;

            ui_dev_setup(fd, &DEVICE_SETUP).map_err(|e| e.to_string())?;
            ui_dev_create(fd).map_err(|e| e.to_string())?;
        }

        // Wait 100ms for uinput virtual device node registration to complete in kernel
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Open physical /dev/input/event* devices (strictly excluding virtual uinput/ti-84 devices)
        let mut physical_fds = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/dev/input") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.starts_with("event")) {
                    if let Ok(f) = File::options().read(true).open(&path) {
                        let mut id_buf = InputId { bustype: 0, vendor: 0, product: 0, version: 0 };
                        let res_id = unsafe {
                            nix::libc::ioctl(f.as_raw_fd(), EVIOCGID, &mut id_buf as *mut _ as *mut nix::libc::c_void)
                        };
                        if res_id >= 0 {
                            // Exclude virtual devices (BUS_VIRTUAL = 0x06) and the TI-84 device
                            if id_buf.bustype == 0x06 || (id_buf.vendor == 0x0451 && id_buf.product == 0xe009) {
                                continue;
                            }
                        }

                        let mut name_buf = [0u8; 80];
                        let res_name = unsafe {
                            nix::libc::ioctl(f.as_raw_fd(), EVIOCGNAME_80, name_buf.as_mut_ptr())
                        };
                        if res_name >= 0 {
                            let name_str = String::from_utf8_lossy(&name_buf).to_lowercase();
                            if !name_str.contains("ti-84") && !name_str.contains("uinput") && !name_str.contains("calculator") && !name_str.contains("virtual") {
                                let mut bit_buf = [0u8; 64];
                                let res_bit = unsafe {
                                    nix::libc::ioctl(f.as_raw_fd(), EVIOCGBIT_KEY_64, bit_buf.as_mut_ptr())
                                };
                                if res_bit >= 0 {
                                    // Check if device supports KEY_W (17)
                                    let has_w = (bit_buf[17 / 8] >> (17 % 8)) & 1 != 0;
                                    if has_w {
                                        let clean_name = String::from_utf8_lossy(&name_buf);
                                        let trimmed = clean_name.trim_matches('\0').trim();
                                        ::utils::info!("[KEYBOARD] Bound physical keyboard device: \"{}\" ({:?})", trimmed, path);
                                        physical_fds.push(f);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            file,
            physical_fds,
            w_pressed: false,
            s_pressed: false,
            a_pressed: false,
            d_pressed: false,
            p_pressed: false,
            o_pressed: false,
            space_pressed: false,
            end_pressed: false,
            jump_timestamps: std::collections::VecDeque::new(),
            total_timestamps: std::collections::VecDeque::new(),
        })
    }

    /// Returns (w, a, s, d) physical key states directly from hardware evdev drivers (/dev/input/event*).
    /// Ignores all synthetic uinput injection.
    pub fn get_physical_wasd(&self) -> (bool, bool, bool, bool) {
        let mut w = false;
        let mut a = false;
        let mut s = false;
        let mut d = false;

        let mut key_buf = [0u8; 64];
        for f in &self.physical_fds {
            let res = unsafe {
                nix::libc::ioctl(f.as_raw_fd(), EVIOCGKEY_64, key_buf.as_mut_ptr())
            };
            if res >= 0 {
                if (key_buf[17 / 8] >> (17 % 8)) & 1 != 0 { w = true; }
                if (key_buf[30 / 8] >> (30 % 8)) & 1 != 0 { a = true; }
                if (key_buf[31 / 8] >> (31 % 8)) & 1 != 0 { s = true; }
                if (key_buf[32 / 8] >> (32 % 8)) & 1 != 0 { d = true; }
            }
        }
        (w, a, s, d)
    }

    /// Checks hardware keyboard input state directly from linux evdev kernel drivers (`/dev/input/event*`).
    /// Returns true if any physical WASD movement key (KEY_W=17, KEY_A=30, KEY_S=31, KEY_D=32) is currently held down on real hardware.
    pub fn is_any_physical_key_pressed(&self) -> bool {
        let (w, a, s, d) = self.get_physical_wasd();
        w || a || s || d
    }

    /// Checks hardware keyboard input state directly from linux evdev kernel drivers (`/dev/input/event*`).
    /// Returns true if physical SPACE key (KEY_SPACE=57) is currently pressed on real hardware.
    pub fn is_physical_space_pressed(&self) -> bool {
        let mut space = false;
        let mut key_buf = [0u8; 64];
        for f in &self.physical_fds {
            let res = unsafe {
                nix::libc::ioctl(f.as_raw_fd(), EVIOCGKEY_64, key_buf.as_mut_ptr())
            };
            if res >= 0 {
                if (key_buf[57 / 8] >> (57 % 8)) & 1 != 0 {
                    space = true;
                }
            }
        }
        space
    }

    /// Checks if Counter-Strike 2 window is currently active/focused in X11
    pub fn is_cs2_focused() -> bool {
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static LAST_CHECK: AtomicU64 = AtomicU64::new(0);
        static IS_FOCUSED: AtomicBool = AtomicBool::new(true);

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let last = LAST_CHECK.load(Ordering::Relaxed);
        if now_ms > last && now_ms - last < 250 {
            return IS_FOCUSED.load(Ordering::Relaxed);
        }

        LAST_CHECK.store(now_ms, Ordering::Relaxed);

        let output = match std::process::Command::new("xdotool")
            .arg("getwindowfocus")
            .arg("getwindowname")
            .output()
        {
            Ok(o) => o,
            Err(_) => {
                IS_FOCUSED.store(true, Ordering::Relaxed);
                return true;
            }
        };

        let focused = if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).to_lowercase();
            name.contains("counter-strike 2") || name.contains("cs2")
        } else {
            true
        };

        IS_FOCUSED.store(focused, Ordering::Relaxed);
        focused
    }

    #[allow(dead_code)]
    pub fn space_press(&mut self) {
        self.key(KEY_SPACE, 1);
    }

    #[allow(dead_code)]
    pub fn space_release(&mut self) {
        self.key(KEY_SPACE, 0);
    }

    pub fn space_pressed(&self) -> bool {
        self.space_pressed
    }

    pub fn end_press(&mut self) {
        self.key(KEY_END, 1);
    }

    pub fn end_release(&mut self) {
        self.key(KEY_END, 0);
    }

    pub fn end_pressed(&self) -> bool {
        self.end_pressed
    }

    pub fn w_press(&mut self) {
        self.key(KEY_W, 1);
    }

    pub fn w_release(&mut self) {
        self.key(KEY_W, 0);
    }

    pub fn s_press(&mut self) {
        self.key(KEY_S, 1);
    }

    pub fn s_release(&mut self) {
        self.key(KEY_S, 0);
    }

    pub fn a_press(&mut self) {
        self.key(KEY_A, 1);
    }

    pub fn a_release(&mut self) {
        self.key(KEY_A, 0);
    }

    pub fn d_press(&mut self) {
        self.key(KEY_D, 1);
    }

    pub fn d_release(&mut self) {
        self.key(KEY_D, 0);
    }

    pub fn w_pressed(&self) -> bool { self.w_pressed }
    pub fn s_pressed(&self) -> bool { self.s_pressed }
    pub fn a_pressed(&self) -> bool { self.a_pressed }
    pub fn d_pressed(&self) -> bool { self.d_pressed }

    pub fn p_press(&mut self) {
        self.key(KEY_P, 1);
    }

    pub fn p_release(&mut self) {
        self.key(KEY_P, 0);
    }

    pub fn o_press(&mut self) {
        self.key(KEY_O, 1);
    }

    pub fn o_release(&mut self) {
        self.key(KEY_O, 0);
    }

    pub fn get_cps(&mut self) -> (u32, u32) {
        let now = std::time::Instant::now();
        while let Some(&t) = self.jump_timestamps.front() {
            if now.duration_since(t) > std::time::Duration::from_secs(1) {
                self.jump_timestamps.pop_front();
            } else {
                break;
            }
        }
        while let Some(&t) = self.total_timestamps.front() {
            if now.duration_since(t) > std::time::Duration::from_secs(1) {
                self.total_timestamps.pop_front();
            } else {
                break;
            }
        }
        (self.jump_timestamps.len() as u32, self.total_timestamps.len() as u32)
    }

    fn key(&mut self, code: u16, pressed: i32) {
        let is_pressed = pressed == 1;
        let state_changed = match code {
            KEY_W => { let c = self.w_pressed != is_pressed; self.w_pressed = is_pressed; c }
            KEY_S => { let c = self.s_pressed != is_pressed; self.s_pressed = is_pressed; c }
            KEY_A => { let c = self.a_pressed != is_pressed; self.a_pressed = is_pressed; c }
            KEY_D => { let c = self.d_pressed != is_pressed; self.d_pressed = is_pressed; c }
            KEY_P => { let c = self.p_pressed != is_pressed; self.p_pressed = is_pressed; c }
            KEY_O => { let c = self.o_pressed != is_pressed; self.o_pressed = is_pressed; c }
            KEY_SPACE => { let c = self.space_pressed != is_pressed; self.space_pressed = is_pressed; c }
            KEY_END => { let c = self.end_pressed != is_pressed; self.end_pressed = is_pressed; c }
            _ => true,
        };

        let now_inst = std::time::Instant::now();
        if is_pressed && state_changed {
            self.total_timestamps.push_back(now_inst);
            if code == KEY_SPACE {
                self.jump_timestamps.push_back(now_inst);
            }
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let press = InputEvent {
            time,
            event_type: EV_KEY,
            code,
            value: pressed,
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        if state_changed {
            self.file.write_all(&press.bytes()).unwrap();
            self.file.write_all(&syn.bytes()).unwrap();
        }
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        let _ = unsafe { ui_dev_destroy(self.file.as_raw_fd()) };
        CREATED.store(false, Ordering::Relaxed);
    }
}
