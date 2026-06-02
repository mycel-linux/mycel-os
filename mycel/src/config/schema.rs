use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MycelConfig {
    pub system:   System,
    pub boot:     Boot,
    pub packages: Packages,
    pub overlays: Option<Overlays>,
    pub desktop:  Desktop,
    pub services: Services,
    pub users:    Vec<User>,
}

#[derive(Deserialize, Debug)]
pub struct System {
    pub hostname:  String,
    pub timezone:  String,
    pub locale:    String,
    pub kernel:    String,
    pub immutable: bool,
}

#[derive(Deserialize, Debug)]
pub struct Boot {
    pub timeout: u32,
    pub cmdline: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Packages {
    pub install: Vec<String>,
    pub lock:    Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Overlays {
    pub sources: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Desktop {
    pub environment: String,
}

#[derive(Deserialize, Debug)]
pub struct Services {
    pub enable: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub name:          String,
    pub shell:         String,
    pub groups:        Vec<String>,
    pub password_hash: String,
}
