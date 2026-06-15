use bevy::prelude::*;
use bevy_dialogue::{
    DialogueComponent,
    DialoguePlugin,
    DialogueRes,
    NextDialogue,
    RequestDialogue,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the plugin
        .add_plugins(DialoguePlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, click_to_talk)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut dialogue_res: ResMut<DialogueRes>) {
    // Plugin can look up for {{hero_name}} and replace it by your value
    dialogue_res.variables.insert("hero_name".to_string(), "Sam".to_string());

    // Load the dialogue file. Multiple files are supported.
    dialogue_res.dialogues.push(asset_server.load("dialogue_sample.ron"));


    // Add `DialogueComponent` to your entity with specific class_id and state_id
    commands
        .spawn(DialogueComponent::new(1, 1))
        .observe(print_dialogue);

    commands
        .spawn(Node {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            width: Val::Percent(90.),
            ..default()
        })
        .with_child(Text::new(""));
    commands.spawn(Camera2d);
}

fn click_to_talk(
    mut commands: Commands,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    npc: Query<Entity, With<DialogueComponent>>,
) {
    for entity in npc.iter() {
        if mouse_btn.just_pressed(MouseButton::Left) {
            // Request random dialogues for character.
            commands.trigger(RequestDialogue::new(entity));
        }
    }
}

// Character dialogue will be returned through event `NextDialogue`
fn print_dialogue(trigger: On<NextDialogue>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        let Some(dialogue) = trigger.dialogues.first() else {
            return;
        };
        let Some((_, content)) = dialogue.contents.first_key_value() else {
            return;
        };

        **text = content.clone();
    }
}
