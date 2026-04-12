use bevy::prelude::*;
use bevy_dialogue::{
    DialogueComponent,
    DialoguePluginAnyState,
    DialogueRes,
    DialogueStateChanged,
    NextDialogue,
    RequestDialogue,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DialoguePluginAnyState::any())
        .add_systems(Startup, startup)
        .add_systems(Update, click_to_talk)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut dialogue_res: ResMut<DialogueRes>) {
    dialogue_res.0 = asset_server.load("sample.ron");
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            align_self: AlignSelf::End,
            justify_self: JustifySelf::Center,
            width: Val::Percent(90.),
            ..default()
        })
        .with_child(Text::new(""));

    commands.spawn(DialogueComponent::new(1)).observe(print_dialogue);
}

fn click_to_talk(
    mut commands: Commands,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    npc: Query<Entity, With<DialogueComponent>>,
) {
    for entity in npc.iter() {
        if mouse_btn.just_pressed(MouseButton::Left) {
            commands.trigger(RequestDialogue { entity });
        } else if mouse_btn.just_pressed(MouseButton::Right) {
            commands.trigger(DialogueStateChanged {
                entity,
                next_state: None,
            });
            commands.trigger(RequestDialogue { entity });
        }
    }
}

fn print_dialogue(trigger: On<NextDialogue>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        **text = trigger.dialogue.clone();
    }
}
