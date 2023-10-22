use bevy_app::App;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::EntityMut;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::serialize::writer::WriteBuffer;
use crate::{BitSerializable, Protocol, ReplicationSend};

// client writes an Enum containing all their message type
// each message must derive message

// that big enum will implement MessageProtocol via a proc macro
// TODO: remove the extra  Serialize + DeserializeOwned + Clone  bounds
pub trait ComponentProtocol:
    BitSerializable + Serialize + DeserializeOwned + ComponentBehaviour
{
    fn add_systems<P: Protocol, R: ReplicationSend<P>>(app: &mut App);
    // fn insert(self, entity: &mut EntityMut);
}

pub trait Replicable: Component + Clone {}

/// Trait to delegate a method from the ComponentProtocol enum to the inner Component type
#[enum_delegate::register]
pub trait ComponentBehaviour {
    /// Insert the component for an entity
    fn insert(self, entity: &mut EntityMut);
}

impl<T: Component> ComponentBehaviour for T {
    fn insert(self, entity: &mut EntityMut) {
        entity.insert(self);
    }
}

pub trait ComponentProtocolKind: BitSerializable + Serialize + DeserializeOwned {}
