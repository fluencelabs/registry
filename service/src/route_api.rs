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
use crate::misc::check_weight_result;
use crate::results::{DhtResult, GetRouteMetadataResult, RegisterRouteResult};
use crate::route::{Route, RouteInternal};
use crate::storage_impl::get_storage;
use crate::tetraplets_checkers::{check_timestamp_tetraplets, check_weight_tetraplets};
use crate::{wrapped_try, WeightResult};
use marine_rs_sdk::marine;

#[marine]
pub fn get_route_bytes(
    label: String,
    mut peer_id: Vec<String>,
    timestamp_created: u64,
    challenge: Vec<u8>,
    challenge_type: String,
) -> Vec<u8> {
    Route {
        label,
        peer_id: peer_id
            .pop()
            .unwrap_or(marine_rs_sdk::get_call_parameters().init_peer_id),
        timestamp_created,
        challenge,
        challenge_type,
        ..Default::default()
    }
    .signature_bytes()
}

#[marine]
pub fn get_route_id(label: String, peer_id: String) -> String {
    Route::get_id(&label, &peer_id)
}

/// register new route if not exists with caller peer_id, update if exists with same peer_id or return error
#[marine]
pub fn register_route(
    label: String,
    peer_id: Vec<String>,
    timestamp_created: u64,
    challenge: Vec<u8>,
    challenge_type: String,
    signature: Vec<u8>,
    pin: bool,
    weight: WeightResult,
    current_timestamp_sec: u64,
) -> RegisterRouteResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_weight_tetraplets(&call_parameters, 7, 0)?;
        check_timestamp_tetraplets(&call_parameters, 8)?;
        let peer_id = peer_id
            .get(0)
            .unwrap_or(&call_parameters.init_peer_id)
            .clone();
        check_weight_result(&peer_id, &weight)?;
        let route = Route::new(
            label,
            peer_id,
            timestamp_created,
            challenge,
            challenge_type,
            signature,
        );
        route.verify(current_timestamp_sec)?;

        let route_id = route.id.clone();
        let weight = weight.weight;
        let storage = get_storage()?;
        storage.update_route_timestamp(&route.id, current_timestamp_sec)?;
        storage.update_route(RouteInternal {
            route,
            timestamp_published: 0,
            pinned: pin,
            weight,
        })?;

        Ok(route_id)
    })
    .into()
}

#[marine]
pub fn get_route_metadata(route_id: String, current_timestamp_sec: u64) -> GetRouteMetadataResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_timestamp_tetraplets(&call_parameters, 1)?;

        let storage = get_storage()?;
        storage.update_route_timestamp(&route_id, current_timestamp_sec)?;
        storage.get_route(route_id)
    })
    .into()
}

/// Used for replication, same as register_route, but route.pinned is ignored, updates timestamp_accessed
#[marine]
pub fn republish_route(
    mut route: Route,
    weight: WeightResult,
    current_timestamp_sec: u64,
) -> DhtResult {
    wrapped_try(|| {
        let call_parameters = marine_rs_sdk::get_call_parameters();
        check_weight_tetraplets(&call_parameters, 1, 0)?;
        check_weight_result(&route.peer_id, &weight)?;
        check_timestamp_tetraplets(&call_parameters, 2)?;
        route.verify(current_timestamp_sec)?;

        // just to be sure
        route.id = Route::get_id(&route.label, &route.peer_id);

        let storage = get_storage()?;
        storage.update_route_timestamp(&route.id, current_timestamp_sec)?;
        match storage.update_route(RouteInternal {
            route,
            timestamp_published: 0,
            pinned: false,
            weight: weight.weight,
        }) {
            // we should ignore this error for republish
            Err(ServiceError::RouteAlreadyExistsNewerTimestamp(_, _)) => Ok(()),
            other => other,
        }
    })
    .into()
}
