mod image_handle;
mod settings;
mod source_image_processing;
mod string_trace;
mod ui_widgets;

use bevy::prelude::*;
use image_handle::*;
use settings::*;
use source_image_processing::*;
use string_trace::*;
use ui_widgets::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            SourceImagePlugin,
            StringTracePlugin,
            UIWidgetsPlugin,
            SettingsPlugin,
        ))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands, mut assets: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle::default());

    let settings = Settings::default();
    let string_trace = StringTrace::new(&settings, &mut assets);
    let result_texture = string_trace.canvas.clone();

    let mut settings_entity = Entity::PLACEHOLDER;
    let page = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            settings_entity = parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Auto,
                            height: Val::Percent(100.),
                            ..default()
                        },
                        image: UiImage::new(result_texture),
                        ..default()
                    },
                    string_trace,
                    ProcessedImage(None),
                    settings,
                ))
                .id();
        })
        .id();

    let settings_ui = SettingsUI::new(&mut commands, settings_entity);
    commands.add(AddChild {
        parent: page,
        child: settings_ui,
    });
}
