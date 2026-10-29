// settings.toml in %LOCALAPPDATA%\pdesk, read once at start (docs/specs/0001-settings-file/index.md).
// Not next to the exe: the installed exe lives in Program Files, where you cannot write (spec 0008).
// Hand edited and only reported through the log, so a bad value resets just that setting
// and a file that is not TOML resets all of them. pdesk always starts.
use std::fmt;
use std::io::Write;
use std::sync::{LazyLock, OnceLock};
use toml::{Table, Value};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub const WIN: u8 = 1;
pub const CTRL: u8 = 2;
pub const ALT: u8 = 4;
pub const SHIFT: u8 = 8;

/// A shortcut: exactly these modifiers (either side of the keyboard) plus one key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Combo {
    pub mods: u8,
    pub vk: u16,
}

/// What text the overview shows under the windows in its spread (spec 0010).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Labels {
    Hover,
    Always,
    None,
}

const LABELS: &[(&str, Labels)] = &[("hover", Labels::Hover), ("always", Labels::Always), ("none", Labels::None)];

#[derive(PartialEq, Debug)]
pub struct Settings {
    pub workspaces: usize,
    pub copilot_as_right_ctrl: bool,
    pub follow_activation: bool,
    pub previous: Vec<Combo>,
    pub next: Vec<Combo>,
    pub send_previous: Vec<Combo>, // these two switch along with the window
    pub send_next: Vec<Combo>,
    pub send_previous_stay: Vec<Combo>, // these two leave the screen where it is
    pub send_next_stay: Vec<Combo>,
    // Modifier sets for the digits 1 to 9 (spec 0005); None = off.
    pub jump: Option<u8>,
    pub send_to: Option<u8>, // and follow
    pub send_to_stay: Option<u8>,
    pub finder: Vec<Combo>, // opens the window finder (spec 0007)
    pub overview: Vec<Combo>, // opens the workspace overview (spec 0010)
    pub indicator_enabled: bool,
    pub hold_ms: u32,
    pub fade_ms: u32,
    pub restore_enabled: bool,
    pub restore_minutes: u32,
    pub overview_labels: Labels,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            workspaces: 4,
            copilot_as_right_ctrl: true,
            follow_activation: true,
            previous: vec![Combo { mods: WIN | CTRL, vk: VK_LEFT.0 }],
            next: vec![Combo { mods: WIN | CTRL, vk: VK_RIGHT.0 }],
            send_previous: vec![Combo { mods: WIN | CTRL | SHIFT, vk: VK_LEFT.0 }],
            send_next: vec![Combo { mods: WIN | CTRL | SHIFT, vk: VK_RIGHT.0 }],
            send_previous_stay: vec![Combo { mods: WIN | CTRL | ALT, vk: VK_LEFT.0 }],
            send_next_stay: vec![Combo { mods: WIN | CTRL | ALT, vk: VK_RIGHT.0 }],
            jump: Some(WIN),
            send_to: Some(WIN | SHIFT),
            send_to_stay: Some(WIN | CTRL),
            finder: vec![Combo { mods: WIN, vk: b'F' as u16 }],
            overview: vec![Combo { mods: WIN, vk: VK_TAB.0 }],
            indicator_enabled: true,
            hold_ms: 550,
            fade_ms: 200,
            restore_enabled: true,
            restore_minutes: 3,
            overview_labels: Labels::Hover,
        }
    }
}

/// Written verbatim when the file is missing. Must parse to `Settings::default()` (tested).
const DEFAULT_FILE: &str = r#"# pdesk settings. Restart pdesk after editing.

workspaces = 4              # 2 to 9, every screen
copilot_as_right_ctrl = true
follow_activation = true    # switch to a hidden window's workspace when it comes to the front

