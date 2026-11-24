#[allow(unused_imports)]
use crate::prelude::*;

pub mod accounts;
pub mod ai;
pub mod history;
pub mod hosting;
pub mod machines;
pub mod refs;
pub mod tabs;
pub mod tasks;
mod repos;
pub mod spaces;
pub mod editors;
mod staging;

#[derive(PartialEq, Clone, Copy)]
pub enum Screen {
    Home,
    Repo,
    Settings,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Changes,
    History,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Section {
    Application,
    Spaces,
    Appearance,
    Keybinds,
    Privacy,
    Git,
    Editor,
    Accounts,
    Ai,
    Remotes,
    RepoStyle,
    Identity,
    Ignore,
    Help,
}

#[derive(Clone)]
pub enum Confirm {
    Discard(String),
    DiscardAll,
    DeleteBranch(String),
    ResetHard(String),
    Rewrite(git::RewriteOp, String),
    AbortOp,
}

pub enum Action {
    Stage(String),
    Unstage(String),
    StageAll,
    UnstageAll,
    Select(String),
}

#[derive(PartialEq, Clone, Copy)]
pub enum Dialog {
    Tags,
    Worktrees,
    Reword,
    Publish,
    PullRequests,
    Switcher,
    AddMachine,
}

/// Short status of a repository, shown on the home screen.
#[derive(Clone, Debug)]
pub struct RepoInfo {
    pub branch: String,
    pub dirty: usize,
    /// (ahead, behind); None when there is no upstream.
    pub sync: Option<(u32, u32)>,
}

pub type Done = (String, Result<String, String>);

pub struct App {
    pub settings: Settings,
    pub screen: Screen,
    pub repos: Vec<PathBuf>,
    pub current: Option<usize>,

    // home
    pub search: String,
    pub grid: bool,
    pub clone_open: bool,
    pub clone_url: String,
    pub clone_dir: Option<PathBuf>,
    pub create_open: bool,
    pub create_dir: Option<PathBuf>,
    pub create_branch: String,
    pub create_anchor: egui::Pos2,
    pub pending_clone: Option<PathBuf>,

    // settings
    pub section: Section,
    pub git_name: String,
    pub git_email: String,

