use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;

module_manifest!();

pub fn main() {}

#[marine]
pub fn echo(msg: String) -> String {
    format!("{}: {}", marine_rs_sdk::get_call_parameters().host_id, msg)
}