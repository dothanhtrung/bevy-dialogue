mod ron_loader;

use crate::ron_loader::RonAssetLoader;
use bevy::prelude::{
    App,
    Asset,
    AssetApp,
    Assets,
    Commands,
    Component,
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

#[derive(Resource, Default)]
pub struct DialogueRes {
    pub dialogues: Handle<DialogueAsset>,
    pub id_map: Handle<DialogueIdMap>,
}

/// List of dialogues by character kind and by state
#[derive(Asset, TypePath, Serialize, Deserialize, Default)]
pub struct DialogueAsset(pub HashMap<u32, BTreeMap<u32, Vec<String>>>);

/// Map from character kind or state to number id
#[derive(Asset, TypePath, Serialize, Deserialize, Default)]
pub struct DialogueIdMap(pub HashMap<String, u32>);

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
    if let Some(dialogue_asset) = dialogue_asset.get(&dialogue_res.dialogues)
        && let Ok(npc) = query.get(trigger.entity)
        && let Some(dialogues) = dialogue_asset.0.get(&npc.class)
        && let Some(dialogue) = dialogues.get(&npc.current_state)
        && !dialogue.is_empty()
    {
        let random_msg_idx = rng.random_range(0..dialogue.len());
        let dialogue = dialogue[random_msg_idx].clone();
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
    if let Some(dialogue_asset) = dialogue_asset.get(&dialogue_res.dialogues)
        && let Ok(mut npc) = query.get_mut(trigger.entity)
        && let Some(dialogues) = dialogue_asset.0.get(&npc.class)
    {
        if let Some(next_state) = trigger.next_state {
            for state in dialogues.keys() {
                if *state == next_state {
                    npc.current_state = next_state;
                    break;
                }
            }
        } else {
            let mut found_current = false;
            for state in dialogues.keys() {
                if found_current {
                    npc.current_state = *state;
                    break;
                }
                if *state == npc.current_state {
                    found_current = true;
                }
            }
        }
    }
}
