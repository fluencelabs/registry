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

use std::collections::HashMap;

use boolinator::Boolinator;
use marine_rs_sdk::CallParameters;
use marine_sqlite_connector::{Connection, Result as SqliteResult, State, Statement, Value};

use crate::config::load_config;
use crate::defaults::{
    DB_PATH, KEYS_TABLE_NAME, TRUSTED_TIMESTAMP_FUNCTION_NAME, TRUSTED_TIMESTAMP_SERVICE_ID,
    VALUES_LIMIT, VALUES_TABLE_NAME,
};
use crate::error::ServiceError;
use crate::error::ServiceError::{
    HostValueNotFound, InternalError, InvalidSetHostValueResult, InvalidSetHostValueTetraplet,
    InvalidTimestampTetraplet, InvalidWeightTetraplet, KeyAlreadyExists, KeyNotExists,
    ValuesLimitExceeded,
};
use crate::results::{EvictStaleItem, Key, PutHostValueResult, Record};
use crate::WeightResult;

fn get_custom_option(value: String) -> Vec<String> {
    if value.is_empty() {
        vec![]
    } else {
        vec![value]
    }
}

fn read_key(statement: &Statement) -> Result<Key, ServiceError> {
    Ok(Key {
        key: statement.read::<String>(0)?,
        peer_id: statement.read::<String>(1)?,
        timestamp_created: statement.read::<i64>(2)? as u64,
        pinned: statement.read::<i64>(3)? != 0,
        weight: statement.read::<i64>(4)? as u32,
    })
}

