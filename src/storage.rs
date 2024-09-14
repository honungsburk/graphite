use uuid::Uuid;

struct Storage {}

enum DataTypes {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(i64),
    Entity(Uuid),
}

enum Event {
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
        events: Vec<Event>,
    },
}

struct Envelope {
    logical_timestamp: u64, // Logical timestamp
    event_type: Event,      // The type of event
    actor: Uuid, // The actor who performed the event, e.g. a user. should be present amoung the entities
    version: u64, // Envelope version
}
