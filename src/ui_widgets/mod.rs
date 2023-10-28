mod slider;
pub use slider::*;

// mod grab::*;
// pub use grab::*;

use bevy::prelude::*;

pub struct UIWidgetsPlugin;

impl Plugin for UIWidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SliderPlugin);
    }
}

#[derive(Component)]
pub struct ValueBind<D, S> {
    pub dst: Entity,
    pub update: fn(dst: &mut D, src: &S),
}

pub fn update_settings<D: Component, S: Component>(
    src_query: Query<(&ValueBind<D, S>, &S), Changed<S>>,
    mut dst_query: Query<&mut D>,
) {
    for (value_bind, source) in &src_query {
        let mut destination = dst_query.get_mut(value_bind.dst).unwrap();
        (value_bind.update)(&mut destination, source);
    }
}