fn read_record(statement: &Statement) -> Result<Record, ServiceError> {
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

fn check_key_existence(
    connection: &Connection,
    key: String,
    current_timestamp_sec: u64,
) -> Result<(), ServiceError> {
    get_key_metadata_helper(connection, key, current_timestamp_sec).map(|_| ())
}

fn insert_or_replace_value(
    connection: &Connection,
    key: String,
    record: Record,
    current_timestamp: u64,
) -> Result<(), ServiceError> {
    let relay_id = if record.relay_id.is_empty() {
        "".to_string()
    } else {
        record.relay_id[0].clone()
    };
    let service_id = if record.service_id.is_empty() {
        "".to_string()
    } else {
        record.service_id[0].clone()
    };
    let mut statement = connection.prepare(f!(
        "INSERT OR REPLACE INTO {VALUES_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))?;

    statement.bind(1, &Value::String(key))?;
    statement.bind(2, &Value::String(record.value))?;
    statement.bind(3, &Value::String(record.peer_id))?;
    statement.bind(4, &Value::String(record.set_by))?;
    statement.bind(5, &Value::String(relay_id))?;
    statement.bind(6, &Value::String(service_id))?;
    statement.bind(7, &Value::Integer(record.timestamp_created as i64))?;
    statement.bind(8, &Value::Integer(current_timestamp as i64))?;
    statement.bind(9, &Value::Integer(record.weight as i64))?;
    statement.next().map(drop)?;

    Ok(())
}

fn delete_value(
    connection: &Connection,
    key: &str,
    peer_id: String,
    set_by: String,
) -> Result<(), ServiceError> {
    let mut statement = connection.prepare(f!(
        "DELETE FROM {VALUES_TABLE_NAME} WHERE key=? AND peer_id=? AND set_by=?"
    ))?;
    statement.bind(1, &Value::String(key.to_string()))?;
    statement.bind(2, &Value::String(peer_id))?;
    statement.bind(3, &Value::String(set_by))?;
    statement.next().map(drop)?;

    Ok(())
}

/// Check timestamps are generated on the current host with builtin ("peer" "timestamp_sec")
pub(crate) fn check_timestamp_tetraplets(
    call_parameters: &CallParameters,
    arg_number: usize,
) -> Result<(), ServiceError> {
    let tetraplets = call_parameters
        .tetraplets
        .get(arg_number)
        .ok_or_else(|| InvalidTimestampTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    let tetraplet = tetraplets
        .get(0)
        .ok_or_else(|| InvalidTimestampTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    (tetraplet.service_id == TRUSTED_TIMESTAMP_SERVICE_ID
        && tetraplet.function_name == TRUSTED_TIMESTAMP_FUNCTION_NAME
        && tetraplet.peer_pk == call_parameters.host_id)
        .then(|| ())
        .ok_or_else(|| InvalidTimestampTetraplet(format!("{:?}", tetraplet)))
}

pub(crate) fn check_host_value_tetraplets(
    call_parameters: &CallParameters,
    arg_number: usize,
    host_value: &Record,
) -> Result<(), ServiceError> {
    let tetraplets = call_parameters
        .tetraplets
        .get(arg_number)
        .ok_or_else(|| InvalidSetHostValueTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    let tetraplet = tetraplets
        .get(0)
        .ok_or_else(|| InvalidSetHostValueTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    (tetraplet.service_id == "aqua-dht"
        && tetraplet.function_name == "put_host_value"
        && tetraplet.peer_pk == host_value.peer_id)
        .then(|| ())
        .ok_or_else(|| InvalidSetHostValueTetraplet(format!("{:?}", tetraplet)))
}

pub(crate) fn check_weight_tetraplets(
    call_parameters: &CallParameters,
    arg_number: usize,
    weight: &WeightResult,
) -> Result<(), ServiceError> {
    let tetraplets = call_parameters
        .tetraplets
        .get(arg_number)
        .ok_or_else(|| InvalidWeightTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    let tetraplet = tetraplets
        .get(0)
        .ok_or_else(|| InvalidWeightTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    (tetraplet.service_id == "trust-graph"
        && tetraplet.function_name == "get_weight"
        && tetraplet.peer_pk == weight.peer_id)
        .then(|| ())
        .ok_or_else(|| InvalidWeightTetraplet(format!("{:?}", tetraplet)))
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
        "))
        .is_ok()
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
        "))
        .is_ok()
}

/// Update timestamp_accessed and return metadata of the key
fn get_key_metadata_helper(
    connection: &Connection,
    key: String,
    current_timestamp_sec: u64,
) -> Result<Key, ServiceError> {
    let mut statement = connection.prepare(f!(
        "UPDATE {KEYS_TABLE_NAME} SET timestamp_accessed = ? WHERE key = ?"
    ))?;
    statement.bind(1, &Value::Integer(current_timestamp_sec as i64))?;
    statement.bind(2, &Value::String(key.clone()))?;
    statement.next()?;

    let mut statement = connection.prepare(f!(
        "SELECT key, peer_id, timestamp_created, pinned, weight \
                              FROM {KEYS_TABLE_NAME} WHERE key = ?"
    ))?;
    statement.bind(1, &Value::String(key.clone()))?;

    if let State::Row = statement.next()? {
        read_key(&statement)
    } else {
        Err(KeyNotExists(key))
    }
}

/// Insert key if not exists or update timestamp if peer_id is same
fn update_key(
    connection: &Connection,
    key: String,
    peer_id: String,
    timestamp_created: u64,
    timestamp_accessed: u64,
    pin: bool,
    weight: u32,
) -> Result<(), ServiceError> {
    let old_key = get_key_metadata_helper(connection, key.clone(), timestamp_accessed);
    let pinned = pin as i32;
    let update_allowed = {
        match old_key {
            Ok(key) => key.peer_id == peer_id && key.timestamp_created < timestamp_created,
            Err(_) => true,
        }
    };

    if update_allowed {
        let mut statement = connection.prepare(f!("
             INSERT OR REPLACE INTO {KEYS_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?);
         "))?;

        statement.bind(1, &Value::String(key))?;
        statement.bind(2, &Value::Integer(timestamp_created as i64))?;
        statement.bind(3, &Value::Integer(timestamp_accessed as i64))?;
        statement.bind(4, &Value::String(peer_id))?;
        statement.bind(5, &Value::Integer(pinned as i64))?;
        statement.bind(6, &Value::Integer(weight as i64))?;
        statement.next()?;
        Ok(())
    } else {
        Err(KeyAlreadyExists(key))
    }
}

pub fn get_key_metadata_impl(key: String, current_timestamp_sec: u64) -> Result<Key, ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)?;

    get_key_metadata_helper(&get_connection()?, key, current_timestamp_sec)
}

/// register new key if not exists with caller peer_id, update if exists with same peer_id or return error
pub fn register_key_impl(
    key: String,
    current_timestamp_sec: u64,
    pin: bool,
    weight: WeightResult,
) -> Result<(), ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    let peer_id = call_parameters.init_peer_id.clone();
    check_timestamp_tetraplets(&call_parameters, 1)?;
    check_weight_tetraplets(&call_parameters, 3, &weight)?;

    update_key(
        &get_connection()?,
        key,
        peer_id,
        current_timestamp_sec,
        current_timestamp_sec,
        pin,
        weight.weight,
    )
}

/// Used for replication, same as register_key, but key.pinned is ignored, updates timestamp_accessed
pub fn republish_key_impl(key: Key, current_timestamp_sec: u64) -> Result<(), ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)?;

    // Key.pinned is ignored in republish
    update_key(
        &get_connection()?,
        key.key,
        key.peer_id,
        key.timestamp_created,
        current_timestamp_sec,
        false,
        key.weight,
    )
}

/// Put value with caller peer_id if the key exists.
/// If the value is NOT a host value and the key already has `VALUES_LIMIT` records, then a value with the smallest weight is removed and the new value is inserted instead.
pub fn put_value_impl(
    key: String,
    value: String,
    current_timestamp_sec: u64,
    relay_id: Vec<String>,
    service_id: Vec<String>,
    weight: WeightResult,
    host: bool,
) -> Result<Record, ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 2)?;
    check_weight_tetraplets(&call_parameters, 5, &weight)?;
    let weight = weight.weight;

    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec)?;
    let records_count = get_non_host_records_count_by_key(&connection, key.clone())?;

    // check values limits for non-host values
    if !host && records_count >= VALUES_LIMIT {
        let min_weight_record = get_min_weight_non_host_record_by_key(&connection, key.clone())?;

        if min_weight_record.weight < weight {
            // delete the lightest record if the new one is heavier
            delete_value(
                &connection,
                &key,
                min_weight_record.peer_id,
                min_weight_record.set_by,
            )?;
        } else {
            // return error if limit is exceeded
            return Err(ValuesLimitExceeded(key));
        }
    }

    let result = Record {
        value,
        peer_id: if host {
            call_parameters.host_id
        } else {
            call_parameters.init_peer_id.clone()
        },
        set_by: call_parameters.init_peer_id,
        relay_id,
        service_id,
        timestamp_created: current_timestamp_sec,
        weight,
    };

    insert_or_replace_value(&connection, key, result.clone(), current_timestamp_sec)?;
    Ok(result)
}

/// Return all values by key
pub fn get_values_helper(
    connection: &Connection,
    key: String,
) -> Result<Vec<Record>, ServiceError> {
    let mut statement = connection.prepare(
        f!("SELECT value, peer_id, set_by, relay_id, service_id, timestamp_created, weight FROM {VALUES_TABLE_NAME} \
                     WHERE key = ? ORDER BY weight DESC"))?;
    statement.bind(1, &Value::String(key))?;

    let mut result: Vec<Record> = vec![];

    while let State::Row = statement.next()? {
        result.push(read_record(&statement)?)
    }

    Ok(result)
}

fn get_non_host_records_count_by_key(
    connection: &Connection,
    key: String,
) -> Result<usize, ServiceError> {
    let host_id = marine_rs_sdk::get_call_parameters().host_id;

    // only only non-host values
    let mut statement = connection.prepare(f!(
        "SELECT COUNT(*) FROM {VALUES_TABLE_NAME} WHERE key = ? AND peer_id != ?"
    ))?;
    statement.bind(1, &Value::String(key))?;
    statement.bind(2, &Value::String(host_id))?;

    if let State::Row = statement.next()? {
        statement
            .read::<i64>(0)
            .map(|n| n as usize)
            .map_err(ServiceError::SqliteError)
    } else {
        Err(InternalError(f!(
            "get_non_host_records_count_by_key: something went totally wrong"
        )))
    }
}

fn get_min_weight_non_host_record_by_key(
    connection: &Connection,
    key: String,
) -> Result<Record, ServiceError> {
    let host_id = marine_rs_sdk::get_call_parameters().host_id;

    // only only non-host values
    let mut statement = connection.prepare(
        f!("SELECT value, peer_id, set_by, relay_id, service_id, timestamp_created, weight FROM {VALUES_TABLE_NAME} \
                     WHERE key = ? AND peer_id != ? ORDER BY weight ASC LIMIT 1"))?;

    statement.bind(1, &Value::String(key.clone()))?;
    statement.bind(2, &Value::String(host_id))?;

    if let State::Row = statement.next()? {
        read_record(&statement)
    } else {
        Err(InternalError(f!(
            "not found non-host records for given key: {key}"
        )))
    }
}

/// Return all values by key and update timestamp_accessed
pub fn get_values_impl(
    key: String,
    current_timestamp_sec: u64,
) -> Result<Vec<Record>, ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)?;

    let connection = get_connection()?;
    check_key_existence(&connection, key.clone(), current_timestamp_sec)?;

    let mut statement = connection.prepare(f!("UPDATE {VALUES_TABLE_NAME} \
                  SET timestamp_accessed = ? \
                  WHERE key = ?"))?;

    statement.bind(1, &Value::Integer(current_timestamp_sec as i64))?;
    statement.bind(2, &Value::String(key.clone()))?;
    statement.next()?;

    get_values_helper(&connection, key)
}

/// If the key exists, then merge new records with existing (last-write-wins) and put
pub fn republish_values_impl(
    key: String,
    records: Vec<Record>,
    current_timestamp_sec: u64,
) -> Result<u64, ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 2)?;

    republish_values_helper(key, records, current_timestamp_sec)
}

