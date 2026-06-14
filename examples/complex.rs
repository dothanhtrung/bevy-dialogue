use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_dialogue::{
    DialogueComponent,
    DialoguePlugin,
    DialogueRes,
    NextDialogue,
    RequestDialogue,
    RequestType,
};
use xxhash_rust::xxh3::xxh3_64;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the plugin
        .add_plugins(DialoguePlugin)
        .insert_resource(IdMap::default())
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component)]
struct Hero;

#[derive(Component)]
struct Villager;

#[derive(Component)]
struct Monster;

#[derive(Resource, Default, DerefMut, Deref)]
struct IdMap(HashMap<String, u64>);

// In this example, the id is hashed from string using xxh3_64, but you do not need to do the same.
// Any unique u64 values are okay.
impl IdMap {
    fn get_id(&mut self, value: &str) -> u64 {
        let lower = value.to_lowercase();
        if let Some(id) = self.get(lower.as_str()) {
            *id
        } else {
            let id = xxh3_64(lower.as_bytes());
            self.insert(lower, id);
            id
        }
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut dialogue_res: ResMut<DialogueRes>,
    mut id_map: ResMut<IdMap>,
) {
    dialogue_res
        .variables
        .insert("hero_name".to_string(), "Sam".to_string());
    dialogue_res.dialogues.push(asset_server.load("dialogue_sample.ron"));

    commands
        .spawn((
            Hero,
            DialogueComponent::new(id_map.get_id("Hero"), id_map.get_id("Normal")),
        ))
        .observe(print_hero_dialogue);

    commands
        .spawn((
            Villager,
            DialogueComponent::new(id_map.get_id("Villager"), id_map.get_id("Normal")),
        ))
        .observe(print_villager_dialogue);

    commands
        .spawn((
            Monster,
            DialogueComponent::new(id_map.get_id("Monster"), id_map.get_id("Normal")),
        ))
        .observe(print_monster_dialogue);

    // ---- Just UI, can be ignored -----
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    height: Val::Percent(100.),
                    width: Val::Percent(30.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("Hero"));
                    parent.spawn((Button, Text::new("Talk to Villager"))).observe(talk);
                    parent
                        .spawn((Button, Text::new("Talk to Monster")))
                        .observe(talk_to_monster);
                    parent.spawn((Hero, Text::default()));
                });

            parent
                .spawn(Node {
                    height: Val::Percent(100.),
                    width: Val::Percent(30.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("Villager"));
                    parent.spawn((Villager, Text::default()));
                });

            parent
                .spawn(Node {
                    height: Val::Percent(100.),
                    width: Val::Percent(30.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("Monster"));
                    parent.spawn((Monster, Text::default()));
                });
        });

    commands.spawn(Camera2d);
}

fn talk_to_monster(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    hero: Single<Entity, (With<Hero>, With<DialogueComponent>)>,
    monster: Single<Entity, (With<Monster>, With<DialogueComponent>)>,
) {
    // Request dialog for Hero
    commands.trigger(RequestDialogue {
        entity: *hero,
        to: Some(*monster),
        request_type: RequestType::Random,
        request_index: None,
    });

    // Request dialog for monster
    commands.trigger(RequestDialogue {
        entity: *monster,
        to: Some(*hero),
        request_type: RequestType::Random,
        request_index: None,
    });
}

fn talk(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    mut hero: Single<(Entity, &mut DialogueComponent), With<Hero>>,
    villager: Single<Entity, With<Villager>>,
    mut id_map: ResMut<IdMap>,
) {
    let (hero_entity, mut dialogue_component) = hero.into_inner();

    dialogue_component.state = id_map.get_id("Normal");

    // Request dialog for Hero
    // commands.trigger(RequestDialogue::new(hero_entity));

    // Request dialog for Villager
    commands.trigger(RequestDialogue {
        entity: *villager,
        to: Some(hero_entity),
        request_type: RequestType::Random,
        request_index: None,
    });
}

fn print_hero_dialogue(trigger: On<NextDialogue>, mut text: Single<&mut Text, With<Hero>>) {
    for dialogue in trigger.dialogues.iter() {
        let Some((_lang, content)) = dialogue.contents.first_key_value() else {
            continue;
        };
        ***text = content.clone();
    }
}

fn print_monster_dialogue(trigger: On<NextDialogue>, mut text: Single<&mut Text, With<Monster>>) {
    for dialogue in trigger.dialogues.iter() {
        let Some((_lang, content)) = dialogue.contents.first_key_value() else {
            continue;
        };
        ***text = content.clone();
    }
}

fn print_villager_dialogue(trigger: On<NextDialogue>, mut text: Single<&mut Text, With<Villager>>) {
    for dialogue in trigger.dialogues.iter() {
        let Some((_lang, content)) = dialogue.contents.first_key_value() else {
            continue;
        };
        ***text = content.clone();
    }
}
