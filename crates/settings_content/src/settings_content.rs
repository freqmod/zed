mod action;
mod agent;
mod editor;
mod extension;
mod fallible_options;
mod language;
mod language_model;
pub mod merge_from;
mod project;
mod serde_helper;
mod terminal;
mod theme;
mod title_bar;
mod workspace;

pub use action::{ActionName, ActionWithArguments};
pub use agent::*;
pub use editor::*;
pub use extension::*;
pub use fallible_options::*;
pub use language::*;
pub use language_model::*;
pub use merge_from::MergeFrom as MergeFromTrait;
pub use project::*;
use serde::de::DeserializeOwned;
pub use serde_helper::{
    serialize_f32_with_two_decimal_places, serialize_optional_f32_with_two_decimal_places,
};
use settings_json::parse_json_with_comments;
pub use terminal::*;
pub use theme::*;
pub use title_bar::*;
pub use workspace::*;

use collections::{HashMap, IndexMap};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

/// Defines a settings override struct where each field is
/// `Option<Box<SettingsContent>>`, along with:
/// - `OVERRIDE_KEYS`: a `&[&str]` of the field names (the JSON keys)
/// - `get_by_key(&self, key) -> Option<&SettingsContent>`: accessor by key
///
/// The field list is the single source of truth for the override key strings.
macro_rules! settings_overrides {
    (
        $(#[$attr:meta])*
        pub struct $name:ident { $($field:ident),* $(,)? }
    ) => {
        $(#[$attr])*
        pub struct $name {
            $(pub $field: Option<Box<SettingsContent>>,)*
        }

        impl $name {
            /// The JSON override keys, derived from the field names on this struct.
            pub const OVERRIDE_KEYS: &[&str] = &[$(stringify!($field)),*];

            /// Look up an override by its JSON key name.
            pub fn get_by_key(&self, key: &str) -> Option<&SettingsContent> {
                match key {
                    $(stringify!($field) => self.$field.as_deref(),)*
                    _ => None,
                }
            }
        }
    }
}
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
pub use util::serde::default_true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseStatus {
    /// Settings were parsed successfully
    Success,
    /// Settings file was not changed, so no parsing was performed
    Unchanged,
    /// Settings failed to parse
    Failed { error: String },
}

#[with_fallible_options]
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct SettingsContent {
    #[serde(flatten)]
    pub project: ProjectSettingsContent,

    #[serde(flatten)]
    pub theme: Box<ThemeSettingsContent>,

    #[serde(flatten)]
    pub extension: ExtensionSettingsContent,

    #[serde(flatten)]
    pub workspace: WorkspaceSettingsContent,

    #[serde(flatten)]
    pub editor: EditorSettingsContent,

    #[serde(flatten)]
    pub remote: RemoteSettingsContent,

    /// Settings related to the file finder.
    pub file_finder: Option<FileFinderSettingsContent>,

    pub git_panel: Option<GitPanelSettingsContent>,

    pub tabs: Option<ItemSettingsContent>,
    pub tab_bar: Option<TabBarSettingsContent>,
    pub status_bar: Option<StatusBarSettingsContent>,

    pub preview_tabs: Option<PreviewTabsSettingsContent>,

    pub agent: Option<AgentSettingsContent>,
    pub agent_servers: Option<AllAgentServersSettings>,

    /// Configuration of audio in Zed.
    pub audio: Option<AudioSettingsContent>,

    /// Whether or not to automatically check for updates.
    ///
    /// Default: true
    pub auto_update: Option<bool>,

    /// This base keymap settings adjusts the default keybindings in Zed to be similar
    /// to other common code editors. By default, Zed's keymap closely follows VSCode's
    /// keymap, with minor adjustments, this corresponds to the "VSCode" setting.
    ///
    /// Default: VSCode
    pub base_keymap: Option<BaseKeymapContent>,

    /// Configuration for the collab panel visual settings.
    pub collaboration_panel: Option<PanelSettingsContent>,

    pub debugger: Option<DebuggerSettingsContent>,

    /// Configuration for Diagnostics-related features.
    pub diagnostics: Option<DiagnosticsSettingsContent>,

    /// Configuration for Git-related features
    pub git: Option<GitSettings>,

    /// Common language server settings.
    pub global_lsp_settings: Option<GlobalLspSettingsContent>,

    /// The settings for the image viewer.
    pub image_viewer: Option<ImageViewerSettingsContent>,

    pub repl: Option<ReplSettingsContent>,

    /// Whether or not to enable Helix mode.
    ///
    /// Default: false
    pub helix_mode: Option<bool>,

    pub journal: Option<JournalSettingsContent>,

    /// A map of log scopes to the desired log level.
    /// Useful for filtering out noisy logs or enabling more verbose logging.
    ///
    /// Example: {"log": {"client": "warn"}}
    pub log: Option<HashMap<String, String>>,

    pub line_indicator_format: Option<LineIndicatorFormat>,

    pub language_models: Option<AllLanguageModelSettingsContent>,

    pub outline_panel: Option<OutlinePanelSettingsContent>,

    pub project_panel: Option<ProjectPanelSettingsContent>,

    /// Configuration for the Message Editor
    pub message_editor: Option<MessageEditorSettings>,

    /// Configuration for Node-related features
    pub node: Option<NodeBinarySettings>,

    pub proxy: Option<String>,

    /// The URL of the Zed server to connect to.
    pub server_url: Option<String>,

    /// The URL used as the key for credential storage.
    ///
    /// When set, credentials are stored under this URL instead of `server_url`.
    /// This allows running multiple Zed instances side by side without them
    /// overwriting each other's keychain entries.
    pub credentials_url: Option<String>,

    /// Configuration for session-related features
    pub session: Option<SessionSettingsContent>,
    /// Control what info is collected by Zed.
    pub telemetry: Option<TelemetrySettingsContent>,

    /// Configuration of the terminal in Zed.
    pub terminal: Option<TerminalSettingsContent>,

    pub title_bar: Option<TitleBarSettingsContent>,

    /// Whether or not to enable Vim mode.
    ///
    /// Default: false
    pub vim_mode: Option<bool>,

    // Settings related to calls in Zed
    pub calls: Option<CallSettingsContent>,

    /// Settings for the which-key popup.
    pub which_key: Option<WhichKeySettingsContent>,

    /// Settings related to Vim mode in Zed.
    pub vim: Option<VimSettingsContent>,

    /// Number of lines to search for modelines at the beginning and end of files.
    /// Modelines contain editor directives (e.g., vim/emacs settings) that configure
    /// the editor behavior for specific files.
    ///
    /// Default: 5
    pub modeline_lines: Option<usize>,

    /// Local overrides for feature flags, keyed by flag name.
    pub feature_flags: Option<FeatureFlagsMap>,

    /// Settings for developer-oriented instrumentation tools (profilers,
    /// tracers, etc.) that can be toggled at runtime.
    pub instrumentation: Option<InstrumentationSettingsContent>,
    
    pub jump_labels: Option<JumpLabelSettingsContent>,
}

/// Configuration for developer-oriented instrumentation tools that collect
/// diagnostic data about a running Zed instance.
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct InstrumentationSettingsContent {
    /// Configuration for the performance profiler, accessed via the
    /// `zed: open performance profiler` action.
    pub performance_profiler: Option<PerformanceProfilerSettingsContent>,
}

/// Configuration for the performance profiler which collects timing data
/// for foreground and background executor tasks.
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct PerformanceProfilerSettingsContent {
    /// Whether to collect timing data for foreground and background executor
    /// tasks. Enabling this may lead to increased memory usage, hence it's
    /// disabled by default for regular builds.
    ///
    /// Default: false
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, MergeFrom)]
#[serde(transparent)]
pub struct FeatureFlagsMap(pub HashMap<String, String>);

// A manual `JsonSchema` impl keeps this type's schema registered under a
// unique name. The derived impl on a `#[serde(transparent)]` newtype around
// `HashMap<String, String>` would inline to the map's own schema name (`Map_of_string`),
// which is shared with every other `HashMap<String, String>` setting field in
// `SettingsContent`. A named placeholder lets `json_schema_store` find and
// replace just this field's schema at runtime without clobbering the others.
impl JsonSchema for FeatureFlagsMap {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FeatureFlagsMap".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "additionalProperties": { "type": "string" }
        })
    }
}

impl std::ops::Deref for FeatureFlagsMap {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FeatureFlagsMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SettingsContent {
    pub fn languages_mut(&mut self) -> &mut HashMap<String, LanguageSettingsContent> {
        &mut self.project.all_languages.languages.0
    }
}

// These impls are there to optimize builds by avoiding monomorphization downstream. Yes, they're repetitive, but using default impls
// break the optimization, for whatever reason.
pub trait RootUserSettings: Sized + DeserializeOwned {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus);
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self>;
}

impl RootUserSettings for SettingsContent {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus) {
        fallible_options::parse_json(json)
    }
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self> {
        parse_json_with_comments(json)
    }
}
// Explicit opt-in instead of blanket impl to avoid monomorphizing downstream. Just a hunch though.
impl RootUserSettings for Option<SettingsContent> {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus) {
        fallible_options::parse_json(json)
    }
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self> {
        parse_json_with_comments(json)
    }
}
impl RootUserSettings for UserSettingsContent {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus) {
        fallible_options::parse_json(json)
    }
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self> {
        parse_json_with_comments(json)
    }
}

settings_overrides! {
    #[with_fallible_options]
    #[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
    pub struct ReleaseChannelOverrides { dev, nightly, preview, stable }
}

settings_overrides! {
    #[with_fallible_options]
    #[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
    pub struct PlatformOverrides { macos, linux, windows }
}

/// Determines what settings a profile starts from before applying its overrides.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom,
)]
#[serde(rename_all = "snake_case")]
pub enum ProfileBase {
    /// Apply profile settings on top of the user's current settings.
    #[default]
    User,
    /// Apply profile settings on top of Zed's default settings, ignoring user customizations.
    Default,
}

/// A named settings profile that can temporarily override settings.
#[with_fallible_options]
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct SettingsProfile {
    /// What base settings to start from before applying this profile's overrides.
    ///
    /// - `user`: Apply on top of user's settings (default)
    /// - `default`: Apply on top of Zed's default settings, ignoring user customizations
    #[serde(default)]
    pub base: ProfileBase,

    /// The settings overrides for this profile.
    #[serde(default)]
    pub settings: Box<SettingsContent>,
}

#[with_fallible_options]
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct UserSettingsContent {
    #[serde(flatten)]
    pub content: Box<SettingsContent>,

    #[serde(flatten)]
    pub release_channel_overrides: ReleaseChannelOverrides,

    #[serde(flatten)]
    pub platform_overrides: PlatformOverrides,

    #[serde(default)]
    pub profiles: IndexMap<String, SettingsProfile>,
}

pub struct ExtensionsSettingsContent {
    pub all_languages: AllLanguageSettingsContent,
}

/// Base key bindings scheme. Base keymaps can be overridden with user keymaps.
///
/// Default: VSCode
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    Default,
    strum::VariantArray,
)]
pub enum BaseKeymapContent {
    #[default]
    VSCode,
    JetBrains,
    SublimeText,
    Atom,
    TextMate,
    Emacs,
    Cursor,
    None,
}

impl strum::VariantNames for BaseKeymapContent {
    const VARIANTS: &'static [&'static str] = &[
        "VSCode",
        "JetBrains",
        "Sublime Text",
        "Atom",
        "TextMate",
        "Emacs",
        "Cursor",
        "None",
    ];
}

/// Configuration of audio in Zed.
#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct AudioSettingsContent {
    /// Automatically increase or decrease you microphone's volume. This affects how
    /// loud you sound to others.
    ///
    /// Recommended: off (default)
    /// Microphones are too quite in zed, until everyone is on experimental
    /// audio and has auto speaker volume on this will make you very loud
    /// compared to other speakers.
    #[serde(rename = "experimental.auto_microphone_volume")]
    pub auto_microphone_volume: Option<bool>,
    /// Remove background noises. Works great for typing, cars, dogs, AC. Does
    /// not work well on music.
    /// Select specific output audio device.
    #[serde(rename = "experimental.output_audio_device")]
    pub output_audio_device: Option<AudioOutputDeviceName>,
    /// Select specific input audio device.
    #[serde(rename = "experimental.input_audio_device")]
    pub input_audio_device: Option<AudioInputDeviceName>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct AudioOutputDeviceName(pub Option<String>);

