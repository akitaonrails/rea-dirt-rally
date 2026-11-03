//! Game controller mappings for the game's 2016 SDL (2.0.5).
//!
//! DiRT Rally's Linux port uses its own SDL 2.0.5, which recognises only the
//! pads it knew in 2016 as game controllers. Newer pads (an 8BitDo Ultimate 2
//! on the xpad driver, for one) show up as plain joysticks, so the game
//! skips its own gamepad preset (`input/xinput_pad_*.xml`), starts on the
//! keyboard, and a hand-made mapping does not come back on the next start
//! (docs/RUNTIME_NOTES.md).
//!
//! Feral decides which pads are gamepads from its own list,
//! `share/inputdevices.json`: vendor and product IDs, a type
//! (`Xbox360Gamepad`) and an SDL mapping. A pad missing from it but known
//! to SDL as a game controller is not offered at all (the owner's test with
//! `SDL_GAMECONTROLLERCONFIG`), so the launcher adds pads to that list in
//! its private copy ([`input_devices`]) instead.
//!
//! Linux drivers for modern pads report the standard gamepad layout
//! (Documentation/input/gamepad.rst): BTN_SOUTH … BTN_THUMBR, sticks on
//! ABS_X/Y and ABS_RX/RY, triggers on ABS_Z/RZ, the D-pad on ABS_HAT0. For
//! every connected joystick with that layout, [`Pad::layout`] gives the
//! button and axis numbers in the order SDL 2.0.5 counts them. A pad without
//! that layout gets no entry.

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

    /// The SDL mapping fields (`a:b0,b:b1,…`), when the pad has the
    /// standard layout.
    pub fn layout(&self) -> Option<String> {
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
        Some(parts.join(","))
    }

    /// The name without commas or control characters.
    fn clean_name(&self) -> String {
        let name: String = self
            .name
            .chars()
            .filter(|c| *c != ',' && !c.is_control())
            .collect();
        name.trim().to_owned()
    }

    /// The entry for Feral's `inputdevices.json`, when the pad has the
    /// standard layout.
    pub fn feral_entry(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "Name": self.clean_name(),
            "VendorID": format!("0x{:04x}", self.vendor),
            "ProductIDs": [format!("0x{:04x}", self.product)],
            "Category": "Gamepad",
            "Type": "Xbox360Gamepad",
            "SDLMapping": self.layout()?,
        }))
    }
}

/// Whether Feral's list already has a device with this vendor and product
/// (a `"*"` product matches every product of the vendor).
fn listed(list: &[serde_json::Value], vendor: u16, product: u16) -> bool {
    let hex = |text: &str| u16::from_str_radix(text.trim().trim_start_matches("0x"), 16).ok();
    list.iter().any(|entry| {
        entry["VendorID"].as_str().and_then(hex) == Some(vendor)
            && entry["ProductIDs"].as_array().is_some_and(|ids| {
                ids.iter()
                    .filter_map(serde_json::Value::as_str)
                    .any(|id| id.trim() == "*" || hex(id) == Some(product))
            })
    })
}

/// The vendor and product of a Feral entry (its first product).
fn ids(entry: &serde_json::Value) -> Option<(u16, u16)> {
    let hex = |text: &str| u16::from_str_radix(text.trim().trim_start_matches("0x"), 16).ok();
    Some((
        entry["VendorID"].as_str().and_then(hex)?,
        entry["ProductIDs"].get(0)?.as_str().and_then(hex)?,
    ))
}

/// `known` (the pads seen on earlier starts) with an entry for each pad in
/// `connected` that is new, in order, one per vendor and product.
pub fn remember(known: &[serde_json::Value], connected: &[Pad]) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> =
        known.iter().filter(|e| ids(e).is_some()).cloned().collect();
    for entry in connected.iter().filter_map(Pad::feral_entry) {
        let (vendor, product) = ids(&entry).expect("our own entry has ids");
        if !listed(&out, vendor, product) {
            out.push(entry);
        }
    }
    out
}