[shortcuts]
# Keep these two if you switch with the touchpad swipe.
previous = ["Win+Ctrl+Left"]
next     = ["Win+Ctrl+Right"]
# Send the focused window to the previous or next workspace (wraps around).
send_previous      = ["Win+Ctrl+Shift+Left"]   # and follow it there
send_next          = ["Win+Ctrl+Shift+Right"]
send_previous_stay = ["Win+Ctrl+Alt+Left"]     # and stay where you are
send_next_stay     = ["Win+Ctrl+Alt+Right"]
# Go straight to workspace N, or send the focused window there (N = 1 to 9, number row or numpad).
jump         = "Win"         # "" turns a set off
send_to      = "Win+Shift"   # and follow it there
send_to_stay = "Win+Ctrl"    # and stay where you are
# Search every window on every screen and workspace; Enter goes there.
finder = ["Win+F"]
# Every workspace of every screen at once, like Task View (which it replaces on Win+Tab).
overview = ["Win+Tab"]

[indicator]
enabled = true
hold_ms = 550               # 0 to 5000
fade_ms = 200               # 0 to 2000

[restore]
enabled = true              # after a reboot or sign in, hide reopened windows back into their workspaces
minutes = 3                 # how long after start to keep matching (1 to 30)

[overview]
labels = "hover"            # "hover", "always" or "none"
"#;

const TOP_KEYS: &[&str] = &["workspaces", "copilot_as_right_ctrl", "follow_activation", "shortcuts", "indicator", "restore", "overview"];
const SHORTCUT_KEYS: &[&str] = &["previous", "next", "send_previous", "send_next", "send_previous_stay", "send_next_stay", "jump", "send_to", "send_to_stay", "finder", "overview"];
const INDICATOR_KEYS: &[&str] = &["enabled", "hold_ms", "fade_ms"];
const RESTORE_KEYS: &[&str] = &["enabled", "minutes"];
const OVERVIEW_KEYS: &[&str] = &["labels"];

static SETTINGS: OnceLock<Settings> = OnceLock::new();
// Kept apart from SETTINGS so an early get() can never block load() from filling it.
static DEFAULTS: LazyLock<Settings> = LazyLock::new(Settings::default);

pub fn get() -> &'static Settings {
    SETTINGS.get().unwrap_or(&DEFAULTS)
}

/// settings.toml in the data folder, shared by the installed and the dev copy.
pub fn path() -> std::path::PathBuf {
    crate::ws::data_dir().join("settings.toml")
}

/// Read the file (writing the default one if missing) and log every problem. Main thread, once.
pub fn load() {
    let path = path();
    let settings = match std::fs::read_to_string(&path) {
        Ok(text) => {
            let (settings, warnings) = parse(&text);
            for w in warnings {
                eprintln!("settings: {w}");
            }
            settings
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // create_new: never overwrite a file that appeared meanwhile.
            let written = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).and_then(|mut f| f.write_all(DEFAULT_FILE.as_bytes()));
            match written {
                Ok(()) => println!("settings: wrote defaults to {}", path.display()),
                Err(e) => eprintln!("settings: cannot write {} ({e}), using all defaults", path.display()),
            }
            Settings::default()
        }
        Err(e) => {
            eprintln!("settings: cannot read {} ({e}), using all defaults", path.display());
            Settings::default()
        }
    };
    println!("settings: {}: {settings}", path.display());
    let _ = SETTINGS.set(settings);
}

