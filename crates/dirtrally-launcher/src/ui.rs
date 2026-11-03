//! The settings window, on `rea_launcher_ui`.
//!
//! [`draw`] is the whole window as a function of [`Window`], so the
//! headless tests below drive the real widgets (egui_kittest) on the host
//! and under `sdk/scripts/wine-test`; `show` wraps it in eframe.
//!
//! Two states share one layout (the hero photo on the left, the title and
//! status, the tabs, the footer with the Play plate):
//!
//! - **Getting ready**: no game folder yet, or one the launcher cannot play
//!   (the reason, and for an unknown build its SHA-256 and the report
//!   link). Play stays off until the folder is right.
//! - **Ready**: the looks, then Play. The first Play makes the private copy
//!   of the game with its progress in place of the tabs ("12.4 of 40.1 GB"),
//!   then starts the game. Later starts copy only what changed.
//!
//! Any change to the window's look or layout goes through a design skill
//! (docs/STANDARDS.md "Settings window and CLI").

use std::path::PathBuf;

use rea_launcher::cfg::{OptionSpec, spec};
use rea_launcher_ui::egui::{self, Rect, RichText, TextureHandle, pos2, vec2};
use rea_launcher_ui::spec::{mark_rendered, spec_path_row, spec_row};
use rea_launcher_ui::theme::{body, condensed};
use rea_launcher_ui::widgets::{self, Crop, LookCard};
use rea_launcher_ui::window::Layout;

use crate::config::{Look, OPTIONS};
use crate::copy;
use crate::edition;
use crate::form::Form;
use crate::palette::{BASE, CHALK, ERROR, INK, MUTED, PANEL};

/// Width of the hero image strip.
pub const PHOTO_W: f32 = 280.0;

/// The hero image: Kris Meeke's Citroën DS3 WRC on a gravel forest stage of
/// the 2015 Wales Rally GB (the year and the country of DiRT Rally's own
/// Welsh stages). Chris Cox, CC BY 2.0, via Wikimedia Commons
/// (THIRD_PARTY.md). Cropped and resized for the strip.
pub const HERO_JPEG: Option<&[u8]> = Some(include_bytes!("../assets/hero.jpg"));

/// The image's credit, shown on it (CC BY asks for attribution).
pub const HERO_CREDIT: &str = "Photo: Chris Cox, CC BY 2.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
    Play,
    Picture,
}

