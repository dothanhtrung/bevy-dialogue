#![doc=include_str!("../README.md")]

use bevy::{
    app::Update,
    ecs::{
        message::{
            Message,
            MessageReader,
            MessageWriter,
        },
        schedule::{
            IntoScheduleConfigs,
            common_conditions::on_message,
        },
        system::ResMut,
    },
    platform::collections::HashMap,
    prelude::{
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
        TypePath,
        With,
    },
    utils::default,
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
use indexmap::IndexMap;
use isolang::Language;
use rand::RngExt;
use serde::{
    Deserialize,
    Serialize,
};

/// The main plugin. Add this to your `App`.
#[derive(Default)]
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntropyPlugin<WyRand>>() {
            app.add_plugins(EntropyPlugin::<WyRand>::default());
        };
        app.add_plugins((
            RonLoaderPlugin::<DialogueAsset>::default(),
            BinLoaderPlugin::<DialogueAsset>::default(),
        ))
        .init_asset::<DialogueAsset>()
        .insert_resource(DialogueHandles::default())
        .insert_resource(DialogueConfig::default())
        .insert_resource(SequenceTracker::default())
        .add_message::<RequestSequence>()
        .add_message::<DialogueSequenceEnd>()
        .add_systems(Update, on_request_sequence.run_if(on_message::<RequestSequence>))
        .add_observer(on_request_dialogue)
        .add_observer(update_state);
    }
}

/// Trigger when ONE specific dialogue is request, like with `RequestType::Random` or (`RequestType::All` but with `request_index`)
#[derive(EntityEvent)]
pub struct DialogueTrigger {
    pub entity: Entity,
    pub event_id: u64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct SequenceItem {
    class: u64,
    state: u64,
    /// If dialogue_pos is not specified, all dialogues in specified state will be used
    dialogue: Option<usize>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Dialogue {
    /// Dialogue content in multiple languages
    #[serde(default)]
    pub contents: IndexMap<Language, String>,

    /// The class this dialogue will affect and the state that class will change to
    #[serde(default)]
    pub affects: HashMap<u64, u64>,

    /// The event ids that this dialogue can trigger
    #[serde(default)]
    pub events: Vec<u64>,
    // TODO: Condition
    // #[serde(default)]
    // pub conditions: HashSet<u64>,
}

/// Multiple loaded asset handles are stored in this.
/// The plugin will scan through all files and stop right after found the desired dialogue.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct DialogueHandles(pub Vec<Handle<DialogueAsset>>);

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct DialogueConfig {
    /// Variable to be replaced in the dialogue
    pub variables: HashMap<String, String>,
    /// Language of all characters. Can be overriden by `DialogueComponent` or `RequestDialogue`
    pub global_lang: Option<Language>,
}

#[derive(Default, Clone)]
struct Tracker {
    participants: HashMap<u64, Entity>,
    progress: usize,
    dialogue_pos: usize,
}

#[derive(Resource, Default, Clone, Deref, DerefMut)]
struct SequenceTracker(HashMap<u64, Tracker>);

/// Dialogue asset type. Support both `.ron` and `.bin`.
#[derive(Asset, TypePath, Serialize, Deserialize, Default)]
pub struct DialogueAsset {
    #[serde(default)]
    dialogues: HashMap<u64, IndexMap<u64, Vec<Dialogue>>>,

    #[serde(default)]
    sequences: IndexMap<u64, Vec<SequenceItem>>,
}

#[derive(Component, Default, Clone, Serialize, Deserialize)]
pub struct DialogueComponent {
    /// Dialogue class id
    pub class: u64,
    /// Dialogue state id
    pub state: u64,
    /// Dialogue index
    pub dialogue: usize,
    /// Default character language
    pub default_lang: Option<Language>,
}

impl DialogueComponent {
    pub fn new(class: u64, state: u64) -> Self {
        Self {
            class,
            state,
            dialogue: 0,
            default_lang: None,
        }
    }

    pub fn with_lang(mut self, lang: Language) -> Self {
        self.default_lang = Some(lang);
        self
    }
}

#[derive(Default, Clone)]
pub enum RequestType {
    #[default]
    /// Request a random dialogue of current state
    Random,
    /// Request all dialogues of current state
    All,
    /// Request a specific dialogue in the current state
    One(usize),
}

#[derive(EntityEvent, Clone)]
pub struct RequestDialogue {
    /// Entity of the one who is talking
    #[entity_event]
    pub entity: Entity,
    /// The target whom character talk to
    pub to: Option<Entity>,
    pub request_type: RequestType,
    /// Override component's default language
    pub request_lang: Option<Language>,
}

impl RequestDialogue {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            to: None,
            request_type: RequestType::default(),
            request_lang: None,
        }
    }