/// Pure, so the tests can run it: the settings plus one message per problem found.
fn parse(text: &str) -> (Settings, Vec<String>) {
    let mut s = Settings::default();
    let mut warn = Vec::new();
    let top: Table = match text.parse() {
        Ok(t) => t,
        Err(e) => {
            // First line of the error carries the line and column; the message says what.
            let full = e.to_string();
            let at = full.lines().next().unwrap_or("");
            warn.push(format!("not valid TOML ({at}: {}), using all defaults", e.message()));
            return (s, warn);
        }
    };
    unknown_keys(&top, "", TOP_KEYS, &mut warn);
    s.workspaces = int(&top, "workspaces", "workspaces", 2, 9, s.workspaces as i64, &mut warn) as usize;
    s.copilot_as_right_ctrl = boolean(&top, "copilot_as_right_ctrl", "copilot_as_right_ctrl", s.copilot_as_right_ctrl, &mut warn);
    s.follow_activation = boolean(&top, "follow_activation", "follow_activation", s.follow_activation, &mut warn);

    if let Some(t) = section(&top, "shortcuts", &mut warn) {
        unknown_keys(t, "shortcuts.", SHORTCUT_KEYS, &mut warn);
        let previous = combos(t, "previous", "shortcuts.previous", &s.previous, &mut warn);
        let next = combos(t, "next", "shortcuts.next", &s.next, &mut warn);
        // Checked after refilling empty lists, so a clash with a default is caught too.
        if let Some(c) = previous.iter().find(|c| next.contains(c)) {
            warn.push(format!("shortcuts: {c} is in both previous and next, using defaults for both"));
        } else {
            (s.previous, s.next) = (previous, next);
        }
        // Switch keys always win: a send list (or the finder, or the overview) sharing a combo with
        // them, or with an earlier list still on, is turned off (an empty list never matches)
        // (specs 0004, 0007, 0010).
        let mut taken = vec![("previous", s.previous.clone()), ("next", s.next.clone())];
        let sends = [
            ("send_previous", &mut s.send_previous),
            ("send_next", &mut s.send_next),
            ("send_previous_stay", &mut s.send_previous_stay),
            ("send_next_stay", &mut s.send_next_stay),
            ("finder", &mut s.finder),
            ("overview", &mut s.overview),
        ];
        for (key, list) in sends {
            let got = combos(t, key, &format!("shortcuts.{key}"), list, &mut warn);
            match got.iter().find_map(|c| Some((c, taken.iter().find(|(_, l)| l.contains(c))?.0))) {
                Some((c, other)) => {
                    warn.push(format!("shortcuts: {c} is in both {other} and {key}, {key} turned off"));
                    list.clear();
                }
                None => {
                    taken.push((key, got.clone()));
                    *list = got;
                }
            }
        }
        // Digit sets (spec 0005): one with the same modifiers as an earlier set is turned off.
        let mut sets: Vec<(&str, u8)> = Vec::new();
        for (key, set) in [("jump", &mut s.jump), ("send_to", &mut s.send_to), ("send_to_stay", &mut s.send_to_stay)] {
            *set = mods(t, key, *set, &mut warn);
            let Some(m) = *set else { continue };
            match sets.iter().find(|(_, other)| *other == m) {
                Some((other, _)) => {
                    warn.push(format!("shortcuts: {} is in both {other} and {key}, {key} turned off", mods_text(*set)));
                    *set = None;
                }
                None => sets.push((key, m)),
            }
        }
        // The hook tries the lists first, so a list keeps its digit and the set the other eight.
        for (list, got) in &taken {
            for c in got.iter().filter(|c| (b'1' as u16..=b'9' as u16).contains(&c.vk)) {
                if let Some((key, _)) = sets.iter().find(|(_, m)| *m == c.mods) {
                    warn.push(format!("shortcuts: {c} is in {list}, {key} skips digit {}", c.vk as u8 as char));
                }
            }
        }
    }

    if let Some(t) = section(&top, "indicator", &mut warn) {
        unknown_keys(t, "indicator.", INDICATOR_KEYS, &mut warn);
        s.indicator_enabled = boolean(t, "enabled", "indicator.enabled", s.indicator_enabled, &mut warn);
        s.hold_ms = int(t, "hold_ms", "indicator.hold_ms", 0, 5000, s.hold_ms as i64, &mut warn) as u32;
        s.fade_ms = int(t, "fade_ms", "indicator.fade_ms", 0, 2000, s.fade_ms as i64, &mut warn) as u32;
    }

    if let Some(t) = section(&top, "restore", &mut warn) {
        unknown_keys(t, "restore.", RESTORE_KEYS, &mut warn);
        s.restore_enabled = boolean(t, "enabled", "restore.enabled", s.restore_enabled, &mut warn);
        s.restore_minutes = int(t, "minutes", "restore.minutes", 1, 30, s.restore_minutes as i64, &mut warn) as u32;
    }

    if let Some(t) = section(&top, "overview", &mut warn) {
        unknown_keys(t, "overview.", OVERVIEW_KEYS, &mut warn);
        s.overview_labels = choice(t, "labels", "overview.labels", LABELS, s.overview_labels, &mut warn);
    }
    (s, warn)
}

