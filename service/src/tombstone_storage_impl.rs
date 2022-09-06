/*
 * Copyright 2022 Fluence Labs Limited
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
use crate::defaults::RECORDS_TABLE_NAME;
use crate::error::ServiceError;
use crate::load_config;
use crate::storage_impl::Storage;
use crate::tombstone::Tombstone;
use marine_sqlite_connector::{State, Statement, Value};

pub fn read_tombstone(statement: &Statement) -> Result<Tombstone, ServiceError> {
    Ok(Tombstone {
        key_id: statement.read::<String>(0)?,
        issued_by: statement.read::<String>(1)?,
        peer_id: statement.read::<String>(2)?,
        timestamp_issued: statement.read::<i64>(3)? as u64,
        solution: statement.read::<Vec<u8>>(4)?,
        issuer_signature: statement.read::<Vec<u8>>(5)?,
    })
}

impl Storage {
    /// insert tombstone if a record or tombstone with `(key_id, issued_by, peer_id)` does not exist
    /// or replace if it has lower `timestamp_issued`
    pub fn write_tombstone(&self, tombstone: Tombstone) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "INSERT OR REPLACE INTO {RECORDS_TABLE_NAME} VALUES \
            (key_id, issued_by, peer_id, timestamp_issued, solution, issuer_signature, is_tombstoned) \
            SELECT ?, ?, ?, ?, ?, ?, ? \
            WHERE NOT EXISTS (SELECT * FROM {RECORDS_TABLE_NAME} WHERE key_id=? AND peer_id=? AND issued_by=? AND timestamp_issued<?"
        ))?;

        let is_tombstoned = 1;
        statement.bind(1, &Value::String(tombstone.key_id.clone()))?;
        statement.bind(2, &Value::String(tombstone.issued_by.clone()))?;
        statement.bind(3, &Value::String(tombstone.peer_id.clone()))?;

        statement.bind(4, &Value::Integer(tombstone.timestamp_issued as i64))?;
        statement.bind(5, &Value::Binary(tombstone.solution))?;
        statement.bind(6, &Value::Binary(tombstone.issuer_signature))?;
        statement.bind(7, &Value::Integer(is_tombstoned))?;

        statement.bind(8, &Value::String(tombstone.key_id))?;
        statement.bind(9, &Value::String(tombstone.issued_by))?;
        statement.bind(10, &Value::String(tombstone.peer_id))?;
        statement.bind(11, &Value::Integer(tombstone.timestamp_issued as i64))?;

        statement.next().map(drop)?;

        Ok(())
    }

    pub fn get_tombstones(
        &self,
        key_id: String,
        current_timestamp_sec: u64,
    ) -> Result<Vec<Tombstone>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT key_id, issued_by, peer_id, timestamp_issued, solution, issuer_signature \
             FROM {RECORDS_TABLE_NAME} WHERE key_id = ? AND is_tombstoned = 1 and timestamp_issued > expired_timestamp"
        ))?;

        let expired_timestamp = current_timestamp_sec - load_config().expired_timeout;
        statement.bind(1, &Value::String(key_id))?;
        statement.bind(2, &Value::Integer(expired_timestamp as i64))?;

        let mut result: Vec<Tombstone> = vec![];

        while let State::Row = statement.next()? {
            result.push(read_tombstone(&statement)?)
        }

        Ok(result)
    }

    /// Remove expired tombstones
    pub fn clear_expired_tombstones(&self, expired_timestamp: u64) -> Result<u64, ServiceError> {
        self.connection.execute(f!(
            "DELETE FROM {RECORDS_TABLE_NAME} WHERE timestamp_created <= {expired_timestamp} AND is_tombstoned = 1"
        ))?;
        Ok(self.connection.changes() as u64)
    }
}
