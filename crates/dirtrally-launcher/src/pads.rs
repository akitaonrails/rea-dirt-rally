//! Game controller mappings for the game's 2016 SDL (2.0.5).
//!
//! DiRT Rally's Linux port uses its own SDL 2.0.5, which recognises only the
//! pads it knew in 2016 as game controllers. Newer pads (an 8BitDo Ultimate 2
//! on the xpad driver, for one) show up as plain joysticks, so the game
//! skips its own gamepad preset (`input/xinput_pad_*.xml`), starts on the
//! keyboard, and a hand-made mapping does not come back on the next start
//! (docs/RUNTIME_NOTES.md).
//!
//! Linux drivers for modern pads report the standard gamepad layout
//! (Documentation/input/gamepad.rst): BTN_SOUTH … BTN_THUMBR, sticks on
//! ABS_X/Y and ABS_RX/RY, triggers on ABS_Z/RZ, the D-pad on ABS_HAT0. For
//! every connected joystick with that layout, [`mappings`] writes the line
//! SDL 2.0.5 needs — its GUID (bus, vendor, product, version) and the button
//! and axis numbers in the order SDL 2.0.5 counts them — for
//! `SDL_GAMECONTROLLERCONFIG`. A pad without that layout gets no line.

use std::fs;
use std::path::Path;

// Linux input event codes (include/uapi/linux/input-event-codes.h).
const BTN_MISC: u32 = 0x100;
const BTN_JOYSTICK: u32 = 0x120;
const KEY_MAX: u32 = 0x2ff;
const ABS_MISC: u32 = 0x28;
const ABS_HAT0X: u32 = 0x10;
const ABS_HAT3Y: u32 = 0x17;

/// The standard layout: SDL's name for each part, and the event code.
const BUTTONS: &[(&str, u32)] = &[
    ("a", 0x130),             // BTN_SOUTH
    ("b", 0x131),             // BTN_EAST
    ("x", 0x133),             // BTN_NORTH (xpad's X)
    ("y", 0x134),             // BTN_WEST (xpad's Y)
    ("leftshoulder", 0x136),  // BTN_TL
    ("rightshoulder", 0x137), // BTN_TR
    ("back", 0x13a),          // BTN_SELECT
    ("start", 0x13b),         // BTN_START
    ("guide", 0x13c),         // BTN_MODE
    ("leftstick", 0x13d),     // BTN_THUMBL
    ("rightstick", 0x13e),    // BTN_THUMBR
];
const AXES: &[(&str, u32)] = &[
    ("leftx", 0x00),        // ABS_X
    ("lefty", 0x01),        // ABS_Y
    ("lefttrigger", 0x02),  // ABS_Z
    ("rightx", 0x03),       // ABS_RX
    ("righty", 0x04),       // ABS_RY
    ("righttrigger", 0x05), // ABS_RZ
];

/// One joystick as sysfs describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pad {
    pub name: String,
    pub bus: u16,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
    /// `capabilities/key` and `capabilities/abs`: hexadecimal words, the
    /// highest first.
    pub keys: String,
    pub abs: String,
}

/// Whether bit `code` is set in a sysfs capability bitmap.
fn has(bitmap: &str, code: u32) -> bool {
    let words: Vec<u64> = bitmap
        .split_whitespace()
        .rev()
        .filter_map(|word| u64::from_str_radix(word, 16).ok())
        .collect();
    let (word, bit) = ((code / 64) as usize, code % 64);
    words.get(word).is_some_and(|w| w & (1 << bit) != 0)
}

impl Pad {
    /// SDL 2.0.5's GUID for this device: bus, vendor, product and version,
    /// each as a little-endian 16-bit value followed by two zero bytes.
    pub fn guid(&self) -> String {
        [self.bus, self.vendor, self.product, self.version]
            .iter()
            .map(|v| format!("{:02x}{:02x}0000", v & 0xff, v >> 8))
            .collect()
    }

