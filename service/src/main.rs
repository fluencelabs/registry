/*
 * Copyright 2021 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;

use crate::config::{create_config, load_config, write_config};
use crate::results::{ClearExpiredResult, EvictStaleResult};
use crate::storage_impl::get_storage;
use crate::tetraplets_checkers::check_timestamp_tetraplets;

mod config;
mod defaults;
mod error;
mod key;
mod key_api;
mod key_storage_impl;
mod misc;
mod record;
mod record_api;
mod record_storage_impl;
mod results;
mod storage_impl;
mod tests;
mod tetraplets_checkers;

#[macro_use]
extern crate fstrings;

/*
   _initialize function that calls __wasm_call_ctors is required to mitigade memory leak
   that is described in https://github.com/WebAssembly/wasi-libc/issues/298

   In short, without this code rust wraps every export function
   with __wasm_call_ctors/__wasm_call_dtors calls. This causes memory leaks. When compiler sees
   an explicit call to __wasm_call_ctors in _initialize function, it disables export wrapping.

   TODO: remove when updating to marine-rs-sdk with fix
*/
extern "C" {
    pub fn __wasm_call_ctors();
}

#[no_mangle]
fn _initialize() {
    unsafe {
        __wasm_call_ctors();
    }
}
//------------------------------

module_manifest!();

pub fn wrapped_try<F, T>(func: F) -> T
where
    F: FnOnce() -> T,
{
    func()
}

// TODO: ship tg results as crate, remove duplication
#[marine]
pub struct WeightResult {
    pub success: bool,
    pub weight: u32,
    pub peer_id: String,
    pub error: String,
}

fn main() {
    _initialize(); // As __wasm_call_ctors still does necessary work, we call it at the start of the module
    let storage = get_storage().unwrap();
    storage.create_key_tables();
    storage.create_values_table();
    create_config();
}

#[marine]
pub fn clear_expired(current_timestamp_sec: u64) -> ClearExpiredResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 0)?;
        get_storage()?.clear_expired(current_timestamp_sec)
    })
    .into()
}

#[marine]
pub fn evict_stale(current_timestamp_sec: u64) -> EvictStaleResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 0)?;
        get_storage()?.evict_stale(current_timestamp_sec)
    })
    .into()
}

#[marine]
pub fn set_expired_timeout(timeout_sec: u64) {
    let mut config = load_config();
    config.expired_timeout = timeout_sec;
    write_config(config);
}

#[marine]
pub fn set_stale_timeout(timeout_sec: u64) {
    let mut config = load_config();
    config.stale_timeout = timeout_sec;
    write_config(config);
}
