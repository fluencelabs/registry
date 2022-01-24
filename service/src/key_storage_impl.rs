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

use crate::defaults::KEYS_TABLE_NAME;

use crate::error::ServiceError;
use crate::error::ServiceError::KeyNotExists;
use crate::key::Key;
use crate::storage_impl::get_connection;
use marine_sqlite_connector::{Connection, State, Statement, Value};

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

pub fn read_key(statement: &Statement) -> Result<Key, ServiceError> {
    Ok(Key {
        key: statement.read::<String>(0)?,
        peer_id: statement.read::<String>(1)?,
        timestamp_created: statement.read::<i64>(2)? as u64,
        signature: statement.read::<Vec<u8>>(3)?,
        pinned: statement.read::<i64>(4)? != 0,
        weight: statement.read::<i64>(5)? as u32,
    })
}

/// Update timestamp_accessed and return metadata of the key
pub fn get_key(
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

pub fn write_key(
    connection: &Connection,
    key: String,
    timestamp_created: u64,
    timestamp_accessed: u64,
    peer_id: String,
    signature: Vec<u8>,
    pin: bool,
    weight: u32,
) -> Result<(), ServiceError> {
    let mut statement = connection.prepare(f!("
             INSERT OR REPLACE INTO {KEYS_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?, ?);
         "))?;

    let pinned = if pin { 1 } else { 0 } as i64;
    statement.bind(1, &Value::String(key))?;
    statement.bind(2, &Value::Integer(timestamp_created as i64))?;
    statement.bind(3, &Value::Integer(timestamp_accessed as i64))?;
    statement.bind(4, &Value::String(peer_id))?;
    statement.bind(5, &Value::Binary(signature))?;
    statement.bind(6, &Value::Integer(pinned))?;
    statement.bind(7, &Value::Integer(weight as i64))?;
    statement.next()?;
    Ok(())
}

pub fn update_key(
    connection: &Connection,
    key: String,
    timestamp_created: u64,
    current_timestamp_sec: u64,
    peer_id: String,
    signature: Vec<u8>,
    pin: bool,
    weight: u32,
) -> Result<(), ServiceError> {
    if let Ok(existing_key) = get_key(&connection, key.clone(), current_timestamp_sec) {
        if existing_key.peer_id != peer_id {
            return Err(ServiceError::KeyAlreadyExists(key));
        }

        if existing_key.timestamp_created > timestamp_created {
            return Err(ServiceError::KeyAlreadyExistsNewerTimestamp(key));
        }
    }

    write_key(
        &connection,
        key,
        timestamp_created,
        current_timestamp_sec,
        peer_id,
        signature,
        pin,
        weight,
    )
}

pub fn check_key_existence(
    connection: &Connection,
    key: String,
    current_timestamp_sec: u64,
) -> Result<(), ServiceError> {
    get_key(connection, key, current_timestamp_sec).map(|_| ())
}