    /// SDL 2.0.5's number for each button code, in the order it counts
    /// them: `BTN_JOYSTICK..KEY_MAX`, then `BTN_MISC..BTN_JOYSTICK`.
    fn button_index(&self, code: u32) -> Option<usize> {
        (BTN_JOYSTICK..KEY_MAX)
            .chain(BTN_MISC..BTN_JOYSTICK)
            .filter(|c| has(&self.keys, *c))
            .position(|c| c == code)
    }

    /// SDL 2.0.5's number for each axis code (hats are not axes).
    fn axis_index(&self, code: u32) -> Option<usize> {
        (0..ABS_MISC)
            .filter(|c| !(ABS_HAT0X..=ABS_HAT3Y).contains(c))
            .filter(|c| has(&self.abs, *c))
            .position(|c| c == code)
    }

    /// The `SDL_GAMECONTROLLERCONFIG` line, when the pad has the standard
    /// layout.
    pub fn mapping(&self) -> Option<String> {
        if self.vendor == 0 || self.product == 0 || self.version == 0 {
            // SDL 2.0.5 then builds the GUID from the name; leave those alone.
            return None;
        }
        if !(has(&self.abs, ABS_HAT0X) && has(&self.abs, ABS_HAT0X + 1)) {
            return None;
        }
        let mut parts = Vec::new();
        for (name, code) in BUTTONS {
            parts.push(format!("{name}:b{}", self.button_index(*code)?));
        }
        for (name, code) in AXES {
            parts.push(format!("{name}:a{}", self.axis_index(*code)?));
        }
        // The first hat, as SDL numbers its directions.
        parts.extend(["dpup:h0.1", "dpright:h0.2", "dpdown:h0.4", "dpleft:h0.8"].map(String::from));
        let name: String = self
            .name
            .chars()
            .filter(|c| *c != ',' && !c.is_control())
            .collect();
        Some(format!(
            "{},{},{},",
            self.guid(),
            name.trim(),
            parts.join(",")
        ))
    }
}

fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_owned())
}

fn hex16(path: &Path) -> Option<u16> {
    u16::from_str_radix(&read(path)?, 16).ok()
}

/// The joysticks under `class_input` (`/sys/class/input`), by `js*` node.
pub fn pads(class_input: &Path) -> Vec<Pad> {
    let Ok(entries) = fs::read_dir(class_input) else {
        return Vec::new();
    };
    let mut nodes: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with("js"))
        .map(|e| e.path())
        .collect();
    nodes.sort();
    nodes
        .into_iter()
        .filter_map(|node| {
            let dev = node.join("device");
            Some(Pad {
                name: read(&dev.join("name"))?,
                bus: hex16(&dev.join("id/bustype"))?,
                vendor: hex16(&dev.join("id/vendor"))?,
                product: hex16(&dev.join("id/product"))?,
                version: hex16(&dev.join("id/version"))?,
                keys: read(&dev.join("capabilities/key"))?,
                abs: read(&dev.join("capabilities/abs"))?,
            })
        })
        .collect()
}

/// The mapping lines for every standard pad in `pads`, one per GUID.
pub fn mappings(pads: &[Pad]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in pads.iter().filter_map(Pad::mapping) {
        if !lines
            .iter()
            .any(|l| l.split(',').next() == line.split(',').next())
        {
            lines.push(line);
        }
    }
    lines
}

