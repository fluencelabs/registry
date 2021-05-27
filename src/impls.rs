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

use crate::{KEYS_TABLE_NAME, VALUES_TABLE_NAME, DB_PATH, TRUSTED_TIMESTAMP_SERVICE_ID, TRUSTED_TIMESTAMP_FUNCTION_NAME, EXPIRED_VALUE_AGE, STALE_VALUE_AGE};
use crate::results::{Key, Record, EvictStaleItem};
use marine_sqlite_connector::{Connection, Result as SqliteResult, Error as SqliteError, State};
use fluence::{CallParameters};
use eyre;
use eyre::ContextCompat;
use std::collections::HashMap;


fn get_custom_option(value: String) -> Vec<String> {
    if value.is_empty() {
        vec![]
    } else {
        vec![value]
    }
}

pub(crate) fn check_timestamp_tetraplets(call_parameters: &CallParameters, arg_number: usize) -> eyre::Result<()> {
    let error_msg = "you should use peer.timestamp_ms to pass timestamp";
    let tetraplets = call_parameters.tetraplets.get(arg_number).wrap_err(error_msg)?;
    let tetraplet = tetraplets.get(0).wrap_err(error_msg)?;
    (tetraplet.service_id == TRUSTED_TIMESTAMP_SERVICE_ID &&
        tetraplet.function_name == TRUSTED_TIMESTAMP_FUNCTION_NAME).then(|| ()).wrap_err(error_msg)
    // TODO check host_id == peer_pk(???)
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
                peer_id TEXT);
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
                relay_id TEXT,
                service_id TEXT,
                timestamp_created INTEGER,
                timestamp_accessed INTEGER,
                PRIMARY KEY (key, peer_id)
                );
        "),
        ).is_ok()
}

fn get_key_metadata_helper(connection: &Connection, key: String, current_timestamp: u64) -> SqliteResult<Key> {
    connection.execute(
        f!("UPDATE {KEYS_TABLE_NAME} \
                     SET timestamp_accessed = '{current_timestamp}' \
                     WHERE key = '{key}'"))?;

    let mut statement = connection
        .prepare(f!("SELECT key, peer_id, timestamp_created \
                              FROM {KEYS_TABLE_NAME} WHERE key = '{key}'"))?;

    if let State::Row = statement.next()? {
        Ok(Key {
            key: statement.read::<String>(0)?,
            peer_id: statement.read::<String>(1)?,
            timestamp_created: statement.read::<i64>(2)? as u64,
        })
    } else {
        Err(SqliteError { code: None, message: Some("not found".to_string()) })
    }
}

fn update_key(connection: &Connection, key: String, peer_id: String, timestamp_created: u64, timestamp_accessed: u64) -> SqliteResult<()> {
    let old_key = get_key_metadata_helper(&connection, key.clone(), timestamp_accessed);

    if old_key.is_err() || old_key?.peer_id == peer_id {
        connection.execute(f!("
             INSERT OR REPLACE INTO {KEYS_TABLE_NAME} VALUES ('{key}', '{timestamp_created}', '{timestamp_accessed}', '{peer_id}');
         "))
    } else {
        Err(SqliteError { code: None, message: Some("key already exists with different peer_id".to_string()) })
    }
}

pub fn get_key_metadata_impl(key: String, current_timestamp: u64) -> SqliteResult<Key> {
    let call_parameters = fluence::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    get_key_metadata_helper(&get_connection()?, key, current_timestamp)
}

pub fn register_key_impl(key: String, current_timestamp: u64) -> SqliteResult<()> {
    let call_parameters = fluence::get_call_parameters();
    let peer_id = call_parameters.init_peer_id.clone();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    update_key(&get_connection()?, key, peer_id, current_timestamp.clone(), current_timestamp)
}

pub fn republish_key_impl(key: Key, current_timestamp: u64) -> SqliteResult<()> {
    let call_parameters = fluence::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    update_key(&get_connection()?, key.key, key.peer_id, key.timestamp_created, current_timestamp)
}

pub fn put_value_impl(key: String, value: String, current_timestamp: u64, relay_id: Vec<String>, service_id: Vec<String>) -> SqliteResult<()> {
    let call_parameters = fluence::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 2)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    let connection = get_connection()?;

    // checking key for existence
    let _key = get_key_metadata_helper(&connection, key.clone(), current_timestamp.clone())?;
    let relay_id = if relay_id.len() == 0 { "".to_string() } else { relay_id[0].clone() };
    let peer_id = call_parameters.init_peer_id;
    let service_id = if service_id.len() == 0 { "".to_string() } else { service_id[0].clone() };

    connection.execute(
        f!("INSERT OR REPLACE INTO {VALUES_TABLE_NAME} \
                    VALUES ('{key}', '{value}', '{peer_id}', '{relay_id}',\
                    '{service_id}', '{current_timestamp}', '{current_timestamp}')")
    )
}

pub fn get_values_helper(connection: &Connection, key: String) -> SqliteResult<Vec<Record>> {
    let mut statement = connection.prepare(
        f!("SELECT value, peer_id, relay_id, service_id, timestamp_created FROM {VALUES_TABLE_NAME} \
                     WHERE key = '{key}'"))?;
    let mut result: Vec<Record> = vec![];

    while let State::Row = statement.next()? {
        result.push(Record {
            value: statement.read::<String>(0)?,
            peer_id: statement.read::<String>(1)?,
            relay_id: get_custom_option(statement.read::<String>(2)?),
            service_id: get_custom_option(statement.read::<String>(3)?),
            timestamp_created: statement.read::<i64>(4)? as u64,
        })
    }

    Ok(result)
}

pub fn get_values_impl(key: String, current_timestamp: u64) -> SqliteResult<Vec<Record>> {
    let call_parameters = fluence::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 1)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;

    let connection = get_connection()?;

    connection.execute(
        f!("UPDATE {VALUES_TABLE_NAME} \
                     SET timestamp_accessed = '{current_timestamp}' \
                     WHERE key = '{key}'"))?;

    get_values_helper(&connection, key)
}

