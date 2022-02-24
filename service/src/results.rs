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

use crate::error::ServiceError;
use crate::key::Key;
use crate::record::Record;
use marine_rs_sdk::marine;

#[marine]
#[derive(Debug)]
pub struct DhtResult {
    pub success: bool,
    pub error: String,
}

impl From<Result<(), ServiceError>> for DhtResult {
    fn from(result: Result<(), ServiceError>) -> Self {
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
pub struct RegisterKeyResult {
    pub success: bool,
    pub error: String,
    pub key_id: String,
}

impl From<Result<String, ServiceError>> for RegisterKeyResult {
    fn from(result: Result<String, ServiceError>) -> Self {
        match result {
            Ok(key_id) => Self {
                success: true,
                error: "".to_string(),
                key_id,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                key_id: "".to_string(),
            },
        }
    }
}

#[marine]
#[derive(Debug)]
pub struct GetValuesResult {
    pub success: bool,
    pub error: String,
    pub result: Vec<Record>,
}

impl From<Result<Vec<Record>, ServiceError>> for GetValuesResult {
    fn from(result: Result<Vec<Record>, ServiceError>) -> Self {
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

impl From<Result<(u64, u64), ServiceError>> for ClearExpiredResult {
    fn from(result: Result<(u64, u64), ServiceError>) -> Self {
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

impl From<Result<Vec<Record>, ServiceError>> for GetStaleRecordsResult {
    fn from(result: Result<Vec<Record>, ServiceError>) -> Self {
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
pub struct GetKeyMetadataResult {
    pub success: bool,
    pub error: String,
    pub key: Key,
}

impl From<Result<Key, ServiceError>> for GetKeyMetadataResult {
    fn from(result: Result<Key, ServiceError>) -> Self {
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

impl From<Result<u64, ServiceError>> for RepublishValuesResult {
    fn from(result: Result<u64, ServiceError>) -> Self {
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

impl From<Result<Vec<EvictStaleItem>, ServiceError>> for EvictStaleResult {
    fn from(result: Result<Vec<EvictStaleItem>, ServiceError>) -> Self {
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
pub struct PutHostRecordResult {
    pub success: bool,
    pub error: String,
    pub value: Vec<Record>,
}

impl From<Result<Record, ServiceError>> for PutHostRecordResult {
    fn from(result: Result<Record, ServiceError>) -> Self {
        match result {
            Ok(result) => Self {
                success: true,
                error: "".to_string(),
                value: vec![result],
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                value: vec![],
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

impl From<Result<Vec<Record>, ServiceError>> for MergeResult {
    fn from(result: Result<Vec<Record>, ServiceError>) -> Self {
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
