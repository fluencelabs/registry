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

use crate::defaults::{
    TRUSTED_REGISTRY_FUNCTION_NAME, TRUSTED_REGISTRY_SERVICE_ID, TRUSTED_TIMESTAMP_FUNCTION_NAME,
    TRUSTED_TIMESTAMP_SERVICE_ID, TRUSTED_WEIGHT_FUNCTION_NAME, TRUSTED_WEIGHT_SERVICE_ID,
};
use crate::error::ServiceError;
use crate::error::ServiceError::{
    InvalidSetHostValueTetraplet, InvalidTimestampTetraplet, InvalidWeightTetraplet,
};
use crate::record::Record;
use marine_rs_sdk::CallParameters;

/// Check timestamps are generated on the current host with builtin ("peer" "timestamp_sec")
pub(crate) fn check_timestamp_tetraplets(
    call_parameters: &CallParameters,
    arg_number: usize,
) -> Result<(), ServiceError> {
    let tetraplets = call_parameters
        .tetraplets
        .get(arg_number)
        .ok_or_else(|| InvalidTimestampTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    let tetraplet = tetraplets
        .get(0)
        .ok_or_else(|| InvalidTimestampTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    (tetraplet.service_id == TRUSTED_TIMESTAMP_SERVICE_ID
        && tetraplet.function_name == TRUSTED_TIMESTAMP_FUNCTION_NAME
        && tetraplet.peer_pk == call_parameters.host_id)
        .then(|| ())
        .ok_or_else(|| InvalidTimestampTetraplet(format!("{:?}", tetraplet)))
}

pub(crate) fn check_host_value_tetraplets(
    call_parameters: &CallParameters,
    arg_number: usize,
    host_value: &Record,
) -> Result<(), ServiceError> {
    let tetraplets = call_parameters
        .tetraplets
        .get(arg_number)
        .ok_or_else(|| InvalidSetHostValueTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    let tetraplet = tetraplets
        .get(0)
        .ok_or_else(|| InvalidSetHostValueTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    (tetraplet.service_id == TRUSTED_REGISTRY_SERVICE_ID
        && tetraplet.function_name == TRUSTED_REGISTRY_FUNCTION_NAME
        && tetraplet.peer_pk == host_value.peer_id)
        .then(|| ())
        .ok_or_else(|| InvalidSetHostValueTetraplet(format!("{:?}", tetraplet)))
}

pub(crate) fn check_weight_tetraplets(
    call_parameters: &CallParameters,
    arg_number: usize,
    index: usize,
) -> Result<(), ServiceError> {
    let tetraplets = call_parameters
        .tetraplets
        .get(arg_number)
        .ok_or_else(|| InvalidWeightTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    let tetraplet = tetraplets
        .get(index)
        .ok_or_else(|| InvalidWeightTetraplet(format!("{:?}", call_parameters.tetraplets)))?;
    (tetraplet.service_id == TRUSTED_WEIGHT_SERVICE_ID
        && tetraplet.function_name == TRUSTED_WEIGHT_FUNCTION_NAME
        && tetraplet.peer_pk == call_parameters.host_id)
        .then(|| ())
        .ok_or_else(|| InvalidWeightTetraplet(format!("{:?}", tetraplet)))
}
