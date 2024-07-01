pub mod prelude {
  pub use bevy::prelude::*;
}

pub mod plugins {

}

mod app {
  use crate::prelude::*;
  #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
  pub enum AppState {
    #[default]
    Entry,
  }

  pub const APP_NAME: &str = "BirdBoxes";
}
