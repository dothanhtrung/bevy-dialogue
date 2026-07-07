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
    DialogueAvailable,
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

#[repr(u64)]
enum CharacterClass {
    Hero = 1,
    Villager,
}

#[repr(u64)]
enum CharacterState {
    Normal = 1,
}

#[derive(Component)]
struct Choice(usize);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut dialogue_res: ResMut<DialogueRes>) {
    dialogue_res
        .variables
        .insert("hero_name".to_string(), "Sam".to_string());
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

    // ----- UI -----
    // Can be ignored
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
                    width: Val::Percent(50.),
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
                            Text::new("Get choices"),
                        ))
                        .observe(get_choices);

                    parent.spawn((
                        Hero,
                        Node {
                            margin: UiRect::top(Val::Percent(10.)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ));
                });

            // UI for villager
            parent
                .spawn(Node {
                    height: Val::Percent(100.),
                    width: Val::Percent(50.),
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
        });

    commands.spawn(Camera2d);
}

fn get_choices(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    hero: Single<Entity, (With<Hero>, With<DialogueComponent>)>,
) {
    // Request all the dialogues in the current state of Hero as choices
    commands.trigger(RequestDialogue::new(*hero).with_type(RequestType::All));
}

fn print_hero_dialogue(
    trigger: On<DialogueAvailable>,
    mut commands: Commands,
    hero_display: Single<Entity, (With<Hero>, With<Node>)>,
) {
    let entity = hero_display.into_inner();

    // Display all received dialogues as choices
    commands.entity(entity).despawn_children();
    commands.entity(entity).with_children(|parent| {
        for (i, dialogue) in trigger.dialogues.iter().enumerate() {
            parent
                .spawn((Button, Choice(i), Text::new(dialogue)))
                .observe(select_dialogue);
        }
    });
}

fn select_dialogue(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    hero: Single<Entity, (With<Hero>, With<DialogueComponent>)>,
    villager: Single<Entity, (With<Villager>, With<DialogueComponent>)>,
    choices: Query<&Choice>,
) {
    let Ok(Choice(choice)) = choices.get(trigger.entity) else {
        return;
    };

    // Request a specific dialogue for Hero
    commands.trigger(
        RequestDialogue::new(*hero)
            .talk_to(*villager)
            .with_type(RequestType::One(*choice)),
    );

    // Request dialogue for Villager
    commands.trigger(RequestDialogue::new(*villager).talk_to(*hero));
}

fn print_villager_dialogue(trigger: On<DialogueAvailable>, mut text: Single<&mut Text, With<Villager>>) {
    if let Some(dialogue) = trigger.dialogues.first() {
        ***text = dialogue.clone();
    }
}
