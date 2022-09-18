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

use crate::defaults::{RECORDS_LIMIT, RECORDS_TABLE_NAME};
use crate::error::ServiceError;
use crate::error::ServiceError::InternalError;
use crate::load_config;
use crate::record::{Record, RecordInternal, RecordMetadata};
use crate::storage_impl::{from_custom_option, get_custom_option, Storage};
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_records_table(&self) {
        // TODO: check table schema
        let result = self.connection.execute(f!("
            CREATE TABLE IF NOT EXISTS {RECORDS_TABLE_NAME} (
                key_id TEXT,
                issued_by TEXT,
                peer_id TEXT,
                timestamp_issued INTEGER NOT NULL,
                solution BLOB,
                issuer_signature BLOB NOT NULL,
                is_tombstoned INTEGER NOT NULL,
                value TEXT,
                relay_id TEXT,
                service_id TEXT,
                timestamp_created INTEGER,
                signature BLOB,
                weight INTEGER,
                PRIMARY KEY (key_id, issued_by, peer_id)
            );
        "));

        if let Err(error) = result {
            println!("create_records_table error: {}", error);
        }
    }

    pub fn update_record(&self, record: RecordInternal) -> Result<(), ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;

        // there is no limits for local service records
        if record.record.metadata.peer_id == host_id {
            self.write_record(record)
        } else {
            let records_count =
                self.get_non_host_records_count_by_key(&record.record.metadata.key_id)?;
            // check values limits for non-host values
            if records_count >= RECORDS_LIMIT {
                let min_weight_record =
                    self.get_min_weight_non_host_record_by_key(&record.record.metadata.key_id)?;

                if min_weight_record.weight < record.weight
                    || (min_weight_record.weight == record.weight
                        && min_weight_record.record.timestamp_created
                            < record.record.timestamp_created)
                {
                    // delete the lightest record if the new one is heavier or newer
                    self.delete_record(
                        min_weight_record.record.metadata.key_id,
                        min_weight_record.record.metadata.peer_id,
                        min_weight_record.record.metadata.issued_by,
                    )?;
                } else {
                    // return error if limit is exceeded
                    return Err(ServiceError::ValuesLimitExceeded(
                        record.record.metadata.key_id,
                    ));
                }
            }

            self.write_record(record)
        }
    }

    /// insert record if a record or tombstone with `(key_id, issued_by, peer_id)` does not exist
    /// or replace if it has lower `timestamp_issued`
    pub fn write_record(&self, record: RecordInternal) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "INSERT OR REPLACE INTO {RECORDS_TABLE_NAME} VALUES \
            (key_id, issued_by, peer_id, timestamp_issued, solution, issuer_signature, is_tombstoned, \
            value, relay_id, service_id, timestamp_created, signature, weight) \
            SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? \
            WHERE NOT EXISTS (SELECT * FROM {RECORDS_TABLE_NAME} WHERE key_id=? AND peer_id=? AND issued_by=? AND timestamp_issued<?"
        ))?;

        let is_tombstoned = 0;
        statement.bind(1, &Value::String(record.record.metadata.key_id.clone()))?;
        statement.bind(2, &Value::String(record.record.metadata.issued_by.clone()))?;
        statement.bind(3, &Value::String(record.record.metadata.peer_id.clone()))?;

        statement.bind(
            4,
            &Value::Integer(record.record.metadata.timestamp_issued as i64),
        )?;
        statement.bind(5, &Value::Binary(record.record.metadata.solution))?;
        statement.bind(6, &Value::Binary(record.record.metadata.issuer_signature))?;
        statement.bind(7, &Value::Integer(is_tombstoned))?;

        statement.bind(8, &Value::String(record.record.metadata.value))?;
        statement.bind(
            9,
            &Value::String(from_custom_option(record.record.metadata.relay_id)),
        )?;
        statement.bind(
            10,
            &Value::String(from_custom_option(record.record.metadata.service_id)),
        )?;

        statement.bind(11, &Value::Integer(record.record.timestamp_created as i64))?;
        statement.bind(12, &Value::Binary(record.record.signature))?;
        statement.bind(13, &Value::Integer(record.weight as i64))?;

        statement.bind(14, &Value::String(record.record.metadata.key_id))?;
        statement.bind(15, &Value::String(record.record.metadata.issued_by))?;
        statement.bind(16, &Value::String(record.record.metadata.peer_id))?;
        statement.bind(
            17,
            &Value::Integer(record.record.metadata.timestamp_issued as i64),
        )?;

        statement.next().map(drop)?;

        Ok(())
    }

    pub fn delete_record(
        &self,
        key_id: String,
        peer_id: String,
        issued_by: String,
    ) -> Result<bool, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "DELETE FROM {RECORDS_TABLE_NAME} WHERE key_id=? AND peer_id=? AND issued_by=?"
        ))?;
        statement.bind(1, &Value::String(key_id))?;
        statement.bind(2, &Value::String(peer_id))?;
        statement.bind(3, &Value::String(issued_by))?;
        statement.next().map(drop)?;

        Ok(self.connection.changes() == 1)
    }

    fn get_min_weight_non_host_record_by_key(
        &self,
        key_id: &str,
    ) -> Result<RecordInternal, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;

        // only only non-host values
        let mut statement = self.connection.prepare(
            f!("SELECT key_id, value, peer_id, set_by, relay_id, service_id, timestamp_created, signature, weight FROM {RECORDS_TABLE_NAME} \
                     WHERE key_id = ? AND peer_id != ? AND is_tombstoned = 0 ORDER BY weight ASC LIMIT 1"))?;

        statement.bind(1, &Value::String(key_id.to_string()))?;
        statement.bind(2, &Value::String(host_id))?;

        if let State::Row = statement.next()? {
            read_record(&statement)
        } else {
            Err(InternalError(f!(
                "not found non-host records for given key_id: {key_id}"
            )))
        }
    }

    fn get_non_host_records_count_by_key(&self, key_id: &str) -> Result<usize, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;

        // only only non-host values
        let mut statement = self.connection.prepare(f!(
            "SELECT COUNT(*) FROM {RECORDS_TABLE_NAME} WHERE key_id = ? AND peer_id != ? AND is_tombstoned = 0"
        ))?;
        statement.bind(1, &Value::String(key_id.to_string()))?;
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

    pub fn get_records_count_by_key(&self, key_id: &str) -> Result<u64, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT COUNT(*) FROM {RECORDS_TABLE_NAME} WHERE key_id = ? and is_tombstoned = 0"
        ))?;
        statement.bind(1, &Value::String(key_id.to_string()))?;

        if let State::Row = statement.next()? {
            statement
                .read::<i64>(0)
                .map(|n| n as u64)
                .map_err(ServiceError::SqliteError)
        } else {
            Err(InternalError(f!(
                "get_records_count_by_key: something went totally wrong"
            )))
        }
    }

    pub fn merge_and_update_records(
        &self,
        key_id: String,
        records: Vec<RecordInternal>,
        current_timestamp_sec: u64,
    ) -> Result<u64, ServiceError> {
        let records = merge_records(
            self.get_records(key_id, current_timestamp_sec)?
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

    pub fn get_records(
        &self,
        key_id: String,
        current_timestamp_sec: u64,
    ) -> Result<Vec<RecordInternal>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT key_id, issued_by, peer_id, timestamp_issued, solution, issuer_signature,\
                    value, relay_id, service_id, timestamp_created, signature \
             FROM {RECORDS_TABLE_NAME} WHERE key_id = ? AND is_tombstoned = 0 AND timestamp_created > ? ORDER BY weight DESC"
        ))?;
        let expired_timestamp = current_timestamp_sec - load_config().expired_timeout;
        statement.bind(1, &Value::String(key_id))?;
        statement.bind(2, &Value::Integer(expired_timestamp as i64))?;

        let mut result: Vec<RecordInternal> = vec![];

        while let State::Row = statement.next()? {
            result.push(read_record(&statement)?)
        }

        Ok(result)
    }

    pub fn get_local_stale_records(
        &self,
        key_id: String,
        stale_timestamp_sec: u64,
    ) -> Result<Vec<RecordInternal>, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;
        let mut statement = self.connection.prepare(f!(
            "SELECT key_id, issued_by, peer_id, timestamp_issued, solution, issuer_signature,\
                    value, relay_id, service_id, timestamp_created, signature \
             FROM {RECORDS_TABLE_NAME} WHERE key_id = ? AND peer_id = ? AND is_tombstoned = 0 AND timestamp_created < ?"
        ))?;
        statement.bind(1, &Value::String(key_id))?;
        statement.bind(2, &Value::String(host_id))?;
        statement.bind(3, &Value::Integer(stale_timestamp_sec as i64))?;

        let mut result: Vec<RecordInternal> = vec![];

        while let State::Row = statement.next()? {
            result.push(read_record(&statement)?)
        }

        Ok(result)
    }

    /// Remove expired records except host records (actually we should not have expired host records
    /// at this stage, all host records should be updated in time or removed via tombstones)
    pub fn clear_expired_records(&self, expired_timestamp: u64) -> Result<u64, ServiceError> {
        let host_id = marine_rs_sdk::get_call_parameters().host_id;
        self.connection.execute(f!(
            "DELETE FROM {RECORDS_TABLE_NAME} WHERE timestamp_created <= {expired_timestamp} AND peer_id != '{host_id}' AND is_tombstoned = 0"
        ))?;
        Ok(self.connection.changes() as u64)
    }
}

