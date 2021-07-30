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

use crate::{Config, KEYS_TABLE_NAME, VALUES_TABLE_NAME, DB_PATH, TRUSTED_TIMESTAMP_SERVICE_ID, TRUSTED_TIMESTAMP_FUNCTION_NAME, DEFAULT_EXPIRED_VALUE_AGE, DEFAULT_STALE_VALUE_AGE, DEFAULT_EXPIRED_HOST_VALUE_AGE, VALUES_LIMIT, CONFIG_FILE};
use crate::results::{Key, Record, EvictStaleItem, PutHostValueResult};
use marine_sqlite_connector::{Connection, Result as SqliteResult, Error as SqliteError, State, Statement};
use marine_rs_sdk::{CallParameters};
use eyre;
use eyre::ContextCompat;
use std::collections::HashMap;
use boolinator::Boolinator;
use toml;
use std::fs;

fn get_custom_option(value: String) -> Vec<String> {
    if value.is_empty() {
        vec![]
    } else {
        vec![value]
    }
}

fn read_key(statement: &Statement) -> SqliteResult<Key> {
    Ok(Key {
        key: statement.read::<String>(0)?,
        peer_id: statement.read::<String>(1)?,
        timestamp_created: statement.read::<i64>(2)? as u64,
        pinned: statement.read::<i64>(3)? != 0,
        weight: statement.read::<i64>(4)? as u32,
    })
}

fn read_record(statement: &Statement) -> SqliteResult<Record> {
    Ok(Record {
        value: statement.read::<String>(0)?,
        peer_id: statement.read::<String>(1)?,
        set_by: statement.read::<String>(2)?,
        relay_id: get_custom_option(statement.read::<String>(3)?),
        service_id: get_custom_option(statement.read::<String>(4)?),
        timestamp_created: statement.read::<i64>(5)? as u64,
        weight: statement.read::<i64>(6)? as u32,
    })
}

fn check_key_existence(connection: &Connection, key: String, current_timestamp_sec: u64) -> SqliteResult<()> {
    get_key_metadata_helper(&connection, key, current_timestamp_sec).map(|_| ())
}


/// Check timestamps are generated on the current host with builtin ("peer" "timestamp_sec")
pub(crate) fn check_timestamp_tetraplets(call_parameters: &CallParameters, arg_number: usize) -> eyre::Result<()> {
    let error_msg = "you should use host peer.timestamp_sec to pass timestamp";
    let tetraplets = call_parameters.tetraplets.get(arg_number).wrap_err(error_msg)?;
    let tetraplet = tetraplets.get(0).wrap_err(error_msg)?;
    (tetraplet.service_id == TRUSTED_TIMESTAMP_SERVICE_ID &&
        tetraplet.function_name == TRUSTED_TIMESTAMP_FUNCTION_NAME &&
        tetraplet.peer_pk == call_parameters.host_id
    ).then(|| ()).wrap_err(error_msg)
}

pub(crate) fn check_host_value_tetraplets(call_parameters: &CallParameters, arg_number: usize, host_value: &Record) -> eyre::Result<()> {
    let error_msg = "you should use put_host_value to pass set_host_value";
    let tetraplets = call_parameters.tetraplets.get(arg_number).wrap_err(error_msg)?;
    let tetraplet = tetraplets.get(0).wrap_err(error_msg)?;
    (tetraplet.service_id == "aqua-dht" &&
        tetraplet.function_name == "put_host_value" &&
        tetraplet.peer_pk == host_value.peer_id
    ).then(|| ()).wrap_err(error_msg)
}

#[inline]
pub(crate) fn get_connection() -> SqliteResult<Connection> {
    marine_sqlite_connector::open(DB_PATH)
}