/// Feral's `inputdevices.json` (`original`, the game's own) with each of
/// `entries` it does not list. The game's text is kept as it is; new
/// entries go before the closing `]`. A file that is not the JSON array
/// Feral writes is an error, never guessed at.
pub fn input_devices(original: &str, entries: &[serde_json::Value]) -> Result<String, String> {
    let list: Vec<serde_json::Value> =
        serde_json::from_str(original).map_err(|error| format!("inputdevices.json: {error}"))?;
    let mut added: Vec<serde_json::Value> = Vec::new();
    for entry in entries {
        let Some((vendor, product)) = ids(entry) else {
            continue;
        };
        if !listed(&list, vendor, product) && !listed(&added, vendor, product) {
            added.push(entry.clone());
        }
    }
    if added.is_empty() {
        return Ok(original.to_owned());
    }
    let close = original
        .rfind(']')
        .ok_or("inputdevices.json: no closing ]")?;
    let mut out = original[..close].trim_end().to_owned();
    for (i, entry) in added.iter().enumerate() {
        let text = serde_json::to_string_pretty(entry).map_err(|error| error.to_string())?;
        // No comma before the first entry of an empty list.
        out.push_str(if list.is_empty() && i == 0 {
            "\n    "
        } else {
            ",\n    "
        });
        out.push_str(&text.replace('\n', "\n    "));
    }
    out.push('\n');
    out.push_str(&original[close..]);
    Ok(out)
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

    /// Feral's own entry for the wired Xbox 360 pad (share/inputdevices.json).
    const X360: &str = "a:b0,b:b1,back:b6,dpdown:h0.4,dpleft:h0.8,dpright:h0.2,dpup:h0.1,guide:b8,leftshoulder:b4,leftstick:b9,lefttrigger:a2,leftx:a0,lefty:a1,rightshoulder:b5,rightstick:b10,righttrigger:a5,rightx:a3,righty:a4,start:b7,x:b2,y:b3";

    fn sorted(mapping: &str) -> Vec<&str> {
        let mut parts: Vec<&str> = mapping.split(',').filter(|p| !p.is_empty()).collect();
        parts.sort();
        parts
    }

    #[test]
    fn an_xpad_pad_gets_feral_s_own_xbox_360_layout() {
        assert_eq!(sorted(&eightbitdo().layout().unwrap()), sorted(X360));
    }

    #[test]
    fn extra_buttons_and_axes_shift_the_numbers_as_sdl_counts_them() {
        // BTN_TRIGGER (0x120) is bit 32 of the highest word (bits 256..319),
        // and a throttle (ABS_THROTTLE 0x06) comes after the triggers.
        let mut pad = eightbitdo();
        pad.keys = "7cdb000100000000 0 0 0 0".into();
        pad.abs = "3007f".into();
        let layout = pad.layout().unwrap();
        assert!(layout.starts_with("a:b1,"), "{layout}");
        assert!(layout.contains("rightstick:b11,"), "{layout}");
        assert!(layout.contains("righttrigger:a5,"), "{layout}");
    }

    #[test]
    fn a_wheel_or_odd_device_gets_no_entry() {
        let mut wheel = eightbitdo();
        wheel.abs = "3".into(); // only X and Y, no hat
        assert_eq!(wheel.feral_entry(), None);
        let mut few = eightbitdo();
        few.keys = "3000000000000 0 0 0 0".into(); // no gamepad buttons
        assert_eq!(few.feral_entry(), None);
        let mut nameless = eightbitdo();
        nameless.version = 0;
        assert_eq!(nameless.feral_entry(), None);
    }

    /// A trimmed inputdevices.json in Feral's shape.
    const FERAL_LIST: &str = r#"[
	{
		"_Comment": "Originally based on mapping database available at https://github.com/gabomdq/SDL_GameControllerDB"
	},
    {
        "Name": "X360 Controller",
        "VendorID": "0x045e",
        "ProductIDs": [
            "0x028e",
			"*"
        ],
        "Category": "Gamepad",
        "Type": "Xbox360Gamepad",
        "SDLMapping": "a:b0,b:b1"
    }
]
"#;

    #[test]
    fn a_pad_feral_does_not_list_is_added_as_an_xbox_360_gamepad() {
        let out = input_devices(FERAL_LIST, &remember(&[], &[eightbitdo(), eightbitdo()])).unwrap();
        assert!(out.starts_with(FERAL_LIST.trim_end().trim_end_matches(']').trim_end()));
        let list: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(
            list.len(),
            3,
            "one new entry, the same pad twice is added once"
        );
        let added = &list[2];
        assert_eq!(added["Name"], "8BitDo Ultimate 2 Wireless Controller");
        assert_eq!(added["VendorID"], "0x2dc8");
        assert_eq!(added["ProductIDs"][0], "0x310b");
        assert_eq!(added["Category"], "Gamepad");
        assert_eq!(added["Type"], "Xbox360Gamepad");
        assert_eq!(sorted(added["SDLMapping"].as_str().unwrap()), sorted(X360));
        // Built again from the game's own file: the same text.
        assert_eq!(
            input_devices(FERAL_LIST, &remember(&[], &[eightbitdo()])).unwrap(),
            out
        );
    }

    #[test]
    fn a_pad_feral_already_lists_is_left_to_feral() {
        let mut xbox = eightbitdo();
        xbox.vendor = 0x045e;
        xbox.product = 0x02ea; // matched by the vendor's "*"
        assert_eq!(
            input_devices(FERAL_LIST, &remember(&[], &[xbox])).unwrap(),
            FERAL_LIST
        );
        assert_eq!(input_devices(FERAL_LIST, &[]).unwrap(), FERAL_LIST);
    }

    #[test]
    fn a_pad_seen_once_is_remembered_and_new_ones_join_it() {
        let known = remember(&[], &[eightbitdo()]);
        assert_eq!(known.len(), 1);
        // Asleep at the next start: still remembered.
        assert_eq!(remember(&known, &[]), known);
        // Seen again, and a second pad: one entry each.
        let mut other = eightbitdo();
        other.product = 0x3106;
        let both = remember(&known, &[eightbitdo(), other]);
        assert_eq!(both.len(), 2);
        assert_eq!(both[1]["ProductIDs"][0], "0x3106");
        // A broken remembered entry is dropped, not passed to the game.
        let junk = vec![serde_json::json!({"Name": "?"})];
        assert!(remember(&junk, &[]).is_empty());
    }

    #[test]
    fn an_empty_list_stays_valid_json() {
        for empty in ["[]", "[\n]\n", "[ ]"] {
            let out = input_devices(empty, &remember(&[], &[eightbitdo()])).unwrap();
            let list: Vec<serde_json::Value> =
                serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
            assert_eq!(list.len(), 1);
        }
    }

    #[test]
    fn a_broken_list_is_refused_not_guessed() {
        assert!(input_devices("not json", &remember(&[], &[eightbitdo()])).is_err());
        assert!(input_devices("{\"a\": 1}", &remember(&[], &[eightbitdo()])).is_err());
    }

    #[test]
    fn commas_in_a_name_are_dropped() {
        let mut pad = eightbitdo();
        pad.name = "Pad, Model X\n".into();
        assert_eq!(pad.feral_entry().unwrap()["Name"], "Pad Model X");
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

/// This machine's pads as Feral entries (manual check):
/// `cargo test -p dirtrally-launcher --lib this_machines_pads -- --ignored --nocapture`.
#[cfg(test)]
#[test]
#[ignore]
fn this_machines_pads() {
    for pad in pads(Path::new("/sys/class/input")) {
        println!("{}: {:?}", pad.name, pad.feral_entry());
    }
}
