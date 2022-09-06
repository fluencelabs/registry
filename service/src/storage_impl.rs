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

use crate::config::load_config;
use crate::defaults::DB_PATH;
use crate::error::ServiceError;
use crate::record::Record;
use crate::results::EvictStaleItem;
use marine_sqlite_connector::{Connection, Result as SqliteResult};

pub struct Storage {
    pub(crate) connection: Connection,
}

#[inline]
pub(crate) fn get_storage() -> SqliteResult<Storage> {
    marine_sqlite_connector::open(DB_PATH).map(|c| Storage { connection: c })
}

pub fn get_custom_option(value: String) -> Vec<String> {
    if value.is_empty() {
        vec![]
    } else {
        vec![value]
    }
}

pub fn from_custom_option(value: Vec<String>) -> String {
    if value.is_empty() {
        "".to_string()
    } else {
        value[0].clone()
    }
}

impl Storage {
    /// Remove expired records (based on `timestamp_created`), expired tombstones (based on `timestamp_issued`)
    /// and then expired keys without actual records
    pub fn clear_expired(
        &self,
        current_timestamp_sec: u64,
    ) -> Result<(u64, u64, u64), ServiceError> {
        let config = load_config();

        let expired_timestamp = current_timestamp_sec - config.expired_timeout;
        let deleted_tombstones = self.clear_expired_tombstones(expired_timestamp)?;
        // delete expired non-host records
        let deleted_records = self.clear_expired_records(expired_timestamp)?;
        let deleted_keys = self.clear_expired_keys(expired_timestamp)?;

        Ok((deleted_keys, deleted_records, deleted_tombstones))
    }

    pub fn evict_stale(
        &self,
        current_timestamp_sec: u64,
    ) -> Result<Vec<EvictStaleItem>, ServiceError> {
        let stale_timestamp = current_timestamp_sec - load_config().stale_timeout;

        let stale_keys = self.get_stale_keys(stale_timestamp)?;
        let mut results: Vec<EvictStaleItem> = vec![];
        for key in stale_keys.into_iter() {
            let records: Vec<Record> = self
                .get_records(key.key.id.clone(), current_timestamp_sec)?
                .into_iter()
                .map(|r| r.record)
                .collect();

            let tombstones = self.get_tombstones(key.key.id.clone(), current_timestamp_sec)?;
            results.push(EvictStaleItem {
                key: key.key,
                records,
                tombstones,
            });
        }

        Ok(results)
    }
}
