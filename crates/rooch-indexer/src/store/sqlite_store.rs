// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diesel::QueryDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use tracing::log;

use crate::errors::{Context, IndexerError};
use crate::models::events::StoredEvent;
use crate::models::states::{StoredFieldState, StoredObjectState, StoredTableChangeSet};
use crate::models::transactions::StoredTransaction;
use crate::schema::{events, field_states, object_states, table_change_sets, transactions};
use crate::types::{
    IndexedEvent, IndexedFieldState, IndexedObjectState, IndexedTableChangeSet, IndexedTransaction,
};
use crate::utils::escape_sql_string;
use crate::{get_sqlite_pool_connection, SqliteConnectionPool};

#[derive(Clone)]
pub struct SqliteIndexerStore {
    pub(crate) connection_pool: SqliteConnectionPool,
}

impl SqliteIndexerStore {
    pub fn new(connection_pool: SqliteConnectionPool) -> Self {
        Self { connection_pool }
    }

    pub fn persist_or_update_object_states(
        &self,
        states: Vec<IndexedObjectState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredObjectState::from)
            .collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "('{}', '{}', {}, '{}', '{}', '{}', {}, {}, {}, {}, {})",
                    escape_sql_string(state.object_id),
                    escape_sql_string(state.owner),
                    state.flag,
                    escape_sql_string(state.value),
                    escape_sql_string(state.object_type),
                    escape_sql_string(state.state_root),
                    state.size,
                    state.tx_order,
                    state.state_index,
                    state.created_at,
                    state.updated_at,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "
                INSERT INTO object_states (object_id, owner, flag, value, object_type, state_root, size, tx_order, state_index, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (object_id) DO UPDATE SET \
                owner = excluded.owner, \
                flag = excluded.flag, \
                value = excluded.value, \
                state_root = excluded.state_root, \
                size = excluded.size, \
                tx_order = excluded.tx_order, \
                state_index = excluded.state_index, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        // // Perform multi-insert with ON CONFLICT update
        // diesel::insert_into(object_states::table)
        //     .values(states.as_slice())
        //     .on_conflict(object_states::object_id)
        //     .do_update()
        //     .set((
        //         object_states::owner.eq(excluded(object_states::owner)),
        //         object_states::flag.eq(excluded(object_states::flag)),
        //         object_states::value.eq(excluded(object_states::value)),
        //         object_states::size.eq(excluded(object_states::size)),
        //         object_states::updated_at.eq(excluded(object_states::updated_at)),
        //     ))
        //     .execute(&mut connection)
        //     .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        //     .context("Failed to write or update global states to SQLiteDB");

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                log::error!("Upsert global states Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to write or update global states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_object_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        diesel::delete(
            object_states::table.filter(object_states::object_id.eq_any(state_pks.as_slice())),
        )
        .execute(&mut connection)
        .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        .context("Failed to delete global states to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_or_update_field_states(
        &self,
        states: Vec<IndexedFieldState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredFieldState::from)
            .collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "('{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {})",
                    escape_sql_string(state.object_id),
                    escape_sql_string(state.key_hex),
                    escape_sql_string(state.key_str),
                    escape_sql_string(state.value),
                    escape_sql_string(state.key_type),
                    escape_sql_string(state.value_type),
                    state.tx_order,
                    state.state_index,
                    state.created_at,
                    state.updated_at,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "
                INSERT INTO field_states (object_id, key_hex, key_str, value, key_type, value_type, tx_order, state_index, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (object_id, key_hex) DO UPDATE SET \
                value = excluded.value, \
                value_type = excluded.value_type, \
                tx_order = excluded.tx_order, \
                state_index = excluded.state_index, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                log::error!("Upsert table states Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to write or update table states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_field_states(
        &self,
        state_pks: Vec<(String, String)>,
    ) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        // Diesel for SQLite don't support batch delete on composite primary key yet, so implements batch delete directly via raw SQL
        let values_clause = state_pks
            .into_iter()
            .map(|pk| {
                format!(
                    "('{}', '{}')",
                    escape_sql_string(pk.0),
                    escape_sql_string(pk.1),
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            "
                DELETE FROM field_states \
                WHERE (object_id, key_hex) IN ({})
            ",
            values_clause
        );

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                log::error!("Delete field states Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to delete field states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_field_states_by_object_id(
        &self,
        object_ids: Vec<String>,
    ) -> Result<(), IndexerError> {
        if object_ids.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        diesel::delete(
            field_states::table.filter(field_states::object_id.eq_any(object_ids.as_slice())),
        )
        .execute(&mut connection)
        .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        .context("Failed to delete table states by table handles to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_table_change_sets(
        &self,
        table_change_sets: Vec<IndexedTableChangeSet>,
    ) -> Result<(), IndexerError> {
        if table_change_sets.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let table_change_sets = table_change_sets
            .into_iter()
            .map(StoredTableChangeSet::from)
            .collect::<Vec<_>>();

        diesel::insert_into(table_change_sets::table)
            .values(table_change_sets.as_slice())
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write table change sets to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        if transactions.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let transactions = transactions
            .into_iter()
            .map(StoredTransaction::from)
            .collect::<Vec<_>>();

        diesel::insert_into(transactions::table)
            .values(transactions.as_slice())
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write transactions to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        if events.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let events = events
            .into_iter()
            .map(StoredEvent::from)
            .collect::<Vec<_>>();

        diesel::insert_into(events::table)
            .values(events.as_slice())
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write events to SQLiteDB")?;

        Ok(())
    }
}
