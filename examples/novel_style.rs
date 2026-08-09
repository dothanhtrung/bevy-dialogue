use bevy::prelude::*;
use bevy_dialogue::{
    DialogueAvailable,
    DialogueComponent,
    DialogueHandles,
    DialoguePlugin,
    DialogueSequenceEnd,
    RequestSequence,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the plugin
        .add_plugins(DialoguePlugin)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (click_to_talk, sequence_end.run_if(on_message::<DialogueSequenceEnd>)),
        )
        .run();
}

#[repr(u64)]
enum Actor {
    Narrator = 15008331116881037351,
    TheBoy = 14537648815543369767,
}

#[repr(u64)]
enum State {
    First = 3967406410050873939,
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut handles: ResMut<DialogueHandles>,
    mut request: MessageWriter<RequestSequence>,
) {
    handles.push(asset_server.load("novel_sample.ron"));

    let narrator_id = commands
        .spawn(DialogueComponent::new(Actor::Narrator as u64, State::First as u64))
        .observe(print_dialogue)
        .id();
    let theboy_id = commands
        .spawn(DialogueComponent::new(Actor::TheBoy as u64, State::First as u64))
        .observe(print_dialogue)
        .id();

    commands
        .spawn(Node {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            width: Val::Percent(90.),
            ..default()
        })
        .with_child(Text::new(""));
    commands.spawn(Camera2d);

    // Request a sequence
    request.write(
        RequestSequence::new(1)
            .with_participant(Actor::Narrator as u64, narrator_id)
            .with_participant(Actor::TheBoy as u64, theboy_id),
    );
}

fn click_to_talk(mouse_btn: Res<ButtonInput<MouseButton>>, mut request: MessageWriter<RequestSequence>) {
    if mouse_btn.just_pressed(MouseButton::Left) {
        request.write(RequestSequence::new(1));
    }
}

// Character dialogue will be returned through event `DialogueAvailable`
fn print_dialogue(trigger: On<DialogueAvailable>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        let Some(dialogue) = trigger.dialogues.first() else {
            return;
        };

        **text = dialogue.clone();
    }
}

fn sequence_end(mut msg: MessageReader<DialogueSequenceEnd>, mut text: Single<&mut Text>) {
    for msg in msg.read() {
        ***text = String::new();
        info!("Sequence {} ended", msg.0);
    }
}
