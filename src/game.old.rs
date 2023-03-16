use bevy::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::consts::*;

const BOARD_OBJS: [(&str, i32, Color); 8] = [
  ("One", 10, Color::rgb(0.0, 0.0, 0.0)),
  ("Five", 10, Color::rgb(10.0, 10.0, 10.0)),
  ("Ten", 10, Color::rgb(20.0, 20.0, 20.0)),
  ("Fifty", 10, Color::rgb(20.0, 0.0, 0.0)),
  ("OneHundred", 10, Color::rgb(0.0, 20.0, 0.0)),
  ("FiveHundred", 5, Color::rgb(0.0, 0.0, 20.0)),
  ("RankUp", 1, Color::rgb(100.0, 100.0, 0.0)),
  ("EraseItem", 1, Color::rgb(0.0, 100.0, 100.0)),
];

#[derive(Component)]
struct BoardObj;
#[derive(Component)]
struct CoinGirl;

pub struct GamePlugin {
  pub player_count: usize,
}

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems((
      setup_game.in_schedule(OnEnter(AppState::Game)),
      // game.in_set(OnUpdate(AppState::Game)),
      // cleanup_game.in_schedule(OnExit(AppState::Game)),
    ));
  }
}

fn setup_game(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  asset_server: Res<AssetServer>,
) {
  commands.spawn(Camera2dBundle::default());

  // CoinGirl
  commands.spawn((
    SpriteBundle {
      transform: Transform {
        translation: Vec3::new(0.0, 100.0, 0.0),
        scale: GIRL_SIZE,
        ..default()
      },
      sprite: Sprite {
        color: Color::rgb(0.5, 0.5, 0.5),
        ..default()
      },
      ..default()
    },
    CoinGirl,
  ));

  // Coins
  let mut rng = thread_rng();
  let dist = WeightedIndex::new(BOARD_OBJS.iter().map(|item| item.1)).unwrap();
  // Initial Coins
  let coin_size = (COIN_SIZE.truncate() + Vec2::splat(GAP_BETWEEN_COINS));
  for row in 0..3 {
    for col in 0..7 {
      let position = Vec2::new(coin_size.x * col as f32, coin_size.y * row as f32);
      let item = BOARD_OBJS[dist.sample(&mut rng)];
      commands.spawn(SpriteBundle {
        transform: Transform {
          translation: position.extend(COIN_Z),
          scale: COIN_SIZE,
          ..default()
        },
        sprite: Sprite {
          color: item.2,
          ..default()
        },
        ..default()
      });
    }
  }
}
