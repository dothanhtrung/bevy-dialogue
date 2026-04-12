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
    EntityEvent,
    Event,
    Handle,
    On,
    Plugin,
    Query,
    Res,
    Resource,
    Single,
    States,
    TypePath,
    With,
};
use bevy_rand::prelude::{
    EntropyPlugin,
    GlobalRng,
    WyRand,
};
use rand::RngExt;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::{
    BTreeMap,
    HashMap,
};

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
        app.add_plugins(EntropyPlugin::<WyRand>::default())
            .init_asset::<DialogueAsset>()
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
pub struct Dialogue(pub BTreeMap<u32, Vec<String>>); // state and list of dialogues

#[derive(Component)]
pub struct DialogueComponent {
    pub class: u32,
    pub current_state: u32,
}

impl DialogueComponent {
    pub fn new(class: u32) -> Self {
        Self {
            class,
            current_state: 0,
        }
    }
}

#[derive(Event)]
pub struct RequestDialogue {
    pub entity: Entity,
}

#[derive(EntityEvent, Clone)]
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
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    if let Some(dialogues) = dialogue_asset.get(&dialogue_res.0)
        && let Ok(dialogue_class) = query.get(trigger.entity)
        && let Some(messages) = dialogues.0.get(&dialogue_class.class)
        && let Some(messages_by_state) = messages.0.get(&dialogue_class.current_state)
        && !messages_by_state.is_empty()
    {
        let random_msg_idx = rng.random_range(0..messages_by_state.len());
        let dialogue = messages_by_state[random_msg_idx].clone();
        let res = NextDialogue {
            entity: trigger.entity,
            dialogue,
        };
        commands.trigger(res);
    }
}

fn update_state(
    trigger: On<DialogueStateChanged>,
    dialogue_res: Res<DialogueRes>,
    dialogue_asset: Res<Assets<DialogueAsset>>,
    mut query: Query<&mut DialogueComponent>,
) {
    if let Some(dialogues) = dialogue_asset.get(&dialogue_res.0)
        && let Ok(mut component) = query.get_mut(trigger.entity)
        && let Some(messages) = dialogues.0.get(&component.class)
    {
        if let Some(next_state) = trigger.next_state {
            for state in messages.0.keys() {
                if *state == next_state {
                    component.current_state = next_state;
                    break;
                }
            }
        } else {
            let mut found_current = false;
            for state in messages.0.keys() {
                if found_current {
                    component.current_state = *state;
                    break;
                }
                if *state == component.current_state {
                    found_current = true;
                }
            }
        }
    }
}