impl AsRef<Option<String>> for AudioInputDeviceName {
    fn as_ref(&self) -> &Option<String> {
        &self.0
    }
}

impl From<Option<String>> for AudioInputDeviceName {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct AudioInputDeviceName(pub Option<String>);

impl AsRef<Option<String>> for AudioOutputDeviceName {
    fn as_ref(&self) -> &Option<String> {
        &self.0
    }
}

impl From<Option<String>> for AudioOutputDeviceName {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

/// Control what info is collected by Zed.
#[with_fallible_options]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Debug, MergeFrom)]
pub struct TelemetrySettingsContent {
    /// Send debug info like crash reports.
    ///
    /// Default: true
    pub diagnostics: Option<bool>,
    /// Send anonymized usage data like what languages you're using Zed with.
    ///
    /// Default: true
    pub metrics: Option<bool>,
}

impl Default for TelemetrySettingsContent {
    fn default() -> Self {
        Self {
            diagnostics: Some(true),
            metrics: Some(true),
        }
    }
}

#[with_fallible_options]
#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Clone, MergeFrom)]
pub struct DebuggerSettingsContent {
    /// Determines the stepping granularity.
    ///
    /// Default: line
    pub stepping_granularity: Option<SteppingGranularity>,
    /// Whether the breakpoints should be reused across Zed sessions.
    ///
    /// Default: true
    pub save_breakpoints: Option<bool>,
    /// Whether to show the debug button in the status bar.
    ///
    /// Default: true
    pub button: Option<bool>,
    /// Time in milliseconds until timeout error when connecting to a TCP debug adapter
    ///
    /// Default: 2000ms
    pub timeout: Option<u64>,
    /// Whether to log messages between active debug adapters and Zed
    ///
    /// Default: true
    pub log_dap_communications: Option<bool>,
    /// Whether to format dap messages in when adding them to debug adapter logger
    ///
    /// Default: true
    pub format_dap_log_messages: Option<bool>,
    /// The dock position of the debug panel
    ///
    /// Default: Bottom
    pub dock: Option<DockPosition>,
}

/// The granularity of one 'step' in the stepping requests `next`, `stepIn`, `stepOut`, and `stepBack`.
#[derive(
    PartialEq,
    Eq,
    Debug,
    Hash,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum SteppingGranularity {
    /// The step should allow the program to run until the current statement has finished executing.
    /// The meaning of a statement is determined by the adapter and it may be considered equivalent to a line.
    /// For example 'for(int i = 0; i < 10; i++)' could be considered to have 3 statements 'int i = 0', 'i < 10', and 'i++'.
    Statement,
    /// The step should allow the program to run until the current source line has executed.
    Line,
    /// The step should allow one instruction to execute (e.g. one x86 instruction).
    Instruction,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DockPosition {
    Left,
    Bottom,
    Right,
}

/// Configuration of voice calls in Zed.
#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct CallSettingsContent {
    /// Whether the microphone should be muted when joining a channel or a call.
    ///
    /// Default: false
    pub mute_on_join: Option<bool>,

    /// Whether your current project should be shared when joining an empty channel.
    ///
    /// Default: false
    pub share_on_join: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct GitPanelSettingsContent {
    /// Whether to show the panel button in the status bar.
    ///
    /// Default: true
    pub button: Option<bool>,
    /// Where to dock the panel.
    ///
    /// Default: left
    pub dock: Option<DockPosition>,
    /// Default width of the panel in pixels.
    ///
    /// Default: 360
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub default_width: Option<f32>,
    /// How entry statuses are displayed.
    ///
    /// Default: icon
    pub status_style: Option<StatusStyle>,

    /// Whether to show file icons in the git panel.
    ///
    /// Default: false
    pub file_icons: Option<bool>,

    /// Whether to show folder icons or chevrons for directories in the git panel.
    ///
    /// Default: true
    pub folder_icons: Option<bool>,

    /// How and when the scrollbar should be displayed.
    ///
    /// Default: inherits editor scrollbar settings
    pub scrollbar: Option<ScrollbarSettings>,

    /// What the default branch name should be when
    /// `init.defaultBranch` is not set in git
    ///
    /// Default: main
    pub fallback_branch_name: Option<String>,

    /// Whether to sort entries in the panel by path
    /// or by status (the default).
    ///
    /// Default: false
    pub sort_by_path: Option<bool>,

    /// Whether to collapse untracked files in the diff panel.
    ///
    /// Default: false
    pub collapse_untracked_diff: Option<bool>,

    /// Whether to show entries with tree or flat view in the panel
    ///
    /// Default: false
    pub tree_view: Option<bool>,

    /// Whether to show the addition/deletion change count next to each file in the Git panel.
    ///
    /// Default: true
    pub diff_stats: Option<bool>,

    /// Whether to show a badge on the git panel icon with the count of uncommitted changes.
    ///
    /// Default: false
    pub show_count_badge: Option<bool>,

    /// Whether the git panel should open on startup.
    ///
    /// Default: false
    pub starts_open: Option<bool>,

    /// Maximum length of the commit message title before a warning is shown.
    /// Set to 0 to disable.
    ///
    /// Default: 72
    pub commit_title_max_length: Option<usize>,
}

#[derive(
    Default,
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum StatusStyle {
    #[default]
    Icon,
    LabelColor,
}

#[with_fallible_options]
#[derive(
    Copy, Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq,
)]
pub struct ScrollbarSettings {
    pub show: Option<ShowScrollbar>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct PanelSettingsContent {
    /// Whether to show the panel button in the status bar.
    ///
    /// Default: true
    pub button: Option<bool>,
    /// Where to dock the panel.
    ///
    /// Default: left
    pub dock: Option<DockPosition>,
    /// Default width of the panel in pixels.
    ///
    /// Default: 240
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub default_width: Option<f32>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct MessageEditorSettings {
    /// Whether to automatically replace emoji shortcodes with emoji characters.
    /// For example: typing `:wave:` gets replaced with `👋`.
    ///
    /// Default: false
    pub auto_replace_emoji_shortcode: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct FileFinderSettingsContent {
    /// Whether to show file icons in the file finder.
    ///
    /// Default: true
    pub file_icons: Option<bool>,
    /// Determines how much space the file finder can take up in relation to the available window width.
    ///
    /// Default: small
    pub modal_max_width: Option<FileFinderWidthContent>,
    /// Determines whether the file finder should skip focus for the active file in search results.
    ///
    /// Default: true
    pub skip_focus_for_active_in_search: Option<bool>,
    /// Whether to use gitignored files when searching.
    /// Only the file Zed had indexed will be used, not necessary all the gitignored files.
    ///
    /// Default: Smart
    pub include_ignored: Option<IncludeIgnoredContent>,
    /// Whether to include text channels in file finder results.
    ///
    /// Default: false
    pub include_channels: Option<bool>,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum IncludeIgnoredContent {
    /// Use all gitignored files
    All,
    /// Use only the files Zed had indexed
    Indexed,
    /// Be smart and search for ignored when called from a gitignored worktree
    #[default]
    Smart,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "lowercase")]
pub enum FileFinderWidthContent {
    #[default]
    Small,
    Medium,
    Large,
    XLarge,
    Full,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug, JsonSchema, MergeFrom)]
pub struct VimSettingsContent {
    pub default_mode: Option<ModeContent>,
    pub toggle_relative_line_numbers: Option<bool>,
    pub use_system_clipboard: Option<UseSystemClipboard>,
    pub use_smartcase_find: Option<bool>,
    pub use_regex_search: Option<bool>,
    /// When enabled, the `:substitute` command replaces all matches in a line
    /// by default. The 'g' flag then toggles this behavior.,
    pub gdefault: Option<bool>,
    pub custom_digraphs: Option<HashMap<String, Arc<str>>>,
    pub highlight_on_yank_duration: Option<u64>,
    pub cursor_shape: Option<CursorShapeSettings>,
}

type JumpLabelSetting = Vec<String>;

fn jump_setting_from_strings(input: &[&str]) -> JumpLabelSetting {
    input
        .iter()
        .map(|ss| String::from_str(ss).unwrap())
        .collect()
}

fn default_screen_before() -> JumpLabelSetting {
    jump_setting_from_strings(&[
        "a", "r", "s", "t", "qr", "qs", "qt", "qx", "qc", "qv", "qb", "wa", "ws", "wt", "wz", "wc",
        "wv", "wb", "fa", "fr", "ft", "fz", "fx", "fv", "fb", "pa", "pr", "ps", "pz", "px", "pc",
        "pb", "ga", "gr", "gs", "gt", "gz", "gx", "gc", "gv", "zwq", "zww", "zwf", "zwp", "zwg",
        "zwa", "zws", "zwt", "zwz", "zwc", "zwv", "zwb", "zfq", "zfw", "zff", "zfp", "zfg", "zfa",
        "zfr", "zft", "zfz", "zfx", "zfv", "zfb", "zpq", "zpw", "zpf", "zpp", "zpg", "zpa", "zpr",
        "zps", "zpz", "zpx", "zpc", "zpb", "zgq", "zgw", "zgf", "zgp", "zgg", "zga", "zgr", "zgs",
        "zgt", "zgz", "zgx", "zgc", "zgv", "zrq", "zrf", "zrp", "zrg", "zra", "zrr", "zrs", "zrt",
        "zrz", "zrc", "zrv", "zrb", "zsq", "zsw", "zsp", "zsg", "zsa", "zsr", "zss", "zst", "zsz",
        "zsx", "zsv", "zsb", "ztq", "ztw", "ztf", "ztg", "zta", "ztr", "zts", "ztt", "ztz", "ztx",
        "ztc", "ztb", "vqq", "vqw", "vqf", "vqp", "vqg", "vqr", "vqs", "vqt", "vqx", "vqc", "vqv",
        "vqb", "vwq", "vww", "vwf", "vwp", "vwg", "vwa", "vws", "vwt", "vwz", "vwc", "vwv", "vwb",
        "vfq", "vfw", "vff", "vfp", "vfg", "vfa", "vfr", "vft", "vfz", "vfx", "vfv", "vfb", "vgq",
        "vgw", "vgf", "vgp", "vgg", "vga", "vgr", "vgs", "vgt", "vgz", "vgx", "vgc", "vgv", "vaw",
        "vaf", "vap", "vag", "vaa", "var", "vas", "vat", "vax", "vac", "vav", "vab", "vrq", "vrf",
        "vrp", "vrg", "vra", "vrr", "vrs", "vrt", "vrz", "vrc", "vrv", "vrb", "vsq", "vsw", "vsp",
        "vsg", "vsa", "vsr", "vss", "vst", "vsz", "vsx", "vsv", "vsb", "bqq", "bqw", "bqf", "bqp",
        "bqg", "bqr", "bqs", "bqt", "bqx", "bqc", "bqv", "bqb", "bwq", "bww", "bwf", "bwp", "bwg",
        "bwa", "bws", "bwt", "bwz", "bwc", "bwv", "bwb", "bfq", "bfw", "bff", "bfp", "bfg", "bfa",
        "bfr", "bft", "bfz", "bfx", "bfv", "bfb", "bpq", "bpw", "bpf", "bpp", "bpg", "bpa", "bpr",
        "bps", "bpz", "bpx", "bpc", "bpb", "baw", "baf", "bap", "bag", "baa", "bar", "bas", "bat",
        "bax", "bac", "bav", "bab", "brq", "brf", "brp", "brg", "bra", "brr", "brs", "brt", "brz",
        "brc", "brv", "brb", "bsq", "bsw", "bsp", "bsg", "bsa", "bsr", "bss", "bst", "bsz", "bsx",
        "bsv", "bsb", "btq", "btw", "btf", "btg", "bta", "btr", "bts", "btt", "btz", "btx", "btc",
        "btb", "xqqq", "xqqw", "xqqf", "xqqp", "xqqg", "xqqr", "xqqs", "xqqt", "xqqx", "xqqc",
        "xqqv", "xqqb", "xqwq", "xqww", "xqwf", "xqwp", "xqwg", "xqwa", "xqws", "xqwt", "xqwz",
        "xqwc", "xqwv", "xqwb", "xqfq", "xqfw", "xqff", "xqfp", "xqfg", "xqfa", "xqfr", "xqft",
        "xqfz", "xqfx", "xqfv", "xqfb", "xqpq", "xqpw", "xqpf", "xqpp", "xqpg", "xqpa", "xqpr",
        "xqps", "xqpz", "xqpx", "xqpc", "xqpb", "xqgq", "xqgw", "xqgf", "xqgp", "xqgg", "xqga",
        "xqgr", "xqgs", "xqgt", "xqgz", "xqgx", "xqgc", "xqgv", "xqrq", "xqrf", "xqrp", "xqrg",
        "xqra", "xqrr", "xqrs", "xqrt", "xqrz", "xqrc", "xqrv", "xqrb", "xqsq", "xqsw", "xqsp",
        "xqsg", "xqsa", "xqsr", "xqss", "xqst", "xqsz", "xqsx", "xqsv", "xqsb", "xqtq", "xqtw",
        "xqtf", "xqtg", "xqta", "xqtr", "xqts", "xqtt", "xqtz", "xqtx", "xqtc", "xqtb", "xqxq",
        "xqxf", "xqxp", "xqxg", "xqxa", "xqxs", "xqxt", "xqxz", "xqxx", "xqxc", "xqxv", "xqxb",
        "xqcq", "xqcw", "xqcp", "xqcg", "xqca", "xqcr", "xqct", "xqcz", "xqcx", "xqcc", "xqcv",
        "xqcb", "xqvq", "xqvw", "xqvf", "xqvg", "xqva", "xqvr", "xqvs", "xqvz", "xqvx", "xqvc",
        "xqvv", "xqvb", "xqbq", "xqbw", "xqbf", "xqbp", "xqba", "xqbr", "xqbs", "xqbt", "xqbz",
        "xqbx", "xqbc", "xqbv", "xqbb", "xfqq", "xfqw", "xfqf", "xfqp", "xfqg", "xfqr", "xfqs",
        "xfqt", "xfqx", "xfqc", "xfqv", "xfqb", "xfwq", "xfww", "xfwf", "xfwp", "xfwg", "xfwa",
        "xfws", "xfwt", "xfwz", "xfwc", "xfwv", "xfwb", "xffq", "xffw", "xfff", "xffp", "xffg",
        "xffa", "xffr", "xfft", "xffz", "xffx", "xffv", "xffb", "xfpq", "xfpw", "xfpf", "xfpp",
        "xfpg", "xfpa", "xfpr", "xfps", "xfpz", "xfpx", "xfpc", "xfpb", "xfgq", "xfgw", "xfgf",
        "xfgp", "xfgg", "xfga", "xfgr", "xfgs", "xfgt", "xfgz", "xfgx", "xfgc", "xfgv", "xfaw",
        "xfaf", "xfap", "xfag", "xfaa", "xfar", "xfas", "xfat", "xfax", "xfac", "xfav", "xfab",
        "xfrq", "xfrf", "xfrp", "xfrg", "xfra", "xfrr", "xfrs", "xfrt", "xfrz", "xfrc", "xfrv",
        "xfrb", "xftq", "xftw", "xftf", "xftg", "xfta", "xftr", "xfts", "xftt", "xftz", "xftx",
        "xftc", "xftb", "xfzw", "xfzf", "xfzp", "xfzg", "xfzr", "xfzs", "xfzt", "xfzz", "xfzx",
        "xfzc", "xfzv", "xfzb", "xfxq", "xfxf", "xfxp", "xfxg", "xfxa", "xfxs", "xfxt", "xfxz",
        "xfxx", "xfxc", "xfxv", "xfxb", "xfvq", "xfvw", "xfvf", "xfvg", "xfva", "xfvr", "xfvs",
        "xfvz", "xfvx", "xfvc", "xfvv", "xfvb", "xfbq", "xfbw", "xfbf", "xfbp", "xfba", "xfbr",
        "xfbs", "xfbt", "xfbz", "xfbx", "xfbc", "xfbv", "xfbb", "xpqq", "xpqw", "xpqf", "xpqp",
        "xpqg", "xpqr", "xpqs", "xpqt", "xpqx", "xpqc", "xpqv", "xpqb", "xpwq", "xpww", "xpwf",
        "xpwp", "xpwg", "xpwa", "xpws", "xpwt", "xpwz", "xpwc", "xpwv", "xpwb", "xpfq", "xpfw",
        "xpff", "xpfp", "xpfg", "xpfa", "xpfr", "xpft", "xpfz", "xpfx", "xpfv", "xpfb", "xppq",
        "xppw", "xppf", "xppp", "xppg", "xppa", "xppr", "xpps", "xppz", "xppx", "xppc", "xppb",
        "xpgq", "xpgw", "xpgf", "xpgp", "xpgg", "xpga", "xpgr", "xpgs", "xpgt", "xpgz", "xpgx",
        "xpgc", "xpgv", "xpaw", "xpaf", "xpap", "xpag", "xpaa", "xpar", "xpas", "xpat", "xpax",
        "xpac", "xpav", "xpab", "xprq", "xprf", "xprp", "xprg", "xpra", "xprr", "xprs", "xprt",
        "xprz", "xprc", "xprv", "xprb", "xpsq", "xpsw", "xpsp", "xpsg", "xpsa", "xpsr", "xpss",
        "xpst", "xpsz", "xpsx", "xpsv", "xpsb", "xpzw", "xpzf", "xpzp", "xpzg", "xpzr", "xpzs",
        "xpzt", "xpzz", "xpzx", "xpzc", "xpzv", "xpzb", "xpxq", "xpxf", "xpxp", "xpxg", "xpxa",
        "xpxs", "xpxt", "xpxz", "xpxx", "xpxc", "xpxv", "xpxb", "xpcq", "xpcw", "xpcp", "xpcg",
        "xpca", "xpcr", "xpct", "xpcz", "xpcx", "xpcc", "xpcv", "xpcb", "xpbq", "xpbw", "xpbf",
        "xpbp", "xpba", "xpbr", "xpbs", "xpbt", "xpbz", "xpbx", "xpbc", "xpbv", "xpbb", "xgqq",
        "xgqw", "xgqf", "xgqp", "xgqg", "xgqr", "xgqs", "xgqt", "xgqx", "xgqc", "xgqv", "xgqb",
        "xgwq", "xgww", "xgwf", "xgwp", "xgwg", "xgwa", "xgws", "xgwt", "xgwz", "xgwc", "xgwv",
        "xgwb", "xgfq", "xgfw", "xgff", "xgfp", "xgfg", "xgfa", "xgfr", "xgft", "xgfz", "xgfx",
        "xgfv", "xgfb", "xgpq", "xgpw", "xgpf", "xgpp", "xgpg", "xgpa", "xgpr", "xgps", "xgpz",
        "xgpx", "xgpc", "xgpb", "xggq", "xggw", "xggf", "xggp", "xggg", "xgga", "xggr", "xggs",
        "xggt", "xggz", "xggx", "xggc", "xggv", "xgaw", "xgaf", "xgap", "xgag", "xgaa", "xgar",
        "xgas", "xgat", "xgax", "xgac", "xgav", "xgab", "xgrq", "xgrf", "xgrp", "xgrg", "xgra",
        "xgrr", "xgrs", "xgrt", "xgrz", "xgrc", "xgrv", "xgrb", "xgsq", "xgsw", "xgsp", "xgsg",
        "xgsa", "xgsr", "xgss", "xgst", "xgsz", "xgsx", "xgsv", "xgsb", "xgtq", "xgtw", "xgtf",
        "xgtg", "xgta", "xgtr", "xgts", "xgtt", "xgtz", "xgtx", "xgtc", "xgtb", "xgzw", "xgzf",
        "xgzp", "xgzg", "xgzr", "xgzs", "xgzt", "xgzz", "xgzx", "xgzc", "xgzv", "xgzb", "xgxq",
        "xgxf", "xgxp", "xgxg", "xgxa", "xgxs", "xgxt", "xgxz", "xgxx", "xgxc", "xgxv", "xgxb",
        "xgcq", "xgcw", "xgcp", "xgcg", "xgca", "xgcr", "xgct", "xgcz", "xgcx", "xgcc", "xgcv",
        "xgcb", "xgvq", "xgvw", "xgvf", "xgvg", "xgva", "xgvr", "xgvs", "xgvz", "xgvx", "xgvc",
        "xgvv", "xgvb", "xawq", "xaww", "xawf", "xawp", "xawg", "xawa", "xaws", "xawt", "xawz",
        "xawc", "xawv", "xawb", "xafq", "xafw", "xaff", "xafp", "xafg", "xafa", "xafr", "xaft",
        "xafz", "xafx", "xafv", "xafb", "xapq", "xapw", "xapf", "xapp", "xapg", "xapa", "xapr",
        "xaps", "xapz", "xapx", "xapc", "xapb", "xagq", "xagw", "xagf", "xagp", "xagg", "xaga",
        "xagr", "xags", "xagt", "xagz", "xagx", "xagc", "xagv", "xaaw", "xaaf", "xaap", "xaag",
        "xaaa", "xaar", "xaas", "xaat", "xaax", "xaac", "xaav", "xaab", "xarq", "xarf", "xarp",
        "xarg", "xara", "xarr", "xars", "xart", "xarz", "xarc", "xarv", "xarb", "xasq", "xasw",
        "xasp", "xasg", "xasa", "xasr", "xass", "xast", "xasz", "xasx", "xasv", "xasb", "xatq",
        "xatw", "xatf", "xatg", "xata", "xatr", "xats", "xatt", "xatz", "xatx", "xatc", "xatb",
        "xaxq", "xaxf", "xaxp", "xaxg", "xaxa", "xaxs", "xaxt", "xaxz", "xaxx", "xaxc", "xaxv",
        "xaxb", "xacq", "xacw", "xacp", "xacg", "xaca", "xacr", "xact", "xacz", "xacx", "xacc",
        "xacv", "xacb", "xavq", "xavw", "xavf", "xavg", "xava", "xavr", "xavs", "xavz", "xavx",
        "xavc", "xavv", "xavb", "xabq", "xabw", "xabf", "xabp", "xaba", "xabr", "xabs", "xabt",
        "xabz", "xabx", "xabc", "xabv", "xabb", "xsqq", "xsqw", "xsqf", "xsqp", "xsqg", "xsqr",
        "xsqs", "xsqt", "xsqx", "xsqc", "xsqv", "xsqb", "xswq", "xsww", "xswf", "xswp", "xswg",
        "xswa", "xsws", "xswt", "xswz", "xswc", "xswv", "xswb", "xspq", "xspw", "xspf", "xspp",
        "xspg", "xspa", "xspr", "xsps", "xspz", "xspx", "xspc", "xspb", "xsgq", "xsgw", "xsgf",
        "xsgp", "xsgg", "xsga", "xsgr", "xsgs", "xsgt", "xsgz", "xsgx", "xsgc", "xsgv", "xsaw",
        "xsaf", "xsap", "xsag", "xsaa", "xsar", "xsas", "xsat", "xsax", "xsac", "xsav", "xsab",
        "xsrq", "xsrf", "xsrp", "xsrg", "xsra", "xsrr", "xsrs", "xsrt", "xsrz", "xsrc", "xsrv",
        "xsrb", "xssq", "xssw", "xssp", "xssg", "xssa", "xssr", "xsss", "xsst", "xssz", "xssx",
        "xssv", "xssb", "xstq", "xstw", "xstf", "xstg", "xsta", "xstr", "xsts", "xstt", "xstz",
        "xstx", "xstc", "xstb", "xszw", "xszf", "xszp", "xszg", "xszr", "xszs", "xszt", "xszz",
        "xszx", "xszc", "xszv", "xszb", "xsxq", "xsxf", "xsxp", "xsxg", "xsxa", "xsxs", "xsxt",
        "xsxz", "xsxx", "xsxc", "xsxv", "xsxb", "xsvq", "xsvw", "xsvf", "xsvg", "xsva", "xsvr",
        "xsvs", "xsvz", "xsvx", "xsvc", "xsvv", "xsvb", "xsbq", "xsbw", "xsbf", "xsbp", "xsba",
        "xsbr", "xsbs", "xsbt", "xsbz", "xsbx", "xsbc", "xsbv", "xsbb", "xtqq", "xtqw", "xtqf",
        "xtqp", "xtqg", "xtqr", "xtqs", "xtqt", "xtqx", "xtqc", "xtqv", "xtqb", "xtwq", "xtww",
        "xtwf", "xtwp", "xtwg", "xtwa", "xtws", "xtwt", "xtwz", "xtwc", "xtwv", "xtwb", "xtfq",
        "xtfw", "xtff", "xtfp", "xtfg", "xtfa", "xtfr", "xtft", "xtfz", "xtfx", "xtfv", "xtfb",
        "xtgq", "xtgw", "xtgf", "xtgp", "xtgg", "xtga", "xtgr", "xtgs", "xtgt", "xtgz", "xtgx",
        "xtgc", "xtgv", "xtaw", "xtaf", "xtap", "xtag", "xtaa", "xtar", "xtas", "xtat", "xtax",
        "xtac", "xtav", "xtab", "xtrq", "xtrf", "xtrp", "xtrg", "xtra", "xtrr", "xtrs", "xtrt",
        "xtrz", "xtrc", "xtrv", "xtrb", "xtsq", "xtsw", "xtsp", "xtsg", "xtsa", "xtsr", "xtss",
        "xtst", "xtsz", "xtsx", "xtsv", "xtsb", "xttq", "xttw", "xttf", "xttg", "xtta", "xttr",
        "xtts", "xttt", "xttz", "xttx", "xttc", "xttb", "xtzw", "xtzf", "xtzp", "xtzg", "xtzr",
        "xtzs", "xtzt", "xtzz", "xtzx", "xtzc", "xtzv", "xtzb", "xtxq", "xtxf", "xtxp", "xtxg",
        "xtxa", "xtxs", "xtxt", "xtxz", "xtxx", "xtxc", "xtxv", "xtxb", "xtcq", "xtcw", "xtcp",
        "xtcg", "xtca", "xtcr", "xtct", "xtcz", "xtcx", "xtcc", "xtcv", "xtcb", "xtbq", "xtbw",
        "xtbf", "xtbp", "xtba", "xtbr", "xtbs", "xtbt", "xtbz", "xtbx", "xtbc", "xtbv", "xtbb",
        "cqqq", "cqqw", "cqqf", "cqqp", "cqqg", "cqqr", "cqqs", "cqqt", "cqqx", "cqqc", "cqqv",
        "cqqb", "cqwq", "cqww", "cqwf", "cqwp", "cqwg", "cqwa", "cqws", "cqwt", "cqwz", "cqwc",
        "cqwv", "cqwb", "cqfq", "cqfw", "cqff", "cqfp", "cqfg", "cqfa", "cqfr", "cqft", "cqfz",
        "cqfx", "cqfv", "cqfb", "cqpq", "cqpw", "cqpf", "cqpp", "cqpg", "cqpa", "cqpr", "cqps",
        "cqpz", "cqpx", "cqpc", "cqpb", "cqgq", "cqgw", "cqgf", "cqgp", "cqgg", "cqga", "cqgr",
        "cqgs", "cqgt", "cqgz", "cqgx", "cqgc", "cqgv", "cqrq", "cqrf", "cqrp", "cqrg", "cqra",
        "cqrr", "cqrs", "cqrt", "cqrz", "cqrc", "cqrv", "cqrb", "cqsq", "cqsw", "cqsp", "cqsg",
        "cqsa", "cqsr", "cqss", "cqst", "cqsz", "cqsx", "cqsv", "cqsb", "cqtq", "cqtw", "cqtf",
        "cqtg", "cqta", "cqtr", "cqts", "cqtt", "cqtz", "cqtx", "cqtc", "cqtb", "cqxq", "cqxf",
        "cqxp", "cqxg", "cqxa", "cqxs", "cqxt", "cqxz", "cqxx", "cqxc", "cqxv", "cqxb", "cqcq",
        "cqcw", "cqcp", "cqcg", "cqca", "cqcr", "cqct", "cqcz", "cqcx", "cqcc", "cqcv", "cqcb",
        "cqvq", "cqvw", "cqvf", "cqvg", "cqva", "cqvr", "cqvs", "cqvz", "cqvx", "cqvc", "cqvv",
        "cqvb", "cqbq", "cqbw", "cqbf", "cqbp", "cqba", "cqbr", "cqbs", "cqbt", "cqbz", "cqbx",
        "cqbc", "cqbv", "cqbb", "cwqq", "cwqw", "cwqf", "cwqp", "cwqg", "cwqr", "cwqs", "cwqt",
        "cwqx", "cwqc", "cwqv", "cwqb", "cwwq", "cwww", "cwwf", "cwwp", "cwwg", "cwwa", "cwws",
        "cwwt", "cwwz", "cwwc", "cwwv", "cwwb", "cwfq", "cwfw", "cwff", "cwfp", "cwfg", "cwfa",
        "cwfr", "cwft", "cwfz", "cwfx", "cwfv", "cwfb", "cwpq", "cwpw", "cwpf", "cwpp", "cwpg",
        "cwpa", "cwpr", "cwps", "cwpz", "cwpx", "cwpc", "cwpb", "cwgq", "cwgw", "cwgf", "cwgp",
        "cwgg", "cwga", "cwgr", "cwgs", "cwgt", "cwgz", "cwgx", "cwgc", "cwgv", "cwaw", "cwaf",
        "cwap", "cwag", "cwaa", "cwar", "cwas", "cwat", "cwax", "cwac", "cwav", "cwab", "cwsq",
        "cwsw", "cwsp", "cwsg", "cwsa", "cwsr", "cwss", "cwst", "cwsz", "cwsx", "cwsv", "cwsb",
        "cwtq", "cwtw", "cwtf", "cwtg", "cwta", "cwtr", "cwts", "cwtt", "cwtz", "cwtx", "cwtc",
        "cwtb", "cwzw", "cwzf", "cwzp", "cwzg", "cwzr", "cwzs", "cwzt", "cwzz", "cwzx", "cwzc",
        "cwzv", "cwzb", "cwcq", "cwcw", "cwcp", "cwcg", "cwca", "cwcr", "cwct", "cwcz", "cwcx",
        "cwcc", "cwcv", "cwcb", "cwvq", "cwvw", "cwvf", "cwvg", "cwva", "cwvr", "cwvs", "cwvz",
        "cwvx", "cwvc", "cwvv", "cwvb", "cwbq", "cwbw", "cwbf", "cwbp", "cwba", "cwbr", "cwbs",
        "cwbt", "cwbz", "cwbx", "cwbc", "cwbv", "cwbb", "cpqq", "cpqw", "cpqf", "cpqp", "cpqg",
        "cpqr", "cpqs", "cpqt", "cpqx", "cpqc", "cpqv", "cpqb", "cpwq", "cpww", "cpwf", "cpwp",
        "cpwg", "cpwa", "cpws", "cpwt", "cpwz", "cpwc", "cpwv", "cpwb", "cpfq", "cpfw", "cpff",
        "cpfp", "cpfg", "cpfa", "cpfr", "cpft", "cpfz", "cpfx", "cpfv", "cpfb", "cppq", "cppw",
        "cppf", "cppp", "cppg", "cppa", "cppr", "cpps", "cppz", "cppx", "cppc", "cppb", "cpgq",
        "cpgw", "cpgf", "cpgp", "cpgg", "cpga", "cpgr", "cpgs", "cpgt", "cpgz", "cpgx", "cpgc",
        "cpgv", "cpaw", "cpaf", "cpap", "cpag", "cpaa", "cpar", "cpas", "cpat", "cpax", "cpac",
        "cpav", "cpab", "cprq", "cprf", "cprp", "cprg", "cpra", "cprr", "cprs", "cprt", "cprz",
        "cprc", "cprv", "cprb", "cpsq", "cpsw", "cpsp", "cpsg", "cpsa", "cpsr", "cpss", "cpst",
        "cpsz", "cpsx", "cpsv", "cpsb", "cpzw", "cpzf", "cpzp", "cpzg", "cpzr", "cpzs", "cpzt",
        "cpzz", "cpzx", "cpzc", "cpzv", "cpzb", "cpxq", "cpxf", "cpxp", "cpxg", "cpxa", "cpxs",
        "cpxt", "cpxz", "cpxx", "cpxc", "cpxv", "cpxb", "cpcq", "cpcw", "cpcp", "cpcg", "cpca",
        "cpcr", "cpct", "cpcz", "cpcx", "cpcc", "cpcv", "cpcb", "cpbq", "cpbw", "cpbf", "cpbp",
        "cpba", "cpbr", "cpbs", "cpbt", "cpbz", "cpbx", "cpbc", "cpbv", "cpbb", "cgqq", "cgqw",
        "cgqf", "cgqp", "cgqg", "cgqr", "cgqs", "cgqt", "cgqx", "cgqc", "cgqv", "cgqb", "cgwq",
        "cgww", "cgwf", "cgwp", "cgwg", "cgwa", "cgws", "cgwt", "cgwz", "cgwc", "cgwv", "cgwb",
        "cgfq", "cgfw", "cgff", "cgfp", "cgfg", "cgfa", "cgfr", "cgft", "cgfz", "cgfx", "cgfv",
        "cgfb", "cgpq", "cgpw", "cgpf", "cgpp", "cgpg", "cgpa", "cgpr", "cgps", "cgpz", "cgpx",
        "cgpc", "cgpb", "cggq", "cggw", "cggf", "cggp", "cggg", "cgga", "cggr", "cggs", "cggt",
        "cggz", "cggx", "cggc", "cggv", "cgaw", "cgaf", "cgap", "cgag", "cgaa", "cgar", "cgas",
        "cgat", "cgax", "cgac", "cgav", "cgab", "cgrq", "cgrf", "cgrp", "cgrg", "cgra", "cgrr",
        "cgrs", "cgrt", "cgrz", "cgrc", "cgrv", "cgrb", "cgsq", "cgsw", "cgsp", "cgsg", "cgsa",
        "cgsr", "cgss", "cgst", "cgsz", "cgsx", "cgsv", "cgsb", "cgtq", "cgtw", "cgtf", "cgtg",
        "cgta", "cgtr", "cgts", "cgtt", "cgtz", "cgtx", "cgtc", "cgtb", "cgzw", "cgzf", "cgzp",
        "cgzg", "cgzr", "cgzs", "cgzt", "cgzz", "cgzx", "cgzc", "cgzv", "cgzb", "cgxq", "cgxf",
        "cgxp", "cgxg", "cgxa", "cgxs", "cgxt", "cgxz", "cgxx", "cgxc", "cgxv", "cgxb", "cgcq",
        "cgcw", "cgcp", "cgcg", "cgca", "cgcr", "cgct", "cgcz", "cgcx", "cgcc", "cgcv", "cgcb",
        "cgvq", "cgvw", "cgvf", "cgvg", "cgva", "cgvr", "cgvs", "cgvz", "cgvx", "cgvc", "cgvv",
        "cgvb", "cawq", "caww", "cawf", "cawp", "cawg", "cawa", "caws", "cawt", "cawz", "cawc",
        "cawv", "cawb", "cafq", "cafw", "caff", "cafp", "cafg", "cafa", "cafr", "caft", "cafz",
        "cafx", "cafv", "cafb", "capq", "capw", "capf", "capp", "capg", "capa", "capr", "caps",
        "capz", "capx", "capc", "capb", "cagq", "cagw", "cagf", "cagp", "cagg", "caga", "cagr",
        "cags", "cagt", "cagz", "cagx", "cagc", "cagv", "caaw", "caaf", "caap", "caag", "caaa",
        "caar", "caas", "caat", "caax", "caac", "caav", "caab", "carq", "carf", "carp", "carg",
        "cara", "carr", "cars", "cart", "carz", "carc", "carv", "carb", "casq", "casw", "casp",
        "casg", "casa", "casr", "cass", "cast", "casz", "casx", "casv", "casb", "catq", "catw",
        "catf", "catg", "cata", "catr", "cats", "catt", "catz", "catx", "catc", "catb", "caxq",
        "caxf", "caxp", "caxg", "caxa", "caxs", "caxt", "caxz", "caxx", "caxc", "caxv", "caxb",
        "cacq", "cacw", "cacp", "cacg", "caca", "cacr", "cact", "cacz", "cacx", "cacc", "cacv",
        "cacb", "cavq", "cavw", "cavf", "cavg", "cava", "cavr", "cavs", "cavz", "cavx", "cavc",
        "cavv", "cavb", "cabq", "cabw", "cabf", "cabp", "caba", "cabr", "cabs", "cabt", "cabz",
        "cabx", "cabc", "cabv", "cabb", "crqq", "crqw", "crqf", "crqp", "crqg", "crqr", "crqs",
        "crqt", "crqx", "crqc", "crqv", "crqb", "crfq", "crfw", "crff", "crfp", "crfg", "crfa",
        "crfr", "crft", "crfz", "crfx", "crfv", "crfb", "crpq", "crpw", "crpf", "crpp", "crpg",
        "crpa", "crpr", "crps", "crpz", "crpx", "crpc", "crpb", "crgq", "crgw", "crgf", "crgp",
        "crgg", "crga", "crgr", "crgs", "crgt", "crgz", "crgx", "crgc", "crgv", "craw", "craf",
        "crap", "crag", "craa", "crar", "cras", "crat", "crax", "crac", "crav", "crab", "crrq",
        "crrf", "crrp", "crrg", "crra", "crrr", "crrs", "crrt", "crrz", "crrc", "crrv", "crrb",
        "crsq", "crsw", "crsp", "crsg", "crsa", "crsr", "crss", "crst", "crsz", "crsx", "crsv",
        "crsb", "crtq", "crtw", "crtf", "crtg", "crta", "crtr", "crts", "crtt", "crtz", "crtx",
        "crtc", "crtb", "crzw", "crzf", "crzp", "crzg", "crzr", "crzs", "crzt", "crzz", "crzx",
        "crzc", "crzv", "crzb", "crcq", "crcw", "crcp", "crcg", "crca", "crcr", "crct", "crcz",
        "crcx", "crcc", "crcv", "crcb", "crvq", "crvw", "crvf", "crvg", "crva", "crvr", "crvs",
        "crvz", "crvx", "crvc", "crvv", "crvb", "crbq", "crbw", "crbf", "crbp", "crba", "crbr",
        "crbs", "crbt", "crbz", "crbx", "crbc", "crbv", "crbb", "ctqq", "ctqw", "ctqf", "ctqp",
        "ctqg", "ctqr", "ctqs", "ctqt", "ctqx", "ctqc", "ctqv", "ctqb", "ctwq", "ctww", "ctwf",
        "ctwp", "ctwg", "ctwa", "ctws", "ctwt", "ctwz", "ctwc", "ctwv", "ctwb", "ctfq", "ctfw",
        "ctff", "ctfp", "ctfg", "ctfa", "ctfr", "ctft", "ctfz", "ctfx", "ctfv", "ctfb", "ctgq",
        "ctgw", "ctgf", "ctgp", "ctgg", "ctga", "ctgr", "ctgs", "ctgt", "ctgz", "ctgx", "ctgc",
        "ctgv", "ctaw", "ctaf", "ctap", "ctag", "ctaa", "ctar", "ctas", "ctat", "ctax", "ctac",
        "ctav", "ctab", "ctrq", "ctrf", "ctrp", "ctrg", "ctra", "ctrr", "ctrs", "ctrt", "ctrz",
        "ctrc", "ctrv", "ctrb", "ctsq", "ctsw", "ctsp", "ctsg", "ctsa", "ctsr", "ctss", "ctst",
        "ctsz", "ctsx", "ctsv", "ctsb", "cttq", "cttw", "cttf", "cttg", "ctta", "cttr", "ctts",
        "cttt", "cttz", "cttx", "cttc", "cttb", "ctzw", "ctzf", "ctzp", "ctzg", "ctzr", "ctzs",
        "ctzt", "ctzz", "ctzx", "ctzc", "ctzv", "ctzb", "ctxq", "ctxf", "ctxp", "ctxg", "ctxa",
        "ctxs", "ctxt", "ctxz", "ctxx", "ctxc", "ctxv", "ctxb", "ctcq", "ctcw", "ctcp", "ctcg",
        "ctca", "ctcr", "ctct", "ctcz", "ctcx", "ctcc", "ctcv", "ctcb", "ctbq", "ctbw", "ctbf",
        "ctbp", "ctba", "ctbr", "ctbs", "ctbt", "ctbz", "ctbx", "ctbc", "ctbv", "ctbb",
    ])
}
//luyo;
//
fn default_screen_after() -> JumpLabelSetting {
    jump_setting_from_strings(&[
        "n", "e", "i", "o", "jn", "je", "ji", "jo", "jm", "j,", "j.", "j/", "le", "li", "lo", "lk",
        "l,", "l.", "l/", "un", "ui", "uo", "uk", "um", "u.", "u/", "yn", "ye", "yo", "yk", "ym",
        "y,", "y/", ";n", ";e", ";i", ";k", ";m", ";,", ";.", "klj", "kll", "klu", "kly", "kl;",
        "kle", "kli", "klo", "klk", "kl,", "kl.", "kl/", "kuj", "kul", "kuu", "kuy", "ku;", "kun",
        "kui", "kuo", "kuk", "kum", "ku.", "ku/", "kyj", "kyl", "kyu", "kyy", "ky;", "kyn", "kye",
        "kyo", "kyk", "kym", "ky,", "ky/", "k;j", "k;l", "k;u", "k;y", "k;;", "k;n", "k;e", "k;i",
        "k;k", "k;m", "k;,", "k;.", "knj", "knu", "kny", "kn;", "knn", "kne", "kni", "kno", "knk",
        "kn,", "kn.", "kn/", "kej", "kel", "key", "ke;", "ken", "kee", "kei", "keo", "kek", "kem",
        "ke.", "ke/", "kij", "kil", "kiu", "ki;", "kin", "kie", "kii", "kio", "kik", "kim", "ki,",
        "ki/", "koj", "kol", "kou", "koy", "kon", "koe", "koi", "koo", "kok", "kom", "ko,", "ko.",
        "mjj", "mjl", "mju", "mjy", "mj;", "mjn", "mje", "mji", "mjo", "mjm", "mj,", "mj.", "mj/",
        "muj", "mul", "muu", "muy", "mu;", "mun", "mui", "muo", "muk", "mum", "mu.", "mu/", "myj",
        "myl", "myu", "myy", "my;", "myn", "mye", "myo", "myk", "mym", "my,", "my/", "m;j", "m;l",
        "m;u", "m;y", "m;;", "m;n", "m;e", "m;i", "m;k", "m;m", "m;,", "m;.", "mej", "mel", "mey",
        "me;", "men", "mee", "mei", "meo", "mek", "mem", "me.", "me/", "mij", "mil", "miu", "mi;",
        "min", "mie", "mii", "mio", "mik", "mim", "mi,", "mi/", "moj", "mol", "mou", "moy", "mon",
        "moe", "moi", "moo", "mok", "mom", "mo,", "mo.", "/jj", "/jl", "/ju", "/jy", "/j;", "/jn",
        "/je", "/ji", "/jo", "/jm", "/j,", "/j.", "/j/", "/lj", "/ll", "/lu", "/ly", "/l;", "/le",
        "/li", "/lo", "/lk", "/l,", "/l.", "/l/", "/uj", "/ul", "/uu", "/uy", "/u;", "/un", "/ui",
        "/uo", "/uk", "/um", "/u.", "/u/", "/yj", "/yl", "/yu", "/yy", "/y;", "/yn", "/ye", "/yo",
        "/yk", "/ym", "/y,", "/y/", "/nj", "/nu", "/ny", "/n;", "/nn", "/ne", "/ni", "/no", "/nk",
        "/n,", "/n.", "/n/", "/ej", "/el", "/ey", "/e;", "/en", "/ee", "/ei", "/eo", "/ek", "/em",
        "/e.", "/e/", "/ij", "/il", "/iu", "/i;", "/in", "/ie", "/ii", "/io", "/ik", "/im", "/i,",
        "/i/", ",jjj", ",jjl", ",jju", ",jjy", ",jj;", ",jjn", ",jje", ",jji", ",jjo", ",jjm",
        ",jj,", ",jj.", ",jj/", ",jlj", ",jll", ",jlu", ",jly", ",jl;", ",jle", ",jli", ",jlo",
        ",jlk", ",jl,", ",jl.", ",jl/", ",juj", ",jul", ",juu", ",juy", ",ju;", ",jun", ",jui",
        ",juo", ",juk", ",jum", ",ju.", ",ju/", ",jyj", ",jyl", ",jyu", ",jyy", ",jy;", ",jyn",
        ",jye", ",jyo", ",jyk", ",jym", ",jy,", ",jy/", ",j;j", ",j;l", ",j;u", ",j;y", ",j;;",
        ",j;n", ",j;e", ",j;i", ",j;k", ",j;m", ",j;,", ",j;.", ",jnj", ",jnu", ",jny", ",jn;",
        ",jnn", ",jne", ",jni", ",jno", ",jnk", ",jn,", ",jn.", ",jn/", ",jej", ",jel", ",jey",
        ",je;", ",jen", ",jee", ",jei", ",jeo", ",jek", ",jem", ",je.", ",je/", ",jij", ",jil",
        ",jiu", ",ji;", ",jin", ",jie", ",jii", ",jio", ",jik", ",jim", ",ji,", ",ji/", ",joj",
        ",jol", ",jou", ",joy", ",jon", ",joe", ",joi", ",joo", ",jok", ",jom", ",jo,", ",jo.",
        ",jmj", ",jmu", ",jmy", ",jm;", ",jme", ",jmi", ",jmo", ",jmk", ",jmm", ",jm,", ",jm.",
        ",jm/", ",j,j", ",j,l", ",j,y", ",j,;", ",j,n", ",j,i", ",j,o", ",j,k", ",j,m", ",j,,",
        ",j,.", ",j,/", ",j.j", ",j.l", ",j.u", ",j.;", ",j.n", ",j.e", ",j.o", ",j.k", ",j.m",
        ",j.,", ",j..", ",j./", ",j/j", ",j/l", ",j/u", ",j/y", ",j/n", ",j/e", ",j/i", ",j/k",
        ",j/m", ",j/,", ",j/.", ",j//", ",ljj", ",ljl", ",lju", ",ljy", ",lj;", ",ljn", ",lje",
        ",lji", ",ljo", ",ljm", ",lj,", ",lj.", ",lj/", ",llj", ",lll", ",llu", ",lly", ",ll;",
        ",lle", ",lli", ",llo", ",llk", ",ll,", ",ll.", ",ll/", ",luj", ",lul", ",luu", ",luy",
        ",lu;", ",lun", ",lui", ",luo", ",luk", ",lum", ",lu.", ",lu/", ",lyj", ",lyl", ",lyu",
        ",lyy", ",ly;", ",lyn", ",lye", ",lyo", ",lyk", ",lym", ",ly,", ",ly/", ",l;j", ",l;l",
        ",l;u", ",l;y", ",l;;", ",l;n", ",l;e", ",l;i", ",l;k", ",l;m", ",l;,", ",l;.", ",lej",
        ",lel", ",ley", ",le;", ",len", ",lee", ",lei", ",leo", ",lek", ",lem", ",le.", ",le/",
        ",lij", ",lil", ",liu", ",li;", ",lin", ",lie", ",lii", ",lio", ",lik", ",lim", ",li,",
        ",li/", ",loj", ",lol", ",lou", ",loy", ",lon", ",loe", ",loi", ",loo", ",lok", ",lom",
        ",lo,", ",lo.", ",lkl", ",lku", ",lky", ",lk;", ",lkn", ",lke", ",lki", ",lko", ",lkk",
        ",lkm", ",lk,", ",lk.", ",lk/", ",l,j", ",l,l", ",l,y", ",l,;", ",l,n", ",l,i", ",l,o",
        ",l,k", ",l,m", ",l,,", ",l,.", ",l,/", ",l.j", ",l.l", ",l.u", ",l.;", ",l.n", ",l.e",
        ",l.o", ",l.k", ",l.m", ",l.,", ",l..", ",l./", ",l/j", ",l/l", ",l/u", ",l/y", ",l/n",
        ",l/e", ",l/i", ",l/k", ",l/m", ",l/,", ",l/.", ",l//", ",yjj", ",yjl", ",yju", ",yjy",
        ",yj;", ",yjn", ",yje", ",yji", ",yjo", ",yjm", ",yj,", ",yj.", ",yj/", ",ylj", ",yll",
        ",ylu", ",yly", ",yl;", ",yle", ",yli", ",ylo", ",ylk", ",yl,", ",yl.", ",yl/", ",yuj",
        ",yul", ",yuu", ",yuy", ",yu;", ",yun", ",yui", ",yuo", ",yuk", ",yum", ",yu.", ",yu/",
        ",yyj", ",yyl", ",yyu", ",yyy", ",yy;", ",yyn", ",yye", ",yyo", ",yyk", ",yym", ",yy,",
        ",yy/", ",y;j", ",y;l", ",y;u", ",y;y", ",y;;", ",y;n", ",y;e", ",y;i", ",y;k", ",y;m",
        ",y;,", ",y;.", ",ynj", ",ynu", ",yny", ",yn;", ",ynn", ",yne", ",yni", ",yno", ",ynk",
        ",yn,", ",yn.", ",yn/", ",yej", ",yel", ",yey", ",ye;", ",yen", ",yee", ",yei", ",yeo",
        ",yek", ",yem", ",ye.", ",ye/", ",yoj", ",yol", ",you", ",yoy", ",yon", ",yoe", ",yoi",
        ",yoo", ",yok", ",yom", ",yo,", ",yo.", ",ykl", ",yku", ",yky", ",yk;", ",ykn", ",yke",
        ",yki", ",yko", ",ykk", ",ykm", ",yk,", ",yk.", ",yk/", ",ymj", ",ymu", ",ymy", ",ym;",
        ",yme", ",ymi", ",ymo", ",ymk", ",ymm", ",ym,", ",ym.", ",ym/", ",y,j", ",y,l", ",y,y",
        ",y,;", ",y,n", ",y,i", ",y,o", ",y,k", ",y,m", ",y,,", ",y,.", ",y,/", ",y/j", ",y/l",
        ",y/u", ",y/y", ",y/n", ",y/e", ",y/i", ",y/k", ",y/m", ",y/,", ",y/.", ",y//", ",;jj",
        ",;jl", ",;ju", ",;jy", ",;j;", ",;jn", ",;je", ",;ji", ",;jo", ",;jm", ",;j,", ",;j.",
        ",;j/", ",;lj", ",;ll", ",;lu", ",;ly", ",;l;", ",;le", ",;li", ",;lo", ",;lk", ",;l,",
        ",;l.", ",;l/", ",;uj", ",;ul", ",;uu", ",;uy", ",;u;", ",;un", ",;ui", ",;uo", ",;uk",
        ",;um", ",;u.", ",;u/", ",;yj", ",;yl", ",;yu", ",;yy", ",;y;", ",;yn", ",;ye", ",;yo",
        ",;yk", ",;ym", ",;y,", ",;y/", ",;;j", ",;;l", ",;;u", ",;;y", ",;;;", ",;;n", ",;;e",
        ",;;i", ",;;k", ",;;m", ",;;,", ",;;.", ",;nj", ",;nu", ",;ny", ",;n;", ",;nn", ",;ne",
        ",;ni", ",;no", ",;nk", ",;n,", ",;n.", ",;n/", ",;ej", ",;el", ",;ey", ",;e;", ",;en",
        ",;ee", ",;ei", ",;eo", ",;ek", ",;em", ",;e.", ",;e/", ",;ij", ",;il", ",;iu", ",;i;",
        ",;in", ",;ie", ",;ii", ",;io", ",;ik", ",;im", ",;i,", ",;i/", ",;kl", ",;ku", ",;ky",
        ",;k;", ",;kn", ",;ke", ",;ki", ",;ko", ",;kk", ",;km", ",;k,", ",;k.", ",;k/", ",;mj",
        ",;mu", ",;my", ",;m;", ",;me", ",;mi", ",;mo", ",;mk", ",;mm", ",;m,", ",;m.", ",;m/",
        ",;,j", ",;,l", ",;,y", ",;,;", ",;,n", ",;,i", ",;,o", ",;,k", ",;,m", ",;,,", ",;,.",
        ",;,/", ",;.j", ",;.l", ",;.u", ",;.;", ",;.n", ",;.e", ",;.o", ",;.k", ",;.m", ",;.,",
        ",;..", ",;./", ",njj", ",njl", ",nju", ",njy", ",nj;", ",njn", ",nje", ",nji", ",njo",
        ",njm", ",nj,", ",nj.", ",nj/", ",nuj", ",nul", ",nuu", ",nuy", ",nu;", ",nun", ",nui",
        ",nuo", ",nuk", ",num", ",nu.", ",nu/", ",nyj", ",nyl", ",nyu", ",nyy", ",ny;", ",nyn",
        ",nye", ",nyo", ",nyk", ",nym", ",ny,", ",ny/", ",n;j", ",n;l", ",n;u", ",n;y", ",n;;",
        ",n;n", ",n;e", ",n;i", ",n;k", ",n;m", ",n;,", ",n;.", ",nnj", ",nnu", ",nny", ",nn;",
        ",nnn", ",nne", ",nni", ",nno", ",nnk", ",nn,", ",nn.", ",nn/", ",nej", ",nel", ",ney",
        ",ne;", ",nen", ",nee", ",nei", ",neo", ",nek", ",nem", ",ne.", ",ne/", ",nij", ",nil",
        ",niu", ",ni;", ",nin", ",nie", ",nii", ",nio", ",nik", ",nim", ",ni,", ",ni/", ",noj",
        ",nol", ",nou", ",noy", ",non", ",noe", ",noi", ",noo", ",nok", ",nom", ",no,", ",no.",
        ",nkl", ",nku", ",nky", ",nk;", ",nkn", ",nke", ",nki", ",nko", ",nkk", ",nkm", ",nk,",
        ",nk.", ",nk/", ",n,j", ",n,l", ",n,y", ",n,;", ",n,n", ",n,i", ",n,o", ",n,k", ",n,m",
        ",n,,", ",n,.", ",n,/", ",n.j", ",n.l", ",n.u", ",n.;", ",n.n", ",n.e", ",n.o", ",n.k",
        ",n.m", ",n.,", ",n..", ",n./", ",n/j", ",n/l", ",n/u", ",n/y", ",n/n", ",n/e", ",n/i",
        ",n/k", ",n/m", ",n/,", ",n/.", ",n//", ",ijj", ",ijl", ",iju", ",ijy", ",ij;", ",ijn",
        ",ije", ",iji", ",ijo", ",ijm", ",ij,", ",ij.", ",ij/", ",ilj", ",ill", ",ilu", ",ily",
        ",il;", ",ile", ",ili", ",ilo", ",ilk", ",il,", ",il.", ",il/", ",iuj", ",iul", ",iuu",
        ",iuy", ",iu;", ",iun", ",iui", ",iuo", ",iuk", ",ium", ",iu.", ",iu/", ",i;j", ",i;l",
        ",i;u", ",i;y", ",i;;", ",i;n", ",i;e", ",i;i", ",i;k", ",i;m", ",i;,", ",i;.", ",inj",
        ",inu", ",iny", ",in;", ",inn", ",ine", ",ini", ",ino", ",ink", ",in,", ",in.", ",in/",
        ",iej", ",iel", ",iey", ",ie;", ",ien", ",iee", ",iei", ",ieo", ",iek", ",iem", ",ie.",
        ",ie/", ",iij", ",iil", ",iiu", ",ii;", ",iin", ",iie", ",iii", ",iio", ",iik", ",iim",
        ",ii,", ",ii/", ",ioj", ",iol", ",iou", ",ioy", ",ion", ",ioe", ",ioi", ",ioo", ",iok",
        ",iom", ",io,", ",io.", ",ikl", ",iku", ",iky", ",ik;", ",ikn", ",ike", ",iki", ",iko",
        ",ikk", ",ikm", ",ik,", ",ik.", ",ik/", ",imj", ",imu", ",imy", ",im;", ",ime", ",imi",
        ",imo", ",imk", ",imm", ",im,", ",im.", ",im/", ",i,j", ",i,l", ",i,y", ",i,;", ",i,n",
        ",i,i", ",i,o", ",i,k", ",i,m", ",i,,", ",i,.", ",i,/", ",i/j", ",i/l", ",i/u", ",i/y",
        ",i/n", ",i/e", ",i/i", ",i/k", ",i/m", ",i/,", ",i/.", ",i//", ",ojj", ",ojl", ",oju",
        ",ojy", ",oj;", ",ojn", ",oje", ",oji", ",ojo", ",ojm", ",oj,", ",oj.", ",oj/", ",olj",
        ",oll", ",olu", ",oly", ",ol;", ",ole", ",oli", ",olo", ",olk", ",ol,", ",ol.", ",ol/",
        ",ouj", ",oul", ",ouu", ",ouy", ",ou;", ",oun", ",oui", ",ouo", ",ouk", ",oum", ",ou.",
        ",ou/", ",oyj", ",oyl", ",oyu", ",oyy", ",oy;", ",oyn", ",oye", ",oyo", ",oyk", ",oym",
        ",oy,", ",oy/", ",onj", ",onu", ",ony", ",on;", ",onn", ",one", ",oni", ",ono", ",onk",
        ",on,", ",on.", ",on/", ",oej", ",oel", ",oey", ",oe;", ",oen", ",oee", ",oei", ",oeo",
        ",oek", ",oem", ",oe.", ",oe/", ",oij", ",oil", ",oiu", ",oi;", ",oin", ",oie", ",oii",
        ",oio", ",oik", ",oim", ",oi,", ",oi/", ",ooj", ",ool", ",oou", ",ooy", ",oon", ",ooe",
        ",ooi", ",ooo", ",ook", ",oom", ",oo,", ",oo.", ",okl", ",oku", ",oky", ",ok;", ",okn",
        ",oke", ",oki", ",oko", ",okk", ",okm", ",ok,", ",ok.", ",ok/", ",omj", ",omu", ",omy",
        ",om;", ",ome", ",omi", ",omo", ",omk", ",omm", ",om,", ",om.", ",om/", ",o,j", ",o,l",
        ",o,y", ",o,;", ",o,n", ",o,i", ",o,o", ",o,k", ",o,m", ",o,,", ",o,.", ",o,/", ",o.j",
        ",o.l", ",o.u", ",o.;", ",o.n", ",o.e", ",o.o", ",o.k", ",o.m", ",o.,", ",o..", ",o./",
        ".jjj", ".jjl", ".jju", ".jjy", ".jj;", ".jjn", ".jje", ".jji", ".jjo", ".jjm", ".jj,",
        ".jj.", ".jj/", ".jlj", ".jll", ".jlu", ".jly", ".jl;", ".jle", ".jli", ".jlo", ".jlk",
        ".jl,", ".jl.", ".jl/", ".juj", ".jul", ".juu", ".juy", ".ju;", ".jun", ".jui", ".juo",
        ".juk", ".jum", ".ju.", ".ju/", ".jyj", ".jyl", ".jyu", ".jyy", ".jy;", ".jyn", ".jye",
        ".jyo", ".jyk", ".jym", ".jy,", ".jy/", ".j;j", ".j;l", ".j;u", ".j;y", ".j;;", ".j;n",
        ".j;e", ".j;i", ".j;k", ".j;m", ".j;,", ".j;.", ".jnj", ".jnu", ".jny", ".jn;", ".jnn",
        ".jne", ".jni", ".jno", ".jnk", ".jn,", ".jn.", ".jn/", ".jej", ".jel", ".jey", ".je;",
        ".jen", ".jee", ".jei", ".jeo", ".jek", ".jem", ".je.", ".je/", ".jij", ".jil", ".jiu",
        ".ji;", ".jin", ".jie", ".jii", ".jio", ".jik", ".jim", ".ji,", ".ji/", ".joj", ".jol",
        ".jou", ".joy", ".jon", ".joe", ".joi", ".joo", ".jok", ".jom", ".jo,", ".jo.", ".jmj",
        ".jmu", ".jmy", ".jm;", ".jme", ".jmi", ".jmo", ".jmk", ".jmm", ".jm,", ".jm.", ".jm/",
        ".j,j", ".j,l", ".j,y", ".j,;", ".j,n", ".j,i", ".j,o", ".j,k", ".j,m", ".j,,", ".j,.",
        ".j,/", ".j.j", ".j.l", ".j.u", ".j.;", ".j.n", ".j.e", ".j.o", ".j.k", ".j.m", ".j.,",
        ".j..", ".j./", ".j/j", ".j/l", ".j/u", ".j/y", ".j/n", ".j/e", ".j/i", ".j/k", ".j/m",
        ".j/,", ".j/.", ".j//", ".ljj", ".ljl", ".lju", ".ljy", ".lj;", ".ljn", ".lje", ".lji",
        ".ljo", ".ljm", ".lj,", ".lj.", ".lj/", ".llj", ".lll", ".llu", ".lly", ".ll;", ".lle",
        ".lli", ".llo", ".llk", ".ll,", ".ll.", ".ll/", ".luj", ".lul", ".luu", ".luy", ".lu;",
        ".lun", ".lui", ".luo", ".luk", ".lum", ".lu.", ".lu/", ".lyj", ".lyl", ".lyu", ".lyy",
        ".ly;", ".lyn", ".lye", ".lyo", ".lyk", ".lym", ".ly,", ".ly/", ".l;j", ".l;l", ".l;u",
        ".l;y", ".l;;", ".l;n", ".l;e", ".l;i", ".l;k", ".l;m", ".l;,", ".l;.", ".lej", ".lel",
        ".ley", ".le;", ".len", ".lee", ".lei", ".leo", ".lek", ".lem", ".le.", ".le/", ".lij",
        ".lil", ".liu", ".li;", ".lin", ".lie", ".lii", ".lio", ".lik", ".lim", ".li,", ".li/",
        ".loj", ".lol", ".lou", ".loy", ".lon", ".loe", ".loi", ".loo", ".lok", ".lom", ".lo,",
        ".lo.", ".lkl", ".lku", ".lky", ".lk;", ".lkn", ".lke", ".lki", ".lko", ".lkk", ".lkm",
        ".lk,", ".lk.", ".lk/", ".l,j", ".l,l", ".l,y", ".l,;", ".l,n", ".l,i", ".l,o", ".l,k",
        ".l,m", ".l,,", ".l,.", ".l,/", ".l.j", ".l.l", ".l.u", ".l.;", ".l.n", ".l.e", ".l.o",
        ".l.k", ".l.m", ".l.,", ".l..", ".l./", ".l/j", ".l/l", ".l/u", ".l/y", ".l/n", ".l/e",
        ".l/i", ".l/k", ".l/m", ".l/,", ".l/.", ".l//", ".ujj", ".ujl", ".uju", ".ujy", ".uj;",
        ".ujn", ".uje", ".uji", ".ujo", ".ujm", ".uj,", ".uj.", ".uj/", ".ulj", ".ull", ".ulu",
        ".uly", ".ul;", ".ule", ".uli", ".ulo", ".ulk", ".ul,", ".ul.", ".ul/", ".uuj", ".uul",
        ".uuu", ".uuy", ".uu;", ".uun", ".uui", ".uuo", ".uuk", ".uum", ".uu.", ".uu/", ".uyj",
        ".uyl", ".uyu", ".uyy", ".uy;", ".uyn", ".uye", ".uyo", ".uyk", ".uym", ".uy,", ".uy/",
        ".u;j", ".u;l", ".u;u", ".u;y", ".u;;", ".u;n", ".u;e", ".u;i", ".u;k", ".u;m", ".u;,",
        ".u;.", ".unj", ".unu", ".uny", ".un;", ".unn", ".une", ".uni", ".uno", ".unk", ".un,",
        ".un.", ".un/", ".uij", ".uil", ".uiu", ".ui;", ".uin", ".uie", ".uii", ".uio", ".uik",
        ".uim", ".ui,", ".ui/", ".uoj", ".uol", ".uou", ".uoy", ".uon", ".uoe", ".uoi", ".uoo",
        ".uok", ".uom", ".uo,", ".uo.", ".ukl", ".uku", ".uky", ".uk;", ".ukn", ".uke", ".uki",
        ".uko", ".ukk", ".ukm", ".uk,", ".uk.", ".uk/", ".umj", ".umu", ".umy", ".um;", ".ume",
        ".umi", ".umo", ".umk", ".umm", ".um,", ".um.", ".um/", ".u.j", ".u.l", ".u.u", ".u.;",
        ".u.n", ".u.e", ".u.o", ".u.k", ".u.m", ".u.,", ".u..", ".u./", ".u/j", ".u/l", ".u/u",
        ".u/y", ".u/n", ".u/e", ".u/i", ".u/k", ".u/m", ".u/,", ".u/.", ".u//", ".;jj", ".;jl",
        ".;ju", ".;jy", ".;j;", ".;jn", ".;je", ".;ji", ".;jo", ".;jm", ".;j,", ".;j.", ".;j/",
        ".;lj", ".;ll", ".;lu", ".;ly", ".;l;", ".;le", ".;li", ".;lo", ".;lk", ".;l,", ".;l.",
        ".;l/", ".;uj", ".;ul", ".;uu", ".;uy", ".;u;", ".;un", ".;ui", ".;uo", ".;uk", ".;um",
        ".;u.", ".;u/", ".;yj", ".;yl", ".;yu", ".;yy", ".;y;", ".;yn", ".;ye", ".;yo", ".;yk",
        ".;ym", ".;y,", ".;y/", ".;;j", ".;;l", ".;;u", ".;;y", ".;;;", ".;;n", ".;;e", ".;;i",
        ".;;k", ".;;m", ".;;,", ".;;.", ".;nj", ".;nu", ".;ny", ".;n;", ".;nn", ".;ne", ".;ni",
        ".;no", ".;nk", ".;n,", ".;n.", ".;n/", ".;ej", ".;el", ".;ey", ".;e;", ".;en", ".;ee",
        ".;ei", ".;eo", ".;ek", ".;em", ".;e.", ".;e/", ".;ij", ".;il", ".;iu", ".;i;", ".;in",
        ".;ie", ".;ii", ".;io", ".;ik", ".;im", ".;i,", ".;i/", ".;kl", ".;ku", ".;ky", ".;k;",
        ".;kn", ".;ke", ".;ki", ".;ko", ".;kk", ".;km", ".;k,", ".;k.", ".;k/", ".;mj", ".;mu",
        ".;my", ".;m;", ".;me", ".;mi", ".;mo", ".;mk", ".;mm", ".;m,", ".;m.", ".;m/", ".;,j",
        ".;,l", ".;,y", ".;,;", ".;,n", ".;,i", ".;,o", ".;,k", ".;,m", ".;,,", ".;,.", ".;,/",
        ".;.j", ".;.l", ".;.u", ".;.;", ".;.n", ".;.e", ".;.o", ".;.k", ".;.m", ".;.,", ".;..",
        ".;./", ".njj", ".njl", ".nju", ".njy", ".nj;", ".njn", ".nje", ".nji", ".njo", ".njm",
        ".nj,", ".nj.", ".nj/", ".nuj", ".nul", ".nuu", ".nuy", ".nu;", ".nun", ".nui", ".nuo",
        ".nuk", ".num", ".nu.", ".nu/", ".nyj", ".nyl", ".nyu", ".nyy", ".ny;", ".nyn", ".nye",
        ".nyo", ".nyk", ".nym", ".ny,", ".ny/", ".n;j", ".n;l", ".n;u", ".n;y", ".n;;", ".n;n",
        ".n;e", ".n;i", ".n;k", ".n;m", ".n;,", ".n;.", ".nnj", ".nnu", ".nny", ".nn;", ".nnn",
        ".nne", ".nni", ".nno", ".nnk", ".nn,", ".nn.", ".nn/", ".nej", ".nel", ".ney", ".ne;",
        ".nen", ".nee", ".nei", ".neo", ".nek", ".nem", ".ne.", ".ne/", ".nij", ".nil", ".niu",
        ".ni;", ".nin", ".nie", ".nii", ".nio", ".nik", ".nim", ".ni,", ".ni/", ".noj", ".nol",
        ".nou", ".noy", ".non", ".noe", ".noi", ".noo", ".nok", ".nom", ".no,", ".no.", ".nkl",
        ".nku", ".nky", ".nk;", ".nkn", ".nke", ".nki", ".nko", ".nkk", ".nkm", ".nk,", ".nk.",
        ".nk/", ".n,j", ".n,l", ".n,y", ".n,;", ".n,n", ".n,i", ".n,o", ".n,k", ".n,m", ".n,,",
        ".n,.", ".n,/", ".n.j", ".n.l", ".n.u", ".n.;", ".n.n", ".n.e", ".n.o", ".n.k", ".n.m",
        ".n.,", ".n..", ".n./", ".n/j", ".n/l", ".n/u", ".n/y", ".n/n", ".n/e", ".n/i", ".n/k",
        ".n/m", ".n/,", ".n/.", ".n//", ".ejj", ".ejl", ".eju", ".ejy", ".ej;", ".ejn", ".eje",
        ".eji", ".ejo", ".ejm", ".ej,", ".ej.", ".ej/", ".elj", ".ell", ".elu", ".ely", ".el;",
        ".ele", ".eli", ".elo", ".elk", ".el,", ".el.", ".el/", ".eyj", ".eyl", ".eyu", ".eyy",
        ".ey;", ".eyn", ".eye", ".eyo", ".eyk", ".eym", ".ey,", ".ey/", ".e;j", ".e;l", ".e;u",
        ".e;y", ".e;;", ".e;n", ".e;e", ".e;i", ".e;k", ".e;m", ".e;,", ".e;.", ".enj", ".enu",
        ".eny", ".en;", ".enn", ".ene", ".eni", ".eno", ".enk", ".en,", ".en.", ".en/", ".eej",
        ".eel", ".eey", ".ee;", ".een", ".eee", ".eei", ".eeo", ".eek", ".eem", ".ee.", ".ee/",
        ".eij", ".eil", ".eiu", ".ei;", ".ein", ".eie", ".eii", ".eio", ".eik", ".eim", ".ei,",
        ".ei/", ".eoj", ".eol", ".eou", ".eoy", ".eon", ".eoe", ".eoi", ".eoo", ".eok", ".eom",
        ".eo,", ".eo.", ".ekl", ".eku", ".eky", ".ek;", ".ekn", ".eke", ".eki", ".eko", ".ekk",
        ".ekm", ".ek,", ".ek.", ".ek/", ".emj", ".emu", ".emy", ".em;", ".eme", ".emi", ".emo",
        ".emk", ".emm", ".em,", ".em.", ".em/", ".e.j", ".e.l", ".e.u", ".e.;", ".e.n", ".e.e",
        ".e.o", ".e.k", ".e.m", ".e.,", ".e..", ".e./", ".e/j", ".e/l", ".e/u", ".e/y", ".e/n",
        ".e/e", ".e/i", ".e/k", ".e/m", ".e/,", ".e/.", ".e//", ".ojj", ".ojl", ".oju", ".ojy",
        ".oj;", ".ojn", ".oje", ".oji", ".ojo", ".ojm", ".oj,", ".oj.", ".oj/", ".olj", ".oll",
        ".olu", ".oly", ".ol;", ".ole", ".oli", ".olo", ".olk", ".ol,", ".ol.", ".ol/", ".ouj",
        ".oul", ".ouu", ".ouy", ".ou;", ".oun", ".oui", ".ouo", ".ouk", ".oum", ".ou.", ".ou/",
        ".oyj", ".oyl", ".oyu", ".oyy", ".oy;", ".oyn", ".oye", ".oyo", ".oyk", ".oym", ".oy,",
        ".oy/", ".onj", ".onu", ".ony", ".on;", ".onn", ".one", ".oni", ".ono", ".onk", ".on,",
        ".on.", ".on/", ".oej", ".oel", ".oey", ".oe;", ".oen", ".oee", ".oei", ".oeo", ".oek",
        ".oem", ".oe.", ".oe/", ".oij", ".oil", ".oiu", ".oi;", ".oin", ".oie", ".oii", ".oio",
        ".oik", ".oim", ".oi,", ".oi/", ".ooj", ".ool", ".oou", ".ooy", ".oon", ".ooe", ".ooi",
        ".ooo", ".ook", ".oom", ".oo,", ".oo.", ".okl", ".oku", ".oky", ".ok;", ".okn", ".oke",
        ".oki", ".oko", ".okk", ".okm", ".ok,", ".ok.", ".ok/", ".omj", ".omu", ".omy", ".om;",
        ".ome", ".omi", ".omo", ".omk", ".omm", ".om,", ".om.", ".om/", ".o,j", ".o,l", ".o,y",
        ".o,;", ".o,n", ".o,i", ".o,o", ".o,k", ".o,m", ".o,,", ".o,.", ".o,/", ".o.j", ".o.l",
        ".o.u", ".o.;", ".o.n", ".o.e", ".o.o", ".o.k", ".o.m", ".o.,", ".o..", ".o./",
    ])
}

#[with_fallible_options]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, JsonSchema, MergeFrom)]
pub struct JumpLabelSettingsContent {
    #[serde(default = "default_screen_before")]
    pub screen_before: JumpLabelSetting,
    #[serde(default = "default_screen_after")]
    pub screen_after: JumpLabelSetting,
}

impl Default for JumpLabelSettingsContent {
    fn default() -> Self {
        Self {
            screen_before: default_screen_before(),
            screen_after: default_screen_after(),
        }
    }
}

#[derive(
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ModeContent {
    #[default]
    Normal,
    Insert,
}

/// Controls when to use system clipboard.
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum UseSystemClipboard {
    /// Don't use system clipboard.
    Never,
    /// Use system clipboard.
    Always,
    /// Use system clipboard for yank operations.
    OnYank,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum VimInsertModeCursorShape {
    /// Inherit cursor shape from the editor's base cursor_shape setting.
    Inherit,
    /// Vertical bar cursor.
    Bar,
    /// Block cursor that surrounds the character.
    Block,
    /// Underline cursor.
    Underline,
    /// Hollow box cursor.
    Hollow,
}

/// The settings for cursor shape.
#[with_fallible_options]
#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom,
)]
pub struct CursorShapeSettings {
    /// Cursor shape for the normal mode.
    ///
    /// Default: block
    pub normal: Option<CursorShape>,
    /// Cursor shape for the replace mode.
    ///
    /// Default: underline
    pub replace: Option<CursorShape>,
    /// Cursor shape for the visual mode.
    ///
    /// Default: block
    pub visual: Option<CursorShape>,
    /// Cursor shape for the insert mode.
    ///
    /// The default value follows the primary cursor_shape.
    pub insert: Option<VimInsertModeCursorShape>,
}

/// Settings specific to journaling
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct JournalSettingsContent {
    /// The path of the directory where journal entries are stored.
    ///
    /// Default: `~`
    pub path: Option<String>,
    /// What format to display the hours in.
    ///
    /// Default: hour12
    pub hour_format: Option<HourFormat>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HourFormat {
    #[default]
    Hour12,
    Hour24,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct OutlinePanelSettingsContent {
    /// Whether to show the outline panel button in the status bar.
    ///
    /// Default: true
    pub button: Option<bool>,
    /// Customize default width (in pixels) taken by outline panel
    ///
    /// Default: 240
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub default_width: Option<f32>,
    /// The position of outline panel
    ///
    /// Default: left
    pub dock: Option<DockSide>,
    /// Whether to show file icons in the outline panel.
    ///
    /// Default: true
    pub file_icons: Option<bool>,
    /// Whether to show folder icons or chevrons for directories in the outline panel.
    ///
    /// Default: true
    pub folder_icons: Option<bool>,
    /// Whether to show the git status in the outline panel.
    ///
    /// Default: true
    pub git_status: Option<bool>,
    /// Amount of indentation (in pixels) for nested items.
    ///
    /// Default: 20
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub indent_size: Option<f32>,
    /// Whether to reveal it in the outline panel automatically,
    /// when a corresponding project entry becomes active.
    /// Gitignored entries are never auto revealed.
    ///
    /// Default: true
    pub auto_reveal_entries: Option<bool>,
    /// Whether to fold directories automatically
    /// when directory has only one directory inside.
    ///
    /// Default: true
    pub auto_fold_dirs: Option<bool>,
    /// Settings related to indent guides in the outline panel.
    pub indent_guides: Option<IndentGuidesSettingsContent>,
    /// Scrollbar-related settings
    pub scrollbar: Option<ScrollbarSettingsContent>,
    /// Default depth to expand outline items in the current file.
    /// The default depth to which outline entries are expanded on reveal.
    /// - Set to 0 to collapse all items that have children
    /// - Set to 1 or higher to collapse items at that depth or deeper
    ///
    /// Default: 100
    pub expand_outlines_with_depth: Option<usize>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DockSide {
    Left,
    Right,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ShowIndentGuides {
    Always,
    Never,
}

#[with_fallible_options]
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq, Default,
)]
pub struct IndentGuidesSettingsContent {
    /// When to show the scrollbar in the outline panel.
    pub show: Option<ShowIndentGuides>,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, JsonSchema, MergeFrom, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineIndicatorFormat {
    Short,
    #[default]
    Long,
}

/// The settings for the image viewer.
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, Default, PartialEq)]
pub struct ImageViewerSettingsContent {
    /// The unit to use for displaying image file sizes.
    ///
    /// Default: "binary"
    pub unit: Option<ImageFileSizeUnit>,
}

#[with_fallible_options]
#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    Default,
    PartialEq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ImageFileSizeUnit {
    /// Displays file size in binary units (e.g., KiB, MiB).
    #[default]
    Binary,
    /// Displays file size in decimal units (e.g., KB, MB).
    Decimal,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct RemoteSettingsContent {
    pub ssh_connections: Option<Vec<SshConnection>>,
    pub wsl_connections: Option<Vec<WslConnection>>,
    pub dev_container_connections: Option<Vec<DevContainerConnection>>,
    pub read_ssh_config: Option<bool>,
    pub use_podman: Option<bool>,
}

#[with_fallible_options]
#[derive(
    Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom, Hash,
)]
pub struct DevContainerConnection {
    pub name: String,
    pub remote_user: String,
    pub container_id: String,
    pub use_podman: bool,
    pub extension_ids: Vec<String>,
    pub remote_env: BTreeMap<String, String>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct SshConnection {
    pub host: String,
    pub username: Option<String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub projects: collections::BTreeSet<RemoteProject>,
    /// Name to use for this server in UI.
    pub nickname: Option<String>,
    // By default Zed will download the binary to the host directly.
    // If this is set to true, Zed will download the binary to your local machine,
    // and then upload it over the SSH connection. Useful if your SSH server has
    // limited outbound internet access.
    pub upload_binary_over_ssh: Option<bool>,

    pub port_forwards: Option<Vec<SshPortForwardOption>>,
    /// Timeout in seconds for SSH connection and downloading the remote server binary.
    /// Defaults to 10 seconds if not specified.
    pub connection_timeout: Option<u16>,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom, Debug)]
pub struct WslConnection {
    pub distro_name: String,
    pub user: Option<String>,
    #[serde(default)]
    pub projects: BTreeSet<RemoteProject>,
}

#[with_fallible_options]
#[derive(
    Clone, Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub struct RemoteProject {
    pub paths: Vec<String>,
}

#[with_fallible_options]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema, MergeFrom)]
pub struct SshPortForwardOption {
    pub local_host: Option<String>,
    pub local_port: u16,
    pub remote_host: Option<String>,
    pub remote_port: u16,
}

/// Settings for configuring REPL display and behavior.
#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ReplSettingsContent {
    /// Maximum number of lines to keep in REPL's scrollback buffer.
    /// Clamped with [4, 256] range.
    ///
    /// Default: 32
    pub max_lines: Option<usize>,
    /// Maximum number of columns to keep in REPL's scrollback buffer.
    /// Clamped with [20, 512] range.
    ///
    /// Default: 128
    pub max_columns: Option<usize>,
    /// Whether to show small single-line outputs inline instead of in a block.
    ///
    /// Default: true
    pub inline_output: Option<bool>,
    /// Maximum number of characters for an output to be shown inline.
    /// Only applies when `inline_output` is true.
    ///
    /// Default: 50
    pub inline_output_max_length: Option<usize>,
    /// Maximum number of lines of output to display before scrolling.
    /// Set to 0 to disable output height limits.
    ///
    /// Default: 0
    pub output_max_height_lines: Option<usize>,
}

/// Settings for configuring the which-key popup behaviour.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct WhichKeySettingsContent {
    /// Whether to show the which-key popup when holding down key combinations
    ///
    /// Default: false
    pub enabled: Option<bool>,
    /// Delay in milliseconds before showing the which-key popup.
    ///
    /// Default: 700
    pub delay_ms: Option<u64>,
}

