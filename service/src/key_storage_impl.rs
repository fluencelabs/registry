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

use crate::defaults::{KEYS_TABLE_NAME, KEYS_TIMESTAMPS_TABLE_NAME};

use crate::error::ServiceError;
use crate::error::ServiceError::{InternalError, KeyNotExists};
use crate::key::Key;
use crate::storage_impl::Storage;
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_keys_tables(&self) -> bool {
        self.connection
            .execute(f!("
            CREATE TABLE IF NOT EXISTS {KEYS_TABLE_NAME} (
                key_id TEXT PRIMARY KEY,
                key TEXT,
                peer_id TEXT,
                timestamp_created INTEGER,
                signature TEXT,
                timestamp_published INTEGER,
                pinned INTEGER,
                weight INTEGER
            );
        "))
            .is_ok()
            && self
                .connection
                .execute(f!("
            CREATE TABLE IF NOT EXISTS {KEYS_TIMESTAMPS_TABLE_NAME} (
                key_id TEXT PRIMARY KEY,
                timestamp_accessed INTEGER
            );
        "))
                .is_ok()
    }

    pub fn update_key_timestamp(
        &self,
        key_id: &str,
        current_timestamp_sec: u64,
    ) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!("
             INSERT OR REPLACE INTO {KEYS_TIMESTAMPS_TABLE_NAME} VALUES (?, ?);
         "))?;

        statement.bind(1, &Value::String(key_id.to_string()))?;
        statement.bind(2, &Value::Integer(current_timestamp_sec as i64))?;
        statement.next()?;
        Ok(())
    }

    pub fn get_key(&self, key_id: String) -> Result<Key, ServiceError> {
        let mut statement = self.connection.prepare(f!(
        "SELECT key_id, key, peer_id, timestamp_created, signature, timestamp_published, pinned, weight \
                              FROM {KEYS_TABLE_NAME} WHERE key_id = ?"
        ))?;
        statement.bind(1, &Value::String(key_id.clone()))?;

        if let State::Row = statement.next()? {
            read_key(&statement)
        } else {
            Err(KeyNotExists(key_id))
        }
    }

    pub fn write_key(&self, key: Key) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!("
             INSERT OR REPLACE INTO {KEYS_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?, ?, ?);
         "))?;

        let pinned = if key.pinned { 1 } else { 0 } as i64;
        statement.bind(1, &Value::String(key.key_id))?;
        statement.bind(2, &Value::String(key.key))?;
        statement.bind(3, &Value::String(key.peer_id))?;
        statement.bind(4, &Value::Integer(key.timestamp_created as i64))?;
        statement.bind(
            5,
            &Value::String(bs58::encode(&key.signature).into_string()),
        )?;
        statement.bind(6, &Value::Integer(key.timestamp_published as i64))?;
        statement.bind(7, &Value::Integer(pinned))?;
        statement.bind(8, &Value::Integer(key.weight as i64))?;
        statement.next()?;
        Ok(())
    }

    pub fn update_key(&self, key: Key) -> Result<(), ServiceError> {
        if let Ok(existing_key) = self.get_key(key.key_id.clone()) {
            if existing_key.timestamp_created > key.timestamp_created {
                return Err(ServiceError::KeyAlreadyExistsNewerTimestamp(
                    key.key,
                    key.peer_id,
                ));
            }
        }

        self.write_key(key)
    }

    pub fn check_key_existence(&self, key_id: &str) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT EXISTS(SELECT 1 FROM {KEYS_TABLE_NAME} WHERE key_id = ? LIMIT 1)"
        ))?;
        statement.bind(1, &Value::String(key_id.to_string()))?;

        if let State::Row = statement.next()? {
            let exists = statement.read::<i64>(0)?;
            if exists == 1 {
                Ok(())
            } else {
                Err(KeyNotExists(key_id.to_string()))
            }
        } else {
            Err(InternalError(
                "EXISTS should always return something".to_string(),
            ))
        }
    }

    pub fn get_stale_keys(&self, stale_timestamp: u64) -> Result<Vec<Key>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
        "SELECT key, peer_id, timestamp_created, signature, timestamp_published, pinned, weight \
                              FROM {KEYS_TABLE_NAME} WHERE timestamp_published <= ?"
        ))?;
        statement.bind(1, &Value::Integer(stale_timestamp as i64))?;

        let mut stale_keys: Vec<Key> = vec![];
        while let State::Row = statement.next()? {
            stale_keys.push(read_key(&statement)?);
        }

        Ok(stale_keys)
    }

    pub fn delete_key(&self, key_id: String) -> Result<(), ServiceError> {
        let mut statement = self
            .connection
            .prepare(f!("DELETE FROM {KEYS_TABLE_NAME} WHERE key_id=?"))?;
        statement.bind(1, &Value::String(key_id.clone()))?;
        statement.next().map(drop)?;

        if self.connection.changes() == 1 {
            Ok(())
        } else {
            Err(KeyNotExists(key_id))
        }
    }

    /// not pinned only
    pub fn get_expired_keys(&self, expired_timestamp: u64) -> Result<Vec<Key>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
        "SELECT key, peer_id, timestamp_created, signature, timestamp_published, pinned, weight \
                              FROM {KEYS_TABLE_NAME} WHERE timestamp_created <= ? and pinned != 1"
        ))?;
        statement.bind(1, &Value::Integer(expired_timestamp as i64))?;

        let mut expired_keys: Vec<Key> = vec![];
        while let State::Row = statement.next()? {
            let key = read_key(&statement)?;
            let timestamp_accessed = self.get_key_timestamp_accessed(&key.key_id)?;
            let with_host_records = self.get_host_records_count_by_key(key.key_id.clone())? != 0;

            if timestamp_accessed <= expired_timestamp && !with_host_records {
                expired_keys.push(key);
            }
        }

        Ok(expired_keys)
    }

    pub fn get_key_timestamp_accessed(&self, key_id: &str) -> Result<u64, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT timestamp_accessed FROM {KEYS_TIMESTAMPS_TABLE_NAME} WHERE key_id != ?"
        ))?;
        statement.bind(1, &Value::String(key_id.to_string()))?;

        if let State::Row = statement.next()? {
            statement
                .read::<i64>(0)
                .map(|t| t as u64)
                .map_err(ServiceError::SqliteError)
        } else {
            Err(KeyNotExists(key_id.to_string()))
        }
    }
}

pub fn read_key(statement: &Statement) -> Result<Key, ServiceError> {
    Ok(Key {
        key_id: statement.read::<String>(0)?,
        key: statement.read::<String>(1)?,
        peer_id: statement.read::<String>(2)?,
        timestamp_created: statement.read::<i64>(3)? as u64,
        signature: bs58::decode(&statement.read::<String>(4)?)
            .into_vec()
            .map_err(|_| InternalError("".to_string()))?,
        timestamp_published: statement.read::<i64>(5)? as u64,
        pinned: statement.read::<i64>(6)? != 0,
        weight: statement.read::<i64>(7)? as u32,
    })
}
