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
use crate::WeightResult;
use boolinator::Boolinator;
use fluence_keypair::PublicKey;
use libp2p_core::PeerId;
use std::convert::TryFrom;
use std::str::FromStr;

fn parse_peer_id(peer_id: String) -> Result<PeerId, ServiceError> {
    PeerId::from_str(&peer_id).map_err(|e| ServiceError::PeerIdParseError(format!("{:?}", e)))
}

pub fn extract_public_key(peer_id: String) -> Result<PublicKey, ServiceError> {
    PublicKey::try_from(
        parse_peer_id(peer_id)
            .map_err(|e| ServiceError::PublicKeyExtractionError(e.to_string()))?,
    )
    .map_err(ServiceError::PublicKeyDecodeError)
}

pub fn check_weight_result(peer_id: &str, weight: &WeightResult) -> Result<(), ServiceError> {
    (weight.success && weight.peer_id.eq(peer_id)).as_result(
        (),
        ServiceError::InvalidWeightPeerId(peer_id.to_string(), weight.peer_id.clone()),
    )
}