// An ExtendingVec in the settings can only accumulate new values.
//
// This is useful for things like private files where you only want
// to allow new values to be added.
//
// Consider using a HashMap<String, bool> instead of this type
// (like auto_install_extensions) so that user settings files can both add
// and remove values from the set.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExtendingVec<T>(pub Vec<T>);

impl<T> Into<Vec<T>> for ExtendingVec<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}
impl<T> From<Vec<T>> for ExtendingVec<T> {
    fn from(vec: Vec<T>) -> Self {
        ExtendingVec(vec)
    }
}

impl<T: Clone> merge_from::MergeFrom for ExtendingVec<T> {
    fn merge_from(&mut self, other: &Self) {
        self.0.extend_from_slice(other.0.as_slice());
    }
}

// A SaturatingBool in the settings can only ever be set to true,
// later attempts to set it to false will be ignored.
//
// Used by `disable_ai`.
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SaturatingBool(pub bool);

impl From<bool> for SaturatingBool {
    fn from(value: bool) -> Self {
        SaturatingBool(value)
    }
}

impl From<SaturatingBool> for bool {
    fn from(value: SaturatingBool) -> bool {
        value.0
    }
}

impl merge_from::MergeFrom for SaturatingBool {
    fn merge_from(&mut self, other: &Self) {
        self.0 |= other.0
    }
}

#[derive(
    Copy,
    Clone,
    Default,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    MergeFrom,
    JsonSchema,
    derive_more::FromStr,
)]
#[serde(transparent)]
pub struct DelayMs(pub u64);

impl From<u64> for DelayMs {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl std::fmt::Display for DelayMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0)
    }
}
