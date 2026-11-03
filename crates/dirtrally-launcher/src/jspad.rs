//! Gamepads for the settings window on Linux: the kernel joystick devices
//! (`/dev/input/js*`) read without blocking, as XInput-style button bits
//! for the SDK's [`rea_launcher::pad::PadNav`].
//!
//! The joystick interface numbers a standard-layout pad's buttons A, B, X,
//! Y, LB, RB, Back, Start, Guide, L3, R3 and its axes left stick X/Y, LT,
//! right stick X/Y, RT, then the D-pad hat X/Y (the same order SDL uses,
//! see `pads.rs`).

use std::fs::File;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use rea_launcher::pad as bits;

const EVENT_BUTTON: u8 = 0x01;
const EVENT_AXIS: u8 = 0x02;
const EVENT_INIT: u8 = 0x80;

/// Joystick button number → XInput bit.
const BUTTONS: [u16; 11] = [
    bits::A,
    bits::B,
    bits::X,
    bits::Y,
    bits::LEFT_SHOULDER,
    bits::RIGHT_SHOULDER,
    bits::BACK,
    bits::START,
    0, // Guide: the system's
    bits::LEFT_THUMB,
    bits::RIGHT_THUMB,
];

/// One pad's state.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct State {
    pub buttons: Vec<bool>,
    pub axes: Vec<i16>,
}

impl State {
    /// Applies one 8-byte `js_event` (time u32, value i16, type u8, number u8).
    pub fn apply(&mut self, event: &[u8; 8]) {
        let value = i16::from_le_bytes([event[4], event[5]]);
        let (kind, number) = (event[6] & !EVENT_INIT, event[7] as usize);
        fn grow<T: Default + Clone>(v: &mut Vec<T>, n: usize) {
            if v.len() <= n {
                v.resize(n + 1, T::default());
            }
        }
        match kind {
            EVENT_BUTTON => {
                grow(&mut self.buttons, number);
                self.buttons[number] = value != 0;
            }
            EVENT_AXIS => {
                grow(&mut self.axes, number);
                self.axes[number] = value;
            }
            _ => {}
        }
    }

    /// The XInput-style bits: buttons, the D-pad hat (axes 6/7) and the left
    /// stick past [`bits::STICK_THRESHOLD`] as the D-pad.
    pub fn mask(&self) -> u16 {
        let mut mask = 0;
        for (i, pressed) in self.buttons.iter().enumerate() {
            if *pressed {
                mask |= BUTTONS.get(i).copied().unwrap_or(0);
            }
        }
        let axis = |i: usize| self.axes.get(i).copied().unwrap_or(0);
        let t = bits::STICK_THRESHOLD;
        let (x, y) = (
            axis(6).saturating_add(axis(0)),
            axis(7).saturating_add(axis(1)),
        );
        if x <= -t {
            mask |= bits::DPAD_LEFT;
        }
        if x >= t {
            mask |= bits::DPAD_RIGHT;
        }
        if y <= -t {
            mask |= bits::DPAD_UP;
        }
        if y >= t {
            mask |= bits::DPAD_DOWN;
        }
        mask
    }
}

/// Every joystick device, polled without blocking; reopened every two
/// seconds so a pad switched on later is found.
#[derive(Default)]
pub struct Pads {
    open: Vec<(File, State)>,
    scanned: Option<Instant>,
}

impl Pads {
    fn scan(&mut self) {
        self.open.clear();
        let Ok(entries) = std::fs::read_dir("/dev/input") else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with("js") {
                continue;
            }
            if let Ok(file) = open_nonblocking(&entry.path()) {
                self.open.push((file, State::default()));
            }
        }
    }

    /// The bits of every pad together.
    pub fn buttons(&mut self) -> u16 {
        if self
            .scanned
            .is_none_or(|at| at.elapsed() > Duration::from_secs(2))
        {
            self.scan();
            self.scanned = Some(Instant::now());
        }
        let mut mask = 0;
        self.open.retain_mut(|(file, state)| {
            let mut event = [0u8; 8];
            loop {
                match file.read(&mut event) {
                    Ok(8) => state.apply(&event),
                    Ok(_) => return false,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => return false,
                }
            }
            mask |= state.mask();
            true
        });
        mask
    }
}

fn open_nonblocking(path: &Path) -> std::io::Result<File> {
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: u8, number: u8, value: i16) -> [u8; 8] {
        let v = value.to_le_bytes();
        [0, 0, 0, 0, v[0], v[1], kind, number]
    }

    #[test]
    fn a_pad_becomes_xinput_bits() {
        let mut state = State::default();
        state.apply(&event(EVENT_BUTTON | EVENT_INIT, 0, 1)); // A, initial state
        state.apply(&event(EVENT_BUTTON, 7, 1)); // Start
        state.apply(&event(EVENT_AXIS, 7, -32_767)); // hat up
        state.apply(&event(EVENT_AXIS, 0, 20_000)); // stick right
        assert_eq!(
            state.mask(),
            bits::A | bits::START | bits::DPAD_UP | bits::DPAD_RIGHT
        );
        state.apply(&event(EVENT_BUTTON, 0, 0));
        state.apply(&event(EVENT_AXIS, 0, 8_000)); // not past the threshold
        assert_eq!(state.mask(), bits::START | bits::DPAD_UP);
    }

    #[test]
    fn unknown_buttons_and_the_guide_do_nothing() {
        let mut state = State::default();
        state.apply(&event(EVENT_BUTTON, 8, 1));
        state.apply(&event(EVENT_BUTTON, 15, 1));
        assert_eq!(state.mask(), 0);
    }

    #[test]
    fn start_plays_once_per_press() {
        let mut state = State::default();
        state.apply(&event(EVENT_BUTTON, 7, 1));
        let mut nav = rea_launcher::pad::PadNav::default();
        assert_eq!(nav.update(state.mask()), [rea_launcher::pad::Nav::Play]);
        assert!(nav.update(state.mask()).is_empty());
    }
}
