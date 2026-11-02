//! The Linux port's own `preferences` file: Feral's registry, as XML
//! (`~/.local/share/feral-interactive/DiRT Rally/preferences`).
//!
//! ```xml
//! <registry>
//!     <key name="HKEY_CURRENT_USER">
//!         <key name="Software">
//!             <key name="Feral Interactive">
//!                 <key name="DiRT Rally">
//!                     <key name="Setup">
//!                         <value name="FullScreen" type="integer">1</value>
//! ```
//!
//! The launcher writes only the values it owns ([`start_values`]). Each edit
//! splices those values into the original text, so every other byte — the
//! player's settings, the order, the indentation — stays as the game wrote
//! it. A missing file is seeded with just the owned values; the game fills
//! in the rest on its first run.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// The key the game keeps its launch settings in.
pub const SETUP: &[&str] = &[
    "HKEY_CURRENT_USER",
    "Software",
    "Feral Interactive",
    "DiRT Rally",
    "Setup",
];

/// A registry value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value<'a> {
    Integer(i64),
    Text(&'a str),
}

impl Value<'_> {
    fn kind(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Text(_) => "string",
        }
    }

    fn text(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Text(text) => escape(text),
        }
    }
}

/// The values the launcher always sets in [`SETUP`]:
/// "Unsupported Linux distribution" was already seen (docs/RESEARCH.md #6).
pub const OWNED: &[(&str, Value<'static>)] = &[(
    "SeenSpecificationAlertUUIDLnxDistributionUnsupported",
    Value::Integer(1),
)];

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn name_of(element: &BytesStart) -> Option<String> {
    element
        .try_get_attribute("name")
        .ok()
        .flatten()
        .and_then(|attr| {
            attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
        })
        .map(|value| value.into_owned())
}

/// One edit: replace `start..end` with `text`.
struct Splice {
    start: usize,
    end: usize,
    text: String,
}

/// The whitespace before byte `at` on its line: the indentation there.
fn indent_before(text: &str, at: usize) -> &str {
    let line_start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    let indent = &text[line_start..at];
    if indent.chars().all(char::is_whitespace) {
        indent
    } else {
        ""
    }
}

/// One `<value>` element.
fn value_element(name: &str, value: &Value) -> String {
    format!(
        "<value name=\"{}\" type=\"{}\">{}</value>",
        escape(name),
        value.kind(),
        value.text()
    )
}

/// The new elements for `values`, each on its own line at `indent`,
/// followed by a line break and `close_indent` (the indentation of the
/// closing tag they go before).
fn new_values(values: &[(&str, Value)], indent: &str, close_indent: &str) -> String {
    let mut out = String::new();
    for (name, value) in values {
        out.push_str(indent);
        out.push_str(&value_element(name, value));
        out.push('\n');
    }
    out.push_str(close_indent);
    out
}

/// The keys from `path[from..]` as nested elements holding `values`.
fn new_keys(path: &[&str], values: &[(&str, Value)], indent: &str, step: &str) -> String {
    let Some((first, rest)) = path.split_first() else {
        return String::new();
    };
    let inner = format!("{indent}{step}");
    let body = if rest.is_empty() {
        new_values(values, &inner, indent)
    } else {
        format!("{}{indent}", new_keys(rest, values, &inner, step))
    };
    format!("{indent}<key name=\"{}\">\n{body}</key>\n", escape(first))
}

