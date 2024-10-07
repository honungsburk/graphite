use crate::hlc;
use crate::hlc::HLTimestamp;
use anyhow::{Context, Result};
use rusqlite::Connection;
use rusqlite::Error as RusqliteError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

pub struct EventStorage {
    conn: Connection,
}

impl EventStorage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<EventStorage> {
        let conn = Connection::open(path).context("Failed to open database")?;
        let storage = EventStorage { conn };
        storage.init()?;
        Ok(storage)
    }

    fn init(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS events (
                id BLOB PRIMARY KEY, // UUID as BLOB
                hlc_seconds INTEGER NOT NULL, // 8 Bytes
                hlc_logical INTEGER NOT NULL, // 2 Bytes
                action TEXT NOT NULL, // JSON
                actor BLOB NOT NULL, // UUID as BLOB
                version INTEGER NOT NULL
            )",
                [],
            )
            .context("Failed to Create events table")?;
        Ok(())
    }

    pub fn play(&self, f: impl FnMut(Event) -> Result<()>) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM events ORDER BY hlc_seconds, hlc_logical")
            .context("Failed to prepare SQL statement to play all events")?;

        Self::play_internal(&mut stmt, f)
    }

    pub fn play_from(&self, hlc: HLTimestamp, f: impl FnMut(Event) -> Result<()>) -> Result<()> {
        let query = format!("SELECT * FROM events WHERE hlc_seconds >= {} AND hlc_logical >= {} ORDER BY hlc_seconds, hlc_logical", hlc.seconds(), hlc.logical());
        let mut stmt = self
            .conn
            .prepare(&query)
            .context("Failed to prepare SQL statement to play subset of events")?;
        Self::play_internal(&mut stmt, f)
    }

    fn play_internal(
        stmt: &mut rusqlite::Statement,
        mut f: impl FnMut(Event) -> Result<()>,
    ) -> Result<()> {
        let rows = stmt
            .query_map([], |row| {
                let id: Uuid = row.get(0)?;
                let hlc_seconds: i64 = row.get(1)?;
                let hlc_logical: u16 = row.get(2)?;
                let action_string: String = row.get(3)?;
                let action: Action = serde_json::from_str(&action_string)
                    .map_err(|e| RusqliteError::ToSqlConversionFailure(Box::new(e)))?;
                let actor: Uuid = row.get(4)?;
                let version: u32 = row.get(5)?;

                Ok(Event {
                    id,
                    hlc: HLTimestamp::new(hlc_seconds, hlc_logical),
                    action,
                    actor,
                    version,
                })
            })
            .context("Failed to play events")?;

        for event in rows {
            let e = event.context("Failed to get event")?;
            f(e).context("Failed to execute on event")?;
        }

        Ok(())
    }

    pub fn record(&self, envelope: Event) -> Result<()> {
        let action =
            serde_json::to_string(&envelope.action).context("Failed to serialize to JSON")?;

        self.conn
            .execute(
                "INSERT INTO events (id, hlc_seconds, hlc_logical, action, actor, version)
      VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    envelope.id,
                    envelope.hlc.seconds(),
                    envelope.hlc.logical(),
                    action,
                    envelope.actor,
                    envelope.version,
                ],
            )
            .context("Failed to insert an event")?;

        Ok(())
    }

    pub fn record_batch(&mut self, envelopes: Vec<Event>) -> Result<()> {
        let tx = self
            .conn
            .transaction()
            .context("Failed to open a transaction")?;
        for envelope in envelopes {
            let action =
                serde_json::to_string(&envelope.action).context("Failed to serialize to JSON")?;
            tx.execute(
                "INSERT INTO events (id, hlc_seconds, hlc_logical, action, actor, version)
              VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    envelope.id,
                    envelope.hlc.seconds(),
                    envelope.hlc.logical(),
                    action,
                    envelope.actor,
                    envelope.version,
                ],
            )
            .context("Failed to insert an event")?;
        }
        tx.commit().context("Failed to commit batch of events")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Datum {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(i64),
    Entity(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    CreateEntity {
        id: Uuid,
    },
    AddFact {
        subject: Uuid,
        predicate: String,
        datum: Datum,
    },
    RemoveFact {
        subject: Uuid,
        predicate: String,
    },
    DeleteEntity {
        id: Uuid,
    },
    Transaction {
        actions: Vec<Action>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    id: Uuid,         // The unique identifier of the event
    hlc: HLTimestamp, // Hybrid Logical timestamp
    action: Action,   // The event that was performed
    actor: Uuid, // The actor who performed the event, e.g. a user. should be present amoung the entities
    version: u32, // Event version
}

pub struct EventCreator {
    actor: Uuid,
    hlc: hlc::State<fn() -> i64>,
}

impl EventCreator {
    pub fn new(actor: Uuid, hlt: HLTimestamp) -> EventCreator {
        let mut hlc = hlc::State::new();
        hlc.update(hlt); // Update the HLC with the given timestamp to have the correct time
        EventCreator { actor, hlc }
    }

    pub fn create(&mut self, action: Action) -> Event {
        let hlc = self.hlc.get_time();
        Event {
            id: Uuid::new_v4(),
            hlc,
            action,
            actor: self.actor,
            version: 0,
        }
    }
}
