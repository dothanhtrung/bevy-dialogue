mod asset_loader;

use crate::asset_loader::RonAssetLoader;
use bevy::prelude::{
    App,
    Asset,
    AssetApp,
    Assets,
    Commands,
    Component,
    Deref,
    DerefMut,
    Entity,
    Event,
    Handle,
    On,
    Plugin,
    Query,
    Res,
    Resource,
    States,
    TypePath,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct DialoguePlugin<T>
where
    T: States,
{
    pub states: Vec<T>,
}

impl<T> DialoguePlugin<T>
where
    T: States,
{
    pub fn new(states: Vec<T>) -> Self {
        Self { states }
    }
    pub fn any() -> Self {
        Self::new(Vec::new())
    }
}

#[derive(States, Clone, Debug, Hash, Eq, PartialEq)]
pub enum DummyState {}

pub struct DialoguePluginAnyState;

impl DialoguePluginAnyState {
    pub fn any() -> DialoguePlugin<DummyState> {
        DialoguePlugin::new(Vec::new())
    }
}

impl<T> Plugin for DialoguePlugin<T>
where
    T: States,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<DialogueAsset>()
            .init_asset_loader::<RonAssetLoader<DialogueAsset>>()
            .insert_resource(DialogueRes::default())
            .add_observer(find_dialogue)
            .add_observer(update_state);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct DialogueRes(pub Handle<DialogueAsset>);

#[derive(Asset, TypePath, Serialize, Deserialize, Default)]
pub struct DialogueAsset(pub HashMap<u32, Dialogue>); // character classes and their dialogues

#[derive(Serialize, Deserialize, Deref, DerefMut)]
pub struct Dialogue(pub HashMap<u32, Vec<String>>); // state and list of dialogues

#[derive(Component)]
pub struct DialogueComponent {
    pub class: u32,
    pub current_state: u32,
}

#[derive(Event)]
pub struct RequestDialogue {
    pub entity: Entity,
}

#[derive(Event, Clone)]
pub struct NextDialogue {
    pub entity: Entity,
    pub dialogue: String,
}

#[derive(Event)]
pub struct DialogueStateChanged {
    pub entity: Entity,
    pub next_state: Option<u32>,
}

fn find_dialogue(
    trigger: On<RequestDialogue>,
    mut commands: Commands,
    dialogue_res: Res<DialogueRes>,
    dialogue_asset: Res<Assets<DialogueAsset>>,
    query: Query<&DialogueComponent>,
) {
    if let Some(dialogues) = dialogue_asset.get(&dialogue_res.0) {
        if let Ok(dialogue_class) = query.get(trigger.entity) {
            if let Some(messages) = dialogues.0.get(&dialogue_class.class) {
                if let Some(current_messages) = messages.0.get(&dialogue_class.current_state) {
                    if let Some(message) = fastrand::choice(current_messages.as_slice()) {
                        let res = NextDialogue {
                            entity: trigger.entity,
                            dialogue: message.to_string(),
                        };
                        commands.trigger(res);
                    }
                }
            }
        }
    }
}

fn update_state(
    trigger: On<DialogueStateChanged>,
    dialogue_res: Res<DialogueRes>,
    dialogue_asset: Res<Assets<DialogueAsset>>,
    mut query: Query<&mut DialogueComponent>,
) {
    if let Some(dialogues) = dialogue_asset.get(&dialogue_res.0) {
        if let Ok(mut component) = query.get_mut(trigger.entity) {
            if let Some(messages) = dialogues.0.get(&component.class) {
                if let Some(next_state) = trigger.next_state {
                    for state in messages.0.keys() {
                        if *state == next_state {
                            component.current_state = next_state;
                        }
                    }
                } else {
                    let mut found_current = false;
                    for state in messages.0.keys() {
                        if found_current {
                            component.current_state = *state;
                        }
                        if *state == component.current_state {
                            found_current = true;
                        }
                    }
                }
            }
        }
    }
}
