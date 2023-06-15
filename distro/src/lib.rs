use maplit::hashmap;
use std::collections::HashMap;
use serde_json::{json, Value as JValue};

pub const TRUST_GRAPH_WASM: &'static [u8] = include_bytes!("../registry-service/registry.wasm");
pub const SQLITE_WASM: &'static [u8] = include_bytes!("../registry-service/sqlite3.wasm");
pub const CONFIG: &'static [u8] = include_bytes!("../registry-service/Config.toml");

pub const REGISTRY_SPELL: &'static str =
    include_str!("../registry-service/air/spell.spell.air");

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

pub struct DistrSpell {
    /// AIR script of the spell
    pub air: &'static str,
    /// Initial key-value records for spells KV storage
    pub init_data: HashMap<&'static str, JValue>,
}


/// Decider's configuration needed for the correct decider start-up
#[derive(Debug)]
pub struct RegistryConfig {
    pub expired_interval: u32,
    pub renew_interval: u32,
    pub replicate_interval: u32
}

pub fn registry_spell(config: RegistryConfig) -> DistrSpell {
    DistrSpell {
        air: REGISTRY_SPELL,
        init_data: hashmap!{
            "config" => json!( {
                "expired_interval": config.expired_interval,
                "renew_interval": config.renew_interval,
                "replicate_interval": config.replicate_interval,
            }),
        },
    }
}
