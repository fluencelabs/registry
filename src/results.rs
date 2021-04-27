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

use fluence::fce;
use fce_sqlite_connector::Result as SqliteResult;

#[fce]
#[derive(Debug)]
pub struct PutValueResult {
    pub success: bool,
    pub error: String,
}

impl From<SqliteResult<()>> for PutValueResult {
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

#[fce]
#[derive(Debug)]
pub struct GetValueResult {
    pub success: bool,
    pub result: String,
}

impl From<SqliteResult<String>> for GetValueResult {
    fn from(result: SqliteResult<String>) -> Self {
        match result {
            Ok(result) => Self {
                success: true,
                result,
            },
            Err(err) => Self {
                success: false,
                result: err.to_string(),
            },
        }
    }
}

#[fce]
#[derive(Debug)]
pub struct ClearExpiredResult {
    pub success: bool,
    pub error: String,
    pub count: u64,
}

impl From<SqliteResult<u64>> for ClearExpiredResult {
    fn from(result: SqliteResult<u64>) -> Self {
        match result {
            Ok(result) => Self {
                success: true,
                count: result,
                error: "".to_string(),
            },
            Err(err) => Self {
                success: false,
                count: 0,
                error: err.to_string(),
            },
        }
    }
}

#[fce]
#[derive(Debug)]
pub struct Record {
    pub key: String,
    pub value: String,
    pub peer_id: String,
}

#[fce]
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