pub fn republish_values_impl(key: String, records: Vec<Record>, current_timestamp: u64) -> SqliteResult<u64> {
    let call_parameters = fluence::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 2)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    let connection = get_connection()?;

    // checking key for existence
    let _key = get_key_metadata_helper(&connection, key.clone(), current_timestamp.clone())?;

    let mut updated = 0u64;
    for record in records.iter() {
        let relay_id = if record.relay_id.is_empty() {"".to_string()} else {record.relay_id[0].clone()};
        let service_id = if record.service_id.is_empty() {"".to_string()} else {record.service_id[0].clone()};
        connection.execute(
            f!("INSERT OR REPLACE INTO {VALUES_TABLE_NAME} \
                    VALUES ('{key}', '{record.value}', '{record.peer_id}', '{relay_id}',\
                    '{service_id}', '{record.timestamp_created}', '{current_timestamp}')"))?;

        updated += connection.changes() as u64;
    }

    Ok(updated)
}

pub fn clear_expired_impl(current_timestamp: u64) -> SqliteResult<(u64, u64)> {
    let call_parameters = fluence::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 0)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    let connection = get_connection()?;

    let expired_timestamp = current_timestamp - EXPIRED_VALUE_AGE;
    connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE key IN (SELECT key FROM {KEYS_TABLE_NAME} WHERE timestamp_created <= {expired_timestamp})"))?;
    let deleted_values = connection.changes() as u64;
    connection.execute(f!("DELETE FROM {KEYS_TABLE_NAME} WHERE timestamp_created <= {expired_timestamp}"))?;
    let deleted_keys = connection.changes() as u64;

    Ok((deleted_keys, deleted_values))
}

pub fn evict_stale_impl(current_timestamp: u64) -> SqliteResult<Vec<EvictStaleItem>> {
    let call_parameters = fluence::get_call_parameters();
    check_timestamp_tetraplets(&call_parameters, 0)
        .map_err(|e| SqliteError { code: None, message: Some(e.to_string()) })?;
    let connection = get_connection()?;

    let stale_timestamp = current_timestamp - STALE_VALUE_AGE;

    let mut stale_keys: Vec<Key> = vec![];
    let mut statement =
        connection.prepare(
            f!("SELECT key, peer_id, timestamp_created FROM {KEYS_TABLE_NAME} \
                         WHERE timestamp_accessed <= {stale_timestamp}"))?;

    while let State::Row = statement.next()? {
        stale_keys.push(Key {
            key: statement.read::<String>(0)?,
            peer_id: statement.read::<String>(1)?,
            timestamp_created: statement.read::<i64>(2)? as u64,
        });
    }

    let mut results: Vec<EvictStaleItem> = vec![];
    for key in stale_keys.iter() {
        results.push(EvictStaleItem { key: key.clone(), records: get_values_helper(&connection, key.key.clone())? });
        connection.execute(f!("DELETE FROM {VALUES_TABLE_NAME} WHERE key='{key.key}'"))?;
        connection.execute(f!("DELETE FROM {KEYS_TABLE_NAME} WHERE key='{key.key}'"))?;
    }

    Ok(results)
}

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