pub(crate) fn create_keys_table() -> bool {
    let connection = get_connection().unwrap();

    connection
        .execute(f!("
            CREATE TABLE IF NOT EXISTS {KEYS_TABLE_NAME} (
                key TEXT PRIMARY KEY,
                timestamp_created INTEGER,
                timestamp_accessed INTEGER,
                peer_id TEXT,
                pinned INTEGER,
                weight INTEGER
            );
        "),
        ).is_ok()
}

pub(crate) fn create_values_table() -> bool {
    let connection = get_connection().unwrap();

    connection
        .execute(f!("
            CREATE TABLE IF NOT EXISTS {VALUES_TABLE_NAME} (
                key TEXT,
                value TEXT,
                peer_id TEXT,
                set_by TEXT,
                relay_id TEXT,
                service_id TEXT,
                timestamp_created INTEGER,
                timestamp_accessed INTEGER,
                weight INTEGER,
                PRIMARY KEY (key, peer_id, set_by)
            );
        "),
        ).is_ok()
}

pub fn write_config(config: Config) {
    fs::write(CONFIG_FILE, toml::to_string(&config).unwrap()).unwrap();
}

pub fn load_config() -> Config {
    let file_content = fs::read_to_string(CONFIG_FILE).unwrap();
    let config: Config = toml::from_str(&file_content).unwrap();
    config
}

pub(crate) fn create_config() {
    if fs::metadata(CONFIG_FILE).is_err() {
        write_config(Config {
            expired_timeout: DEFAULT_EXPIRED_VALUE_AGE,
            stale_timeout: DEFAULT_STALE_VALUE_AGE,
            host_expired_timeout: DEFAULT_EXPIRED_HOST_VALUE_AGE,
        });
    }
}

/// Update timestamp_accessed and return metadata of the key
fn get_key_metadata_helper(connection: &Connection, key: String, current_timestamp_sec: u64) -> SqliteResult<Key> {
    connection.execute(
        f!("UPDATE {KEYS_TABLE_NAME} \
                     SET timestamp_accessed = '{current_timestamp_sec}' \
                     WHERE key = '{key}'"))?;

    let mut statement = connection
        .prepare(f!("SELECT key, peer_id, timestamp_created, pinned, weight \
                              FROM {KEYS_TABLE_NAME} WHERE key = '{key}'"))?;

    if let State::Row = statement.next()? {
        read_key(&statement)
    } else {
        Err(SqliteError { code: None, message: Some("not found".to_string()) })
    }
}

/// Insert key if not exists or update timestamp if peer_id is same
fn update_key(connection: &Connection, key: String, peer_id: String, timestamp_created: u64, timestamp_accessed: u64, pin: bool, weight: u32) -> SqliteResult<()> {
    let old_key = get_key_metadata_helper(&connection, key.clone(), timestamp_accessed);
    let pinned = pin as i32;
    let update_allowed = {
        match old_key {
            Ok(key) => key.peer_id == peer_id && key.timestamp_created < timestamp_created,
            Err(_) => true,
        }
    };

    if update_allowed {
        connection.execute(f!("
             INSERT OR REPLACE INTO {KEYS_TABLE_NAME} \
             VALUES ('{key}', '{timestamp_created}', '{timestamp_accessed}', '{peer_id}', '{pinned}', '{weight}');
         "))
    } else {
        Err(SqliteError { code: None, message: Some("key already exists with different peer_id".to_string()) })
    }
}

pub fn get_key_metadata_impl(key: String, current_timestamp_sec: u64) -> SqliteResult<Key> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    get_key_metadata_helper(&get_connection()?, key, current_timestamp_sec)
}

/// register new key if not exists with caller peer_id, update if exists with same peer_id or return error
pub fn register_key_impl(key: String, current_timestamp_sec: u64, pin: bool, weight: u32) -> SqliteResult<()> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    let peer_id = call_parameters.init_peer_id.clone();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    update_key(&get_connection()?, key, peer_id, current_timestamp_sec.clone(), current_timestamp_sec, pin, weight)
}

/// Used for replication, same as register_key, but key.pinned is ignored, updates timestamp_accessed
pub fn republish_key_impl(key: Key, current_timestamp_sec: u64) -> SqliteResult<()> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    // Key.pinned is ignored in republish
    update_key(&get_connection()?, key.key, key.peer_id, key.timestamp_created, current_timestamp_sec, false, key.weight)
}

/// Put value with caller peer_id if the key exists. 
/// If the value is NOT a host value and the key already has `VALUES_LIMIT` records, then a value with the smallest weight is removed and the new value is inserted instead.
pub fn put_value_impl(key: String, value: String, current_timestamp_sec: u64, relay_id: Vec<String>, service_id: Vec<String>, weight: u32, host: bool) -> SqliteResult<Record> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 2)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec.clone())?;
    let records_count = get_non_host_records_count_by_key(&connection, key.clone())?;

    // check values limits for non-host values
    if !host && records_count >= VALUES_LIMIT {
        let min_weight_record = get_min_weight_non_host_record_by_key(&connection, key.clone())?;

        if min_weight_record.weight < weight {
            // delete the lightest record if the new one is heavier
            connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE set_by='{min_weight_record.set_by}' AND peer_id='{min_weight_record.peer_id}'"))?;
        } else {
            // return error if limit is exceeded
            return Err(SqliteError { code: None, message: Some("values limit is exceeded".to_string()) });
        }
    }

    let relay_id = if relay_id.len() == 0 { "".to_string() } else { relay_id[0].clone() };
    let peer_id = if host { call_parameters.host_id } else { call_parameters.init_peer_id.clone() };
    let set_by = call_parameters.init_peer_id;
    let service_id = if service_id.len() == 0 { "".to_string() } else { service_id[0].clone() };

    connection.execute(
        f!("INSERT OR REPLACE INTO {VALUES_TABLE_NAME} \
                    VALUES ('{key}', '{value}', '{peer_id}', '{set_by}', '{relay_id}',\
                    '{service_id}', '{current_timestamp_sec}', '{current_timestamp_sec}', '{weight}')")
    )?;

    Ok(Record {
        value,
        peer_id,
        set_by,
        relay_id: vec![relay_id],
        service_id: vec![service_id],
        timestamp_created: current_timestamp_sec,
        weight,
    })
}

