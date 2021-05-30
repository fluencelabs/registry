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

use fluence::marine;
use marine_sqlite_connector::Result as SqliteResult;

#[marine]
#[derive(Debug)]
pub struct DhtResult {
    pub success: bool,
    pub error: String,
}

impl From<SqliteResult<()>> for DhtResult {
    fn from(result: SqliteResult<()>) -> Self {
        match result {
            Ok(_) => Self {
                success: true,
                error: "".to_string(),
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
            },
        }
    }
}

#[marine]
#[derive(Debug)]
pub struct Record {
    pub value: String,
    pub peer_id: String,
    pub set_by: String,
    pub relay_id: Vec<String>,
    pub service_id: Vec<String>,
    pub timestamp_created: u64,
    pub weight: u32,
}

#[marine]
#[derive(Debug)]
pub struct GetValuesResult {
    pub success: bool,
    pub error: String,
    pub result: Vec<Record>,
}

impl From<SqliteResult<Vec<Record>>> for GetValuesResult {
    fn from(result: SqliteResult<Vec<Record>>) -> Self {
        match result {
            Ok(result) => Self {
                success: true,
                error: "".to_string(),
                result,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                result: vec![],
            },
        }
    }
}

#[marine]
#[derive(Debug)]
pub struct ClearExpiredResult {
    pub success: bool,
    pub error: String,
    pub count_keys: u64,
    pub count_values: u64,
}

impl From<SqliteResult<(u64, u64)>> for ClearExpiredResult {
    fn from(result: SqliteResult<(u64, u64)>) -> Self {
        match result {
            Ok((keys, values)) => Self {
                success: true,
                count_keys: keys,
                count_values: values,
                error: "".to_string(),
            },
            Err(err) => Self {
                success: false,
                count_keys: 0,
                count_values: 0,
                error: err.to_string(),
            },
        }
    }
}

#[marine]
#[derive(Debug)]
pub struct GetStaleRecordsResult {
    pub success: bool,
    pub error: String,
    pub result: Vec<Record>,
}

impl From<SqliteResult<Vec<Record>>> for GetStaleRecordsResult {
    fn from(result: SqliteResult<Vec<Record>>) -> Self {
        match result {
            Ok(result) => Self {
                success: true,
                error: "".to_string(),
                result,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                result: vec![],
            },
        }
    }
}

#[marine]
#[derive(Default, Clone)]
pub struct Key {
    pub key: String,
    pub peer_id: String,
    pub timestamp_created: u64,
    pub pinned: bool,
    pub weight: u32,
}

#[marine]
pub struct GetKeyMetadataResult {
    pub success: bool,
    pub error: String,
    pub key: Key,
}

impl From<SqliteResult<Key>> for GetKeyMetadataResult {
    fn from(result: SqliteResult<Key>) -> Self {
        match result {
            Ok(key) => Self {
                success: true,
                error: "".to_string(),
                key,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                key: Key::default(),
            },
        }
    }
}

#[marine]
pub struct RepublishValuesResult {
    pub success: bool,
    pub error: String,
    pub updated: u64,
}

impl From<SqliteResult<u64>> for RepublishValuesResult {
    fn from(result: SqliteResult<u64>) -> Self {
        match result {
            Ok(count) => Self {
                success: true,
                error: "".to_string(),
                updated: count,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                updated: 0,
            },
        }
    }
}

#[marine]
pub struct EvictStaleItem {
    pub key: Key,
    pub records: Vec<Record>,
}

#[marine]
pub struct EvictStaleResult {
    pub success: bool,
    pub error: String,
    pub results: Vec<EvictStaleItem>,
}

impl From<SqliteResult<Vec<EvictStaleItem>>> for EvictStaleResult {
    fn from(result: SqliteResult<Vec<EvictStaleItem>>) -> Self {
        match result {
            Ok(results) => Self {
                success: true,
                error: "".to_string(),
                results,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                results: vec![],
            },
        }
    }
}

#[marine]
#[derive(Debug)]
pub struct MergeResult {
    pub success: bool,
    pub error: String,
    pub result: Vec<Record>,
}

impl From<SqliteResult<Vec<Record>>> for MergeResult {
    fn from(result: SqliteResult<Vec<Record>>) -> Self {
        match result {
            Ok(result) => Self {
                success: true,
                error: "".to_string(),
                result,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                result: vec![],
            },
        }
    }
}
