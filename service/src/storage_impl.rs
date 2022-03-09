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
    /// Remove expired values and expired empty keys.
    /// Expired means that `timestamp_created` has surpassed `expired_timeout`.
    /// Return number of keys and values removed
    pub fn clear_expired(&self, current_timestamp_sec: u64) -> Result<(u64, u64), ServiceError> {
        let config = load_config();

        let expired_timestamp = current_timestamp_sec - config.expired_timeout;
        let mut deleted_values = 0u64;
        let mut deleted_keys = 0u64;

        // delete expired non-host records
        deleted_values += self.clear_expired_records(expired_timestamp)?;
        let expired_keys = self.get_expired_routes(expired_timestamp)?;

        for key in expired_keys {
            self.delete_key(key.id)?;
            deleted_keys += self.connection.changes() as u64;
        }

        // TODO: clear expired timestamp accessed for keys
        Ok((deleted_keys, deleted_values))
    }

    /// Delete all stale keys and values except for pinned keys and host values.
    /// Stale means that `timestamp_accessed` has surpassed `stale_timeout`.
    /// Returns all deleted items
    pub fn evict_stale(
        &self,
        current_timestamp_sec: u64,
    ) -> Result<Vec<EvictStaleItem>, ServiceError> {
        let stale_timestamp = current_timestamp_sec - load_config().stale_timeout;

        let stale_keys = self.get_stale_routes(stale_timestamp)?;
        let mut key_to_delete: Vec<String> = vec![];
        let mut results: Vec<EvictStaleItem> = vec![];
        let host_id = marine_rs_sdk::get_call_parameters().host_id;
        for route in stale_keys.into_iter() {
            let records: Vec<Record> = self
                .get_records(route.route.id.clone())?
                .into_iter()
                .map(|r| r.record)
                .collect();

            if !route.pinned && !records.iter().any(|r| r.peer_id == host_id) {
                key_to_delete.push(route.route.id.clone());
            }

            results.push(EvictStaleItem {
                route: route.route,
                records,
            });
        }

        for route_id in key_to_delete {
            self.delete_key(route_id.clone())?;
            self.delete_records_by_key(route_id)?;
        }

        Ok(results)
    }
}
