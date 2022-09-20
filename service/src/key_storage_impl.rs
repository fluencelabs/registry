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
use crate::error::ServiceError::{InternalError, KeyNotExists};
use crate::key::{Key, KeyInternal};
use crate::storage_impl::Storage;
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_key_tables(&self) {
        // TODO: check table schema
        let result = self.connection.execute(f!("
            CREATE TABLE IF NOT EXISTS {KEYS_TABLE_NAME} (
                key_id TEXT PRIMARY KEY,
                label TEXT,
                owner_peer_id TEXT,
                timestamp_created INTEGER,
                challenge BLOB,
                challenge_type TEXT,
                signature BLOB NOT NULL,
                timestamp_published INTEGER,
                weight INTEGER
            );
        "));

        if let Err(error) = result {
            println!("create_keys_table error: {}", error);
        }
    }

    pub fn get_key(&self, key_id: String) -> Result<Key, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT key_id, label, owner_peer_id, timestamp_created, challenge, challenge_type, signature \
                              FROM {KEYS_TABLE_NAME} WHERE key_id = ?"
        ))?;
        statement.bind(1, &Value::String(key_id.clone()))?;

        if let State::Row = statement.next()? {
            read_key(&statement)
        } else {
            Err(KeyNotExists(key_id))
        }
    }

    pub fn write_key(&self, key: KeyInternal) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!("
             INSERT OR REPLACE INTO {KEYS_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
         "))?;

        statement.bind(1, &Value::String(key.key.id))?;
        statement.bind(2, &Value::String(key.key.label))?;
        statement.bind(3, &Value::String(key.key.owner_peer_id))?;
        statement.bind(4, &Value::Integer(key.key.timestamp_created as i64))?;
        statement.bind(5, &Value::Binary(key.key.challenge))?;
        statement.bind(6, &Value::String(key.key.challenge_type))?;
        statement.bind(7, &Value::Binary(key.key.signature))?;
        statement.bind(8, &Value::Integer(key.timestamp_published as i64))?;
        statement.bind(9, &Value::Integer(key.weight as i64))?;
        statement.next()?;
        Ok(())
    }

    pub fn update_key(&self, key: KeyInternal) -> Result<(), ServiceError> {
        if let Ok(existing_key) = self.get_key(key.key.id.clone()) {
            if existing_key.timestamp_created > key.key.timestamp_created {
                return Err(ServiceError::KeyAlreadyExistsNewerTimestamp(
                    key.key.label,
                    key.key.owner_peer_id,
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

    pub fn get_stale_keys(&self, stale_timestamp: u64) -> Result<Vec<KeyInternal>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT key_id, label, owner_peer_id, timestamp_created, challenge, challenge_type, signature, timestamp_published, weight \
                              FROM {KEYS_TABLE_NAME} WHERE timestamp_published <= ?"
        ))?;
        statement.bind(1, &Value::Integer(stale_timestamp as i64))?;

        let mut stale_keys: Vec<KeyInternal> = vec![];
        while let State::Row = statement.next()? {
            stale_keys.push(read_internal_key(&statement)?);
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

    pub fn clear_expired_keys(&self, expired_timestamp: u64) -> Result<u64, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT key_id FROM {KEYS_TABLE_NAME} WHERE timestamp_created <= ?"
        ))?;
        statement.bind(1, &Value::Integer(expired_timestamp as i64))?;

        let mut expired_keys: Vec<String> = vec![];
        while let State::Row = statement.next()? {
            let key_id = statement.read::<String>(0)?;
            if self.get_records_count_by_key(&key_id)? == 0 {
                expired_keys.push(key_id);
            }
        }

        let removed_keys = expired_keys.len();
        for id in expired_keys.into_iter() {
            self.delete_key(id)?;
        }

        Ok(removed_keys as u64)
    }
}

pub fn read_key(statement: &Statement) -> Result<Key, ServiceError> {
    Ok(Key {
        id: statement.read::<String>(0)?,
        label: statement.read::<String>(1)?,
        owner_peer_id: statement.read::<String>(2)?,
        timestamp_created: statement.read::<i64>(3)? as u64,
        challenge: statement.read::<Vec<u8>>(4)?,
        challenge_type: statement.read::<String>(5)?,
        signature: statement.read::<Vec<u8>>(6)?,
    })
}

pub fn read_internal_key(statement: &Statement) -> Result<KeyInternal, ServiceError> {
    Ok(KeyInternal {
        key: read_key(statement)?,
        timestamp_published: statement.read::<i64>(7)? as u64,
        weight: statement.read::<i64>(8)? as u32,
    })
}
