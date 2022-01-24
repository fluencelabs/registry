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

use crate::defaults::DB_PATH;
use marine_sqlite_connector::{Connection, Result as SqliteResult};

#[inline]
pub(crate) fn get_connection() -> SqliteResult<Connection> {
    marine_sqlite_connector::open(DB_PATH)
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
