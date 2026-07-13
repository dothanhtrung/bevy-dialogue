use bevy::{prelude::*, window::WindowResolution};
use bevy_2dviewangle::View2dCollection;
use bevy_asset_loader::prelude::{
    AssetCollection,
    ConfigureLoadingState,
    LoadingState,
    LoadingStateAppExt,
};
use bevy_auto_timer::{
    AutoTimer,
    AutoTimerFinished,
};
use bevy_dialogue::{
    DialogueAvailable,
    DialogueComponent,
    DialogueConfig,
    DialogueHandles,
    DialoguePlugin,
    RequestDialogue,
};
use bevy_text_edit::{
    TextEditPluginAnyState,
    TextEditable,
    TextEdited,
};
use isolang::Language;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    InGame,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy".to_string(),
                resolution: WindowResolution::new(640, 320),
                ..default()
            }),
            ..default()
        }),
        DialoguePlugin,
        View2DAnglePluginAnyState::any(),
        TextEditPluginAnyState::any(),
    ))
    .init_state::<MyStates>()
    .add_loading_state(
        LoadingState::new(MyStates::AssetLoading)
            .continue_to_state(MyStates::InGame)
            .load_collection::<MySprite>(),
    )
    .add_systems(OnEnter(MyStates::InGame), setup)
    .run();
}

#[derive(AssetCollection, View2dCollection, Resource)]
struct MySprite {
    #[asset(path = "front.png")]
    #[textureview(actor = "dummy", action = "idle", angle = "front")]
    idle_front: Handle<Image>,

    #[asset(path = "left.png")]
    #[textureview(angle = "left")]
    pub idle_left: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 1, rows = 3))]
    #[textureview(angle = "any")]
    front_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
struct Hero;
#[derive(Component)]
struct Villager;
#[derive(Component)]
struct Monster;

#[repr(u64)]
enum DialogueClass {
    Hero = 6499684068401589489,
    Villager = 8391887828555666002,
    Monster = 918655561745181211,
}

#[repr(u64)]
enum DialogueState {
    Greeting = 2290091270614255425,
}

fn setup(
    mut commands: Commands,
    mut animation2d: ResMut<ActorSpriteSheets>,
    my_sprite: Res<MySprite>,
    mut action_event: MessageWriter<ViewChanged>,
    mut dialogue_config: ResMut<DialogueConfig>,
    mut dialogue_handles: ResMut<DialogueHandles>,
    asset_server: Res<AssetServer>,
) {
    animation2d.load_asset_loader(my_sprite.as_ref());
    dialogue_config.global_lang = Some(Language::Eng);
    dialogue_config
        .variables
        .insert("hero_name".to_string(), "Sam".to_string());
    dialogue_handles.push(asset_server.load("dialogue_sample.ron"));

    commands.spawn(Camera2d);

    let villager = commands
        .spawn((
            Villager,
            DialogueComponent::new(DialogueClass::Villager as u64, DialogueState::Greeting as u64),
            Transform::from_scale(Vec3::splat(10.)).with_translation(Vec3::new(-160., -80., 0.)),
            View2dActor {
                actor: ActorMySprite::Dummy.into(),
                action: ActionMySprite::Idle.into(),
                animation_timer: Some(Timer::from_seconds(1.5, TimerMode::Repeating)),
                ..default()
            },
            DespawnOnExit(MyStates::InGame),
        ))
        .observe(print_villager_dialogue)
        .id();

    let hero = commands
        .spawn((
            Hero,
            DialogueComponent::new(DialogueClass::Hero as u64, DialogueState::Greeting as u64),
            AutoTimer::from_seconds(1.7, TimerMode::Repeating), // timer to say_something
            Transform::from_scale(Vec3::splat(10.)).with_translation(Vec3::new(0., -80., 0.)),
            View2dActor {
                actor: ActorMySprite::Dummy.into(),
                action: ActionMySprite::Idle.into(),
                animation_timer: Some(Timer::from_seconds(1., TimerMode::Repeating)),
                ..default()
            },
            DespawnOnExit(MyStates::InGame),
        ))
        .observe(say_something)
        .observe(print_hero_dialogue)
        .id();

    let monster = commands
        .spawn((
            Monster,
            DialogueComponent::new(DialogueClass::Monster as u64, DialogueState::Greeting as u64),
            Transform::from_scale(Vec3::splat(10.)).with_translation(Vec3::new(160., -80., 0.)),
            View2dActor {
                actor: ActorMySprite::Dummy.into(),
                action: ActionMySprite::Idle.into(),
                animation_timer: Some(Timer::from_seconds(0.5, TimerMode::Repeating)),
                ..default()
            },
            DespawnOnExit(MyStates::InGame),
        ))
        .observe(print_monster_dialogue)
        .id();

    action_event.write(ViewChanged { entity: villager });
    action_event.write(ViewChanged { entity: hero });
    action_event.write(ViewChanged { entity: monster });

    commands
        .spawn(Node {
            width: Val::Percent(90.),
            flex_direction: FlexDirection::Row,
            align_content: AlignContent::Center,
            justify_self: JustifySelf::Center,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        margin: UiRect::right(Val::Percent(2.)),
                        ..default()
                    },
                    Button,
                    Text::new("English"),
                    BackgroundColor(Color::BLACK),
                ))
                .observe(set_eng);
            parent
                .spawn((
                    Button,
                    Node {
                        margin: UiRect::right(Val::Percent(2.)),
                        ..default()
                    },
                    Text::new("Japanese"),
                    BackgroundColor(Color::BLACK),
                ))
                .observe(set_jap);

            parent
                .spawn((
                    Text::new("Sam"),
                    TextEditable::default(),
                    Node {
                        border: UiRect::all(Val::Px(2.)),
                        width: Val::Percent(60.),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                ))
                .observe(set_name);
        });
}