/// `SDL_GAMECONTROLLERCONFIG` for the game: our lines, then the caller's
/// own value, which SDL reads last, so a player's own mapping wins.
pub fn config(ours: &[String], theirs: Option<&str>) -> Option<String> {
    let mut lines: Vec<&str> = ours.iter().map(String::as_str).collect();
    if let Some(theirs) = theirs.map(str::trim).filter(|t| !t.is_empty()) {
        lines.push(theirs);
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The owner's 8BitDo Ultimate 2 Wireless on xpad, as sysfs shows it.
    fn eightbitdo() -> Pad {
        Pad {
            name: "8BitDo Ultimate 2 Wireless Controller".into(),
            bus: 0x0003,
            vendor: 0x2dc8,
            product: 0x310b,
            version: 0x0114,
            keys: "7cdb000000000000 0 0 0 0".into(),
            abs: "3003f".into(),
        }
    }

    #[test]
    fn an_xpad_pad_gets_the_standard_mapping_with_sdl_2_0_5_numbers() {
        assert_eq!(
            eightbitdo().mapping().unwrap(),
            "03000000c82d00000b31000014010000,8BitDo Ultimate 2 Wireless Controller,\
             a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,back:b6,start:b7,guide:b8,\
             leftstick:b9,rightstick:b10,leftx:a0,lefty:a1,lefttrigger:a2,rightx:a3,righty:a4,\
             righttrigger:a5,dpup:h0.1,dpright:h0.2,dpdown:h0.4,dpleft:h0.8,"
        );
    }

    #[test]
    fn extra_buttons_and_axes_shift_the_numbers_as_sdl_counts_them() {
        // A pad with BTN_TRIGGER (0x120) before the gamepad buttons, and a
        // throttle (ABS_THROTTLE 0x06) after the triggers.
        // BTN_TRIGGER (0x120) is bit 32 of the highest word (bits 256..319).
        let mut pad = eightbitdo();
        pad.keys = "7cdb000100000000 0 0 0 0".into();
        pad.abs = "3007f".into();
        let line = pad.mapping().unwrap();
        assert!(line.contains(",a:b1,"), "{line}");
        assert!(line.contains("rightstick:b11,"), "{line}");
        assert!(line.contains("righttrigger:a5,"), "{line}");
    }

    #[test]
    fn a_wheel_or_odd_device_gets_no_mapping() {
        let mut wheel = eightbitdo();
        wheel.abs = "3".into(); // only X and Y, no hat
        assert_eq!(wheel.mapping(), None);
        let mut few = eightbitdo();
        few.keys = "3000000000000 0 0 0 0".into(); // no gamepad buttons
        assert_eq!(few.mapping(), None);
        let mut nameless = eightbitdo();
        nameless.version = 0;
        assert_eq!(nameless.mapping(), None);
    }

    #[test]
    fn commas_in_a_name_cannot_break_the_line() {
        let mut pad = eightbitdo();
        pad.name = "Pad, Model X\n".into();
        assert!(pad.mapping().unwrap().contains(",Pad Model X,a:b0"));
    }

    #[test]
    fn the_players_own_mapping_comes_last_and_wins() {
        let ours = mappings(&[eightbitdo(), eightbitdo()]);
        assert_eq!(ours.len(), 1, "one line per GUID");
        let config = config(&ours, Some("030000005e0400008e02000014010000,Mine,a:b1,")).unwrap();
        let lines: Vec<&str> = config.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].starts_with("030000005e04"));
        assert_eq!(super::config(&[], Some("  ")), None);
    }

    #[test]
    fn joysticks_are_read_from_sysfs() {
        let root = std::env::temp_dir().join(format!("dirtrally-pads-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let dev = root.join("js0/device");
        fs::create_dir_all(dev.join("id")).unwrap();
        fs::create_dir_all(dev.join("capabilities")).unwrap();
        fs::write(dev.join("name"), "8BitDo Ultimate 2 Wireless Controller\n").unwrap();
        for (file, value) in [
            ("bustype", "0003"),
            ("vendor", "2dc8"),
            ("product", "310b"),
            ("version", "0114"),
        ] {
            fs::write(dev.join("id").join(file), format!("{value}\n")).unwrap();
        }
        fs::write(dev.join("capabilities/key"), "7cdb000000000000 0 0 0 0\n").unwrap();
        fs::write(dev.join("capabilities/abs"), "3003f\n").unwrap();
        fs::create_dir_all(root.join("event3")).unwrap(); // not a joystick
        assert_eq!(pads(&root), [eightbitdo()]);
        assert!(pads(&root.join("missing")).is_empty());
        fs::remove_dir_all(&root).unwrap();
    }
}

/// This machine's pads (manual check): `cargo test -p dirtrally-launcher
/// --lib this_machines_pads -- --ignored --nocapture`.
#[cfg(test)]
#[test]
#[ignore]
fn this_machines_pads() {
    for line in mappings(&pads(Path::new("/sys/class/input"))) {
        println!("{line}");
    }
}
