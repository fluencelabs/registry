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
use marine_sqlite_connector::Error as SqliteError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum ServiceError {
    #[error("Internal Sqlite error: {0}")]
    SqliteError(
        #[from]
        #[source]
        SqliteError,
    ),
    #[error("Requested key {0} does not exist")]
    KeyNotExists(String),
    #[error("Key {0} already exists with different peer_id")]
    KeyAlreadyExists(String),
    #[error("Values limit for key {0} is exceeded")]
    ValuesLimitExceeded(String),
    #[error("Host value for key {0} not found ")]
    HostValueNotFound(String),
    #[error("Invalid set_host_value result: success is false or value is missing")]
    InvalidSetHostValueResult,
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error(
        "Invalid timestamp tetraplet: you should use host peer.timestamp_sec to pass timestamp: {0}"
    )]
    InvalidTimestampTetraplet(String),
    #[error(
        "Invalid set_host_value tetraplet: you should use put_host_value to pass set_host_value: {0}"
    )]
    InvalidSetHostValueTetraplet(String),
}