pub fn republish_values_helper(
    key: String,
    mut records: Vec<Record>,
    current_timestamp_sec: u64,
) -> Result<u64, ServiceError> {
    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec)?;

    records = merge_impl(
        get_values_helper(&connection, key.clone())?
            .into_iter()
            .chain(records.into_iter())
            .collect(),
    )?;

    let mut updated = 0u64;
    for record in records.into_iter() {
        insert_or_replace_value(&connection, key.clone(), record, current_timestamp_sec)?;
        updated += connection.changes() as u64;
    }

    Ok(updated)
}

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
    let host_id = call_parameters.host_id;
    connection.execute(f!(
        "DELETE FROM {VALUES_TABLE_NAME} WHERE key IN (SELECT key FROM {KEYS_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_host_timestamp})"
    ))?;
    deleted_values += connection.changes() as u64;
    connection.execute(f!(
        "DELETE FROM {VALUES_TABLE_NAME} WHERE timestamp_created <= {expired_host_timestamp}"
    ))?;
    deleted_values += connection.changes() as u64;

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
        let values = get_values_helper(&connection, key.key.clone())?;
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

/// Merge values with same peer_id by timestamp_created (last-write-wins)
pub fn merge_impl(records: Vec<Record>) -> Result<Vec<Record>, ServiceError> {
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
pub fn renew_host_value_impl(key: String, current_timestamp_sec: u64) -> Result<(), ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)?;
    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec)?;

    let set_by = call_parameters.init_peer_id;
    let host_id = call_parameters.host_id;

    let mut statement = connection.prepare(f!("UPDATE {VALUES_TABLE_NAME} \
                     SET timestamp_created = ?, timestamp_accessed = ? \
                     WHERE key = ? AND set_by = ? AND peer_id = ?"))?;
    statement.bind(1, &Value::Integer(current_timestamp_sec as i64))?;
    statement.bind(2, &Value::Integer(current_timestamp_sec as i64))?;
    statement.bind(3, &Value::String(key.clone()))?;
    statement.bind(4, &Value::String(set_by))?;
    statement.bind(5, &Value::String(host_id))?;
    statement.next()?;

    (connection.changes() == 1).as_result((), HostValueNotFound(key))
}

