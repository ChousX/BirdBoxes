use BirdBoxes::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.9)))
        .add_plugins(DefaultPlugins)
        .add_plugins(BirdBoxesPlugin)
        .add_systems(Startup, test)
        .run();
}

fn test(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());
    let mut iso_field = IsoField::new((2, 2));
    iso_field.set((1, 1), (0,0), 1.0);
    commands.spawn(BirdBoxeBundle{
        iso_field,
        ..default()
    });
}
