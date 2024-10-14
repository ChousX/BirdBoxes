use BirdBoxes::*;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.9)))
        .add_plugins(BirdBoxesPlugin)
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, test)
        .run();
}

fn test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    commands.spawn(Camera2dBundle::default());
    let mut iso_field = IsoField::new((2, 2));
    iso_field.set(1, 1, 1.0);
    let material = materials.add(Color::PINK);
    commands.spawn(BirdBoxeBundle{
        iso_field,
        transform: Transform::from_scale(Vec3::new(10.0, 10.0, 0.0)),
        material,
        ..default()
    });
    /*
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(Color::from(Color::PURPLE)),
        ..default()
    });
    */
}