impl Tab {
    pub const NAMES: [(Tab, &'static str); 2] = [(Tab::Play, "Play"), (Tab::Picture, "Picture")];

    /// The next (`by` = 1) or previous (-1) tab, wrapping (pad LB/RB).
    pub fn step(self, by: isize) -> Tab {
        let at = Tab::NAMES
            .iter()
            .position(|(tab, _)| *tab == self)
            .unwrap_or(0) as isize;
        let len = Tab::NAMES.len() as isize;
        Tab::NAMES[(at + by).rem_euclid(len) as usize].0
    }
}

/// What the game-folder check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Empty,
    /// A playable edition: its name and the game folder it is in (the
    /// chosen folder or its only subfolder).
    Found(&'static str, PathBuf),
    /// Why the folder cannot be played: no game, an unknown build (its
    /// SHA-256 and the report link), or a known build that does not run.
    Problem(String),
}

/// Checks a game-folder text as typed or picked.
pub fn check_game(folder: &str) -> Check {
    let folder = folder.trim();
    if folder.is_empty() {
        return Check::Empty;
    }
    match edition::find_playable(std::path::Path::new(folder)) {
        Ok(found) => Check::Found(found.edition.name, found.root),
        Err(problem) => Check::Problem(problem),
    }
}

/// The window's state.
pub struct Window {
    pub form: Form,
    pub tab: Tab,
    /// The screen the window opened on, for the looks.
    pub screen: Option<(u32, u32)>,
    /// Started by the Linux helper.
    pub linux: bool,
    pub check: Check,
    /// The folder text [`Window::check`] is for.
    checked: String,
    /// The folder picker.
    pub pick: fn() -> Option<PathBuf>,
    pub hero: Option<TextureHandle>,
    /// A message that replaces the status line (a failed save or copy).
    pub error: Option<String>,
    /// The private copy being made after Play, while it runs.
    pub copy: Option<copy::State>,
    /// Play was pressed while ready: the caller saves, makes the copy and
    /// closes.
    pub play: bool,
}

impl Window {
    /// The window over a `launcher.cfg` text.
    pub fn new(text: &str, screen: Option<(u32, u32)>) -> Window {
        let form = Form::from_text(text);
        let checked = form.get("game").to_owned();
        Window {
            check: check_game(&checked),
            checked,
            form,
            tab: Tab::Play,
            screen,
            linux: false,
            pick: || None,
            hero: None,
            error: None,
            copy: None,
            play: false,
        }
    }

    /// A playable edition is chosen, every value can be saved, and no copy
    /// is running.
    pub fn ready(&self) -> bool {
        matches!(self.check, Check::Found(..))
            && self.form.problems().is_empty()
            && self.copy.is_none()
    }

    /// Play (the plate, Enter or the pad's Start): only when ready; else
    /// back to the Play tab, where the reason is shown.
    pub fn press_play(&mut self) {
        if self.ready() {
            self.play = true;
        } else if self.copy.is_none() {
            self.tab = Tab::Play;
        }
    }

    fn refresh_check(&mut self) {
        if self.form.get("game") != self.checked {
            self.checked = self.form.get("game").to_owned();
            self.check = check_game(&self.checked);
        }
    }
}

fn row(key: &str) -> &'static OptionSpec {
    spec::find(OPTIONS, key).unwrap_or_else(|| panic!("no option row {key}"))
}

/// The whole window, filling `ui`.
pub fn draw(ui: &mut egui::Ui, w: &mut Window) {
    w.refresh_check();
    let full = ui.max_rect();
    ui.painter().rect_filled(full, 0.0, BASE);
    let layout = Layout::new(full, PHOTO_W);
    draw_hero(ui, layout.photo, w.hero.as_ref());
    ui.scope_builder(egui::UiBuilder::new().max_rect(layout.body), |ui| {
        ui.add(
            egui::Label::new(RichText::new("DiRT Rally").font(condensed(38.0)).color(INK)).wrap(),
        );
        status(ui, w);
        ui.add_space(6.0);
        if let Some(state) = w.copy.clone() {
            copy_panel(ui, &state);
            return;
        }
        widgets::tabs_bar(ui, &Tab::NAMES, &mut w.tab);
        ui.add_space(14.0);
        widgets::tab_body(ui, w.tab, |ui| match w.tab {
            Tab::Play => play_tab(ui, w),
            Tab::Picture => picture_tab(ui, w),
        });
    });
    let hint = match (&w.copy, &w.check) {
        (Some(_), _) => "The game starts as soon as the copy is ready.",
        (None, Check::Found(..)) => {
            "Your game folder is only read. Settings are saved when you press Play."
        }
        (None, _) => "Play starts once the launcher has found your game.",
    };
    if widgets::footer(ui, layout.footer, hint, w.ready()).clicked() {
        w.press_play();
    }
}

fn draw_hero(ui: &egui::Ui, rect: Rect, hero: Option<&TextureHandle>) {
    let painter = ui.painter();
    if hero.is_none() {
        painter.rect_filled(rect, 0.0, PANEL);
    }
    widgets::hero_image(painter, rect, hero, Crop::Center, BASE);
    if !HERO_CREDIT.is_empty() {
        // A solid backing so the small credit stays readable over the photo.
        let text = painter.layout_no_wrap(HERO_CREDIT.to_owned(), body(11.0), INK);
        let at = pos2(rect.left() + 12.0, rect.bottom() - 12.0 - text.size().y);
        let back = Rect::from_min_size(at, text.size()).expand2(vec2(6.0, 3.0));
        painter.rect_filled(back, 3.0, BASE);
        painter.galley(at, text, INK);
    }
}

fn status(ui: &mut egui::Ui, w: &Window) {
    let problems = w.form.problems();
    let (text, color) = match (&w.error, &w.copy, &w.check) {
        (Some(error), _, _) => (error.clone(), ERROR),
        (None, Some(_), _) => ("Making your private copy of the game.".to_owned(), MUTED),
        (None, None, Check::Found(..)) if !problems.is_empty() => (problems.join(" "), ERROR),
        (None, None, Check::Found(..)) => ("Ready. Press Play.".to_owned(), MUTED),
        (None, None, Check::Problem(_)) => (
            "The launcher can't play this folder. The reason is below.".to_owned(),
            ERROR,
        ),
        (None, None, Check::Empty) => ("Choose your DiRT Rally folder to start.".to_owned(), MUTED),
    };
    ui.add(egui::Label::new(RichText::new(text).font(body(14.0)).color(color)).wrap());
}

/// The first copy's progress, in place of the tabs.
fn copy_panel(ui: &mut egui::Ui, state: &copy::State) {
    ui.add_space(18.0);
    widgets::heading(ui, "Copying the game");
    ui.add_space(6.0);
    let line = match state {
        copy::State::Copying { done, total } if *total > 0 => copy::amount(*done, *total),
        copy::State::Copying { .. } => "Counting the game's files.".to_owned(),
        copy::State::Done { .. } => "Done. Starting the game.".to_owned(),
        copy::State::Failed(error) => error.clone(),
    };
    ui.label(RichText::new(line).font(body(15.0)).color(INK));
    ui.add_space(8.0);
    widgets::progress_bar(ui, state.fraction());
    ui.add_space(10.0);
    widgets::note(
        ui,
        "The launcher plays from its own copy so your game folder is never changed. \
         This happens once; later starts copy only what changed. From another disk or \
         a network share the first copy takes a while.",
    );
}

fn play_tab(ui: &mut egui::Ui, w: &mut Window) {
    if spec_path_row(ui, row("game"), w.form.value_mut("game"), w.pick).changed() {
        w.error = None;
    }
    // The field's own description says what to choose; this says what was found.
    let found = match &w.check {
        Check::Empty => None,
        Check::Found(name, _) => Some((format!("Found DiRT Rally {name}."), CHALK)),
        Check::Problem(problem) => Some((problem.clone(), ERROR)),
    };
    if let Some((note, color)) = found {
        ui.add(
            egui::Label::new(RichText::new(note).font(body(13.0)).color(color)).selectable(true),
        );
    }
    ui.add_space(12.0);

    widgets::heading(ui, row("look").label);
    // The cards are the `look` option's window control.
    mark_rendered(ui.ctx(), "look");
    ui.add_space(6.0);
    let recommended = Look::recommended(w.screen);
    let gap = 10.0;
    let count = Look::ALL.len() as f32;
    let width = (ui.available_width() - (count - 1.0) * gap) / count;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for look in Look::ALL {
            let note = look_note(look);
            let card = LookCard {
                title: look.label(),
                note: &note,
                selected: w.form.look() == Some(look),
                badge: (look == recommended).then_some("For this screen"),
                ..LookCard::default()
            };
            if widgets::look_card(ui, vec2(width, 96.0), &card).clicked() {
                w.form.choose_look(look);
            }
        }
    });
}

