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

use crate::defaults::{RECORDS_TABLE_NAME, VALUES_LIMIT};
use crate::error::ServiceError;
use crate::error::ServiceError::InternalError;
use crate::record::Record;
use crate::storage_impl::{from_custom_option, get_custom_option, Storage};
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_values_table(&self) -> bool {
        self.connection
            .execute(f!("
            CREATE TABLE IF NOT EXISTS {RECORDS_TABLE_NAME} (
                key_id TEXT,
                value TEXT,
                peer_id TEXT,
                set_by TEXT,
                relay_id TEXT,
                service_id TEXT,
                timestamp_created INTEGER,
                signature BLOB,
                weight INTEGER,
                PRIMARY KEY (key_id, record_peer_id, set_by)
            );
        "))
            .is_ok()
    }

    /// Put value with caller peer_id if the key exists.
    /// If the value is NOT a host value and the key already has `VALUES_LIMIT` records, then a value with the smallest weight is removed and the new value is inserted instead.
    pub fn update_record(&self, record: Record, host: bool) -> Result<(), ServiceError> {
        let records_count = self.get_non_host_records_count_by_key(record.key_id.clone())?;

        // check values limits for non-host values
        if !host && records_count >= VALUES_LIMIT {
            let min_weight_record =
                self.get_min_weight_non_host_record_by_key(record.key_id.clone())?;

            if min_weight_record.weight < record.weight {
                // delete the lightest record if the new one is heavier
                self.delete_record(
                    min_weight_record.key_id,
                    min_weight_record.peer_id,
                    min_weight_record.set_by,
                )?;
            } else {
                // return error if limit is exceeded
                return Err(ServiceError::ValuesLimitExceeded(record.key_id));
            }
        }

        self.write_record(record)?;
        Ok(())
    }

    pub fn write_record(&self, record: Record) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "INSERT OR REPLACE INTO {RECORDS_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))?;

        statement.bind(1, &Value::String(record.key_id))?;
        statement.bind(2, &Value::String(record.value))?;
        statement.bind(3, &Value::String(record.peer_id))?;
        statement.bind(4, &Value::String(record.set_by))?;
        statement.bind(5, &Value::String(from_custom_option(record.relay_id)))?;
        statement.bind(6, &Value::String(from_custom_option(record.service_id)))?;
        statement.bind(7, &Value::Integer(record.timestamp_created as i64))?;
        statement.bind(8, &Value::Binary(record.signature))?;
        statement.bind(9, &Value::Integer(record.weight as i64))?;
        statement.next().map(drop)?;

        Ok(())
    }

    pub fn delete_record(
        &self,
        key_id: String,
        peer_id: String,
        set_by: String,
    ) -> Result<bool, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "DELETE FROM {RECORDS_TABLE_NAME} WHERE key_id=? AND peer_id=? AND set_by=?"
        ))?;
        statement.bind(1, &Value::String(key_id))?;
        statement.bind(2, &Value::String(peer_id))?;
        statement.bind(3, &Value::String(set_by))?;
        statement.next().map(drop)?;

        Ok(self.connection.changes() == 1)
    }

    fn get_min_weight_non_host_record_by_key(
        &self,
        key_id: String,
    ) -> Result<Record, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;

        // only only non-host values
        let mut statement = self.connection.prepare(
            f!("SELECT key_id, value, peer_id, set_by, relay_id, service_id, timestamp_created, signature, weight FROM {RECORDS_TABLE_NAME} \
                     WHERE key_id = ? AND peer_id != ? ORDER BY weight ASC LIMIT 1"))?;

        statement.bind(1, &Value::String(key_id.clone()))?;
        statement.bind(2, &Value::String(host_id))?;

        if let State::Row = statement.next()? {
            read_record(&statement)
        } else {
            Err(InternalError(f!(
                "not found non-host records for given key_id: {key_id}"
            )))
        }
    }

    fn get_non_host_records_count_by_key(&self, key: String) -> Result<usize, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;

        // only only non-host values
        let mut statement = self.connection.prepare(f!(
            "SELECT COUNT(*) FROM {RECORDS_TABLE_NAME} WHERE key_id = ? AND peer_id != ?"
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

    pub fn get_host_records_count_by_key(
        &self,
        key: String,
        key_peer_id: String,
    ) -> Result<u64, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;

        // only only non-host values
        let mut statement = self.connection.prepare(f!(
            "SELECT COUNT(*) FROM {RECORDS_TABLE_NAME} WHERE key = ? AND key_peer_id = ? AND record_peer_id = ?"
        ))?;
        statement.bind(1, &Value::String(key))?;
        statement.bind(1, &Value::String(key_peer_id))?;
        statement.bind(2, &Value::String(host_id))?;

        if let State::Row = statement.next()? {
            statement
                .read::<i64>(0)
                .map(|n| n as u64)
                .map_err(ServiceError::SqliteError)
        } else {
            Err(InternalError(f!(
                "get_non_host_records_count_by_key: something went totally wrong"
            )))
        }
    }

    pub fn merge_and_update_records(
        &self,
        key_id: String,
        records: Vec<Record>,
    ) -> Result<u64, ServiceError> {
        let records = merge_records(
            self.get_records(key_id)?
                .into_iter()
                .chain(records.into_iter())
                .collect(),
        )?;

        let mut updated = 0u64;
        for record in records.into_iter() {
            self.write_record(record)?;
            updated += self.connection.changes() as u64;
        }

        Ok(updated)
    }

    pub fn get_records(&self, key_id: String) -> Result<Vec<Record>, ServiceError> {
        let mut statement = self.connection.prepare(
            f!("SELECT key_id, value, peer_id, set_by, relay_id, service_id, timestamp_created, signature, weight FROM {RECORDS_TABLE_NAME} \
                     WHERE key_id = ? ORDER BY weight DESC"))?;
        statement.bind(1, &Value::String(key_id))?;

        let mut result: Vec<Record> = vec![];

        while let State::Row = statement.next()? {
            result.push(read_record(&statement)?)
        }

        Ok(result)
    }

    /// except host records
    pub fn clear_expired_records(&self, expired_timestamp: u64) -> Result<u64, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;
        self.connection.execute(f!(
            "DELETE FROM {RECORDS_TABLE_NAME} WHERE timestamp_created <= {expired_timestamp} AND record_peer_id != {host_id}"
        ))?;
        Ok(self.connection.changes() as u64)
    }

    /// except host records and for pinned keys
    pub fn delete_records_by_key(&self, key_id: String) -> Result<u64, ServiceError> {
        let mut statement = self
            .connection
            .prepare(f!("DELETE FROM {RECORDS_TABLE_NAME} WHERE key_id = ?"))?;

        statement.bind(1, &Value::String(key_id))?;

        statement.next().map(drop)?;
        Ok(self.connection.changes() as u64)
    }
}

pub fn read_record(statement: &Statement) -> Result<Record, ServiceError> {
    Ok(Record {
        key_id: statement.read::<String>(0)?,
        value: statement.read::<String>(1)?,
        peer_id: statement.read::<String>(2)?,
        set_by: statement.read::<String>(3)?,
        relay_id: get_custom_option(statement.read::<String>(4)?),
        service_id: get_custom_option(statement.read::<String>(5)?),
        timestamp_created: statement.read::<i64>(6)? as u64,
        signature: statement.read::<Vec<u8>>(7)?,
        weight: statement.read::<i64>(8)? as u32,
    })
}

/// Merge values with same peer_id by timestamp_created (last-write-wins)
pub fn merge_records(records: Vec<Record>) -> Result<Vec<Record>, ServiceError> {
    // key is (record_peer_id, set_by)
    let mut result: HashMap<(String, String), Record> = HashMap::new();

    for rec in records.into_iter() {
        let key = (rec.peer_id.clone(), rec.set_by.clone());

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