    pub fn talk_to(mut self, to: Entity) -> Self {
        self.to = Some(to);
        self
    }

    pub fn with_type(mut self, request_type: RequestType) -> Self {
        self.request_type = request_type;
        self
    }

    pub fn with_lang(mut self, lang: Language) -> Self {
        self.request_lang = Some(lang);
        self
    }
}

#[derive(Message, Default)]
pub struct RequestSequence {
    pub sequence_id: u64,
    /// Participant by class_id.
    /// Only need to set once at the first request of each sequence id.
    pub participants: HashMap<u64, Entity>,
    /// Override component's default language
    pub request_lang: Option<Language>,
}

impl RequestSequence {
    pub fn new(sequence_id: u64) -> Self {
        Self {
            sequence_id,
            participants: HashMap::new(),
            request_lang: None,
        }
    }

    pub fn with_participant(mut self, class_id: u64, entity: Entity) -> Self {
        self.participants.insert(class_id, entity);
        self
    }

    pub fn with_participants(mut self, participants: HashMap<u64, Entity>) -> Self {
        self.participants = participants;
        self
    }

    pub fn with_lang(mut self, lang: Language) -> Self {
        self.request_lang = Some(lang);
        self
    }
}

/// Returned when found the dialogue
#[derive(EntityEvent, Clone)]
pub struct DialogueAvailable {
    pub entity: Entity,
    pub dialogues: Vec<String>,
}

/// Change the character's state.
/// You can use this event or modify the component directly.
#[derive(Event)]
pub struct DialogueStateChanged {
    pub entity: Entity,
    /// If the next_state is not set, the next state will be detect by order in the asset
    pub next_state: Option<u64>,
}

/// Notify that sequence has end
#[derive(Message)]
pub struct DialogueSequenceEnd(pub u64);

fn on_request_dialogue(
    trigger: On<RequestDialogue>,
    mut commands: Commands,
    dialogue_config: Res<DialogueConfig>,
    dialogue_handles: Res<DialogueHandles>,
    dialogue_asset: Res<Assets<DialogueAsset>>,
    mut query: Query<&mut DialogueComponent>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let Ok(character) = query.get(trigger.entity) else {
        return;
    };
    for handle in dialogue_handles.iter() {
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
                let request_lang = if trigger.request_lang.is_some() {
                    trigger.request_lang
                } else {
                    if character.default_lang.is_some() {
                        character.default_lang
                    } else {
                        if dialogue_config.global_lang.is_some() { dialogue_config.global_lang } else { None }
                    }
                };

                if !dialogues.is_empty() {
                    if let Ok(mut character) = query.get_mut(trigger.entity) {
                        character.state = character_state;
                    }

                    let mut ret_dialogues = Vec::new();
                    match trigger.request_type {
                        RequestType::One(index) => {
                            if let Some(dialogue) = dialogues.get(index) {
                                for event in dialogue.events.iter() {
                                    commands.trigger(DialogueTrigger {
                                        entity: trigger.entity,
                                        event_id: *event,
                                    });
                                }

                                for (lang, content) in dialogue.contents.iter() {
                                    if let Some(request_lang) = request_lang
                                        && *lang != request_lang
                                    {
                                        continue;
                                    }
                                    let content = replace_templates(content, &dialogue_config.variables);
                                    ret_dialogues.push(content);
                                    break;
                                }
                                if let Ok(mut character) = query.get_mut(trigger.entity) {
                                    character.dialogue = index;
                                }
                            }
                        }
                        RequestType::All => {
                            for dialogue in dialogues.iter() {
                                for (lang, content) in dialogue.contents.iter() {
                                    if let Some(request_lang) = request_lang
                                        && *lang != request_lang
                                    {
                                        continue;
                                    }

                                    let content = replace_templates(content, &dialogue_config.variables);
                                    ret_dialogues.push(content);
                                    break;
                                }
                            }
                        }
                        RequestType::Random => {
                            let random_msg_idx = rng.random_range(0..dialogues.len());
                            let dialogue = dialogues[random_msg_idx].clone();

                            for event in dialogue.events.iter() {
                                commands.trigger(DialogueTrigger {
                                    entity: trigger.entity,
                                    event_id: *event,
                                });
                            }

                            for (lang, content) in dialogue.contents.iter() {
                                if let Some(request_lang) = request_lang
                                    && *lang != request_lang
                                {
                                    continue;
                                }
                                let content = replace_templates(content, &dialogue_config.variables);
                                ret_dialogues.push(content);
                                break;
                            }

                            if let Ok(mut character) = query.get_mut(trigger.entity) {
                                character.dialogue = random_msg_idx;
                            }
                        }
                    }

                    let res = DialogueAvailable {
                        entity: trigger.entity,
                        dialogues: ret_dialogues,
                    };
                    commands.trigger(res);
                }

                // Dialogue found. No need to check other assets
                break;
            }
        }
    }
}

