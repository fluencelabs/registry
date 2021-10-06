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

use serde::{Deserialize, Serialize};
use std::fs;

use crate::defaults::{
    CONFIG_FILE, DEFAULT_EXPIRED_HOST_VALUE_AGE, DEFAULT_EXPIRED_VALUE_AGE, DEFAULT_STALE_VALUE_AGE,
};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub expired_timeout: u64,
    pub stale_timeout: u64,
    pub host_expired_timeout: u64,
}

pub fn write_config(config: Config) {
    fs::write(CONFIG_FILE, toml::to_string(&config).unwrap()).unwrap();
}

pub fn load_config() -> Config {
    let file_content = fs::read_to_string(CONFIG_FILE).unwrap();
    let config: Config = toml::from_str(&file_content).unwrap();
    config
}

pub fn create_config() {
    if fs::metadata(CONFIG_FILE).is_err() {
        write_config(Config {
            expired_timeout: DEFAULT_EXPIRED_VALUE_AGE,
            stale_timeout: DEFAULT_STALE_VALUE_AGE,
            host_expired_timeout: DEFAULT_EXPIRED_HOST_VALUE_AGE,
        });
    }
}
