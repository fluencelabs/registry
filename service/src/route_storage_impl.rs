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

use crate::defaults::{ROUTES_TABLE_NAME, ROUTES_TIMESTAMPS_TABLE_NAME};

use crate::error::ServiceError;
use crate::error::ServiceError::{InternalError, RouteNotExists};
use crate::route::{Route, RouteInternal};
use crate::storage_impl::Storage;
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_route_tables(&self) -> bool {
        self.connection
            .execute(f!("
            CREATE TABLE IF NOT EXISTS {ROUTES_TABLE_NAME} (
                route_id TEXT PRIMARY KEY,
                label TEXT,
                peer_id TEXT,
                timestamp_created INTEGER,
                challenge BLOB,
                challenge_type TEXT,
                signature BLOB NOT NULL,
                timestamp_published INTEGER,
                pinned INTEGER,
                weight INTEGER
            );
        "))
            .is_ok()
            && self
                .connection
                .execute(f!("
            CREATE TABLE IF NOT EXISTS {ROUTES_TIMESTAMPS_TABLE_NAME} (
                route_id TEXT PRIMARY KEY,
                timestamp_accessed INTEGER
            );
        "))
                .is_ok()
    }

    pub fn update_route_timestamp(
        &self,
        route_id: &str,
        current_timestamp_sec: u64,
    ) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!("
             INSERT OR REPLACE INTO {ROUTES_TIMESTAMPS_TABLE_NAME} VALUES (?, ?);
         "))?;

        statement.bind(1, &Value::String(route_id.to_string()))?;
        statement.bind(2, &Value::Integer(current_timestamp_sec as i64))?;
        statement.next()?;
        Ok(())
    }

    pub fn get_route(&self, route_id: String) -> Result<Route, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT route_id, label, peer_id, timestamp_created, challenge, challenge_type, signature \
                              FROM {ROUTES_TABLE_NAME} WHERE route_id = ?"
        ))?;
        statement.bind(1, &Value::String(route_id.clone()))?;

        if let State::Row = statement.next()? {
            read_route(&statement)
        } else {
            Err(RouteNotExists(route_id))
        }
    }

    pub fn write_route(&self, route: RouteInternal) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!("
             INSERT OR REPLACE INTO {ROUTES_TABLE_NAME} VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
         "))?;

        let pinned = if route.pinned { 1 } else { 0 } as i64;
        statement.bind(1, &Value::String(route.route.id))?;
        statement.bind(2, &Value::String(route.route.label))?;
        statement.bind(3, &Value::String(route.route.peer_id))?;
        statement.bind(4, &Value::Integer(route.route.timestamp_created as i64))?;
        statement.bind(5, &Value::Binary(route.route.challenge))?;
        statement.bind(6, &Value::String(route.route.challenge_type))?;
        statement.bind(7, &Value::Binary(route.route.signature))?;
        statement.bind(8, &Value::Integer(route.timestamp_published as i64))?;
        statement.bind(9, &Value::Integer(pinned))?;
        statement.bind(10, &Value::Integer(route.weight as i64))?;
        statement.next()?;
        Ok(())
    }

    pub fn update_route(&self, route: RouteInternal) -> Result<(), ServiceError> {
        if let Ok(existing_route) = self.get_route(route.route.id.clone()) {
            if existing_route.timestamp_created > route.route.timestamp_created {
                return Err(ServiceError::RouteAlreadyExistsNewerTimestamp(
                    route.route.label,
                    route.route.peer_id,
                ));
            }
        }

        self.write_route(route)
    }

    pub fn check_route_existence(&self, route_id: &str) -> Result<(), ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT EXISTS(SELECT 1 FROM {ROUTES_TABLE_NAME} WHERE route_id = ? LIMIT 1)"
        ))?;
        statement.bind(1, &Value::String(route_id.to_string()))?;

        if let State::Row = statement.next()? {
            let exists = statement.read::<i64>(0)?;
            if exists == 1 {
                Ok(())
            } else {
                Err(RouteNotExists(route_id.to_string()))
            }
        } else {
            Err(InternalError(
                "EXISTS should always return something".to_string(),
            ))
        }
    }

    pub fn get_stale_routes(
        &self,
        stale_timestamp: u64,
    ) -> Result<Vec<RouteInternal>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT route_id, label, peer_id, timestamp_created, challenge, challenge_type, signature, timestamp_published, pinned, weight \
                              FROM {ROUTES_TABLE_NAME} WHERE timestamp_published <= ?"
        ))?;
        statement.bind(1, &Value::Integer(stale_timestamp as i64))?;

        let mut stale_keys: Vec<RouteInternal> = vec![];
        while let State::Row = statement.next()? {
            stale_keys.push(read_internal_route(&statement)?);
        }

        Ok(stale_keys)
    }

    pub fn delete_key(&self, route_id: String) -> Result<(), ServiceError> {
        let mut statement = self
            .connection
            .prepare(f!("DELETE FROM {ROUTES_TABLE_NAME} WHERE route_id=?"))?;
        statement.bind(1, &Value::String(route_id.clone()))?;
        statement.next().map(drop)?;

        if self.connection.changes() == 1 {
            Ok(())
        } else {
            Err(RouteNotExists(route_id))
        }
    }

    /// not pinned only
    pub fn get_expired_routes(&self, expired_timestamp: u64) -> Result<Vec<Route>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT route_id, label, peer_id, timestamp_created, challenge, challenge_type, signature \
                              FROM {ROUTES_TABLE_NAME} WHERE timestamp_created <= ? and pinned != 1"
        ))?;
        statement.bind(1, &Value::Integer(expired_timestamp as i64))?;

        let mut expired_routes: Vec<Route> = vec![];
        while let State::Row = statement.next()? {
            let route = read_route(&statement)?;
            let timestamp_accessed = self.get_route_timestamp_accessed(&route.id)?;
            let with_host_records = self.get_host_records_count_by_key(route.id.clone())? != 0;

            if timestamp_accessed <= expired_timestamp && !with_host_records {
                expired_routes.push(route);
            }
        }

        Ok(expired_routes)
    }

    pub fn get_route_timestamp_accessed(&self, route_id: &str) -> Result<u64, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT timestamp_accessed FROM {ROUTES_TIMESTAMPS_TABLE_NAME} WHERE route_id = ?"
        ))?;
        statement.bind(1, &Value::String(route_id.to_string()))?;

        if let State::Row = statement.next()? {
            statement
                .read::<i64>(0)
                .map(|t| t as u64)
                .map_err(ServiceError::SqliteError)
        } else {
            Err(RouteNotExists(route_id.to_string()))
        }
    }
}

pub fn read_route(statement: &Statement) -> Result<Route, ServiceError> {
    Ok(Route {
        id: statement.read::<String>(0)?,
        label: statement.read::<String>(1)?,
        peer_id: statement.read::<String>(2)?,
        timestamp_created: statement.read::<i64>(3)? as u64,
        challenge: statement.read::<Vec<u8>>(4)?,
        challenge_type: statement.read::<String>(5)?,
        signature: statement.read::<Vec<u8>>(6)?,
    })
}

pub fn read_internal_route(statement: &Statement) -> Result<RouteInternal, ServiceError> {
    Ok(RouteInternal {
        route: read_route(statement)?,
        timestamp_published: statement.read::<i64>(7)? as u64,
        pinned: statement.read::<i64>(8)? != 0,
        weight: statement.read::<i64>(9)? as u32,
    })
}