fn on_request_sequence(
    mut request: MessageReader<RequestSequence>,
    mut commands: Commands,
    mut query: Query<&mut DialogueComponent>,
    mut trackers: ResMut<SequenceTracker>,
    mut sequence_end: MessageWriter<DialogueSequenceEnd>,
    dialogue_config: Res<DialogueConfig>,
    dialogue_handles: Res<DialogueHandles>,
    dialogue_asset: Res<Assets<DialogueAsset>>,
) {
    'msg_loop: for request in request.read() {
        for handle in dialogue_handles.iter() {
            let Some(dialogue_asset) = dialogue_asset.get(handle) else {
                continue;
            };

            let Some(sequences) = dialogue_asset.sequences.get(&request.sequence_id) else {
                continue 'msg_loop;
            };

            if !trackers.contains_key(&request.sequence_id) {
                trackers.insert(
                    request.sequence_id,
                    Tracker {
                        participants: request.participants.clone(),
                        ..default()
                    },
                );
            }

            let tracker = trackers.get_mut(&request.sequence_id).unwrap();

            // End of sequence
            if tracker.progress >= sequences.len() {
                continue 'msg_loop;
            }

            let sequence = &sequences[tracker.progress];
            let Some(character_id) = tracker.participants.get(&sequence.class_id) else {
                continue 'msg_loop;
            };
            let Ok(mut character) = query.get_mut(*character_id) else {
                continue 'msg_loop;
            };

            if let Some(class) = dialogue_asset.dialogues.get(&sequence.class_id)
                && let Some(state) = class.get(&sequence.state_id)
            {
                let dialogue = if let Some(dialogue_pos) = sequence.dialogue_pos
                    && dialogue_pos < state.len()
                {
                    tracker.progress += 1;
                    if tracker.progress > sequences.len() {
                        tracker.progress = 0;
                        sequence_end.write(DialogueSequenceEnd(request.sequence_id));
                    }

                    character.dialogue = dialogue_pos;
                    state[dialogue_pos].clone()
                } else {
                    if tracker.dialogue_pos < state.len() {
                        let ret = state[tracker.dialogue_pos].clone();
                        character.dialogue = tracker.dialogue_pos;
                        tracker.dialogue_pos += 1;
                        if tracker.dialogue_pos >= state.len() {
                            tracker.dialogue_pos = 0;
                            tracker.progress += 1;
                            if tracker.progress > sequences.len() {
                                tracker.progress = 0;
                                sequence_end.write(DialogueSequenceEnd(request.sequence_id));
                            }
                        }
                        ret
                    } else {
                        continue 'msg_loop;
                    }
                };

                let request_lang = if request.request_lang.is_some() {
                    request.request_lang
                } else {
                    if character.default_lang.is_some() {
                        character.default_lang
                    } else {
                        if dialogue_config.global_lang.is_some() { dialogue_config.global_lang } else { None }
                    }
                };

                for event in dialogue.events.iter() {
                    commands.trigger(DialogueTrigger {
                        entity: *character_id,
                        event_id: *event,
                    });
                }
                for (lang, content) in dialogue.contents.iter() {
                    if let Some(request_lang) = request_lang
                        && *lang != request_lang
                    {
                        continue;
                    }
                    let content = replace_templates(content, &dialogue_config.variables);

                    commands.trigger(DialogueAvailable {
                        entity: *character_id,
                        dialogues: vec![content],
                    });

                    break;
                }
            } else {
                continue 'msg_loop;
            }
        }
    }
}

fn update_state(
    trigger: On<DialogueStateChanged>,
    dialogue_handles: Res<DialogueHandles>,
    dialogue_asset: Res<Assets<DialogueAsset>>,
    mut query: Query<&mut DialogueComponent>,
) {
    for handle in dialogue_handles.iter() {
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
                if !found_current && let Some((first_state, _)) = dialogues.first() {
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
