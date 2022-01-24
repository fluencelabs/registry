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

use marine_sqlite_connector::{State, Value};

use crate::config::load_config;
use crate::defaults::{KEYS_TABLE_NAME, VALUES_TABLE_NAME};
use crate::error::ServiceError;
use crate::key::Key;
use crate::key_storage_impl::read_key;
use crate::record_storage_impl::{
    clear_expired_records, clear_records_for_expired_keys, get_records,
};
use crate::results::EvictStaleItem;
use crate::storage_impl::get_connection;
use crate::tetraplets_checkers::check_timestamp_tetraplets;

/// Remove expired values and expired empty keys.
/// Expired means that `timestamp_created` has surpassed `expired_timeout`.
/// Return number of keys and values removed
pub fn clear_expired_impl(current_timestamp_sec: u64) -> Result<(u64, u64), ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 0)?;
    let connection = get_connection()?;
    let config = load_config();

    let expired_host_timestamp = current_timestamp_sec - config.host_expired_timeout;
    let expired_timestamp = current_timestamp_sec - config.expired_timeout;
    let mut deleted_values = 0u64;
    deleted_values += clear_records_for_expired_keys(&connection, expired_host_timestamp)?;
    deleted_values += clear_expired_records(&connection, expired_host_timestamp)?;

    let host_id = call_parameters.host_id;
    let mut statement = connection.prepare(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE key IN (SELECT key FROM {KEYS_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_timestamp}) AND peer_id != ?"))?;
    statement.bind(1, &Value::String(host_id.clone()))?;
    statement.next()?;
    deleted_values += connection.changes() as u64;

    let mut statement = connection.prepare(f!("DELETE FROM {VALUES_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_timestamp} AND peer_id != ?"))?;
    statement.bind(1, &Value::String(host_id.clone()))?;
    statement.next()?;
    deleted_values += connection.changes() as u64;

    connection.execute(f!(
        "DELETE FROM {KEYS_TABLE_NAME} WHERE timestamp_created <= {expired_host_timestamp}"
    ))?;
    let mut deleted_keys = connection.changes() as u64;

    let mut statement = connection.prepare(f!("DELETE FROM {KEYS_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_timestamp} AND pinned=0 AND \
                                    key NOT IN (SELECT key FROM {VALUES_TABLE_NAME} WHERE peer_id = ?)"))?;
    statement.bind(1, &Value::String(host_id))?;
    statement.next()?;
    deleted_keys += connection.changes() as u64;

    Ok((deleted_keys, deleted_values))
}

/// Delete all stale keys and values except for pinned keys and host values.
/// Stale means that `timestamp_accessed` has surpassed `stale_timeout`.
/// Returns all deleted items
pub fn evict_stale_impl(current_timestamp_sec: u64) -> Result<Vec<EvictStaleItem>, ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 0)?;
    let connection = get_connection()?;
    let stale_timestamp = current_timestamp_sec - load_config().stale_timeout;

    let mut stale_keys: Vec<Key> = vec![];
    let mut statement = connection.prepare(f!(
        "SELECT key, peer_id, timestamp_created, pinned, weight FROM {KEYS_TABLE_NAME} \
                         WHERE timestamp_accessed <= {stale_timestamp}"
    ))?;

    while let State::Row = statement.next()? {
        stale_keys.push(read_key(&statement)?);
    }

    let mut results: Vec<EvictStaleItem> = vec![];
    let host_id = call_parameters.host_id;
    for key in stale_keys.into_iter() {
        let values = get_records(&connection, key.key.clone(), None)?;
        let mut statement = connection.prepare(f!(
            "DELETE FROM {VALUES_TABLE_NAME} WHERE key = ? AND set_by != ?"
        ))?;
        statement.bind(1, &Value::String(key.key.clone()))?;
        statement.bind(2, &Value::String(host_id.clone()))?;
        statement.next()?;

        if !key.pinned && !values.iter().any(|val| val.peer_id == host_id) {
            let mut statement =
                connection.prepare(f!("DELETE FROM {KEYS_TABLE_NAME} WHERE key = ?"))?;
            statement.bind(1, &Value::String(key.key.clone()))?;
            statement.next()?;
        }

        results.push(EvictStaleItem {
            key,
            records: values,
        });
    }

    Ok(results)
}