pub fn read_record(statement: &Statement) -> Result<RecordInternal, ServiceError> {
    Ok(RecordInternal {
        record: Record {
            metadata: RecordMetadata {
                key_id: statement.read::<String>(0)?,
                issued_by: statement.read::<String>(1)?,
                peer_id: statement.read::<String>(2)?,
                timestamp_issued: statement.read::<i64>(3)? as u64,
                solution: statement.read::<Vec<u8>>(4)?,
                issuer_signature: statement.read::<Vec<u8>>(5)?,
                value: statement.read::<String>(6)?,
                relay_id: get_custom_option(statement.read::<String>(7)?),
                service_id: get_custom_option(statement.read::<String>(8)?),
            },
            timestamp_created: statement.read::<i64>(9)? as u64,
            signature: statement.read::<Vec<u8>>(10)?,
        },
        weight: statement.read::<i64>(11)? as u32,
    })
}

/// Merge values with same peer_id by timestamp_created (last-write-wins)
pub fn merge_records(records: Vec<RecordInternal>) -> Result<Vec<RecordInternal>, ServiceError> {
    // key is (peer_id, set_by)
    let mut result: HashMap<(String, String), RecordInternal> = HashMap::new();

    for rec in records.into_iter() {
        let key = (
            rec.record.metadata.peer_id.clone(),
            rec.record.metadata.issued_by.clone(),
        );

        if let Some(other_rec) = result.get_mut(&key) {
            if other_rec.record.timestamp_created < rec.record.timestamp_created {
                *other_rec = rec;
            }
        } else {
            result.insert(key, rec);
        }
    }

    Ok(result.into_iter().map(|(_, rec)| rec).collect())
}