/// One word out of `options`, any case. A bad value keeps `default`.
fn choice<T: Copy + PartialEq>(t: &Table, key: &str, name: &str, options: &[(&str, T)], default: T, warn: &mut Vec<String>) -> T {
    let words = || options.iter().map(|(w, _)| format!("\"{w}\"")).collect::<Vec<_>>().join(", ");
    let fallback = options.iter().find(|(_, v)| *v == default).map_or("", |(w, _)| w);
    match t.get(key) {
        None => default,
        Some(Value::String(text)) => match options.iter().find(|(w, _)| w.eq_ignore_ascii_case(text)) {
            Some((_, v)) => *v,
            None => {
                warn.push(format!("{name}: \"{text}\" is not one of {}, using default \"{fallback}\"", words()));
                default
            }
        },
        Some(v) => {
            warn.push(format!("{name}: expected one of {}, got a {}, using default \"{fallback}\"", words(), v.type_str()));
            default
        }
    }
}

fn unknown_keys(t: &Table, prefix: &str, known: &[&str], warn: &mut Vec<String>) {
    for key in t.keys().filter(|k| !known.contains(&k.as_str())) {
        warn.push(format!("unknown key {prefix}{key}, ignored"));
    }
}

fn section<'a>(t: &'a Table, key: &str, warn: &mut Vec<String>) -> Option<&'a Table> {
    match t.get(key)? {
        Value::Table(t) => Some(t),
        v => {
            warn.push(format!("{key}: expected a [{key}] section, got a {}, using defaults for it", v.type_str()));
            None
        }
    }
}

fn int(t: &Table, key: &str, name: &str, lo: i64, hi: i64, default: i64, warn: &mut Vec<String>) -> i64 {
    match t.get(key) {
        None => default,
        Some(Value::Integer(n)) if (lo..=hi).contains(n) => *n,
        Some(Value::Integer(n)) => {
            warn.push(format!("{name}: {n} is not between {lo} and {hi}, using default {default}"));
            default
        }
        Some(v) => {
            warn.push(format!("{name}: expected a whole number, got a {}, using default {default}", v.type_str()));
            default
        }
    }
}

fn boolean(t: &Table, key: &str, name: &str, default: bool, warn: &mut Vec<String>) -> bool {
    match t.get(key) {
        None => default,
        Some(Value::Boolean(b)) => *b,
        Some(v) => {
            warn.push(format!("{name}: expected true or false, got a {}, using default {default}", v.type_str()));
            default
        }
    }
}

/// Bad entries are dropped one by one; an empty result falls back to `default`.
fn combos(t: &Table, key: &str, name: &str, default: &[Combo], warn: &mut Vec<String>) -> Vec<Combo> {
    let fallback = || default.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ");
    let items = match t.get(key) {
        None => return default.to_vec(),
        Some(Value::Array(items)) => items,
        Some(v) => {
            warn.push(format!("{name}: expected a list like [\"Win+Ctrl+Left\"], got a {}, using default [{}]", v.type_str(), fallback()));
            return default.to_vec();
        }
    };
    let mut out = Vec::new();
    for item in items {
        let Some(text) = item.as_str() else {
            warn.push(format!("{name}: entry {item} is not text, dropped"));
            continue;
        };
        match parse_combo(text) {
            Ok(c) if !out.contains(&c) => out.push(c),
            Ok(_) => {} // listed twice: harmless
            Err(why) => warn.push(format!("{name}: \"{text}\" {why}, dropped")),
        }
    }
    if out.is_empty() {
        warn.push(format!("{name}: no valid entry left, using default [{}]", fallback()));
        return default.to_vec();
    }
    out
}