/// Remove host value by key and caller peer_id
pub fn clear_host_value_impl(key: String, current_timestamp_sec: u64) -> Result<(), ServiceError> {
    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)?;
    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec)?;

    let peer_id = call_parameters.host_id;
    let set_by = call_parameters.init_peer_id;
    delete_value(&connection, &key, peer_id, set_by)?;

    (connection.changes() == 1).as_result((), HostValueNotFound(key))
}

/// Used for replication of host values to other nodes.
/// Similar to republish_values but with an additional check that value.set_by == init_peer_id
pub fn propagate_host_value_impl(
    mut set_host_value: PutHostValueResult,
    current_timestamp_sec: u64,
    weight: WeightResult,
) -> Result<(), ServiceError> {
    if !set_host_value.success || set_host_value.value.len() != 1 {
        return Err(InvalidSetHostValueResult);
    }

    let call_parameters = marine_rs_sdk::get_call_parameters();
    check_host_value_tetraplets(&call_parameters, 0, &set_host_value.value[0])?;
    check_timestamp_tetraplets(&call_parameters, 1)?;
    check_weight_tetraplets(&call_parameters, 2, &weight)?;

    set_host_value.value[0].weight = weight.weight;
    republish_values_helper(
        set_host_value.key,
        set_host_value.value,
        current_timestamp_sec,
    )
    .map(|_| ())
}
