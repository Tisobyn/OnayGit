#[allow(unused_imports)]
use crate::prelude::*;

pub mod history;
pub mod refs;
pub mod tabs;
mod repos;
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
    Appearance,
    Git,
    Editor,
    Remotes,
    RepoStyle,
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
    pub group_open: bool,
    pub clone_open: bool,
    pub clone_url: String,
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
    pub desc: String,
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
        egui_extras::install_image_loaders(&cc.egui_ctx);
        set_accent(settings.accent);
        apply_visuals(&cc.egui_ctx, settings.theme);
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
            group_open: true,
            clone_open: false,
            clone_url: String::new(),
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
            desc: String::new(),
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
        app
    }

    pub fn pal(&self) -> Pal {
        pal(self.settings.theme)
    }

    pub fn repo(&self) -> Option<PathBuf> {
        self.current.and_then(|i| self.repos.get(i).cloned())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let before = self.settings.clone();
        set_accent(self.settings.accent);
        apply_visuals(ctx, self.settings.theme);

        while let Ok((label, res)) = self.rx.try_recv() {
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

        let every = Duration::from_secs(self.settings.refresh_secs.max(1));
        if self.screen == Screen::Repo && self.last_refresh.elapsed() > every && self.busy.is_none() {
            self.refresh();
        }
        ctx.request_repaint_after(every);

        self.onboarding_window(ctx);
        self.confirm_window(ctx);
        self.manage_windows(ctx);

        self.tab_shortcuts(ctx);
        match self.screen {
            Screen::Home => {
                self.tab_bar(ctx);
                self.status_bar(ctx);
                self.home(ctx);
            }
            Screen::Settings => {
                self.status_bar(ctx);
                self.settings_screen(ctx);
            }
            Screen::Repo => {
                self.tab_bar(ctx);
                self.toolbar(ctx);
                self.status_bar(ctx);
                self.sidebar(ctx);
                self.main_panel(ctx);
            }
        }

        if self.settings != before {
            self.settings.save();
        }
    }
}