/// A digit set: text like "Win+Ctrl", or "" for off. A bad value keeps `default`.
fn mods(t: &Table, key: &str, default: Option<u8>, warn: &mut Vec<String>) -> Option<u8> {
    let text = match t.get(key) {
        None => return default,
        Some(Value::String(text)) => text,
        Some(v) => {
            warn.push(format!("shortcuts.{key}: expected text like \"Win+Ctrl\", got a {}, using {}", v.type_str(), mods_text(default)));
            return default;
        }
    };
    match parse_mods(text) {
        Ok(m) => m,
        Err(why) => {
            warn.push(format!("shortcuts.{key}: \"{text}\" {why}, using {}", mods_text(default)));
            default
        }
    }
}

/// Modifiers only, joined by `+`, any case. Shift alone would eat typed digits and symbols.
fn parse_mods(text: &str) -> Result<Option<u8>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    let mut mods = 0u8;
    for part in text.split('+').map(str::trim) {
        let bit = mod_bit(part);
        if bit == 0 {
            return Err(format!("has \"{part}\", which is not Win, Ctrl, Alt or Shift"));
        }
        if mods & bit != 0 {
            return Err(format!("repeats {part}"));
        }
        mods |= bit;
    }
    if mods & (WIN | CTRL | ALT) == 0 {
        return Err("needs Win, Ctrl or Alt".into());
    }
    Ok(Some(mods))
}

fn mods_text(mods: Option<u8>) -> String {
    let Some(mods) = mods else { return "off".into() };
    let names = [(WIN, "Win"), (CTRL, "Ctrl"), (ALT, "Alt"), (SHIFT, "Shift")];
    names.iter().filter(|(bit, _)| mods & bit != 0).map(|(_, name)| *name).collect::<Vec<_>>().join("+")
}

fn mod_bit(name: &str) -> u8 {
    match name.to_ascii_lowercase().as_str() {
        "win" => WIN,
        "ctrl" => CTRL,
        "alt" => ALT,
        "shift" => SHIFT,
        _ => 0,
    }
}

/// "Win+Ctrl+Left": parts joined by `+`, any order, any case, at least one modifier.
fn parse_combo(text: &str) -> Result<Combo, String> {
    let (mut mods, mut vk) = (0u8, None);
    for part in text.split('+').map(str::trim) {
        let bit = mod_bit(part);
        if bit != 0 {
            if mods & bit != 0 {
                return Err(format!("repeats {part}"));
            }
            mods |= bit;
        } else if vk.is_some() {
            return Err("has more than one key".into());
        } else {
            vk = Some(vk_of(part).ok_or_else(|| format!("has \"{part}\", which is not a key pdesk knows"))?);
        }
    }
    let vk = vk.ok_or("has no key")?;
    if mods == 0 {
        return Err("needs at least one of Win, Ctrl, Alt, Shift".into());
    }
    Ok(Combo { mods, vk })
}

const NAMED_KEYS: &[(&str, VIRTUAL_KEY)] = &[
    ("Left", VK_LEFT), ("Right", VK_RIGHT), ("Up", VK_UP), ("Down", VK_DOWN),
    ("Home", VK_HOME), ("End", VK_END), ("PageUp", VK_PRIOR), ("PageDown", VK_NEXT),
    ("Insert", VK_INSERT), ("Delete", VK_DELETE), ("Space", VK_SPACE), ("Enter", VK_RETURN),
    ("Tab", VK_TAB), ("Escape", VK_ESCAPE),
];

/// Named keys, A to Z, 0 to 9 (their vk is the ASCII code), F1 to F24.
fn vk_of(name: &str) -> Option<u16> {
    if let Some((_, vk)) = NAMED_KEYS.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) {
        return Some(vk.0);
    }
    let up = name.to_ascii_uppercase();
    match up.as_bytes() {
        [c @ (b'A'..=b'Z' | b'0'..=b'9')] => Some(*c as u16),
        [b'F', _, ..] => up[1..].parse::<u16>().ok().filter(|n| (1..=24).contains(n)).map(|n| VK_F1.0 + n - 1),
        _ => None,
    }
}

