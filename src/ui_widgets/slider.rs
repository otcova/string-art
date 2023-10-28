use bevy::ecs::system::EntityCommands;
use bevy::ui::RelativeCursorPosition;

use super::*;
use std::ops::RangeInclusive;

pub(super) struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (slider_thumb_update, slider_interact));
    }
}

#[derive(Component)]
pub struct Slider {
    pub value: f32,
    range: RangeInclusive<f32>,
    round_value: bool,
}

#[derive(Component)]
struct SliderThumb;

pub struct SliderSettings<'a> {
    pub width: Val,
    pub track_width: f32,
    pub thumb_size: f32,
    pub value: f32,
    pub range: RangeInclusive<f32>,
    pub track_color: Color,
    pub thumb_color: Color,
    pub lable: &'a str,
    pub text_color: Color,
    pub round_value: bool,
}

impl<'a> Default for SliderSettings<'a> {
    fn default() -> Self {
        Self {
            width: Val::Px(350.),
            track_width: 200.,
            thumb_size: 20.,
            value: 0.3,
            range: 0.0..=1.0,
            track_color: Color::rgb(0.6, 0.6, 0.6),
            thumb_color: Color::WHITE,
            lable: "Slider",
            text_color: Color::rgb(1., 1., 1.),
            round_value: false,
        }
    }
}

impl Slider {
    pub fn spawn<D: 'static>(
        commands: &mut Commands,
        settings: SliderSettings,
        value_bind: ValueBind<D, Slider>,
    ) -> Entity {
        let slider_track = NodeBundle {
            style: Style {
                width: Val::Px(settings.track_width),
                height: Val::Px(settings.thumb_size),
                overflow: Overflow::clip(),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: settings.track_color.into(),
            ..default()
        };
        let slider_thumb = NodeBundle {
            style: Style {
                width: Val::Percent(0.),
                height: Val::Px(settings.thumb_size),
                ..default()
            },
            background_color: settings.thumb_color.into(),
            ..default()
        };

        let mut entity_commands = commands.spawn(NodeBundle {
            style: Style {
                width: settings.width,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(5.),
                ..default()
            },
            ..default()
        });
        entity_commands.with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                settings.lable,
                TextStyle {
                    font_size: 18.0,
                    color: settings.text_color,
                    ..default()
                },
            ));
            parent
                .spawn((
                    Slider {
                        value: settings
                            .value
                            .clamp(*settings.range.start(), *settings.range.end()),
                        range: settings.range,
                        round_value: settings.round_value,
                    },
                    slider_track,
                    Interaction::default(),
                    RelativeCursorPosition::default(),
                    value_bind,
                ))
                .with_children(|parent| {
                    parent.spawn((SliderThumb, slider_thumb));
                    parent.spawn(TextBundle {
                        text: Text {
                            sections: vec![TextSection::new(
                                "",
                                TextStyle {
                                    font_size: 18.0,
                                    color: Color::BLACK,
                                    ..default()
                                },
                            )],
                            ..default()
                        },
                        style: Style {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        ..default()
                    });
                });
        });
        entity_commands.id()
    }
}

fn slider_thumb_update(
    slider_query: Query<(&Slider, &Children), Changed<Slider>>,
    mut thumb_query: Query<&mut Style, With<SliderThumb>>,
    mut text_query: Query<&mut Text>,
) {
    for (slider, slider_children) in &slider_query {
        let mut children = slider_children.into_iter();
        let mut thumb_style = thumb_query.get_mut(*children.next().unwrap()).unwrap();
        let mut text = text_query.get_mut(*children.next().unwrap()).unwrap();

        let min = *slider.range.start();
        let max = *slider.range.end();
        let pos = (slider.value - min) / (max - min);
        thumb_style.width = Val::Percent(pos * 100.);
        thumb_style.margin.right = Val::Percent(100. - pos * 100.);

        if slider.round_value {
            text.sections[0].value = (slider.value as i64).to_string();
        } else {
            text.sections[0].value = format!("{:.3}", slider.value);
        }
    }
}

fn slider_interact(mut query: Query<(&mut Slider, &RelativeCursorPosition, &Interaction)>) {
    for (mut slider, cursor, interaction) in &mut query {
        match interaction {
            Interaction::Pressed => {
                if let Some(cursor_pos) = cursor.normalized {
                    let min = *slider.range.start();
                    let max = *slider.range.end();
                    let value = (cursor_pos.x * (max - min) + min).clamp(min, max);
                    let value = if slider.round_value {
                        value.round()
                    } else {
                        value
                    };

                    if slider.value != value {
                        slider.value = value;
                    }
                }
            }
            _ => {}
        }
    }
}