/// Return all values by key
pub fn get_values_helper(connection: &Connection, key: String) -> SqliteResult<Vec<Record>> {
    let mut statement = connection.prepare(
        f!("SELECT value, peer_id, set_by, relay_id, service_id, timestamp_created, weight FROM {VALUES_TABLE_NAME} \
                     WHERE key = '{key}' ORDER BY weight DESC"))?;
    let mut result: Vec<Record> = vec![];

    while let State::Row = statement.next()? {
        result.push(read_record(&statement)?)
    }

    Ok(result)
}

fn get_non_host_records_count_by_key(connection: &Connection, key: String) -> SqliteResult<usize> {
    let host_id = marine_rs_sdk::get_call_parameters().host_id;

    // only only non-host values
    let mut statement = connection.prepare(
        f!("SELECT COUNT(*) FROM {VALUES_TABLE_NAME} \
                     WHERE key = '{key}' AND peer_id != '{host_id}'"))?;

    if let State::Row = statement.next()? {
        statement.read::<i64>(0).map(|n| n as usize)
    } else {
        Err(SqliteError { code: None, message: Some(f!("get_non_host_records_count_by_key: something went totally wrong")) })
    }
}

fn get_min_weight_non_host_record_by_key(connection: &Connection, key: String) -> SqliteResult<Record> {
    let host_id = marine_rs_sdk::get_call_parameters().host_id;

    // only only non-host values
    let mut statement = connection.prepare(
        f!("SELECT value, peer_id, set_by, relay_id, service_id, timestamp_created, weight FROM {VALUES_TABLE_NAME} \
                     WHERE key = '{key}' AND peer_id != '{host_id}' ORDER BY weight ASC LIMIT 1"))?;

    if let State::Row = statement.next()? {
        read_record(&statement)
    } else {
        Err(SqliteError { code: None, message: Some(f!("not found non-host records for given key: {key}")) })
    }
}