/// `text` with `values` set in the key at `path`. Values already present
/// keep their place and type attribute; missing values and keys are added
/// at the end of their parent, indented like their siblings.
pub fn set_values(text: &str, path: &[&str], values: &[(&str, Value)]) -> Result<String, String> {
    let mut reader = Reader::from_str(text);
    let mut stack: Vec<String> = Vec::new();
    // How deep `stack` matches `path` (the stack starts with "registry").
    let matched = |stack: &[String]| {
        stack
            .iter()
            .skip(1)
            .zip(path)
            .take_while(|(have, want)| have == want)
            .count()
    };
    // Which of `values` this element is, when it sits right in the key at `path`.
    let wanted = |stack: &[String], element: &BytesStart| {
        let at_path = stack.len() == path.len() + 1 && matched(stack) == path.len();
        name_of(element)
            .filter(|_| at_path)
            .and_then(|name| values.iter().position(|(n, _)| *n == name))
    };
    let mut found: Vec<bool> = vec![false; values.len()];
    let mut splices: Vec<Splice> = Vec::new();
    // The value whose text is being read: (index in `values`, text start).
    let mut in_value: Option<(usize, usize)> = None;
    let mut saw_registry = false;
    loop {
        let before = reader.buffer_position() as usize;
        let event = reader
            .read_event()
            .map_err(|error| format!("preferences: {error} at byte {before}"))?;
        let after = reader.buffer_position() as usize;
        match event {
            Event::Start(element) => {
                let tag = element.name().as_ref().to_vec();
                match tag.as_slice() {
                    b"registry" if stack.is_empty() => {
                        saw_registry = true;
                        stack.push("registry".into());
                    }
                    b"key" => stack.push(name_of(&element).unwrap_or_default()),
                    b"value" => {
                        if let Some(i) = wanted(&stack, &element) {
                            in_value = Some((i, after));
                        }
                        stack.push(String::new());
                    }
                    _ => stack.push(String::new()),
                }
            }
            Event::Empty(element) if element.name().as_ref() == b"value" => {
                // <value name="X" type="integer"/>: give it its text.
                if let Some(i) = wanted(&stack, &element) {
                    found[i] = true;
                    let (name, value) = values[i];
                    splices.push(Splice {
                        start: before,
                        end: after,
                        text: value_element(name, &value),
                    });
                }
            }
            Event::End(element) => {
                if let Some((i, start)) = in_value
                    .take()
                    .filter(|_| element.name().as_ref() == b"value")
                {
                    found[i] = true;
                    splices.push(Splice {
                        start,
                        end: before,
                        text: values[i].1.text(),
                    });
                }
                let depth = matched(&stack);
                let is_key =
                    element.name().as_ref() == b"key" || element.name().as_ref() == b"registry";
                // Closing the deepest existing key on the way to `path`:
                // whatever is still missing goes in here.
                if is_key && depth == stack.len().saturating_sub(1) {
                    let missing: Vec<(&str, Value)> = values
                        .iter()
                        .zip(&found)
                        .filter(|(_, f)| !**f)
                        .map(|(v, _)| *v)
                        .collect();
                    if !missing.is_empty() {
                        let close_indent = indent_before(text, before);
                        let step = "    ";
                        let indent = format!("{close_indent}{step}");
                        // Back up over the closing tag's own indentation.
                        let at = before - close_indent.len();
                        let insert = if depth == path.len() {
                            new_values(&missing, &indent, close_indent)
                        } else {
                            format!(
                                "{}{close_indent}",
                                new_keys(&path[depth..], &missing, &indent, step)
                            )
                        };
                        splices.push(Splice {
                            start: at,
                            end: before,
                            text: insert,
                        });
                        found.fill(true);
                    }
                }
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !saw_registry {
        return Err("preferences: no <registry> element".into());
    }
    splices.sort_by_key(|s| s.start);
    let mut out = String::with_capacity(text.len() + 256);
    let mut at = 0;
    for splice in splices {
        out.push_str(&text[at..splice.start]);
        out.push_str(&splice.text);
        at = splice.end;
    }
    out.push_str(&text[at..]);
    Ok(out)
}

/// A new `preferences` file holding only `values` at `path`.
pub fn seed(path: &[&str], values: &[(&str, Value)]) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<registry>\n{}</registry>\n",
        new_keys(path, values, "    ", "    ")
    )
}

/// How the game starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// Feral's pre-game window first, as shipped: the launcher leaves its
    /// values alone. `restore` puts the window back once after the launcher
    /// had hidden it.
    FeralWindow { restore: bool },
    /// Straight into the game at this size. The game crashes when its
    /// pre-game window is skipped while `ScreenW`/`ScreenH` are still 0, so
    /// both are always set; `CrashCounter` above 0 would bring the window
    /// back after a crash (docs/RUNTIME_NOTES.md).
    Direct { width: u32, height: u32 },
}