/// A look card's note: its values in words.
fn look_note(look: Look) -> String {
    use rea_launcher::cfg::Resolution;
    match look.picture().resolution {
        Some(Resolution::Size(w, h)) => format!("Straight into the game at {w}×{h}."),
        Some(Resolution::Fullscreen) | None => {
            "Straight into the game at your screen's size.".to_owned()
        }
        Some(Resolution::Original) => "The game's own pre-game window first.".to_owned(),
    }
}

/// Every option row that is not on the Play tab and has a window control.
fn picture_tab(ui: &mut egui::Ui, w: &mut Window) {
    let mut changed = false;
    for row in OPTIONS
        .iter()
        .filter(|row| !row.launcher_only && !matches!(row.key, "game" | "look"))
    {
        changed |= spec_row(ui, row, w.form.value_mut(row.key)).changed();
        ui.add_space(10.0);
    }
    widgets::note(
        ui,
        "Graphics quality is the game's own: change it in the game's Options, under Graphics.",
    );
    if changed {
        w.form.picture_edited();
    }
}

pub use app::{Outcome, show};

mod app {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use rea_launcher::fsx;
    use rea_launcher_ui::eframe;
    use rea_launcher_ui::egui;
    #[cfg(target_os = "linux")]
    use rea_launcher_ui::pad::joystick::Joysticks;
    use rea_launcher_ui::pad::{self, Nav, PadNav};
    use rea_launcher_ui::theme;
    use rea_launcher_ui::widgets;
    use rea_launcher_ui::window::{SIZE, WindowFit};