/// Return all values by key and update timestamp_accessed
pub fn get_values_impl(key: String, current_timestamp_sec: u64) -> SqliteResult<Vec<Record>> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    let connection = get_connection()?;

    connection.execute(
        f!("UPDATE {VALUES_TABLE_NAME} \
                     SET timestamp_accessed = '{current_timestamp_sec}' \
                     WHERE key = '{key}'"))?;

    get_values_helper(&connection, key)
}

/// If the key exists, then merge new records with existing (last-write-wins) and put
pub fn republish_values_impl(key: String, records: Vec<Record>, current_timestamp_sec: u64) -> SqliteResult<u64> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 2)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    republish_values_helper(key, records, current_timestamp_sec)
}

pub fn republish_values_helper(key: String, mut records: Vec<Record>, current_timestamp_sec: u64) -> SqliteResult<u64> {
    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec.clone())?;

    records = merge_impl(get_values_helper(&connection, key.clone())?.into_iter().chain(records.into_iter()).collect())?;

    let mut updated = 0u64;
    for record in records.iter() {
        let relay_id = if record.relay_id.is_empty() { "".to_string() } else { record.relay_id[0].clone() };
        let service_id = if record.service_id.is_empty() { "".to_string() } else { record.service_id[0].clone() };
        connection.execute(
            f!("INSERT OR REPLACE INTO {VALUES_TABLE_NAME} \
                    VALUES ('{key}', '{record.value}', '{record.peer_id}', '{record.set_by}', '{relay_id}', \
                    '{service_id}', '{record.timestamp_created}', '{current_timestamp_sec}', '{record.weight}')"))?;

        updated += connection.changes() as u64;
    }

    Ok(updated)
}

/// Remove expired values and expired empty keys. 
/// Expired means  that `timestamp_created` has surpassed `expired_timeout`.
/// Return number of keys and values removed
pub fn clear_expired_impl(current_timestamp_sec: u64) -> SqliteResult<(u64, u64)> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 0)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    let connection = get_connection()?;
    let config = load_config();

    let expired_host_timestamp = current_timestamp_sec - config.host_expired_timeout;
    let expired_timestamp = current_timestamp_sec - config.expired_timeout;
    let mut deleted_values = 0u64;
    let host_id = call_parameters.host_id;
    connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE key IN (SELECT key FROM {KEYS_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_host_timestamp})"))?;
    deleted_values += connection.changes() as u64;
    connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE timestamp_created <= {expired_host_timestamp}"))?;
    deleted_values += connection.changes() as u64;

    connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE key IN (SELECT key FROM {KEYS_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_timestamp}) AND peer_id != '{host_id}'"))?;
    deleted_values += connection.changes() as u64;
    connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_timestamp} AND peer_id != '{host_id}'"))?;
    deleted_values += connection.changes() as u64;


    connection.execute(f!("DELETE FROM {KEYS_TABLE_NAME} WHERE timestamp_created <= {expired_host_timestamp}"))?;
    let mut deleted_keys = connection.changes() as u64;
    connection.execute(f!("DELETE FROM {KEYS_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_timestamp} AND pinned=0 AND \
                                    key NOT IN (SELECT key FROM {VALUES_TABLE_NAME} WHERE peer_id='{host_id}')"))?;
    deleted_keys += connection.changes() as u64;

    Ok((deleted_keys, deleted_values))
}

