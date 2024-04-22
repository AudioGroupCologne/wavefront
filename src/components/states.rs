use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Move;

#[derive(Component, Debug)]
pub struct Selected;

#[derive(Component, Debug)]
pub struct MenuSelected;