    use super::{Check, HERO_JPEG, Window, draw};
    use crate::config;
    use crate::copy::{self, CopyJob};
    use crate::palette::{METRICS, PALETTE};

    /// How the window ended.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Outcome {
        /// Play was pressed; the settings are saved and the copy is made.
        Play,
        /// Closed without Play.
        Closed,
    }

    /// The pads, as XInput-style bits: the SDK's `/dev/input/js*` reader
    /// on Linux, XInput on Windows.
    #[derive(Default)]
    pub(super) struct Pads {
        #[cfg(target_os = "linux")]
        pub(super) js: Joysticks,
    }

    impl Pads {
        pub(super) fn buttons(&mut self) -> u16 {
            #[cfg(target_os = "linux")]
            {
                self.js.buttons()
            }
            #[cfg(windows)]
            {
                rea_launcher_win32::xinput::buttons()
            }
            #[cfg(not(any(windows, target_os = "linux")))]
            {
                0
            }
        }
    }

    /// The folder picker: the native dialog on Windows, zenity or kdialog
    /// on Linux (whichever the desktop has).
    fn pick_folder() -> Option<PathBuf> {
        const TITLE: &str = "Choose your DiRT Rally folder";
        #[cfg(windows)]
        {
            rea_launcher_win32::dialog::pick_folder(TITLE)
        }
        #[cfg(not(windows))]
        {
            use std::process::Command;
            let tries: [(&str, &[&str]); 2] = [
                (
                    "zenity",
                    &["--file-selection", "--directory", "--title", TITLE],
                ),
                (
                    "kdialog",
                    &["--getexistingdirectory", ".", "--title", TITLE],
                ),
            ];
            for (program, args) in tries {
                if let Ok(out) = Command::new(program).args(args).output()
                    && out.status.success()
                {
                    let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
                    if !text.is_empty() {
                        return Some(PathBuf::from(text));
                    }
                }
            }
            None
        }
    }

    /// The screen size, for the looks (a Steam Deck's screen picks its look).
    fn screen() -> Option<(u32, u32)> {
        #[cfg(windows)]
        {
            rea_launcher_win32::display::screen_size()
        }
        #[cfg(not(windows))]
        {
            let out = std::process::Command::new("xrandr")
                .arg("--current")
                .output()
                .ok()?;
            crate::start::screen_from_xrandr(&String::from_utf8_lossy(&out.stdout))
        }
    }

    struct App<'a> {
        window: Window,
        fit: WindowFit,
        path: PathBuf,
        /// The `launcher.cfg` text the window's values are saved into.
        text: String,
        runtime_default: PathBuf,
        job: Option<CopyJob>,
        pads: Pads,
        pad: PadNav,
        pending: Vec<Nav>,
        outcome: &'a mut Outcome,
    }

    /// Shows the window over `saved` (the `launcher.cfg` text, or `None` on
    /// the first run). Play saves to `path`, makes the private copy (into
    /// the `runtime` setting, or `runtime_default`) with its progress, then
    /// closes with [`Outcome::Play`].
    pub fn show(
        path: &Path,
        saved: Option<&str>,
        linux: bool,
        runtime_default: &Path,
    ) -> Result<Outcome, String> {
        let screen = screen();
        #[cfg(windows)]
        let fit = {
            use rea_launcher_win32::{display, wine};
            let scale = display::wine_dpi_scale();
            WindowFit::new(SIZE, wine::is_wine(), screen, scale).with_native_fit(true)
        };
        // On Linux the desktop scales the window; the X screen may span
        // several monitors, so it does not size the window.
        #[cfg(not(windows))]
        let fit = WindowFit::new(SIZE, false, None, 1.0);
        let options = fit.native_options("DiRT Rally");
        let text = saved.map_or_else(|| config::first_run_text(screen), str::to_owned);
        let mut outcome = Outcome::Closed;
        eframe::run_native(
            "DiRT Rally",
            options,
            Box::new(|cc| {
                theme::install_with(&cc.egui_ctx, &PALETTE, &METRICS);
                let mut window = Window::new(&text, screen);
                window.linux = linux;
                window.pick = pick_folder;
                window.hero =
                    HERO_JPEG.and_then(|jpeg| widgets::load_jpeg(&cc.egui_ctx, "hero", jpeg));
                Ok(Box::new(App {
                    window,
                    fit,
                    path: path.to_path_buf(),
                    text: text.clone(),
                    runtime_default: runtime_default.to_path_buf(),
                    job: None,
                    pads: Pads::default(),
                    pad: PadNav::default(),
                    pending: Vec::new(),
                    outcome: &mut outcome,
                }))
            }),
        )
        .map_err(|error| error.to_string())?;
        Ok(outcome)
    }

    impl App<'_> {
        /// Saves the settings, then starts the private copy.
        fn save_and_copy(&mut self) {
            let text = self.window.form.save(&self.text);
            if let Some(dir) = self.path.parent()
                && let Err(error) = std::fs::create_dir_all(dir)
            {
                self.window.error = Some(format!("Cannot save the settings: {error}"));
                return;
            }
            if let Err(error) = fsx::write_atomic(&self.path, &text) {
                self.window.error = Some(format!("Cannot save the settings: {error}"));
                return;
            }
            self.text = text;
            let Check::Found(_, root) = &self.window.check else {
                return;
            };
            let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
            let home = std::env::var_os(home_var)
                .map(PathBuf::from)
                .unwrap_or_default();
            let cwd = std::env::current_dir().unwrap_or_default();
            let runtime = self.window.form.settings().runtime.as_deref().map_or_else(
                || self.runtime_default.clone(),
                |path| crate::start::resolve_runtime(path, &home, &cwd),
            );
            self.window.error = None;
            self.window.copy = Some(copy::State::Copying { done: 0, total: 0 });
            self.job = Some(CopyJob::start(root.clone(), runtime));
        }

        /// Follows the copy: done closes with Play, a failure says why.
        fn follow_copy(&mut self, ctx: &egui::Context) {
            let Some(job) = &self.job else {
                return;
            };
            let state = job.state();
            match &state {
                copy::State::Done { .. } => {
                    *self.outcome = Outcome::Play;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                copy::State::Failed(error) => {
                    self.window.error = Some(format!(
                        "The copy stopped: {error} Check the free space on that disk, then press Play to try again."
                    ));
                    self.window.copy = None;
                    self.job = None;
                    return;
                }
                copy::State::Copying { .. } => {}
            }
            self.window.copy = Some(state);
        }
    }

    impl eframe::App for App<'_> {
        fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
            let navs = self.pad.update(self.pads.buttons());
            pad::inject(ctx, raw, &navs);
            self.pending.extend(navs);
        }

        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            let ctx = ui.ctx().clone();
            self.fit.first_frame(&ctx);
            // Taken, so a second egui pass in one frame can't play twice.
            for nav in std::mem::take(&mut self.pending) {
                match nav {
                    Nav::Play => self.window.press_play(),
                    Nav::NextTab => self.window.tab = self.window.tab.step(1),
                    Nav::PreviousTab => self.window.tab = self.window.tab.step(-1),
                    _ => {}
                }
            }
            if ctx.input(|input| input.key_pressed(egui::Key::Enter))
                && !ctx.egui_wants_keyboard_input()
            {
                self.window.press_play();
            }
            self.follow_copy(&ctx);
            draw(ui, &mut self.window);
            if std::mem::take(&mut self.window.play) {
                self.save_and_copy();
            }
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{self, Settings};
    use crate::palette::{METRICS, PALETTE};
    use egui::Role;
    use rea_launcher::cfg::Resolution;
    use rea_launcher_ui::spec::rendered_keys;
    use rea_launcher_ui::testing::{self, Harness, Queryable};

    fn open(text: &str) -> Harness<'static, Window> {
        testing::harness(
            &PALETTE,
            &METRICS,
            Window::new(text, Some((1920, 1080))),
            draw,
        )
    }

    fn flags(args: &[&str]) -> Settings {
        let args = args.iter().map(|arg| (*arg).to_owned());
        config::parse_args(args).unwrap().0
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("dirtrally-ui-{name}-{}", std::process::id()))
            .join("Jogos d'Ação");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_window_has_a_control_for_every_option_and_no_other() {
        let mut ui = open("");
        ui.get_by_role_and_label(Role::Tab, "Picture").click();
        ui.run();
        spec::assert_parity(OPTIONS, &rendered_keys(&ui.ctx));
    }

    #[test]
    fn a_look_card_saves_what_its_flag_sets() {
        let mut ui = open("");
        ui.get_by_label("Steam Deck").click();
        ui.run();
        let saved = ui.state().form.settings();
        assert_eq!(saved.look, Some(Look::SteamDeck));
        let flag = flags(&["--look", "steam_deck"]);
        assert_eq!(saved.picture(), flag.picture());
        assert_eq!(
            saved.picture().resolution,
            Some(Resolution::Size(1280, 800))
        );
        assert!(testing::checked(&ui.get_by_label("Steam Deck")));
    }

    #[test]
    fn the_resolution_row_saves_what_its_flag_sets() {
        let mut ui = open("");
        ui.get_by_role_and_label(Role::Tab, "Picture").click();
        ui.run();
        *ui.state_mut().form.value_mut("resolution") = "2560x1440".into();
        ui.run();
        let saved = ui.state().form.settings();
        assert_eq!(
            saved.resolution,
            flags(&["--resolution", "2560x1440"]).resolution
        );
    }

    #[test]
    fn a_size_the_option_refuses_says_what_it_needs() {
        let mut ui = open("");
        ui.state_mut().form = Form::from_text("resolution = huge\n");
        ui.get_by_role_and_label(Role::Tab, "Picture").click();
        ui.run();
        assert!(ui.query_by_label_contains("Resolution: needs").is_some());
    }

    #[test]
    fn getting_ready_play_waits_and_an_unknown_build_is_refused() {
        let mut ui = open("");
        assert!(
            ui.query_by_label_contains("Choose your DiRT Rally folder")
                .is_some()
        );
        assert!(testing::disabled(
            &ui.get_by_role_and_label(Role::Button, "Play")
        ));

        let dir = scratch("unknown");
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::write(dir.join("bin/DirtRally"), b"not the game").unwrap();
        *ui.state_mut().form.value_mut("game") = dir.display().to_string();
        ui.run();
        let digest = rea_launcher::identify::hex(&rea_launcher::identify::sha256(b"not the game"));
        assert!(
            ui.query_by_label_contains(&digest).is_some(),
            "the SHA-256 is shown"
        );
        assert!(
            ui.query_by_label_contains(edition::UNKNOWN_EDITION_ISSUE)
                .is_some()
        );
        let play = ui.get_by_role_and_label(Role::Button, "Play");
        assert!(testing::disabled(&play));
        play.click();
        ui.run();
        assert!(!ui.state().play);
    }

    #[test]
    fn a_folder_without_the_game_says_where_to_look() {
        let mut ui = open("");
        *ui.state_mut().form.value_mut("game") = scratch("empty").display().to_string();
        ui.run();
        assert!(
            ui.query_by_label_contains("There is no DiRT Rally")
                .is_some()
        );
    }

    #[test]
    fn while_the_copy_runs_its_progress_replaces_the_tabs_and_play_is_off() {
        let mut ui = open("");
        ui.state_mut().copy = Some(copy::State::Copying {
            done: 12_400_000_000,
            total: 40_100_000_000,
        });
        ui.run();
        assert!(ui.query_by_label_contains("12.4 of 40.1 GB").is_some());
        assert!(
            ui.query_by_label_contains("Making your private copy")
                .is_some()
        );
        assert!(ui.query_by_role_and_label(Role::Tab, "Picture").is_none());
        assert!(testing::disabled(
            &ui.get_by_role_and_label(Role::Button, "Play")
        ));
        ui.state_mut().press_play();
        assert!(!ui.state().play);
    }

    #[test]
    fn a_failed_copy_says_why() {
        let mut ui = open("");
        ui.state_mut().error = Some("Cannot prepare /x: disk full".into());
        ui.run();
        assert!(ui.query_by_label_contains("disk full").is_some());
    }

    /// The owner's copy (env-gated): the window finds it and Play is on.
    #[test]
    #[ignore]
    fn the_owners_copy_makes_the_window_ready() {
        let Some(dir) = std::env::var_os("DIRTRALLY_GAME_DIR") else {
            return;
        };
        let mut ui = open(&format!("game = {}\n", PathBuf::from(dir).display()));
        ui.run();
        assert!(ui.query_by_label_contains("Found DiRT Rally").is_some());
        assert!(!testing::disabled(
            &ui.get_by_role_and_label(Role::Button, "Play")
        ));
        ui.get_by_role_and_label(Role::Button, "Play").click();
        ui.run();
        assert!(ui.state().play);
    }

    /// The window reads every pad through the SDK's joystick reader: a
    /// pad's Start (joystick button 7) is Play, once per press.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_pads_start_button_plays() {
        use rea_launcher_ui::pad::joystick::{self, Device, Joysticks};
        use rea_launcher_ui::pad::{Nav, PadNav};
        use std::time::Duration;

        struct Pressed(Vec<[u8; 8]>);
        impl std::io::Read for Pressed {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                match self.0.pop() {
                    Some(event) => {
                        buf[..8].copy_from_slice(&event);
                        Ok(8)
                    }
                    None => Err(std::io::ErrorKind::WouldBlock.into()),
                }
            }
        }
        let start = [0, 0, 0, 0, 1, 0, joystick::EVENT_BUTTON, 7];
        let mut pads = app::Pads {
            js: Joysticks::with_scan(Duration::from_secs(3600), move || {
                vec![Box::new(Pressed(vec![start])) as Device]
            }),
        };
        let mut nav = PadNav::default();
        assert_eq!(nav.update(pads.buttons()), [Nav::Play]);
        assert!(
            nav.update(pads.buttons()).is_empty(),
            "held, not pressed again"
        );
    }

    #[test]
    fn tabs_step_and_wrap() {
        assert_eq!(Tab::Play.step(1), Tab::Picture);
        assert_eq!(Tab::Play.step(-1), Tab::Picture);
        assert_eq!(Tab::Picture.step(1), Tab::Play);
    }
}