impl fmt::Display for Combo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (bit, name) in [(WIN, "Win"), (CTRL, "Ctrl"), (ALT, "Alt"), (SHIFT, "Shift")] {
            if self.mods & bit != 0 {
                write!(f, "{name}+")?;
            }
        }
        match NAMED_KEYS.iter().find(|(_, vk)| vk.0 == self.vk) {
            Some((name, _)) => write!(f, "{name}"),
            None if (VK_F1.0..=VK_F24.0).contains(&self.vk) => write!(f, "F{}", self.vk - VK_F1.0 + 1),
            None => write!(f, "{}", self.vk as u8 as char),
        }
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let list = |l: &[Combo]| l.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ");
        write!(
            f,
            "workspaces={} copilot_as_right_ctrl={} follow_activation={} shortcuts.previous=[{}] shortcuts.next=[{}] shortcuts.send_previous=[{}] shortcuts.send_next=[{}] shortcuts.send_previous_stay=[{}] shortcuts.send_next_stay=[{}] shortcuts.jump={} shortcuts.send_to={} shortcuts.send_to_stay={} shortcuts.finder=[{}] shortcuts.overview=[{}] indicator.enabled={} indicator.hold_ms={} indicator.fade_ms={} restore.enabled={} restore.minutes={} overview.labels={}",
            self.workspaces, self.copilot_as_right_ctrl, self.follow_activation, list(&self.previous), list(&self.next), list(&self.send_previous), list(&self.send_next), list(&self.send_previous_stay), list(&self.send_next_stay), mods_text(self.jump), mods_text(self.send_to), mods_text(self.send_to_stay), list(&self.finder), list(&self.overview), self.indicator_enabled, self.hold_ms, self.fade_ms, self.restore_enabled, self.restore_minutes, LABELS.iter().find(|(_, v)| *v == self.overview_labels).map_or("", |(w, _)| w)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The file pdesk writes for you and the values it falls back to are two copies of the
    // defaults. If they drift, a fresh install behaves differently from a broken file.
    #[test]
    fn default_file_is_the_defaults() {
        let (s, warn) = parse(DEFAULT_FILE);
        assert!(warn.is_empty(), "{warn:?}");
        assert_eq!(s, Settings::default());
        // parse() starts from the defaults, so also prove every key is spelled out in the file.
        let top: Table = DEFAULT_FILE.parse().unwrap();
        let (sc, ind, res) = (top["shortcuts"].as_table().unwrap(), top["indicator"].as_table().unwrap(), top["restore"].as_table().unwrap());
        assert!(TOP_KEYS.iter().all(|k| top.contains_key(*k)));
        assert!(SHORTCUT_KEYS.iter().all(|k| sc.contains_key(*k)));
        assert!(INDICATOR_KEYS.iter().all(|k| ind.contains_key(*k)));
        assert!(RESTORE_KEYS.iter().all(|k| res.contains_key(*k)));
        assert!(OVERVIEW_KEYS.iter().all(|k| top["overview"].as_table().unwrap().contains_key(*k)));
    }

    // "none" is how you get a spread with no text at all, so it must stick; a typo costs only
    // the labels, back to titles on hover.
    #[test]
    fn a_bad_overview_labels_only_resets_itself() {
        let (s, warn) = parse("[overview]\nlabels = \"None\"\n");
        assert_eq!(s.overview_labels, Labels::None);
        assert!(warn.is_empty(), "{warn:?}");
        let (s, warn) = parse("workspaces = 6\n[overview]\nlabels = \"sometimes\"\n");
        assert_eq!((s.workspaces, s.overview_labels), (6, Labels::Hover));
        assert_eq!(warn, ["overview.labels: \"sometimes\" is not one of \"hover\", \"always\", \"none\", using default \"hover\""]);
    }

    // minutes = 0 would end the restore before any app reopens; it must not also switch restore off.
    #[test]
    fn a_bad_restore_minutes_only_resets_itself() {
        let (s, warn) = parse("[restore]\nenabled = false\nminutes = 0\n");
        assert!(!s.restore_enabled);
        assert_eq!(s.restore_minutes, 3);
        assert_eq!(warn.len(), 1, "{warn:?}");
    }

    // The whole point of reading key by key: a typo costs one setting, never the rest.
    #[test]
    fn a_bad_value_only_resets_itself() {
        let text = "workspaces = 12\nfollow_activation = \"yes\"\ntypo = 1\n[shortcuts]\nprevious = [\"alt+CTRL+left\", \"Ctrl+Banana\", 7]\n[indicator]\nhold_ms = \"long\"\nfade_ms = 0\n";
        let (s, warn) = parse(text);
        assert_eq!(s.workspaces, 4);
        // A typo must not silently switch follow off: on is the safe side, typing never goes invisible.
        assert!(s.follow_activation);
        assert_eq!(s.previous, [Combo { mods: ALT | CTRL, vk: VK_LEFT.0 }]);
        assert_eq!(s.hold_ms, 550);
        assert_eq!(s.fade_ms, 0);
        assert_eq!(warn.len(), 6, "{warn:?}"); // workspaces, follow_activation, typo, Banana, 7, hold_ms
    }

    // follow_activation = false is the escape from an app that keeps stealing focus, so it must stick.
    #[test]
    fn follow_activation_can_be_turned_off() {
        let (s, warn) = parse("follow_activation = false\n");
        assert!(!s.follow_activation);
        assert!(warn.is_empty(), "{warn:?}");
    }

    // One combo in both lists would make a key do two things; neither list can be trusted.
    #[test]
    fn a_clash_resets_both_lists() {
        let (s, warn) = parse("[shortcuts]\nprevious = [\"Win+Ctrl+Right\"]\n");
        assert_eq!((s.previous, s.next), (Settings::default().previous, Settings::default().next));
        assert_eq!(warn.len(), 1, "{warn:?}");
    }

    // Win+Ctrl+Left/Right and the swipe must keep switching, whatever the send lists say.
    #[test]
    fn a_send_list_clashing_with_a_switch_list_is_turned_off() {
        let (s, warn) = parse("[shortcuts]
send_next_stay = [\"Win+Ctrl+Right\"]
");
        let d = Settings::default();
        assert!(s.send_next_stay.is_empty());
        assert_eq!((&s.previous, &s.next), (&d.previous, &d.next));
        assert_eq!((&s.send_previous, &s.send_next, &s.send_previous_stay), (&d.send_previous, &d.send_next, &d.send_previous_stay));
        assert_eq!(warn, ["shortcuts: Win+Ctrl+Right is in both next and send_next_stay, send_next_stay turned off"]);
    }

    // Moving a switch shortcut onto a default send combo must not make one key do two things.
    #[test]
    fn a_switch_list_beats_a_default_send_list() {
        let (s, warn) = parse("[shortcuts]
previous = [\"Win+Ctrl+Shift+Left\"]
");
        assert_eq!(s.previous, [Combo { mods: WIN | CTRL | SHIFT, vk: VK_LEFT.0 }]);
        assert!(s.send_previous.is_empty());
        assert_eq!(warn.len(), 1, "{warn:?}");
    }

    // A stay combo that is also a follow combo would send and switch at once; the earlier list keeps it.
    #[test]
    fn a_stay_list_clashing_with_a_follow_list_is_turned_off() {
        let (s, warn) = parse("[shortcuts]
send_previous_stay = [\"Win+Ctrl+Shift+Right\"]
");
        assert_eq!(s.send_next, Settings::default().send_next);
        assert!(s.send_previous_stay.is_empty());
        assert_eq!(warn.len(), 1, "{warn:?}");
    }

    // A typo in one send list costs that list only, back to its default.
    #[test]
    fn a_bad_send_entry_only_resets_its_list() {
        let (s, warn) = parse("[shortcuts]
send_next = [\"Win+Banana\"]
send_previous = [\"Win+Alt+N\"]
");
        let d = Settings::default();
        assert_eq!(s.send_next, d.send_next);
        assert_eq!(s.send_previous, [Combo { mods: WIN | ALT, vk: b'N' as u16 }]);
        assert_eq!((&s.send_previous_stay, &s.send_next_stay), (&d.send_previous_stay, &d.send_next_stay));
        assert_eq!(warn.len(), 2, "{warn:?}"); // Banana dropped, then no valid entry left
    }

    // "" is how you give Win+N back to the taskbar, so it must turn only that set off.
    #[test]
    fn an_empty_digit_set_is_off() {
        let (s, warn) = parse("[shortcuts]\njump = \"\"\n");
        let d = Settings::default();
        assert_eq!(s.jump, None);
        assert_eq!((s.send_to, s.send_to_stay), (d.send_to, d.send_to_stay));
        assert!(warn.is_empty(), "{warn:?}");
    }

    // Shift alone would swallow every typed ! @ #, so it is refused; a typo costs only its own key.
    #[test]
    fn a_bad_digit_set_only_resets_itself() {
        let (s, warn) = parse("[shortcuts]\njump = \"Shift\"\nsend_to = \"Win+Hyper\"\nsend_to_stay = \"alt+WIN\"\n");
        let d = Settings::default();
        assert_eq!((s.jump, s.send_to), (d.jump, d.send_to));
        assert_eq!(s.send_to_stay, Some(WIN | ALT));
        assert_eq!(warn.len(), 2, "{warn:?}");
    }

    // One modifier set for two actions would make a digit both jump and send; the earlier set keeps it.
    #[test]
    fn a_digit_set_clashing_with_an_earlier_one_is_turned_off() {
        let (s, warn) = parse("[shortcuts]\nsend_to_stay = \"win\"\n");
        assert_eq!(s.jump, Some(WIN));
        assert_eq!(s.send_to_stay, None);
        assert_eq!(warn, ["shortcuts: Win is in both jump and send_to_stay, send_to_stay turned off"]);
    }

    // An explicit list on one digit must not cost the set its other eight digits.
    #[test]
    fn a_list_on_a_digit_only_takes_that_digit() {
        let (s, warn) = parse("[shortcuts]\nnext = [\"Win+Ctrl+2\"]\n");
        assert_eq!(s.next, [Combo { mods: WIN | CTRL, vk: b'2' as u16 }]);
        assert_eq!(s.send_to_stay, Some(WIN | CTRL));
        assert_eq!(warn, ["shortcuts: Win+Ctrl+2 is in next, send_to_stay skips digit 2"]);
    }

    // Win+Ctrl+Right must keep switching (and the swipe with it) even if the finder is set onto it.
    #[test]
    fn a_finder_combo_clashing_with_a_switch_list_turns_the_finder_off() {
        let (s, warn) = parse("[shortcuts]
finder = [\"Win+Ctrl+Right\"]
");
        assert_eq!(s.next, Settings::default().next);
        assert!(s.finder.is_empty());
        assert_eq!(warn, ["shortcuts: Win+Ctrl+Right is in both next and finder, finder turned off"]);
    }

    // The hook tries the lists first, so a finder on Win+3 really does take 3 from jump; the log must say so.
    #[test]
    fn a_finder_on_a_digit_takes_that_digit_from_its_set() {
        let (s, warn) = parse("[shortcuts]
finder = [\"Win+3\"]
");
        assert_eq!(s.finder, [Combo { mods: WIN, vk: b'3' as u16 }]);
        assert_eq!(s.jump, Some(WIN));
        assert_eq!(warn, ["shortcuts: Win+3 is in finder, jump skips digit 3"]);
    }

    // Win+F must keep opening the finder if the overview is set onto it: the earlier list keeps
    // the combo, so one key never opens both.
    #[test]
    fn an_overview_combo_clashing_with_the_finder_turns_the_overview_off() {
        let (s, warn) = parse("[shortcuts]
overview = [\"Win+F\"]
");
        assert_eq!(s.finder, Settings::default().finder);
        assert!(s.overview.is_empty());
        assert_eq!(warn, ["shortcuts: Win+F is in both finder and overview, overview turned off"]);
    }

    #[test]
    fn combo_text_round_trips() {
        for text in ["Win+Ctrl+Left", "Ctrl+Shift+A", "Alt+F24", "Win+7", "Ctrl+PageDown"] {
            assert_eq!(parse_combo(text).unwrap().to_string(), text);
        }
        for bad in ["Left", "Win+Win+Left", "Win+A+B", "Win+", "Win+F25", "Win+Ctrl"] {
            assert!(parse_combo(bad).is_err(), "{bad}");
        }
    }
}
