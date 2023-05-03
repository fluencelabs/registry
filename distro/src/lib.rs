pub const TRUST_GRAPH_WASM: &'static [u8] = include_bytes!("../registry-service/registry.wasm");
pub const SQLITE_WASM: &'static [u8] = include_bytes!("../registry-service/sqlite3.wasm");
pub const CONFIG: &'static [u8] = include_bytes!("../registry-service/Config.toml");
pub const SCHEDULED_SCRIPT_1: &'static str = include_str!("../registry-service/air/registry-scheduled-scripts.clearExpired_86400.air");
pub const SCHEDULED_SCRIPT_1_PERIOD: u32 = 86400;

pub const SCHEDULED_SCRIPT_2: &'static str = include_str!("../registry-service/air/registry-scheduled-scripts.renew_43200.air");
pub const SCHEDULED_SCRIPT_2_PERIOD: u32 = 43200;

pub const SCHEDULED_SCRIPT_3: &'static str = include_str!("../registry-service/air/registry-scheduled-scripts.replicate_3600.air");
pub const SCHEDULED_SCRIPT_3_PERIOD: u32 = 3600;

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

pub fn scripts() -> Vec<(&'static str, u32)> {
    vec![
        (SCHEDULED_SCRIPT_1, SCHEDULED_SCRIPT_1_PERIOD),
        (SCHEDULED_SCRIPT_2, SCHEDULED_SCRIPT_2_PERIOD),
        (SCHEDULED_SCRIPT_3, SCHEDULED_SCRIPT_3_PERIOD)
    ]
}
