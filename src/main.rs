use BirdBoxes::{plugins::*, prelude::*};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::GOLD))
        .add_plugins(DefaultPlugins)
        .run();
}
