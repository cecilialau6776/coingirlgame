use bevy::{prelude::*, window::WindowResolution};

/// Stage for our systems
pub const APP_STATE_STAGE: &str = "app_state_stage";

/// States
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
  Menu,
  #[default]
  Game,
  Lost,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
pub enum ObjType {
  One,
  Five,
  Ten,
  Fifty,
  OneHundred,
  FiveHundred,
  RankUp,
  EraseItem,
}
impl ObjType {
  pub fn get_path(&self) -> &str {
    match self {
      ObjType::One => "one.png",
      ObjType::Five => "five.png",
      ObjType::Ten => "ten.png",
      ObjType::Fifty => "fifty.png",
      ObjType::OneHundred => "one_hundred.png",
      ObjType::FiveHundred => "five_hundred.png",
      ObjType::RankUp => "rank_up.png",
      ObjType::EraseItem => "erase.png",
    }
  }
  pub fn get_merge_count(&self) -> usize {
    match self {
      ObjType::One => 5,
      ObjType::Five => 2,
      ObjType::Ten => 5,
      ObjType::Fifty => 2,
      ObjType::OneHundred => 5,
      ObjType::FiveHundred => 2,
      ObjType::RankUp => 2,
      ObjType::EraseItem => 2,
    }
  }
  pub fn get_upgrade(&self) -> Option<Self> {
    match self {
      ObjType::One => Some(ObjType::Five),
      ObjType::Five => Some(ObjType::Ten),
      ObjType::Ten => Some(ObjType::Fifty),
      ObjType::Fifty => Some(ObjType::OneHundred),
      ObjType::OneHundred => Some(ObjType::FiveHundred),
      ObjType::FiveHundred => None,
      ObjType::RankUp => None,
      ObjType::EraseItem => None,
    }
  }
}

// Resolution
pub const RESOLUTION_X: f32 = 1080.0;
pub const RESOLUTION_Y: f32 = 2560.0;

pub const BOARD_MARGIN: Val = Val::Percent(2.0);

pub const COIN_SIZE_PX: f32 = 16.0;
pub const COIN_SIZE: Vec3 = Vec3::splat(COIN_SIZE_PX);
pub const GIRL_SIZE_FACTOR: f32 = 1.25;
pub const GIRL_SIZE: Vec3 = Vec3::splat(COIN_SIZE_PX * GIRL_SIZE_FACTOR);
pub const GAP_BETWEEN_COINS: f32 = 3.0;

// Z indices
pub const BACKGROUND_Z: f32 = 0.0;
pub const BOARD_Z: f32 = 1.0;
pub const GIRL_Z: f32 = 2.0;
pub const COIN_Z: f32 = 10.0;
pub const UI_Z: f32 = 20.0;

// Board Dimensions (width, height)
pub const BOARD_DIM: (i32, i32) = (7, 12);

#[derive(Component, Clone, Copy)]
pub struct ObjInfo {
  pub obj_type: ObjType,
  pub weight: i32,
  pub color: Color,
}

pub const BOARD_OBJS: [ObjInfo; 8] = [
  ObjInfo {
    obj_type: ObjType::One,
    weight: 10,
    color: Color::rgb(0.0, 0.0, 0.0),
  },
  ObjInfo {
    obj_type: ObjType::Five,
    weight: 10,
    color: Color::rgb(10.0, 10.0, 10.0),
  },
  ObjInfo {
    obj_type: ObjType::Ten,
    weight: 10,
    color: Color::rgb(20.0, 20.0, 20.0),
  },
  ObjInfo {
    obj_type: ObjType::Fifty,
    weight: 10,
    color: Color::rgb(20.0, 0.0, 0.0),
  },
  ObjInfo {
    obj_type: ObjType::OneHundred,
    weight: 10,
    color: Color::rgb(0.0, 20.0, 0.0),
  },
  ObjInfo {
    obj_type: ObjType::FiveHundred,
    weight: 5,
    color: Color::rgb(0.0, 0.0, 20.0),
  },
  ObjInfo {
    obj_type: ObjType::RankUp,
    weight: 1,
    color: Color::rgb(100.0, 100.0, 0.0),
  },
  ObjInfo {
    obj_type: ObjType::EraseItem,
    weight: 1,
    color: Color::rgb(0.0, 100.0, 100.0),
  },
];

pub const GAME_BOARD: UiRect = UiRect {
  left: Val::Percent(20.0),
  right: Val::Percent(20.0),
  top: Val::Percent(10.0),
  bottom: Val::Percent(10.0),
};

pub const TOP_HALF: UiRect = UiRect {
  left: Val::Percent(100.0),
  right: Val::Percent(100.0),
  top: Val::Percent(100.0),
  bottom: Val::Percent(50.0),
};

pub const BOTTOM_HALF: UiRect = UiRect {
  left: Val::Percent(100.0),
  right: Val::Percent(100.0),
  top: Val::Percent(50.0),
  bottom: Val::Percent(100.0),
};
