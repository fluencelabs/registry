pub const TRUST_GRAPH_WASM: &'static [u8] = include_bytes!("../registry-service/registry.wasm");
pub const SQLITE_WASM: &'static [u8] = include_bytes!("../registry-service/sqlite3.wasm");
pub const CONFIG: &'static [u8] = include_bytes!("../registry-service/Config.toml");
pub const CLEAR_EXPIRED_AIR: &'static str =
    include_str!("../registry-service/air/registry-scheduled-scripts.clearExpired_86400.air");
pub const CLEAR_EXPIRED_PERIOD_SEC: u32 = 86400;

pub const RENEW_AIR: &'static str =
    include_str!("../registry-service/air/registry-scheduled-scripts.renew_43200.air");
pub const RENEW_PERIOD_SEC: u32 = 43200;

pub const REPLICATE_AIR: &'static str =
    include_str!("../registry-service/air/registry-scheduled-scripts.replicate_3600.air");
pub const REPLICATE_PERIOD_SEC: u32 = 3600;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub use build_info::PKG_VERSION as VERSION;

pub fn modules() -> std::collections::HashMap<&'static str, &'static [u8]> {
    maplit::hashmap! {
        "sqlite3" => SQLITE_WASM,
        "registry" => TRUST_GRAPH_WASM,
    }
}

pub struct Script {
    pub name: &'static str,
    pub air: &'static str,
    pub period_sec: u32,
}

pub fn scripts() -> Vec<Script> {
    vec![
        Script {
            name: "registry-clear-expired",
            air: CLEAR_EXPIRED_AIR,
            period_sec: CLEAR_EXPIRED_PERIOD_SEC,
        },
        Script {
            name: "registry-renew",
            air: RENEW_AIR,
            period_sec: RENEW_PERIOD_SEC,
        },
        Script {
            name: "registry-replicate",
            air: REPLICATE_AIR,
            period_sec: REPLICATE_PERIOD_SEC,
        },
    ]
}