fn print_villager_dialogue(trigger: On<DialogueAvailable>, mut commands: Commands) {
    for dialogue in trigger.dialogues.iter() {
        commands
            .spawn((
                Text2d::new(dialogue),
                TextColor(Color::BLACK),
                TextBackgroundColor(Color::WHITE),
                Transform::from_xyz(-160., 40., 0.),
                AutoTimer::from_seconds(1.5, TimerMode::Once), // despawn timer
            ))
            .observe(despawn_dialogue);
    }
}
fn print_hero_dialogue(trigger: On<DialogueAvailable>, mut commands: Commands) {
    for dialogue in trigger.dialogues.iter() {
        commands
            .spawn((
                Text2d::new(dialogue),
                TextColor(Color::BLACK),
                TextBackgroundColor(Color::WHITE),
                Transform::from_xyz(0., 80., 0.),
                AutoTimer::from_seconds(1.5, TimerMode::Once), // despawn timer
            ))
            .observe(despawn_dialogue);
    }
}
fn print_monster_dialogue(trigger: On<DialogueAvailable>, mut commands: Commands) {
    for dialogue in trigger.dialogues.iter() {
        commands
            .spawn((
                Text2d::new(dialogue),
                TextColor(Color::BLACK),
                TextBackgroundColor(Color::WHITE),
                Transform::from_xyz(160., 40., 0.),
                AutoTimer::from_seconds(1.5, TimerMode::Once), // despawn timer
            ))
            .observe(despawn_dialogue);
    }
}

fn say_something(
    trigger: On<AutoTimerFinished>,
    mut commands: Commands,
    delta: Res<Time>,
    monster: Single<Entity, (With<Monster>, With<DialogueComponent>)>,
    villager: Single<Entity, (With<Villager>, With<DialogueComponent>)>,
    mut hero: Single<&mut View2dActor, With<Hero>>,
    mut view_changed: MessageWriter<ViewChanged>,
) {
    let hero_entity = trigger.entity;
    let decision = (delta.delta().as_millis()) % 3;
    let talk_to = if decision == 0 {
        hero.angle = Angle::Right;
        *monster
    } else if decision == 1 {
        hero.angle = Angle::Left;
        *villager
    } else {
        hero.angle = Angle::Front;
        hero_entity
    };

    if talk_to != hero_entity {
        commands.trigger(RequestDialogue::new(hero_entity).talk_to(talk_to));
        commands.trigger(RequestDialogue::new(talk_to).talk_to(hero_entity));
    }

    view_changed.write(ViewChanged { entity: hero_entity });
}

fn despawn_dialogue(trigger: On<AutoTimerFinished>, mut commands: Commands) {
    commands.entity(trigger.entity).despawn();
}

fn set_eng(_: On<Pointer<Click>>, mut dialogue_config: ResMut<DialogueConfig>) {
    dialogue_config.global_lang = Some(Language::Eng);
}

fn set_jap(_: On<Pointer<Click>>, mut dialogue_config: ResMut<DialogueConfig>) {
    dialogue_config.global_lang = Some(Language::Jpn);
}

fn set_name(trigger: On<TextEdited>, mut dialogue_config: ResMut<DialogueConfig>) {
    dialogue_config
        .variables
        .insert("hero_name".to_string(), trigger.text.clone());
}
