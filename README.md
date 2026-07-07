<div align="center">

bevy_dialogue
=============

[![crates.io](https://img.shields.io/crates/v/bevy_dialogue)](https://crates.io/crates/bevy_dialogue)
[![docs.rs](https://docs.rs/bevy_dialogue/badge.svg)](https://docs.rs/bevy_dialogue)
[![dependency status](https://deps.rs/crate/bevy_dialogue/latest/status.svg)](https://deps.rs/crate/bevy_dialogue/latest)
[![pipeline status](https://gitlab.com/245project/bevy-plugin/bevy-dialogue/badges/master/pipeline.svg)](https://gitlab.com/245project/bevy-plugin/bevy-dialogue/-/commits/master)

[![Gitlab](https://img.shields.io/badge/gitlab-%23181717.svg?style=for-the-badge&logo=gitlab&logoColor=white)](https://gitlab.com/245project/bevy-plugin/bevy-dialogue)
[![Github](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/dothanhtrung/bevy-dialogue)

</div>

Bevy plugin for load and retrieve characters dialogues. (**_UI not included_**)

Asset Syntax
------------

> Dialogue asset can be **text file** (RON format) or **binary file**. There is a GUI tool [Dialogue Editor](https://gitlab.com/kimtinh/dialogue-editor)
> for creating and exporting dialogue asset.

```ron
(
    dialogues: {
        <class_id>: {
            <state_id>: [
                (
                    contents: {
                        "<language code>": "Dialogue content. {{variable}} is supported",
                    },
                    affects: {
                        <target_class_id>: <target_state_id>,
                    },
                    events: [event_id]
                ),
            ],
        },
    },
)
```

* `class_id`: `u64`. The character class id. For example: Villager, Hero, etc. should have unique id.
* `state_id`: `u64`. The character state id. For example: Idle, Arguing, Cheering, etc. should have unique id.
* `language_code`: `String`. 3 character language code by ISO 639-3. For example: `eng`, `spa`, etc.
* `affects`: This mean the state of entity with `target_class_id` will be change to `target_state_id` after this dialogue.
* `events`: Array of event id. They will be triggered by plugin if the dialogue is used.

Usage
-----

Please see [examples](./examples) for more detail.

### Plugin

```rust
let mut = App::new();
app.add_plugins(DialoguePlugin);
```

### Load asset

Multiple dialogue assets can be loaded and pushed to `DialogueRes`:

```rust
fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut dialogue_res: ResMut<DialogueRes>) {
    dialogue_res.dialogues.push(asset_server.load("dialogue_stage1.ron"));
    dialogue_res.dialogues.push(asset_server.load("dialogue_stage2.bin"));
}
```

### Dialogue variables

Variables can be put in asset in syntax `{{your_variable}}`. Their value can be defined in `DialogueRes`.

```rust
fn startup(mut dialogue_res: ResMut<DialogueRes>) {
    // Plugin will look up for variable {{hero_name}} and replace it
    dialogue_res
        .variables
        .insert("hero_name".to_string(), "Sam".to_string());
}
```

### Spawn

Spawn entity with component `DialogueComponent`.

```rust
#[repr(u64)]
enum CharacterClass {
    Hero = 1,
    Villager,
    Monster,
}

#[repr(u64)]
enum CharacterState {
    Normal = 1,
    Attack,
    Disagree,
}

fn spawn(mut commands: Commands) {
    commands.spawn(DialogueComponent::new(
        CharacterClass::Hero as u64,
        CharacterState::Normal as u64)
    )
    .observe(dialogue_available) // Listen for `DialogueAvailable` event
    .observe(dialogue_event)     // One dialogue can trigger a specific event through `DialogueTrigger`
    ;
}
```

### Get dialogue

To get a dialogue, you need to send the request first:
```rust
commands.trigger(RequestDialogue::new(hero_entity));
```

Then, the dialogue will be returned through `DialogueAvailable` event:
```rust
fn dialogue_available(trigger: On<DialogueAvailable>) {
    for dialogue in trigger.dialogues.iter() {
        info!(dialogue);
    }
}
```

### Talk to

When request dialogue, you can specify which one the character is talking to:

```rust
commands.trigger(RequestDialogue::new(hero_entity).talk_to(monster_entity));
```

### Event

The event id in asset can be listen through `DialogueTrigger`

```rust
fn event_trigger(trigger: On<DialogueTrigger>) {
    info!("Dialogue trigger event {}", trigger.event_id);
}
```

### Choice

All dialogues in a same state of character can be treated as choices:
```rust
commands.trigger(RequestDialogue::new(hero_entity).with_type(RequestType::All));
```

Then a choice can be set by request again with the dialogue index. For example, if we select the first choice,
send a request with index `0`:
```rust
commands. trigger(RequestDialogue::new(hero_entity).with_type(RequestType::One(0)));
```

License
-------

Please see [LICENSE](./LICENSE).


Compatible Bevy Versions
------------------------

| bevy | bevy_dialogue |
|------|---------------|
| 0.19 | 0.2-0.3       |
| 0.18 | 0.1           |

---------

<div align="center">

![git_bevy-dialogue](https://count.getloli.com/@git_bevy-dialogue?name=git_bevy-dialogue&theme=random&padding=10&offset=0&align=top&scale=1&pixelated=1&darkmode=auto)

</div>
