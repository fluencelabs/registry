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

use crate::results::{Key, GetKeyMetadataResult, DhtResult, GetValuesResult, Record, RepublishValuesResult, ClearExpiredResult, EvictStaleResult, MergeResult, PutHostValueResult};
use crate::impls::{create_keys_table, create_values_table, register_key_impl, get_key_metadata_impl, republish_key_impl, put_value_impl, get_values_impl, republish_values_impl, clear_expired_impl, evict_stale_impl, merge_impl, renew_host_value_impl, clear_host_value_impl, create_config, load_config, write_config, propagate_host_value_impl};

use fluence::marine;
use fluence::module_manifest;

use serde::{Deserialize, Serialize};

#[macro_use]
extern crate fstrings;

module_manifest!();

pub static KEYS_TABLE_NAME: &str = "dht_keys";
pub static VALUES_TABLE_NAME: &str = "dht_values";
pub static CONFIG_FILE: &str = "/tmp/Config.toml";
pub static DB_PATH: &str = "/tmp/dht.db";
pub static DEFAULT_STALE_VALUE_AGE: u64 = 60 * 60;
pub static DEFAULT_EXPIRED_VALUE_AGE: u64 = 24 * 60 * 60;
pub static DEFAULT_EXPIRED_HOST_VALUE_AGE: u64 = 10 * DEFAULT_EXPIRED_VALUE_AGE;
pub static VALUES_LIMIT: usize = 20;

pub static TRUSTED_TIMESTAMP_SERVICE_ID: &str = "peer";
pub static TRUSTED_TIMESTAMP_FUNCTION_NAME: &str = "timestamp_sec";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub expired_timeout: u64,
    pub stale_timeout: u64,
    pub host_expired_timeout: u64,
}

fn main() {
    create_keys_table();
    create_values_table();
    create_config();
}

// KEYS
#[marine]
pub fn register_key(key: String, current_timestamp_sec: u64, pin: bool, weight: u32) -> DhtResult {
    register_key_impl(key, current_timestamp_sec, pin, weight).into()
}

#[marine]
pub fn get_key_metadata(key: String, current_timestamp_sec: u64) -> GetKeyMetadataResult {
    get_key_metadata_impl(key, current_timestamp_sec).into()
}

#[marine]
pub fn republish_key(key: Key, current_timestamp_sec: u64) -> DhtResult {
    republish_key_impl(key, current_timestamp_sec).into()
}

// VALUES
#[marine]
pub fn put_value(key: String, value: String, current_timestamp_sec: u64, relay_id: Vec<String>, service_id: Vec<String>, weight: u32) -> DhtResult {
    put_value_impl(key, value, current_timestamp_sec, relay_id, service_id, weight, false).map(|_| ()).into()
}

#[marine]
pub fn put_host_value(key: String, value: String, current_timestamp_sec: u64, relay_id: Vec<String>, service_id: Vec<String>, weight: u32) -> PutHostValueResult {
    let mut result: PutHostValueResult = put_value_impl(key.clone(), value, current_timestamp_sec, relay_id, service_id, weight, true).into();
    result.key = key;

    result
}

#[marine]
pub fn propagate_host_value(set_host_value: PutHostValueResult, current_timestamp_sec: u64, weight: u32) -> DhtResult {
    propagate_host_value_impl(set_host_value, current_timestamp_sec, weight).into()
}

#[marine]
pub fn get_values(key: String, current_timestamp_sec: u64) -> GetValuesResult {
    get_values_impl(key, current_timestamp_sec).into()
}

#[marine]
pub fn republish_values(key: String, records: Vec<Record>, current_timestamp_sec: u64) -> RepublishValuesResult {
    republish_values_impl(key, records, current_timestamp_sec).into()
}

#[marine]
pub fn renew_host_value(key: String, current_timestamp_sec: u64) -> DhtResult {
    renew_host_value_impl(key, current_timestamp_sec).into()
}

#[marine]
pub fn clear_host_value(key: String, current_timestamp_sec: u64) -> DhtResult {
    clear_host_value_impl(key, current_timestamp_sec).into()
}

// BOTH
#[marine]
pub fn clear_expired(current_timestamp_sec: u64) -> ClearExpiredResult {
    clear_expired_impl(current_timestamp_sec).into()
}

#[marine]
pub fn evict_stale(current_timestamp_sec: u64) -> EvictStaleResult {
    evict_stale_impl(current_timestamp_sec).into()
}

#[marine]
pub fn merge(records: Vec<Vec<Record>>) -> MergeResult {
    merge_impl(records.into_iter().flatten().collect()).into()
}

#[marine]
pub fn merge_two(a: Vec<Record>, b: Vec<Record>) -> MergeResult {
    merge_impl(a.into_iter().chain(b.into_iter()).collect()).into()
}

#[marine]
pub fn merge_hack_get_values(records: Vec<GetValuesResult>) -> MergeResult {
    merge_impl(
        records
            .into_iter()
            .filter(|elem| elem.success)
            .map(|elem| elem.result)
            .flatten()
            .collect()
    ).into()
}

#[marine]
pub fn set_expired_timeout(timeout_sec: u64) {
    let mut config = load_config();
    config.expired_timeout = timeout_sec;
    write_config(config);
}

#[marine]
pub fn set_host_expired_timeout(timeout_sec: u64) {
    let mut config = load_config();
    config.host_expired_timeout = timeout_sec;
    write_config(config);
}

#[marine]
pub fn set_stale_timeout(timeout_sec: u64) {
    let mut config = load_config();
    config.stale_timeout = timeout_sec;
    write_config(config);
}