/// Delete all stale keys and values except for pinned keys and host values. 
/// Stale means that `timestamp_accessed` has surpassed `stale_timeout`.
/// Returns all deleted items
pub fn evict_stale_impl(current_timestamp_sec: u64) -> SqliteResult<Vec<EvictStaleItem>> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 0)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    let connection = get_connection()?;
    let stale_timestamp = current_timestamp_sec - load_config().stale_timeout;

    let mut stale_keys: Vec<Key> = vec![];
    let mut statement =
        connection.prepare(
            f!("SELECT key, peer_id, timestamp_created, pinned, weight FROM {KEYS_TABLE_NAME} \
                         WHERE timestamp_accessed <= {stale_timestamp}"))?;

    while let State::Row = statement.next()? {
        stale_keys.push(read_key(&statement)?);
    }

    let mut results: Vec<EvictStaleItem> = vec![];
    let host_id = call_parameters.host_id;
    for key in stale_keys.into_iter() {
        let values = get_values_helper(&connection, key.key.clone())?;
        connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE key = '{key.key}' AND set_by != '{host_id}'"))?;

        if !key.pinned && !values.iter().any(|val| val.peer_id == host_id) {
            connection.execute(f!("DELETE FROM {KEYS_TABLE_NAME} WHERE key='{key.key}'"))?;
        }

        results.push(EvictStaleItem { key, records: values });
    }

    Ok(results)
}

/// Merge values with same peer_id by timestamp_created (last-write-wins)
pub fn merge_impl(records: Vec<Record>) -> SqliteResult<Vec<Record>> {
    let mut result: HashMap<String, Record> = HashMap::new();

    for rec in records.into_iter() {
        let key = rec.peer_id.clone();

        if let Some(other_rec) = result.get_mut(&key) {
            if other_rec.timestamp_created < rec.timestamp_created {
                *other_rec = rec;
            }
        } else {
            result.insert(key, rec);
        }
    }

    Ok(result.into_iter().map(|(_, rec)| rec).collect())
}

/// Update timestamp_created of host value by key and caller peer_id
pub fn renew_host_value_impl(key: String, current_timestamp_sec: u64) -> SqliteResult<()> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec.clone())?;

    let set_by = call_parameters.init_peer_id;
    let host_id = call_parameters.host_id;

    connection.execute(
        f!("UPDATE {VALUES_TABLE_NAME} \
                     SET timestamp_created = '{current_timestamp_sec}', timestamp_accessed = '{current_timestamp_sec}' \
                     WHERE key = '{key}' AND set_by = '{set_by}' AND peer_id = '{host_id}'"))?;

    (connection.changes() == 1).as_result((), SqliteError { code: None, message: Some("host value not found".to_string()) })
}

/// Remove host value by key and caller peer_id
pub fn clear_host_value_impl(key: String, current_timestamp_sec: u64) -> SqliteResult<()> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec.clone())?;

    let host_id = call_parameters.host_id;
    let set_by = call_parameters.init_peer_id;

    connection.execute(
        f!("DELETE FROM {VALUES_TABLE_NAME} \
                     WHERE key = '{key}' AND set_by = '{set_by}' AND peer_id = '{host_id}'"))?;

    (connection.changes() == 1).as_result((), SqliteError { code: None, message: Some("host value not found".to_string()) })
}

/// Used for replication of host values to other nodes.
/// Similar to republish_values but with an additional check that value.set_by == init_peer_id
pub fn propagate_host_value_impl(mut set_host_value: PutHostValueResult, current_timestamp_sec: u64, weight: u32) -> SqliteResult<()> {
    if !set_host_value.success || set_host_value.value.len() != 1 {
        return Err(SqliteError { code: None, message: Some("invalid set_host_value".to_string()) });
    }

    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_host_value_tetraplets(&call_parameters, 0, &set_host_value.value[0])
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    if set_host_value.value[0].set_by != call_parameters.init_peer_id {
        return Err(SqliteError { code: None, message: Some("value is set by another peer".to_string()) });
    }

    set_host_value.value[0].weight = weight;
    republish_values_helper(set_host_value.key, set_host_value.value, current_timestamp_sec).map(|_| ())
}