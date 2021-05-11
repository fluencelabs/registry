#![feature(try_trait)]
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

mod results;
mod tests;
mod impls;

use crate::results::{Key, GetKeyMetadataResult, RegisterKeyResult, RepublishKeyResult};
use crate::impls::{create_keys_table, create_values_table, register_key_impl, get_key_metadata_impl, republish_key_impl};

use fluence::marine;
use fluence::module_manifest;

#[macro_use]
extern crate fstrings;

module_manifest!();

pub static KEYS_TABLE_NAME: &str = "dht_keys";
pub static VALUES_TABLE_NAME: &str = "dht_values";
pub static DB_PATH: &str = "/tmp/dht.db";
pub static STALE_VALUE_AGE: u64 = 60 * 60 * 1000;
pub static EXPIRED_VALUE_AGE: u64 = 24 * 60 * 60 * 1000;

pub static TRUSTED_TIMESTAMP_SERVICE_ID: &str = "peer";
pub static TRUSTED_TIMESTAMP_FUNCTION_NAME: &str = "timestamp_ms";

fn main() {
    create_keys_table();
    create_values_table();
}

// KEYS
#[marine]
pub fn register_key(key: String, current_timestamp: u64) -> RegisterKeyResult {
    register_key_impl(key, current_timestamp).into()
}

#[marine]
pub fn get_key_metadata(key: String) -> GetKeyMetadataResult {
    get_key_metadata_impl(key).into()
}

#[marine]
pub fn republish_key(key: Key, current_timestamp: u64) -> RepublishKeyResult {
    republish_key_impl(key, current_timestamp).into()
}

// VALUES
// #[fce]
// pub fn put_value(key: String, value: String, current_timestamp: u64, relay_id: Vec<String>) -> PutValueResult {
//     put_value_impl(key, value, current_timestamp, relay_id).into()
// }
//
// #[fce]
// pub fn get_values(key: String, current_timestamp: u64) -> GetValuesResult {
//     get_values_impl(key, current_timestamp).into()
// }

// #[fce]
// pub fn republish_values(key: String, records: Vec<Record>, current_timestamp: u64) -> RepublishValuesResult  {
//     republish_values_impl(key, records, current_timestamp).into()
// }

// BOTH
// #[fce]
// pub fn clear_expired(current_timestamp: u64) -> ClearExpiredResult {
//     clear_expired_impl(current_timestamp).into()
// }
//
// #[fce]
// pub fn evict_stale(current_timestamp: u64) -> GetStaleResult {
//     evict_stale_impl(current_timestamp).into()
// }