    // repo
    pub tab: Tab,
    pub filter: String,
    pub files: Vec<FileChange>,
    pub selected_file: Option<String>,
    pub diff: Vec<DiffLine>,
    pub summary: String,
    pub filter_open: bool,
    /// Author, date and message body of the selected commit.
    pub commit_info: Option<(String, String, String)>,
    pub branch: String,
    pub branches: Vec<String>,
    pub new_branch: String,
    pub ahead_behind: Option<(u32, u32)>,
    pub has_remote: bool,
    pub last_fetched: Option<Instant>,
    pub commits: Vec<Commit>,
    pub selected_commit: Option<String>,
    pub commit_diff: Vec<DiffLine>,
    pub raw_diff: String,
    pub diff_staged: bool,
    pub sel_lines: HashSet<usize>,
    pub compare: Option<String>,
    pub compare_commits: Vec<Commit>,
    pub compare_counts: (u32, u32),
    pub compare_outgoing: bool,
    pub rebase_onto: Option<String>,
    pub upstream: Option<String>,
    pub ignore_text: String,
    pub stashes: Vec<git::Stash>,
    pub stash_open: bool,
    pub selected_stash: Option<String>,
    pub stash_diff: Vec<DiffLine>,
    pub tags: Vec<git::Tag>,
    pub worktrees: Vec<git::Worktree>,
    pub dialog: Option<Dialog>,
    pub new_tag: String,
    pub new_tag_msg: String,
    pub tag_target: String,
    pub wt_name: String,
    pub wt_new_branch: bool,
    pub commit_files: Vec<(char, String)>,
    pub selected_commit_file: Option<String>,
    pub graph: Vec<git::GraphRow>,
    pub graph_width: usize,
    pub repo_op: Option<git::RepoOp>,
    pub reword_hash: String,
    pub reword_text: String,
    pub remotes: Vec<git::Remote>,
    pub remote_name: String,
    pub new_remote_name: String,
    pub new_remote_url: String,
    pub remote_edit: Option<(String, String)>,
    pub remote_rename: Option<(String, String)>,
    pub tabs: Vec<PathBuf>,
    pub repo_styles: HashMap<PathBuf, RepoStyle>,
    pub tip: usize,
    pub tasks: HashSet<String>,
    pub tree_view: bool,
    pub system_dark: bool,
    pub acct_adding: bool,
    /// 0 browser, 1 access token, 2 GitHub CLI.
    pub acct_method: usize,
    pub acct_client_id: String,
    pub acct_private: bool,
    pub acct_advanced: bool,
    pub device: Option<git::DeviceCode>,
    pub device_waiting: bool,
    pub pending_cli: bool,
    pub cli_checked: bool,
    pub acct_status: HashMap<String, String>,
    pub acct_cli: bool,
    pub launcher_msg: String,
    pub keybind_search: String,
    pub editor_rules: Vec<editors::EditorRule>,
    pub rule_dialog: Option<Option<usize>>,
    pub rule_patterns: String,
    pub rule_editor: String,
    pub git_src: Option<usize>,
    pub fonts_applied: (String, String),
    pub grain_tex: Option<egui::TextureHandle>,
    /// Open rail menu: kind (0 settings, 1 spaces, 2 account), anchor and the frame it opened in.
    pub rail_menu: Option<(u8, egui::Pos2, f64)>,
    pub keybinds: HashMap<String, String>,
    pub recording: Option<&'static str>,
    pub keybind_msg: String,
    pub repo_name_cfg: String,
    pub repo_email_cfg: String,
    pub repo_info: HashMap<PathBuf, RepoInfo>,
    pub info_at: Option<Instant>,
    pub card_rect: egui::Rect,
    pub spaces: Vec<spaces::Space>,
    pub selected_space: Option<usize>,
    pub space_name: String,
    pub space_icon: String,
    pub space_color: Option<usize>,
    /// Space dialog: Some(None) creates, Some(Some(i)) edits space i.
    pub space_dialog: Option<Option<usize>>,
    pub git_editing: bool,
    pub ai_adding: bool,
    pub switcher_text: String,
    pub switcher_sel: usize,
    pub img_before: Option<PathBuf>,
    pub collapsed: HashSet<String>,
    pub accounts: Vec<git::Account>,
    pub acct_provider: git::Provider,
    pub acct_host: String,
    pub acct_token: String,
    pub pending_account: Option<(git::Provider, String, String)>,
    pub publish_account: usize,
    pub publish_name: String,
    pub publish_private: bool,
    pub publish_desc: String,
    pub prs: Vec<git::PullRequest>,
    pub pr_error: String,
    pub pr_title: String,
    pub pr_body: String,
    pub pr_base: String,
    pub ai_key_set: bool,
    pub ai_key_input: String,
    pub machines: Vec<Machine>,
    pub selected_machine: Option<usize>,
    pub machine_repos: HashMap<String, Vec<String>>,
    pub machine_name: String,
    pub machine_host: String,
    pub machine_path: String,
    pub pending_machine: Option<Machine>,
    pub amend: bool,
    pub confirm: Option<Confirm>,
    pub last_commit: Option<(String, String)>,
    pub remote_branches: Vec<String>,
    pub branch_filter: String,

    pub status_msg: String,
    pub busy: Option<String>,
    pub tx: Sender<Done>,
    pub rx: Receiver<Done>,
    pub last_refresh: Instant,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let fonts_applied = (settings.ui_font.clone(), settings.code_font.clone());
        egui_extras::install_image_loaders(&cc.egui_ctx);
        set_accent(settings.accent);
        apply_visuals(&cc.egui_ctx, settings.theme);
        apply_fonts(&cc.egui_ctx, &settings.ui_font, &settings.code_font);
        cc.egui_ctx.set_zoom_factor(settings.zoom);

