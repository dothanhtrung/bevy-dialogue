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
    dialogue_res.dialogues.push(asset_server.load("dialogue_sample.ron"));
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            align_self: AlignSelf::End,
            justify_self: JustifySelf::Center,
            width: Val::Percent(90.),
            ..default()
        })
        .with_child(Text::new(""));

    // The id is hashed from string. You can get this id by `xxh3_64(string_value)`.
    commands
        .spawn(DialogueComponent::new(2087775913480848054, 7173758463185314716))
        .observe(print_dialogue);
}

fn click_to_talk(
    mut commands: Commands,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    npc: Query<Entity, With<DialogueComponent>>,
) {
    for entity in npc.iter() {
        if mouse_btn.just_pressed(MouseButton::Left) {
            commands.trigger(RequestDialogue::new(entity));
        } else if mouse_btn.just_pressed(MouseButton::Right) {
            commands.trigger(DialogueStateChanged {
                entity,
                next_state: None,
            });
            commands.trigger(RequestDialogue::new(entity));
        }
    }
}

fn print_dialogue(trigger: On<NextDialogue>, mut query: Query<&mut Text>) {
    // TODO: Improve example
    for mut text in query.iter_mut() {
        if let Some((_, content)) = trigger.dialogues[0].contents.first_key_value() {
            **text = content.clone();
        }
    }
}
