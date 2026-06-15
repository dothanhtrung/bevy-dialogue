use bevy::{
    color::palettes::css::{
        GRAY,
    },
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the plugin
        .add_plugins(DialoguePlugin)
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component)]
struct Hero;

#[derive(Component)]
struct Villager;

#[derive(Component)]
struct Monster;

#[repr(u64)]
enum CharacterClass {
    Hero = 1,
    Villager,
    Monster,
}

#[repr(u64)]
enum CharacterState {
    Normal = 10,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut dialogue_res: ResMut<DialogueRes>) {
    // Plugin can look up for {{variable}} and replace it by your value
    dialogue_res
        .variables
        .insert("hero_name".to_string(), "Sam".to_string());

    // Load the dialogue file. Multiple files are supported.
    dialogue_res.dialogues.push(asset_server.load("dialogue_sample.ron"));

    // Spawn entity with DialogueComponent and listen the dialogue event on it
    commands
        .spawn((
            Hero,
            DialogueComponent::new(CharacterClass::Hero as u64, CharacterState::Normal as u64),
        ))
        .observe(print_hero_dialogue);

    commands
        .spawn((
            Villager,
            DialogueComponent::new(CharacterClass::Villager as u64, CharacterState::Normal as u64),
        ))
        .observe(print_villager_dialogue);

    commands
        .spawn((
            Monster,
            DialogueComponent::new(CharacterClass::Monster as u64, CharacterState::Normal as u64),
        ))
        .observe(print_monster_dialogue);

    // ----- UI -----
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        })
        .with_children(|parent| {
            // UI for hero
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
                    parent
                        .spawn((
                            Button,
                            BackgroundColor(GRAY.into()),
                            Node {
                                border_radius: BorderRadius::all(Val::Px(5.)),
                                margin: UiRect::top(Val::Percent(5.)),
                                ..default()
                            },
                            Text::new("Talk to Villager"),
                        ))
                        .observe(talk);
                    parent
                        .spawn((
                            Button,
                            BackgroundColor(GRAY.into()),
                            Node {
                                border_radius: BorderRadius::all(Val::Px(5.)),
                                margin: UiRect::top(Val::Percent(5.)),
                                ..default()
                            },
                            Text::new("Talk to Monster"),
                        ))
                        .observe(talk_to_monster);
                    parent.spawn((
                        Hero,
                        Node {
                            margin: UiRect::top(Val::Percent(10.)),
                            ..default()
                        },
                        Text::default(),
                    ));
                });

            // UI for villager
            parent
                .spawn(Node {
                    height: Val::Percent(100.),
                    width: Val::Percent(30.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("Villager"));
                    parent.spawn((
                        Villager,
                        Node {
                            margin: UiRect::top(Val::Percent(10.)),
                            ..default()
                        },
                        Text::default(),
                    ));
                });

            // UI for monster
            parent
                .spawn(Node {
                    height: Val::Percent(100.),
                    width: Val::Percent(30.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("Monster"));
                    parent.spawn((
                        Monster,
                        Node {
                            margin: UiRect::top(Val::Percent(10.)),
                            ..default()
                        },
                        Text::default(),
                    ));
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
    hero: Single<(Entity, &mut DialogueComponent), With<Hero>>,
    villager: Single<Entity, (With<Villager>, With<DialogueComponent>)>,
) {
    let (hero_entity, mut dialogue_component) = hero.into_inner();
    // Reset hero state to normal
    dialogue_component.state = CharacterState::Normal as u64;

    // Request dialog for Hero
    commands.trigger(RequestDialogue {
        entity: hero_entity,
        to: Some(*villager),
        request_type: RequestType::Random,
        request_index: None,
    });

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
