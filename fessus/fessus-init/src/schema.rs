use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FessusConfig {
    pub fessus:        Fessus,
    pub window:        Window,
    pub bar:           Bar,
    pub launcher:      Launcher,
    pub notifications: Notifications,
    pub radial:        Radial,
    pub workspaces:    Workspaces,
    pub keybindings:   Keybindings,
    pub autostart:     Option<Autostart>,
    pub hardware:      Option<Hardware>,
}

#[derive(Deserialize, Debug)]
pub struct Fessus {
    pub compositor:    Option<String>,  // "sway" (default) or "hyprland"
    pub accent_color:  String,
    pub theme:         String,
    pub font:          String,
    pub icon_theme:    String,
    pub cursor_theme:  String,
    pub cursor_size:   u32,
    pub wallpaper:     String,
}

#[derive(Deserialize, Debug)]
pub struct Window {
    pub gaps_inner: u32,
    pub gaps_outer: u32,
    pub border:     u32,
}

#[derive(Deserialize, Debug)]
pub struct Bar {
    pub position:       String,
    pub clock_format:   String,
    pub show_battery:   bool,
    pub show_network:   bool,
    pub show_bluetooth: bool,
    pub show_tray:      bool,
}

#[derive(Deserialize, Debug)]
pub struct Launcher {
    pub provider: String,
}

#[derive(Deserialize, Debug)]
pub struct Notifications {
    pub position:    String,
    pub timeout:     u32,
    pub max_visible: u32,
}

#[derive(Deserialize, Debug)]
pub struct Radial {
    pub corner: String,
    pub pinned: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Workspaces {
    pub count: u32,
}

#[derive(Deserialize, Debug)]
pub struct Keybindings {
    #[serde(rename = "mod")]
    pub mod_key:  String,
    pub terminal: String,
}

#[derive(Deserialize, Debug)]
pub struct Autostart {
    pub apps: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Hardware {
    pub touchscreen:  bool,
    pub auto_rotate:  bool,
}
