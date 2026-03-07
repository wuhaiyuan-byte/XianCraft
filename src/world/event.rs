use std::any::{Any, TypeId};
use std::collections::HashMap;

// --- Event Structs ---

pub struct PlayerAttackEvent {
    pub attacker_id: u64,
    pub defender_id: u64,
}

pub struct PlayerKilledEvent {
    pub victim_id: u64,
    pub killer_id: u64,
}

pub struct SkillPerformedEvent {
    pub player_id: u64,
    pub skill_name: String,
}

// --- Event Trait ---

pub trait Event: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

impl Event for PlayerAttackEvent {
    fn as_any(&self) -> &dyn Any { self }
}
impl Event for PlayerKilledEvent {
    fn as_any(&self) -> &dyn Any { self }
}
impl Event for SkillPerformedEvent {
    fn as_any(&self) -> &dyn Any { self }
}

// --- Event Handler ---

pub trait EventHandler: Send + Sync {
    fn handle(&mut self, event: &dyn Event);
}

// --- Event Bus ---

pub type Subscriber = Box<dyn EventHandler>;

pub struct EventBus {
    subscribers: HashMap<TypeId, Vec<Subscriber>>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe<E: Event + 'static>(&mut self, handler: Subscriber) {
        let event_type_id = TypeId::of::<E>();
        self.subscribers
            .entry(event_type_id)
            .or_default()
            .push(handler);
    }

    pub fn post<E: Event + 'static>(&mut self, event: E) {
        let event_type_id = TypeId::of::<E>();
        if let Some(handlers) = self.subscribers.get_mut(&event_type_id) {
            for handler in handlers {
                handler.handle(&event);
            }
        }
    }
}