        let (tx, rx) = channel();
        let repos = load_repos();
        let mut app = App {
            settings,
            screen: if repos.is_empty() { Screen::Home } else { Screen::Repo },
            current: if repos.is_empty() { None } else { Some(0) },
            repos,
            search: String::new(),
            grid: false,
            clone_open: false,
            clone_url: String::new(),
            clone_dir: None,
            create_open: false,
            create_dir: None,
            create_branch: "main".into(),
            create_anchor: egui::Pos2::ZERO,
            pending_clone: None,
            section: Section::Application,
            git_name: String::new(),
            git_email: String::new(),
            tab: Tab::Changes,
            filter: String::new(),
            files: vec![],
            selected_file: None,
            diff: vec![],
            summary: String::new(),
            filter_open: false,
            commit_info: None,
            branch: String::new(),
            branches: vec![],
            new_branch: String::new(),
            ahead_behind: None,
            has_remote: false,
            last_fetched: None,
            commits: vec![],
            selected_commit: None,
            commit_diff: vec![],
            raw_diff: String::new(),
            diff_staged: false,
            sel_lines: HashSet::new(),
            compare: None,
            compare_commits: vec![],
            compare_counts: (0, 0),
            compare_outgoing: false,
            rebase_onto: None,
            upstream: None,
            ignore_text: String::new(),
            stashes: vec![],
            stash_open: true,
            selected_stash: None,
            stash_diff: vec![],
            tags: vec![],
            worktrees: vec![],
            dialog: None,
            new_tag: String::new(),
            new_tag_msg: String::new(),
            tag_target: String::new(),
            wt_name: String::new(),
            wt_new_branch: true,
            commit_files: vec![],
            selected_commit_file: None,
            graph: vec![],
            graph_width: 1,
            repo_op: None,
            reword_hash: String::new(),
            reword_text: String::new(),
            remotes: vec![],
            remote_name: "origin".to_string(),
            new_remote_name: String::new(),
            new_remote_url: String::new(),
            remote_edit: None,
            remote_rename: None,
            tabs: vec![],
            repo_styles: tabs::load_styles(),
            tip: 0,
            tasks: HashSet::new(),
            system_dark: true,
            tree_view: false,
            acct_adding: false,
            acct_cli: false,
            launcher_msg: String::new(),
            keybind_search: String::new(),
            editor_rules: editors::load_rules(),
            rule_dialog: None,
            rule_patterns: String::new(),
            rule_editor: "open".into(),
            git_src: None,
            fonts_applied,
            grain_tex: None,
            rail_menu: None,
            keybinds: crate::keybinds::load(),
            recording: None,
            keybind_msg: String::new(),
            repo_name_cfg: String::new(),
            repo_email_cfg: String::new(),
            repo_info: HashMap::new(),
            info_at: None,
            card_rect: egui::Rect::NOTHING,
            spaces: spaces::load_spaces(),
            selected_space: None,
            space_name: String::new(),
            space_icon: "layers".into(),
            space_color: None,
            space_dialog: None,
            git_editing: false,
            ai_adding: false,
            switcher_text: String::new(),
            switcher_sel: 0,
            img_before: None,
            collapsed: HashSet::new(),
            accounts: accounts::load_accounts(),
            acct_provider: git::Provider::GitHub,
            acct_host: String::new(),
            acct_token: String::new(),
            pending_account: None,
            acct_method: 0,
            acct_client_id: String::new(),
            acct_private: false,
            acct_advanced: false,
            device: None,
            device_waiting: false,
            pending_cli: false,
            cli_checked: false,
            acct_status: HashMap::new(),
            publish_account: 0,
            publish_name: String::new(),
            publish_private: true,
            publish_desc: String::new(),
            prs: vec![],
            pr_error: String::new(),
            pr_title: String::new(),
            pr_body: String::new(),
            pr_base: String::new(),
            ai_key_set: false,
            ai_key_input: String::new(),
            machines: machines::load_machines(),
            selected_machine: None,
            machine_repos: HashMap::new(),
            machine_name: String::new(),
            machine_host: String::new(),
            machine_path: String::new(),
            pending_machine: None,
            amend: false,
            confirm: None,
            last_commit: None,
            remote_branches: vec![],
            branch_filter: String::new(),
            status_msg: String::new(),
            busy: None,
            tx,
            rx,
            last_refresh: Instant::now(),
        };
        app.tabs = tabs::load_tabs().into_iter().filter(|t| app.repos.contains(t)).collect();
        if app.tabs.is_empty() {
            if let Some(first) = app.repos.first() {
                app.tabs.push(first.clone());
            }
        }
        app.refresh();
        // `onaygit <path>` opens that folder as a repository.
        if let Some(arg) = std::env::args().nth(1) {
            if let Ok(path) = std::fs::canonicalize(&arg) {
                if path.is_dir() && git::is_repo(&path) {
                    app.open_repo(path);
                }
            }
        }
        // Handy for screenshots: ONAYGIT_SCREEN=home|settings
        match std::env::var("ONAYGIT_SCREEN").as_deref() {
            Ok("home") => app.screen = Screen::Home,
            Ok(s) if s.starts_with("settings") => {
                app.open_settings();
                app.section = match s.strip_prefix("settings:").unwrap_or("") {
                    "spaces" => Section::Spaces,
                    "spacenew" => {
                        app.space_dialog = Some(None);
                        Section::Spaces
                    }
                    "appearance" => Section::Appearance,
                    "keybinds" => Section::Keybinds,
                    "privacy" => Section::Privacy,
                    "accounts" => Section::Accounts,
                    "accountadd" => {
                        app.acct_adding = true;
                        Section::Accounts
                    }
                    "git" => Section::Git,
                    "editor" => Section::Editor,
                    "ai" => Section::Ai,
                    "help" => Section::Help,
                    "remotes" => Section::Remotes,
                    "repostyle" => Section::RepoStyle,
                    "identity" => Section::Identity,
                    "ignore" => Section::Ignore,
                    _ => app.section,
                };
            }
            Ok("repo") => {
                if let Some(r) = app.repos.first().cloned() {
                    app.open_repo(r);
                }
            }
            Ok(m) if m.starts_with("dialog:") => {
                if let Some(r) = app.repos.first().cloned() {
                    app.open_repo(r);
                    app.refresh();
                }
                match &m["dialog:".len()..] {
                    "tags" => app.dialog = Some(Dialog::Tags),
                    "worktrees" => app.dialog = Some(Dialog::Worktrees),
                    "publish" => app.dialog = Some(Dialog::Publish),
                    "reword" => {
                        app.reword_text = "Update readme".into();
                        app.dialog = Some(Dialog::Reword);
                    }
                    "welcome" => app.settings.onboarded = false,
                    "switcher" => app.dialog = Some(Dialog::Switcher),
                    _ => {}
                }
            }
            Ok("machine") => app.dialog = Some(Dialog::AddMachine),
            Ok("clone") => {
                app.screen = Screen::Home;
                app.clone_open = true;
            }
            Ok("create") => {
                app.screen = Screen::Home;
                app.create_open = true;
                app.create_anchor = egui::pos2(200.0, 380.0);
            }
            Ok("history") => {
                if let Some(r) = app.repos.first().cloned() {
                    app.open_repo(r);
                    app.tab = Tab::History;
                    app.refresh();
                    if let Some(h) = app.commits.first().map(|c| c.hash.clone()) {
                        app.select_commit(h);
                    }
                }
            }
            _ => {}
        }
        app
    }

