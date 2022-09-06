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
use fluence_keypair::error::DecodingError;
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
    #[error("Key {0} for {1} peer_id already exists with newer timestamp")]
    KeyAlreadyExistsNewerTimestamp(String, String),
    #[error("Values limit for key_d {0} is exceeded")]
    ValuesLimitExceeded(String),
    #[error("Host value for key_id {0} not found ")]
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
    #[error(
        "Invalid weight tetraplet: you should use host trust-graph.get_weight to pass weight: {0}"
    )]
    InvalidWeightTetraplet(String),
    #[error("Invalid weight peer_id: expected {0}, found {1}")]
    InvalidWeightPeerId(String, String),
    #[error("Invalid signature for key_id {0}; label {1} and peer_id {2}: {3}")]
    InvalidKeySignature(
        String,
        String,
        String,
        #[source] fluence_keypair::error::VerificationError,
    ),
    #[error("Invalid record metadata signature for key_id {0} and issued by {1}: {2}")]
    InvalidRecordMetadataSignature(
        String,
        String,
        #[source] fluence_keypair::error::VerificationError,
    ),
    #[error("Invalid record signature for key_id {0} and issued by {1}: {2}")]
    InvalidRecordSignature(
        String,
        String,
        #[source] fluence_keypair::error::VerificationError,
    ),
    #[error("Invalid tombstone signature for key_id {0} and issued by {1}: {2}")]
    InvalidTombstoneSignature(
        String,
        String,
        #[source] fluence_keypair::error::VerificationError,
    ),
    #[error("Key can't be registered in the future")]
    InvalidKeyTimestamp,
    #[error("Record metadata can't be issued in the future")]
    InvalidRecordMetadataTimestamp,
    #[error("Record can't be registered in the future")]
    InvalidRecordTimestamp,
    #[error("Tombstone can't be issued in the future")]
    InvalidTombstoneTimestamp,
    #[error("Records to publish should belong to one key id")]
    RecordsPublishingError,
    #[error("Tombstones to publish should belong to one key id")]
    TombstonesPublishingError,
    #[error("peer id parse error: {0}")]
    PeerIdParseError(String),
    #[error("public key extraction from peer id failed: {0}")]
    PublicKeyExtractionError(String),
    #[error("{0}")]
    PublicKeyDecodeError(
        #[from]
        #[source]
        DecodingError,
    ),
    #[error("Weight for record with peer_id {0} and set_by {1} is missing ")]
    MissingRecordWeight(String, String),
    #[error("merge_keys: keys argument is empty")]
    KeysArgumentEmpty,
}