/// The values the launcher sets for `start` (on top of [`OWNED`]).
pub fn start_values(start: Start) -> Vec<(&'static str, Value<'static>)> {
    let mut values = OWNED.to_vec();
    match start {
        Start::FeralWindow { restore: false } => {}
        Start::FeralWindow { restore: true } => {
            values.push(("GameOptionsDialogShouldShow", Value::Integer(1)));
        }
        Start::Direct { width, height } => values.extend([
            ("GameOptionsDialogShown", Value::Integer(1)),
            ("GameOptionsDialogShouldShow", Value::Integer(0)),
            ("ScreenW", Value::Integer(width.into())),
            ("ScreenH", Value::Integer(height.into())),
            ("CrashCounter", Value::Integer(0)),
        ]),
    }
    values
}

/// The file's new text for `start`: the values set in `existing`, or a
/// seeded file when there is none.
pub fn configure(existing: Option<&str>, start: Start) -> Result<String, String> {
    let values = start_values(start);
    match existing {
        Some(text) => set_values(text, SETUP, &values),
        None => Ok(seed(SETUP, &values)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape the game writes (trimmed).
    const GAME_WRITTEN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<registry>
    <key name="HKEY_CLASSES_ROOT">
    </key>
    <key name="HKEY_CURRENT_USER">
        <key name="Software">
            <key name="Feral Interactive">
                <key name="DiRT Rally">
                    <key name="Setup">
                        <value name="CrashCounter" type="integer">0</value>
                        <value name="FullScreen" type="integer">1</value>
                        <value name="SpecificationUpdateURL" type="string">https://u1.feralinteractive.com/a?b=1&amp;c=2</value>
                    </key>
                </key>
            </key>
            <key name="IndirectX">
                <key name="Direct3D">
                    <key name="Config">
                        <value name="FSAALevel" type="integer">0</value>
                    </key>
                </key>
            </key>
        </key>
    </key>
</registry>
"#;

    #[test]
    fn a_missing_value_is_added_with_its_siblings_indentation_and_nothing_else_moves() {
        let out = configure(Some(GAME_WRITTEN), Start::FeralWindow { restore: false }).unwrap();
        let expected = GAME_WRITTEN.replace(
            "&amp;c=2</value>\n                    </key>",
            "&amp;c=2</value>\n                        <value name=\"SeenSpecificationAlertUUIDLnxDistributionUnsupported\" type=\"integer\">1</value>\n                    </key>",
        );
        assert_eq!(out, expected);
        // A second run changes nothing.
        assert_eq!(
            configure(Some(&out), Start::FeralWindow { restore: false }).unwrap(),
            out
        );
    }

    #[test]
    fn an_existing_value_is_replaced_in_place() {
        let out = set_values(GAME_WRITTEN, SETUP, &[("FullScreen", Value::Integer(0))]).unwrap();
        assert_eq!(
            out,
            GAME_WRITTEN.replace(
                "\"FullScreen\" type=\"integer\">1<",
                "\"FullScreen\" type=\"integer\">0<"
            )
        );
        // The same name elsewhere is not the one at the path.
        let out = set_values(GAME_WRITTEN, SETUP, &[("FSAALevel", Value::Integer(4))]).unwrap();
        assert!(out.contains("\"FSAALevel\" type=\"integer\">0<"));
        assert!(out.contains(
            "                        <value name=\"FSAALevel\" type=\"integer\">4</value>\n                    </key>\n                </key>\n            </key>\n            <key name=\"IndirectX\">"
        ));
    }

    #[test]
    fn text_values_are_escaped_and_an_empty_element_gets_its_text() {
        let text = GAME_WRITTEN.replace(
            "<value name=\"CrashCounter\" type=\"integer\">0</value>",
            "<value name=\"CrashCounter\" type=\"integer\"/>",
        );
        let out = set_values(
            &text,
            SETUP,
            &[
                ("CrashCounter", Value::Integer(0)),
                ("Note", Value::Text("a<b&\"c\"")),
            ],
        )
        .unwrap();
        assert!(out.contains("<value name=\"CrashCounter\" type=\"integer\">0</value>"));
        assert!(
            out.contains("<value name=\"Note\" type=\"string\">a&lt;b&amp;&quot;c&quot;</value>")
        );
    }

    #[test]
    fn missing_keys_are_created_under_the_deepest_existing_one() {
        let text = "<?xml version=\"1.0\"?>\n<registry>\n    <key name=\"HKEY_CURRENT_USER\">\n    </key>\n</registry>\n";
        let out = set_values(text, SETUP, OWNED).unwrap();
        let expected = "<?xml version=\"1.0\"?>\n<registry>\n    <key name=\"HKEY_CURRENT_USER\">\n        <key name=\"Software\">\n            <key name=\"Feral Interactive\">\n                <key name=\"DiRT Rally\">\n                    <key name=\"Setup\">\n                        <value name=\"SeenSpecificationAlertUUIDLnxDistributionUnsupported\" type=\"integer\">1</value>\n                    </key>\n                </key>\n            </key>\n        </key>\n    </key>\n</registry>\n";
        assert_eq!(out, expected);
        assert_eq!(set_values(&out, SETUP, OWNED).unwrap(), out);
    }

    #[test]
    fn a_seeded_file_reads_back_as_the_same_values() {
        let seeded = configure(None, Start::FeralWindow { restore: false }).unwrap();
        assert!(seeded.starts_with("<?xml"));
        assert_eq!(set_values(&seeded, SETUP, OWNED).unwrap(), seeded);
        assert!(seeded.contains("<key name=\"Feral Interactive\">"));
    }

    #[test]
    fn a_broken_file_is_refused_not_guessed() {
        assert!(set_values("not xml at all", SETUP, OWNED).is_err());
        assert!(set_values("<registry><key name=\"a\"></registry>", SETUP, OWNED).is_err());
    }

    fn value_of(text: &str, name: &str) -> Option<String> {
        let start = text.find(&format!("<value name=\"{name}\""))?;
        let rest = &text[start..];
        let open = rest.find('>')? + 1;
        let close = rest.find("</value>")?;
        Some(rest[open..close].to_owned())
    }

    #[test]
    fn a_direct_start_skips_the_pre_game_window_at_the_chosen_size() {
        let crashed = GAME_WRITTEN.replace(
            "\"CrashCounter\" type=\"integer\">0<",
            "\"CrashCounter\" type=\"integer\">3<",
        );
        let out = configure(
            Some(&crashed),
            Start::Direct {
                width: 2560,
                height: 1440,
            },
        )
        .unwrap();
        for (name, want) in [
            ("GameOptionsDialogShown", "1"),
            ("GameOptionsDialogShouldShow", "0"),
            ("ScreenW", "2560"),
            ("ScreenH", "1440"),
            ("CrashCounter", "0"),
            ("SeenSpecificationAlertUUIDLnxDistributionUnsupported", "1"),
            // The player's own values stay.
            ("FullScreen", "1"),
        ] {
            assert_eq!(value_of(&out, name).as_deref(), Some(want), "{name}");
        }
        // The values are set once, not added again.
        let again = configure(
            Some(&out),
            Start::Direct {
                width: 2560,
                height: 1440,
            },
        )
        .unwrap();
        assert_eq!(again, out);
    }

    #[test]
    fn going_back_to_the_original_look_brings_the_pre_game_window_back_once() {
        let direct = configure(
            None,
            Start::Direct {
                width: 1280,
                height: 800,
            },
        )
        .unwrap();
        let back = configure(Some(&direct), Start::FeralWindow { restore: true }).unwrap();
        assert_eq!(
            value_of(&back, "GameOptionsDialogShouldShow").as_deref(),
            Some("1")
        );
        // Without a restore, Original leaves the player's choice in the window alone.
        let theirs = back.replace(
            "\"GameOptionsDialogShouldShow\" type=\"integer\">1<",
            "\"GameOptionsDialogShouldShow\" type=\"integer\">0<",
        );
        let kept = configure(Some(&theirs), Start::FeralWindow { restore: false }).unwrap();
        assert_eq!(kept, theirs);
    }
}
