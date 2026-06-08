use bevy::{
    prelude::{
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
    },
};
use bevy_rand::prelude::{
    EntropyPlugin,
    GlobalRng,
    WyRand,
};
use bevy_support_misc::{
    bincode_asset_loader::BincodeLoaderPlugin,
    ron_asset_loader::RonLoaderPlugin,
};
use isolang::Language;
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
        if !app.is_plugin_added::<EntropyPlugin<WyRand>>() {
            app.add_plugins(EntropyPlugin::<WyRand>::default());
        };
        app.add_plugins((
            RonLoaderPlugin::<DialogueAsset>::default(),
            BincodeLoaderPlugin::<DialogueAsset>::default(),
        ))
        .init_asset::<DialogueAsset>()
        .insert_resource(DialogueRes::default())
        .add_observer(find_dialogue)
        .add_observer(update_state);
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Dialogue {
    #[serde(default)]
    pub contents: BTreeMap<Language, String>,
    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    pub affects: BTreeMap<u64, u64>,
}

#[derive(Resource, Default)]
pub struct DialogueRes {
    pub dialogues: Handle<DialogueAsset>,
}

/// List of dialogues by character kind and by state
#[derive(Asset, TypePath, Serialize, Deserialize, Default)]
pub struct DialogueAsset {
    pub dialogues: HashMap<u64, BTreeMap<u64, Vec<Dialogue>>>,
    pub class_name_map: HashMap<u64, String>,
    pub state_name_map: HashMap<u64, String>,
}

#[derive(Component, Default, Clone)]
pub struct DialogueComponent {
    pub class: u64,
    pub state: u64,
    pub dialogue: usize,
}

impl DialogueComponent {
    pub fn new(class: u64, state: u64) -> Self {
        Self {
            class,
            state,
            dialogue: 0,
        }
    }
}

#[derive(EntityEvent)]
pub struct RequestDialogue {
    #[entity_event]
    pub entity: Entity,
    #[event_target]
    pub to: Option<Entity>,
}

#[derive(EntityEvent, Clone)]
pub struct NextDialogue {
    pub entity: Entity,
    pub dialogue: Dialogue,
}

#[derive(Event)]
pub struct DialogueStateChanged {
    pub entity: Entity,
    pub next_state: Option<u64>,
}

fn find_dialogue(
    trigger: On<RequestDialogue>,
    mut commands: Commands,
    dialogue_res: Res<DialogueRes>,
    dialogue_asset: Res<Assets<DialogueAsset>>,
    mut query: Query<&mut DialogueComponent>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let Ok(character) = query.get(trigger.entity) else {
        return;
    };
    let Some(dialogue_asset) = dialogue_asset.get(&dialogue_res.dialogues) else {
        return;
    };

    let affect_state = if let Some(target_entity) = trigger.to
        && let Ok(target) = query.get(target_entity)
        && let Some(target_states) = dialogue_asset.dialogues.get(&target.class)
        && let Some(target_dialogues) = target_states.get(&target.state)
        && let Some(target_dialogue) = target_dialogues.get(target.dialogue)
        && let Some(affect_state) = target_dialogue.affects.get(&character.class)
    {
        Some(*affect_state)
    } else {
        None
    };

    if let Some(dialogues) = dialogue_asset.dialogues.get(&character.class) {
        let character_state = affect_state.unwrap_or(character.state);
        if let Some(dialogue) = dialogues.get(&character_state)
            && !dialogue.is_empty()
        {
            let random_msg_idx = rng.random_range(0..dialogue.len());
            let dialogue = dialogue[random_msg_idx].clone();
            let res = NextDialogue {
                entity: trigger.entity,
                dialogue,
            };
            commands.trigger(res);

            if let Ok(mut character) = query.get_mut(trigger.entity) {
                character.state = character_state;
                character.dialogue = random_msg_idx;
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
    if let Some(dialogue_asset) = dialogue_asset.get(&dialogue_res.dialogues)
        && let Ok(mut npc) = query.get_mut(trigger.entity)
        && let Some(dialogues) = dialogue_asset.dialogues.get(&npc.class)
    {
        if let Some(next_state) = trigger.next_state {
            for state in dialogues.keys() {
                if *state == next_state {
                    npc.state = next_state;
                    break;
                }
            }
        } else {
            let mut found_current = false;
            for state in dialogues.keys() {
                if found_current {
                    npc.state = *state;
                    break;
                }
                if *state == npc.state {
                    found_current = true;
                }
            }
            // If current state is not in the state list
            if !found_current && let Some((first_state, _)) = dialogues.first_key_value() {
                npc.state = *first_state;
            }
        }
    }
}
