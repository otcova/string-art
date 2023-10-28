use crate::*;

pub enum ColorDistanceFn {
    ABS,
    SQUARE,
}

#[derive(Component)]
pub struct Settings {
    pub diameter: u32,
    pub darken: u8,
    pub source_image_index: usize,

    pub nodes: u16,
    pub nodes_offset: f32,
    pub string_alpha: f32,
    pub max_lines: usize,
    pub repeat_lines: bool,
    pub color_distance_fn: ColorDistanceFn,
}

pub const IMAGES_PATHS: &[&'static str] = &[
    "assets/a.jpg",
    "assets/b.jpg",
    "assets/b.png",
    "assets/e.jpg",
    "assets/mountains.webp",
    "assets/photo.jpg",
    "assets/robot-13.png",
    "assets/sample.png",
];

impl Default for Settings {
    fn default() -> Self {
        Self {
            diameter: 500,
            darken: 100,
            source_image_index: 0,

            nodes: 200,
            nodes_offset: 1.,
            string_alpha: 0.3,
            max_lines: 6000,
            repeat_lines: false,
            color_distance_fn: ColorDistanceFn::SQUARE,
        }
    }
}

#[derive(Bundle)]
pub struct SettingsUI {
    node: NodeBundle,
}

macro_rules! slider {
    ($commands:ident, $settings:ident, $name:ident, $range:expr, float) => {
        Slider::spawn(
            $commands,
            SliderSettings {
                lable: stringify!($name),
                value: Settings::default().$name as f32,
                range: $range,
                round_value: false,
                ..default()
            },
            ValueBind::<Settings, Slider> {
                dst: $settings,
                update: |settings, slider| settings.$name = slider.value as _,
            },
        )
    };
    ($commands:ident, $settings:ident, $name:ident, $range:expr, int) => {
        Slider::spawn(
            $commands,
            SliderSettings {
                lable: stringify!($name),
                value: Settings::default().$name as f32,
                range: $range,
                round_value: true,
                ..default()
            },
            ValueBind::<Settings, Slider> {
                dst: $settings,
                update: |settings, slider| settings.$name = slider.value as _,
            },
        )
    };
    ($commands:ident, $settings:ident, $name:ident, $range:expr, $getter:expr) => {
        Slider::spawn(
            $commands,
            SliderSettings {
                lable: stringify!($name),
                value: 0.,
                range: $range,
                round_value: true,
                ..default()
            },
            ValueBind::<Settings, Slider> {
                dst: $settings,
                update: |settings, slider| settings.$name = ($getter)(slider.value),
            },
        )
    };
}

impl SettingsUI {
    pub fn new(cmd: &mut Commands, settings: Entity) -> Entity {
        let panel = NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(4.),
                ..default()
            },
            background_color: Color::rgb(0., 0., 0.).into(),
            ..default()
        };

        let rows = &[
            slider!(cmd, settings, diameter, 16.0..=2048.0, int),
            slider!(cmd, settings, darken, 0.0..=254.0, int),
            slider!(
                cmd,
                settings,
                source_image_index,
                0.0..=IMAGES_PATHS.len() as f32,
                int
            ),
            slider!(cmd, settings, nodes, 16.0..=512.0, int),
            slider!(cmd, settings, nodes_offset, 0.0..=1.0, float),
            slider!(cmd, settings, string_alpha, 0.0..=1.0, float),
            slider!(cmd, settings, max_lines, 10.0..=12000.0, int),
            slider!(cmd, settings, repeat_lines, 0.0..=1.0, |value| value != 0.),
            slider!(
                cmd,
                settings,
                color_distance_fn,
                0.0..=1.0,
                |value| if value == 0. {
                    ColorDistanceFn::SQUARE
                } else {
                    ColorDistanceFn::ABS
                }
            ),
        ];

        let mut menu = cmd.spawn(panel);
        menu.push_children(rows);
        menu.id()
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_settings::<Settings, Slider>);
    }
}
