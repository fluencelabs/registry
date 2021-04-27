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

mod results;

use crate::results::{Record, GetValueResult, PutValueResult, ClearExpiredResult, GetStaleRecordsResult};

use fluence::fce;
use fluence::module_manifest;

use fce_sqlite_connector;
use fce_sqlite_connector::{Connection, Result as SqliteResult, Error as SqliteError, State};

#[macro_use]
extern crate fstrings;

module_manifest!();

pub static TABLE_NAME: &str = "dht";
pub static DB_PATH: &str = "/tmp/dht.db";
pub static STALE_VALUE_AGE: u64 = 60 * 60 * 1000;
pub static EXPIRED_VALUE_AGE: u64 = 24 * 60 * 60 * 1000;

#[inline]
fn get_connection() -> SqliteResult<Connection> {
    fce_sqlite_connector::open(DB_PATH)
}

fn create_table(table_name: String) -> bool {
    let connection = get_connection().unwrap();

    connection
        .execute(f!("
            CREATE TABLE IF NOT EXISTS {table_name} (
                key TEXT PRIMARY KEY,
                value TEXT,
                timestamp INTEGER,
                peer_id TEXT);
        "),
        ).is_ok()
}

fn main() {
    create_table(TABLE_NAME.to_string());
}

#[fce]
pub fn put_value(key: String, value: String, timestamp: u64) -> PutValueResult {
    fn put_value_impl(key: String, value: String, timestamp: u64) -> SqliteResult<()> {
        let connection = get_connection()?;
        let peer_id = fluence::get_call_parameters().init_peer_id;

        let statement = f!("
            INSERT OR REPLACE INTO {TABLE_NAME} VALUES ('{key}', '{value}', {timestamp}, '{peer_id}');
        ");
        println!("{}", statement);
        connection.execute(statement)
    }

    put_value_impl(key, value, timestamp).into()
}

#[fce]
pub fn get_value(key: String) -> GetValueResult {
    fn get_value_impl(key: String) -> SqliteResult<String> {
        let connection = get_connection()?;

        let mut statement = connection
            .prepare(f!("SELECT value FROM {TABLE_NAME} WHERE key = {key}"))?;

        if let State::Row = statement.next()? {
            statement.read::<String>(0)
        } else {
            Err(SqliteError { code: None, message: Some("not found".to_string()) })
        }
    }

    get_value_impl(key).into()
}

#[fce]
pub fn clear_expired(current_timestamp: u64) -> ClearExpiredResult {
    fn clear_expired_impl(current_timestamp: u64) -> SqliteResult<u64> {
        let connection = get_connection()?;

        let expired_timestamp = current_timestamp - EXPIRED_VALUE_AGE;
        connection
            .execute(f!("DELETE FROM {TABLE_NAME} WHERE timestamp <= {expired_timestamp}"))?;

        Ok(connection.changes() as u64)
    }

    clear_expired_impl(current_timestamp).into()
}

#[fce]
pub fn get_stale_records(current_timestamp: u64) -> GetStaleRecordsResult {
    fn get_stale_records_impl(current_timestamp: u64) -> SqliteResult<Vec<Record>> {
        let connection = get_connection()?;

        let expired_timestamp = current_timestamp.saturating_sub(EXPIRED_VALUE_AGE);
        let stale_timestamp = current_timestamp.saturating_sub(STALE_VALUE_AGE);

        let mut statement = connection
            .prepare(f!("SELECT key, value, peer_id FROM {TABLE_NAME} WHERE timestamp BETWEEN {expired_timestamp} AND {stale_timestamp}"))?;

        let mut result: Vec<Record> = Vec::new();
        while let State::Row = statement.next()? {
            let key = statement.read::<String>(0)?;
            let value = statement.read::<String>(1)?;
            let peer_id = statement.read::<String>(2)?;

            result.push(Record { key, value, peer_id });
        }

        Ok(result)
    }

    get_stale_records_impl(current_timestamp).into()
}