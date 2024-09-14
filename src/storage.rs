use crate::hlc;
use crate::hlc::HLTimestamp;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

struct Storage {}

impl Storage {
    fn new() -> Storage {
        Storage {}
    }

    fn play() {
        // Play all events in order. Returns an iterator that must be consumed.
        todo!("Implement play")
    }

    fn play_from(hlc: HLTimestamp) {
        // Play all events in order starting from the given timestamp. Returns an iterator that must be consumed.
        todo!("Implement play_from")
    }

    fn record(envelope: Event) {
        // Record an event
        todo!("Implement record")
    }

    fn record_batch(envelopes: Vec<Event>) {
        // Record a batch of events
        todo!("Implement record_batch")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum DataTypes {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(i64),
    Entity(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Action {
    CreateEntity {
        id: Uuid,
    },
    AddFact {
        subject: Uuid,
        predicate: String,
        datum: DataTypes,
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
struct Event {
    id: Uuid,         // The unique identifier of the event
    hlc: HLTimestamp, // Hybrid Logical timestamp
    action: Action,   // The event that was performed
    actor: Uuid, // The actor who performed the event, e.g. a user. should be present amoung the entities
    version: u32, // Event version
}

struct EventCreator {
    actor: Uuid,
    hlc: hlc::State<fn() -> i64>,
}

impl EventCreator {
    fn new(actor: Uuid, hlt: HLTimestamp) -> EventCreator {
        let mut hlc = hlc::State::new();
        hlc.update(hlt); // Update the HLC with the given timestamp to have the correct time
        EventCreator { actor, hlc }
    }

    fn create(&mut self, action: Action) -> Event {
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
