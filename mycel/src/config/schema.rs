use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MycelConfig {
    pub system:   System,
    pub boot:     Boot,
    pub packages: Packages,
    pub overlays: Option<Overlays>,
    pub desktop:  Desktop,
    pub services: Services,
    pub users:    Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct System {
    pub hostname:  String,
    pub timezone:  String,
    pub locale:    String,
    pub kernel:    String,
    pub immutable: bool,
    pub channel:          Option<String>,
    pub keep_generations: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Boot {
    pub timeout: u32,
    pub cmdline: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packages {
    pub install: Vec<String>,
    pub lock:    Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Overlays {
    pub sources: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Desktop {
    pub environment: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Services {
    pub enable: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name:          String,
    pub shell:         String,
    pub groups:        Vec<String>,
    pub password_hash: String,
}
