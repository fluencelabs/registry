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

// TODO: sanitize tables' names in SQL expressions
pub static KEYS_TABLE_NAME: &str = "dht_keys";
pub static VALUES_TABLE_NAME: &str = "dht_values";
pub static CONFIG_FILE: &str = "/tmp/Config.toml";
pub static DB_PATH: &str = "/tmp/dht.db";
pub static DEFAULT_STALE_VALUE_AGE: u64 = 60 * 60;
pub static DEFAULT_EXPIRED_VALUE_AGE: u64 = 24 * 60 * 60;
pub static DEFAULT_EXPIRED_HOST_VALUE_AGE: u64 = 10 * DEFAULT_EXPIRED_VALUE_AGE;
pub static VALUES_LIMIT: usize = 32;

pub static TRUSTED_TIMESTAMP_SERVICE_ID: &str = "peer";
pub static TRUSTED_TIMESTAMP_FUNCTION_NAME: &str = "timestamp_sec";
