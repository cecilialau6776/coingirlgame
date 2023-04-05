#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
mod consts;
use crate::consts::*;
use bevy::window::WindowResolution;
use bevy::{prelude::*, window::WindowResized};
use devcaders::{Button, DevcadeControls, Player};

mod menu;
use menu::MenuPlugin;
mod game;
use game::{get_board_quad, get_board_transform, GamePlugin};
use rand::rngs::ThreadRng;

#[derive(Resource, Default)]
pub struct GameInfo {
  players: usize,
}

#[derive(Resource, Default)]
pub struct RenderInfo {
  coin_size: f32,
  transform_p1: Transform,
  transform_p2: Transform,
}
impl RenderInfo {
  pub fn default() -> Self {
    RenderInfo {
      coin_size: 0.0,
      transform_p1: Transform::from_translation(Vec3::ZERO),
      transform_p2: Transform::from_translation(Vec3::ZERO),
    }
  }
  pub fn board_transform(&self, player: Player) -> Transform {
    match player {
      Player::P1 => self.transform_p1,
      Player::P2 => self.transform_p2,
    }
  }

  pub fn obj_translate(&self, player: Player, col: i32, row: i32) -> Vec2 {
    Vec2::new(
      self.board_transform(player).translation.x + (-3.0 + col as f32) * self.coin_size,
      self.board_transform(player).translation.y + (5.5 - row as f32) * self.coin_size,
    )
  }
}

fn init_render_info(
  mut render_info: ResMut<RenderInfo>,
  game_info: Res<GameInfo>,
  window: Query<&mut Window>,
) {
  let board_quad = get_board_quad(game_info.players, Player::P1, &window.single().resolution);
  render_info.transform_p1 =
    get_board_transform(game_info.players, Player::P1, &window.single().resolution);
  render_info.transform_p2 =
    get_board_transform(game_info.players, Player::P2, &window.single().resolution);
  render_info.coin_size = (board_quad.size.x
    - 2.0 * BOARD_MARGIN.evaluate(board_quad.size.x).unwrap())
    / BOARD_DIM.0 as f32;
}

fn update_render_info(
  mut event: EventReader<WindowResized>,
  mut render_info: ResMut<RenderInfo>,
  game_info: Res<GameInfo>,
  window: Query<&mut Window>,
) {
  if game_info.is_changed() {
    let board_quad = get_board_quad(game_info.players, Player::P1, &window.single().resolution);
    render_info.transform_p1 =
      get_board_transform(game_info.players, Player::P1, &window.single().resolution);
    render_info.transform_p2 =
      get_board_transform(game_info.players, Player::P2, &window.single().resolution);
    render_info.coin_size = (board_quad.size.x
      - 2.0 * BOARD_MARGIN.evaluate(board_quad.size.x).unwrap())
      / BOARD_DIM.0 as f32;
  }
  for ev in event.iter() {
    let board_quad = get_board_quad(game_info.players, Player::P1, &window.single().resolution);
    render_info.transform_p1 =
      get_board_transform(game_info.players, Player::P1, &window.single().resolution);
    render_info.transform_p2 =
      get_board_transform(game_info.players, Player::P2, &window.single().resolution);
    render_info.coin_size = (board_quad.size.x
      - 2.0 * BOARD_MARGIN.evaluate(board_quad.size.x).unwrap())
      / BOARD_DIM.0 as f32;
  }
}

fn setup(mut commands: Commands) {
  commands.spawn(Camera2dBundle::default());
  return;

  // // CoinGirl
  // commands.spawn((
  //   SpriteBundle {
  //     transform: Transform {
  //       translation: Vec3::new(0.0, 100.0, 0.0),
  //       scale: COIN_SIZE,
  //       ..default()
  //     },
  //     sprite: Sprite {
  //       color: Color::rgb(50.0, 168.0, 82.0),
  //       ..default()
  //     },
  //     ..default()
  //   },
  //   CoinGirl,
  // ));

  // let mut rng = thread_rng();
  // let offset_x = 10.0;
  // let offset_y = 10.0;
  // let dist = WeightedIndex::new(BOARD_OBJS.iter().map(|item| item.1)).unwrap();
  // // Initial Coins
  // for row in 0..3 {
  //   for col in 0..7 {
  //     let position = COIN_SIZE.truncate() + Vec2::splat(GAP_BETWEEN_COINS);
  //     let item = BOARD_OBJS[dist.sample(&mut rng)];
  //     commands.spawn(SpriteBundle {
  //       transform: Transform {
  //         translation: position.extend(0.0),
  //         scale: COIN_SIZE,
  //         ..default()
  //       },
  //       sprite: Sprite {
  //         color: item.2,
  //         ..default()
  //       },
  //       ..default()
  //     });
  //   }
  // }
}

// fn girl_movement(
//   input: DevcadeControls,
//   mut girl_positions: Query<&mut Transform, With<CoinGirl>>,
// ) {
//   for mut transform in girl_positions.iter_mut() {
//     if input.just_pressed(Player::P1, Button::StickLeft) {
//       transform.translation.x -= COIN_SIZE + GAP_BETWEEN_COINS;
//     }
//     if input.just_pressed(Player::P1, Button::StickRight) {
//       transform.translation.x += COIN_SIZE + GAP_BETWEEN_COINS;
//     }
//   }
// }

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(SpriteBundle {
    // texture: asset_server.load("branding/icon.png"),
    ..default()
  });
}

fn main() {
  App::new()
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            mode: bevy::window::WindowMode::Fullscreen,
            resolution: WindowResolution::new(RESOLUTION_X, RESOLUTION_Y), //.with_scale_factor_override(0.5874),
            // resize_constraints: WindowResizeConstraints {
            //   min_width: 1080.0,
            //   min_height: 2560.0,
            //   max_width: 1080.0,
            //   max_height: 2560.0,
            // },
            ..default()
          }),
          ..default()
        })
        .set(ImagePlugin::default_nearest()),
    )
    .add_state::<AppState>()
    .add_startup_system(setup)
    .insert_resource(GameInfo { players: 2 })
    .insert_resource(RenderInfo::default())
    .add_startup_system(init_render_info)
    .add_plugin(MenuPlugin)
    .add_plugin(GamePlugin)
    .add_systems((devcaders::close_on_menu_buttons, update_render_info))
    // .insert_resource(FixedTime::new_from_secs(1.0 / 30.0))
    .run();
}
