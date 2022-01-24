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

use boolinator::Boolinator;
use std::collections::HashMap;

use crate::defaults::{KEYS_TABLE_NAME, VALUES_LIMIT, VALUES_TABLE_NAME};
use crate::error::ServiceError;
use crate::error::ServiceError::InternalError;
use crate::key_storage_impl::check_key_existence;
use crate::record::Record;
use crate::storage_impl::{from_custom_option, get_connection, get_custom_option};
use marine_sqlite_connector::{Connection, State, Statement, Value};

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

pub fn read_record(statement: &Statement) -> Result<Record, ServiceError> {
    Ok(Record {
        value: statement.read::<String>(0)?,
        peer_id: statement.read::<String>(1)?,
        set_by: statement.read::<String>(2)?,
        relay_id: get_custom_option(statement.read::<String>(3)?),
        service_id: get_custom_option(statement.read::<String>(4)?),
        timestamp_created: statement.read::<i64>(5)? as u64,
        signature: statement.read::<Vec<u8>>(6)?,
        weight: statement.read::<i64>(7)? as u32,
    })
}

/// Put value with caller peer_id if the key exists.
/// If the value is NOT a host value and the key already has `VALUES_LIMIT` records, then a value with the smallest weight is removed and the new value is inserted instead.
pub fn put_record(
    key: String,
    record: Record,
    host: bool,
    current_timestamp_sec: u64,
) -> Result<(), ServiceError> {
    let connection = get_connection()?;

    check_key_existence(&connection, key.clone(), current_timestamp_sec)?;
    let records_count = get_non_host_records_count_by_key(&connection, key.clone())?;

    // check values limits for non-host values
    if !host && records_count >= VALUES_LIMIT {
        let min_weight_record = get_min_weight_non_host_record_by_key(&connection, key.clone())?;

        if min_weight_record.weight < record.weight {
            // delete the lightest record if the new one is heavier
            delete_record(
                &connection,
                &key,
                min_weight_record.peer_id,
                min_weight_record.set_by,
            )?;
        } else {
            // return error if limit is exceeded
            return Err(ServiceError::ValuesLimitExceeded(key));
        }
    }

    write_record(&connection, key, record, current_timestamp_sec)?;
    Ok(())
}

pub fn write_record(
    connection: &Connection,
    key: String,
    record: Record,
    current_timestamp: u64,
) -> Result<(), ServiceError> {
    let mut statement = connection.prepare(f!(
        "INSERT OR REPLACE INTO {VALUES_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))?;

    statement.bind(1, &Value::String(key))?;
    statement.bind(2, &Value::String(record.value))?;
    statement.bind(3, &Value::String(record.peer_id))?;
    statement.bind(4, &Value::String(record.set_by))?;
    statement.bind(5, &Value::String(from_custom_option(record.relay_id)))?;
    statement.bind(6, &Value::String(from_custom_option(record.service_id)))?;
    statement.bind(7, &Value::Integer(record.timestamp_created as i64))?;
    statement.bind(8, &Value::Integer(current_timestamp as i64))?;
    statement.bind(9, &Value::Integer(record.weight as i64))?;
    statement.next().map(drop)?;

    Ok(())
}

pub fn delete_record(
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

pub fn touch_record(
    connection: &Connection,
    key: String,
    peer_id: String,
    set_by: String,
    current_timestamp_sec: u64,
) -> Result<(), ServiceError> {
    let mut statement = connection.prepare(f!("UPDATE {VALUES_TABLE_NAME} \
                     SET timestamp_created = ?, timestamp_accessed = ? \
                     WHERE key = ? AND set_by = ? AND peer_id = ?"))?;
    statement.bind(1, &Value::Integer(current_timestamp_sec as i64))?;
    statement.bind(2, &Value::Integer(current_timestamp_sec as i64))?;
    statement.bind(3, &Value::String(key.clone()))?;
    statement.bind(4, &Value::String(set_by))?;
    statement.bind(5, &Value::String(peer_id))?;
    statement.next()?;

    (connection.changes() == 1).as_result((), ServiceError::HostValueNotFound(key))
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

pub fn get_non_host_records_count_by_key(
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

pub fn merge_and_update_records(
    key: String,
    records: Vec<Record>,
    current_timestamp_sec: u64,
) -> Result<u64, ServiceError> {
    let connection = get_connection()?;
    check_key_existence(&connection, key.clone(), current_timestamp_sec)?;

    let records = merge_records(
        get_records(&connection, key.clone(), None)?
            .into_iter()
            .chain(records.into_iter())
            .collect(),
    )?;

    let mut updated = 0u64;
    for record in records.into_iter() {
        write_record(&connection, key.clone(), record, current_timestamp_sec)?;
        updated += connection.changes() as u64;
    }

    Ok(updated)
}

pub fn get_records(
    connection: &Connection,
    key: String,
    current_timestamp_sec: Option<u64>,
) -> Result<Vec<Record>, ServiceError> {
    if let Some(current_timestamp_sec) = current_timestamp_sec {
        let mut statement = connection.prepare(f!("UPDATE {VALUES_TABLE_NAME} \
                  SET timestamp_accessed = ? \
                  WHERE key = ?"))?;

        statement.bind(1, &Value::Integer(current_timestamp_sec as i64))?;
        statement.bind(2, &Value::String(key.clone()))?;
        statement.next()?;
    }

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

/// Merge values with same peer_id by timestamp_created (last-write-wins)
pub fn merge_records(records: Vec<Record>) -> Result<Vec<Record>, ServiceError> {
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

pub fn clear_records_for_expired_keys(
    connection: &Connection,
    expired_timestamp: u64,
) -> Result<u64, ServiceError> {
    connection.execute(f!(
        "DELETE FROM {VALUES_TABLE_NAME} WHERE key IN (SELECT key FROM {KEYS_TABLE_NAME} \
                                    WHERE timestamp_created <= {expired_timestamp})"
    ))?;
    Ok(connection.changes() as u64)
}

pub fn clear_expired_records(
    connection: &Connection,
    expired_timestamp: u64,
) -> Result<u64, ServiceError> {
    connection.execute(f!(
        "DELETE FROM {VALUES_TABLE_NAME} WHERE timestamp_created <= {expired_timestamp}"
    ))?;
    Ok(connection.changes() as u64)
}
