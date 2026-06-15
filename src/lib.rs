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
    bin_asset_loader::BinLoaderPlugin,
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
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin
{
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntropyPlugin<WyRand>>() {
            app.add_plugins(EntropyPlugin::<WyRand>::default());
        };
        app.add_plugins((
            RonLoaderPlugin::<DialogueAsset>::default(),
            BinLoaderPlugin::<DialogueAsset>::default(),
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
    pub dialogues: Vec<Handle<DialogueAsset>>,
    /// Variable to be replaced in the dialogue
    pub variables: HashMap<String, String>,
}

/// List of dialogues by character kind and by state
#[derive(Asset, TypePath, Serialize, Deserialize, Default)]
pub struct DialogueAsset {
    pub dialogues: HashMap<u64, BTreeMap<u64, Vec<Dialogue>>>,
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

#[derive(Default)]
pub enum RequestType {
    #[default]
    /// Return a random dialogue of current state
    Random,
    /// Return all dialogues of current state
    All,
}

#[derive(EntityEvent)]
pub struct RequestDialogue {
    #[entity_event]
    pub entity: Entity,
    /// The target whom character talk to
    pub to: Option<Entity>,
    pub request_type: RequestType,
    /// Request exactly a dialogue by index. This is often used when select choice
    pub request_index: Option<usize>,
}

impl RequestDialogue {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            to: None,
            request_type: RequestType::default(),
            request_index: None,
        }
    }

    pub fn to(entity: Entity, to: Entity) -> Self {
        Self {
            entity,
            to: Some(to),
            request_type: RequestType::default(),
            request_index: None,
        }
    }
}

#[derive(EntityEvent, Clone)]
pub struct NextDialogue {
    pub entity: Entity,
    pub dialogues: Vec<Dialogue>,
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
    for handle in dialogue_res.dialogues.iter() {
        let Some(dialogue_asset) = dialogue_asset.get(handle) else {
            continue;
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

        if let Some(states) = dialogue_asset.dialogues.get(&character.class) {
            let character_state = affect_state.unwrap_or(character.state);
            if let Some(dialogues) = states.get(&character_state) {
                if !dialogues.is_empty() {
                    if let Ok(mut character) = query.get_mut(trigger.entity) {
                        character.state = character_state;
                    }

                    let mut ret_dialogues = Vec::new();
                    match trigger.request_type {
                        RequestType::All => {
                            if let Some(index) = trigger.request_index
                                && let Some(dialog) = dialogues.get(index)
                            {
                                let mut dialog = dialog.clone();
                                for (_, content) in dialog.contents.iter_mut() {
                                    *content = replace_templates(content, &dialogue_res.variables)
                                }
                                ret_dialogues.push(dialog);
                                if let Ok(mut character) = query.get_mut(trigger.entity) {
                                    character.dialogue = index;
                                }
                            } else {
                                ret_dialogues = dialogues.clone();
                            }
                        }
                        RequestType::Random => {
                            let random_msg_idx = rng.random_range(0..dialogues.len());
                            let mut dialog = dialogues[random_msg_idx].clone();
                            for (_, content) in dialog.contents.iter_mut() {
                                *content = replace_templates(content, &dialogue_res.variables)
                            }

                            ret_dialogues.push(dialog);

                            if let Ok(mut character) = query.get_mut(trigger.entity) {
                                character.dialogue = random_msg_idx;
                            }
                        }
                    }

                    let res = NextDialogue {
                        entity: trigger.entity,
                        dialogues: ret_dialogues,
                    };
                    commands.trigger(res);
                }
                break;
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
    for handle in dialogue_res.dialogues.iter() {
        let Some(dialogue_asset) = dialogue_asset.get(handle) else {
            continue;
        };
        if let Ok(mut npc) = query.get_mut(trigger.entity)
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

            break;
        }
    }
}

fn replace_templates(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut cursor = template;

    while let Some(start_idx) = cursor.find("{{") {
        result.push_str(&cursor[..start_idx]);

        let remainder = &cursor[start_idx + 2..];

        if let Some(end_idx) = remainder.find("}}") {
            let key = &remainder[..end_idx];

            if let Some(value) = vars.get(key) {
                result.push_str(value);
            } else {
                result.push_str("{{");
                result.push_str(key);
                result.push_str("}}");
            }

            cursor = &remainder[end_idx + 2..];
        } else {
            result.push_str("{{");
            cursor = remainder;
            break;
        }
    }

    result.push_str(cursor);
    result
}