    /// The theme in use now. With Auto it follows the system and keeps the chosen family.
    pub fn theme_now(&self) -> Theme {
        let t = self.settings.theme;
        if !self.settings.auto_theme || t.is_dark() == self.system_dark {
            t
        } else if self.system_dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn pal(&self) -> Pal {
        pal(self.theme_now())
    }

    pub fn repo(&self) -> Option<PathBuf> {
        self.current.and_then(|i| self.repos.get(i).cloned())
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let c = self.pal().chrome;
        c.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let before = self.settings.clone();
        if let Some(t) = ctx.input(|i| i.raw.system_theme) {
            self.system_dark = t == egui::Theme::Dark;
        }
        set_accent(self.settings.accent);
        apply_visuals(ctx, self.theme_now());
        if self.fonts_applied != (self.settings.ui_font.clone(), self.settings.code_font.clone()) {
            apply_fonts(ctx, &self.settings.ui_font, &self.settings.code_font);
            self.fonts_applied = (self.settings.ui_font.clone(), self.settings.code_font.clone());
        }
        ctx.style_mut(|st| {
            st.animation_time = if self.settings.smooth { 1.0 / 12.0 } else { 0.0 };
            if !self.settings.scrollbars {
                st.spacing.scroll.bar_width = 0.0;
                st.spacing.scroll.floating_width = 0.0;
                st.spacing.scroll.bar_inner_margin = 0.0;
                st.spacing.scroll.bar_outer_margin = 0.0;
            } else {
                st.spacing.scroll = egui::style::ScrollStyle::floating();
            }
        });

        if !self.cli_checked {
            self.cli_checked = true;
            self.auto_add_cli(ctx);
        }
        while let Ok((label, res)) = self.rx.try_recv() {
            if self.handle_task_result(&label, &res) {
                continue;
            }
            self.busy = None;
            match res {
                Ok(_) => {
                    if label == "Fetching" || label == "Pulling" || label == "Pushing" || label == "Publishing" {
                        self.last_fetched = Some(Instant::now());
                    }
                    if label == "Cloning" {
                        if let Some(p) = self.pending_clone.take() {
                            self.open_repo(p);
                        }
                    }
                }
                Err(e) => {
                    self.pending_clone = None;
                    self.status_msg = format!("{label} failed: {e}");
                }
            }
            self.refresh();
        }

        let mut every = Duration::from_secs(self.settings.refresh_secs.max(1));
        if self.repo().is_some_and(|r| git::is_remote(&r)) {
            every = every.max(Duration::from_secs(20));
        }
        if self.screen == Screen::Repo && self.last_refresh.elapsed() > every && self.busy.is_none() {
            self.refresh();
        }
        ctx.request_repaint_after(every);

        self.onboarding_window(ctx);
        self.confirm_window(ctx);
        if let (Some(other), Some(repo)) = (self.rebase_onto.take(), self.repo()) {
            self.spawn_in(ctx, repo, "Rebasing", &["rebase", &other]);
        }
        self.manage_windows(ctx);

        self.tab_shortcuts(ctx);
        match self.screen {
            Screen::Home => {
                self.shell(ctx);
                self.fill_card(ctx);
                self.status_bar(ctx);
                self.home(ctx);
            }
            Screen::Settings => {
                self.shell(ctx);
                self.fill_card(ctx);
                self.status_bar(ctx);
                self.settings_screen(ctx);
            }
            Screen::Repo => {
                self.shell(ctx);
                self.fill_card(ctx);
                self.toolbar(ctx);
                self.status_bar(ctx);
                self.sidebar(ctx);
                self.main_panel(ctx);
            }
        }

        self.paint_corners(ctx);
        self.paint_grain(ctx);

        if self.settings != before {
            self.settings.save();
        }
    }
}

/// Fonts offered in Settings, with the file each one is loaded from.
pub const UI_FONTS: [(&str, &str); 6] = [
    ("System font", ""),
    ("Helvetica Neue", "/System/Library/Fonts/HelveticaNeue.ttc"),
    ("Helvetica", "/System/Library/Fonts/Helvetica.ttc"),
    ("Arial", "/System/Library/Fonts/Supplemental/Arial.ttf"),
    ("Verdana", "/System/Library/Fonts/Supplemental/Verdana.ttf"),
    ("Georgia", "/System/Library/Fonts/Supplemental/Georgia.ttf"),
];

pub const CODE_FONTS: [(&str, &str); 4] = [
    ("Menlo", "/System/Library/Fonts/Menlo.ttc"),
    ("Monaco", "/System/Library/Fonts/Monaco.ttf"),
    ("Courier New", "/System/Library/Fonts/Supplemental/Courier New.ttf"),
    ("Andale Mono", "/System/Library/Fonts/Supplemental/Andale Mono.ttf"),
];

/// Install the chosen interface and code fonts, with macOS symbols as a fallback for key glyphs.
pub fn apply_fonts(ctx: &egui::Context, ui_font: &str, code_font: &str) {
    let mut fonts = egui::FontDefinitions::default();
    let mut add = |name: &str, path: &str, fams: &[(egui::FontFamily, bool)]| {
        let Ok(bytes) = std::fs::read(path) else { return };
        fonts.font_data.insert(name.to_string(), egui::FontData::from_owned(bytes));
        for (fam, first) in fams {
            let list = fonts.families.entry(fam.clone()).or_default();
            if *first {
                list.insert(0, name.to_string());
            } else {
                list.push(name.to_string());
            }
        }
    };
    if let Some((_, path)) = UI_FONTS.iter().find(|(n, p)| *n == ui_font && !p.is_empty()) {
        add("ui_font", path, &[(egui::FontFamily::Proportional, true)]);
    }
    if let Some((_, path)) = CODE_FONTS.iter().find(|(n, _)| *n == code_font) {
        add("code_font", path, &[(egui::FontFamily::Monospace, true)]);
    }
    add("symbols", "/System/Library/Fonts/Apple Symbols.ttf", &[(egui::FontFamily::Proportional, false), (egui::FontFamily::Monospace, false)]);
    ctx.set_fonts(fonts);
}
