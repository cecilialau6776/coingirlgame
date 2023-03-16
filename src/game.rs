use std::time::Duration;

use bevy::prelude::shape::Quad;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::Stopwatch;
use bevy::{prelude::*, window::WindowResolution};
use devcaders::{DevcadeControls, Player};
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::{consts::*, GameInfo, RenderInfo};

enum ActionType {
  CoinPull,
  CoinPush,
  NewRow,
}

struct GameActionEvent {
  player: Player,
  action_type: ActionType,
}

#[derive(Component)]
struct CoinGirl {
  pub player: Player,
  pub col: usize,
  pub timer: Timer,
  obj_count: usize,
  obj_type: ObjType,
}
impl CoinGirl {
  fn new(player: Player) -> CoinGirl {
    let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
    timer.pause();
    CoinGirl {
      player,
      col: 3,
      timer,
      obj_count: 0,
      obj_type: ObjType::One,
    }
  }
}

#[derive(Component, Clone)]
struct BoardObj {
  pub col: usize,
  pub row: usize,
  pub obj_info: ObjInfo,
  pub owned: bool,
  pub player: Player,
}

#[derive(Bundle, Clone)]
struct BoardObjBundle {
  board_obj: BoardObj,
  sprite_bundle: SpriteBundle,
}
impl BoardObjBundle {
  fn new(
    obj_info: ObjInfo,
    col: usize,
    row: usize,
    player: Player,
    asset_server: &Res<AssetServer>,
    render_info: &Res<RenderInfo>,
  ) -> Self {
    BoardObjBundle {
      board_obj: BoardObj {
        col,
        row,
        obj_info,
        player,
        owned: false,
      },
      sprite_bundle: SpriteBundle {
        transform: Transform {
          translation: render_info
            .obj_translate(player, col, row) //(top_left_coin_pos + Vec2::new(col as f32, -(row as f32)) * coin_size)
            .extend(COIN_Z),
          scale: Vec3::splat(render_info.coin_size / COIN_SIZE_PX),
          ..default()
        },
        texture: asset_server.load(obj_info.obj_type.get_path()),
        ..default()
      },
    }
  }

  fn other_player(&self, render_info: &Res<RenderInfo>) -> Self {
    let mut obj = self.clone();
    obj.board_obj.player = Player::P2;
    obj.sprite_bundle.transform.translation = render_info
      .obj_translate(Player::P2, obj.board_obj.col, obj.board_obj.row)
      .extend(COIN_Z);
    obj
  }
}

#[derive(Component)]
struct Game {
  // pub rect: UiRect,
  pub player: Player,
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<GameActionEvent>()
      .add_systems((
        setup_game.in_schedule(OnEnter(AppState::Game)),
        time.in_set(OnUpdate(AppState::Game)),
        render.in_set(OnUpdate(AppState::Game)),
        // cleanup_game.in_schedule(OnExit(AppState::Game)),
      ))
      .add_systems((
        game_input.in_set(OnUpdate(AppState::Game)),
        game_action_event_handler.in_set(OnUpdate(AppState::Game)),
      ));
  }
}

fn time(time: Res<Time>, mut girl_query: Query<&mut CoinGirl>) {
  for mut girl in girl_query.iter_mut() {
    girl.timer.tick(time.delta());
  }
}

fn render(
  mut set: ParamSet<(
    Query<(&mut Transform, &BoardObj), With<BoardObj>>,
    Query<(&mut Transform, &CoinGirl), With<CoinGirl>>,
  )>,
  game_state: Res<GameInfo>,
  render_info: Res<RenderInfo>,
) {
  // render girl
  for (mut girl_transform, girl) in set.p1().iter_mut() {
    girl_transform.translation = render_info
      .obj_translate(girl.player, girl.col, BOARD_DIM.1 - 1)
      .extend(GIRL_Z);
  }

  // render coins
  for (mut transform, obj) in set.p0().iter_mut() {
    transform.translation = render_info
      .obj_translate(obj.player, obj.col, obj.row)
      .extend(GIRL_Z);
  }
}

fn game_input(
  input: DevcadeControls,
  mut query: Query<&mut CoinGirl, With<CoinGirl>>,
  mut action_writer: EventWriter<GameActionEvent>,
  time_step: Res<FixedTime>,
) {
  for mut girl in &mut query {
    if input.just_pressed(girl.player, devcaders::Button::StickLeft) {
      if girl.col > 0 {
        girl.col -= 1;
      }
    }
    if input.just_pressed(girl.player, devcaders::Button::StickRight) {
      if girl.col < 6 {
        girl.col += 1;
      }
    }

    if input.just_pressed(girl.player, devcaders::Button::StickDown) {
      // timer is running and finished
      if !girl.timer.paused() && !girl.timer.finished() {
        // new row
        action_writer.send(GameActionEvent {
          player: girl.player,
          action_type: ActionType::NewRow,
        });
        girl.timer.pause();
      } else {
        // timer is paused or finished
        girl.timer.reset();
        girl.timer.unpause();
      }
    }

    if input.just_pressed(girl.player, devcaders::Button::A1) {
      action_writer.send(GameActionEvent {
        player: girl.player,
        action_type: ActionType::CoinPull,
      });
    }
    if input.just_pressed(girl.player, devcaders::Button::A2) {
      action_writer.send(GameActionEvent {
        player: girl.player,
        action_type: ActionType::CoinPush,
      });
    }
  }
}

fn game_action_event_handler(
  mut events: EventReader<GameActionEvent>,
  mut set: ParamSet<(
    Query<&mut BoardObj>,
    Query<(&mut Transform, &CoinGirl), With<CoinGirl>>,
  )>,
  asset_server: Res<AssetServer>,
  render_info: Res<RenderInfo>,
  mut commands: Commands,
) {
  for ev in events.iter() {
    match ev.action_type {
      ActionType::CoinPull => {
        // for (_, girl) in set.p1().iter() {
        //   for mut obj in set.p0().iter_mut() {
        //     // if !obj.owned && obj.player == ev.player && obj.col ==
        //   }
        // }
      }
      ActionType::CoinPush => {}
      ActionType::NewRow => {
        // move existing coins down one
        for mut obj in set.p0().iter_mut() {
          if !obj.owned && obj.player == ev.player {
            obj.row += 1;
          }
        }
        // spawn a new row
        let mut rng = thread_rng();
        let dist = WeightedIndex::new(BOARD_OBJS.iter().map(|item| item.weight)).unwrap();
        for col in 0..BOARD_DIM.0 {
          let item = BOARD_OBJS[dist.sample(&mut rng)].clone();
          let obj = BoardObjBundle::new(item, col, 0, ev.player, &asset_server, &render_info);
          commands.spawn(obj);
        }
      }
    }
  }
}

fn setup_game(
  mut commands: Commands,
  mut materials: ResMut<Assets<ColorMaterial>>,
  mut meshes: ResMut<Assets<Mesh>>,
  asset_server: Res<AssetServer>,
  game_state: Res<GameInfo>,
  render_info: Res<RenderInfo>,
  window: Query<&mut Window>,
) {
  let resolution = &window.single().resolution;
  let mut rng = thread_rng();

  let transform_p1 = get_board_transform(game_state.players, Player::P1, resolution);
  let transform_p2 = get_board_transform(game_state.players, Player::P2, resolution);
  let board_quad = get_board_quad(game_state.players, Player::P1, resolution);

  // Board
  commands.spawn(MaterialMesh2dBundle {
    mesh: meshes.add(board_quad.into()).into(),
    material: materials.add(ColorMaterial::from(Color::rgb(0.5, 0.5, 0.5))),
    transform: transform_p1,
    ..default()
  });
  if game_state.players == 2 {
    commands.spawn(MaterialMesh2dBundle {
      mesh: meshes.add(board_quad.into()).into(),
      material: materials.add(ColorMaterial::from(Color::rgb(0.5, 0.5, 0.5))),
      transform: transform_p2,
      ..default()
    });
  }

  // Initial Coins
  let dist = WeightedIndex::new(BOARD_OBJS.iter().map(|item| item.weight)).unwrap();
  let coin_size: f32 = (board_quad.size.x
    - 2.0 * BOARD_MARGIN.evaluate(board_quad.size.x).unwrap())
    / BOARD_DIM.0 as f32;
  // let top_left_coin_pos_p1: Vec2 = Vec2::new(
  //   transform_p1.translation.x - 3.0 * coin_size,
  //   transform_p1.translation.y + 5.5 * coin_size,
  // );
  // let top_left_coin_pos_p2: Vec2 = Vec2::new(
  //   transform_p2.translation.x - 3.0 * coin_size,
  //   transform_p2.translation.y + 5.5 * coin_size,
  // );

  for row in 0..3 {
    for col in 0..BOARD_DIM.0 {
      // let position_p1 = top_left_coin_pos_p1 + Vec2::new(col as f32, -(row as f32)) * coin_size;
      // let position_p2 = top_left_coin_pos_p2 + Vec2::new(col as f32, -(row as f32)) * coin_size;
      let item = BOARD_OBJS[dist.sample(&mut rng)].clone();
      let obj = BoardObjBundle::new(item, col, row, Player::P1, &asset_server, &render_info);
      if game_state.players == 2 {
        commands.spawn(obj.other_player(&render_info));
      }
      commands.spawn(obj);
    }
  }

  // Coin Girl
  let coin_girl_pos =
    (Vec2::new((BOARD_DIM.0 / 2) as f32, -((BOARD_DIM.1 - 1) as f32))) * coin_size;
  commands.spawn((
    SpriteBundle {
      transform: Transform {
        translation: render_info
          .obj_translate(Player::P1, BOARD_DIM.0 / 2, BOARD_DIM.1 - 1)
          .extend(GIRL_Z),
        scale: Vec3::splat(coin_size * GIRL_SIZE_FACTOR),
        ..default()
      },
      sprite: Sprite {
        color: Color::rgb(50.0 / 255.0, 168.0 / 255.0, 82.0 / 255.0),
        ..default()
      },
      ..default()
    },
    CoinGirl::new(Player::P1),
  ));
  println!(
    "{:?}",
    render_info.obj_translate(Player::P1, BOARD_DIM.0 / 2, BOARD_DIM.1 - 1)
  );
  if game_state.players == 2 {
    commands.spawn((
      SpriteBundle {
        transform: Transform {
          translation: render_info
            .obj_translate(Player::P1, BOARD_DIM.0 / 2, BOARD_DIM.1 - 1)
            .extend(GIRL_Z),
          scale: Vec3::splat(coin_size * GIRL_SIZE_FACTOR),
          ..default()
        },
        sprite: Sprite {
          color: Color::rgb(50.0 / 255.0, 168.0 / 255.0, 82.0 / 255.0),
          ..default()
        },
        ..default()
      },
      CoinGirl::new(Player::P2),
    ));
  }
}

pub fn get_board_quad(players: usize, player: Player, resolution: &WindowResolution) -> Quad {
  let avail_x: f32;
  let avail_y: f32;
  if players == 2 {
    // figure out divide screen vertically or horizontally
    if resolution.width() > resolution.height() {
      // split horizontally
      avail_x = resolution.width() / 2.0;
      avail_y = resolution.height();
    } else {
      // split vertically
      avail_x = resolution.width();
      avail_y = resolution.height() / 2.0;
    }
  } else {
    avail_x = resolution.width();
    avail_y = resolution.height();
  }
  let width: f32;
  let height: f32;
  let margin = BOARD_MARGIN.evaluate(avail_x).unwrap();
  if (avail_y * BOARD_DIM.0 as f32 / BOARD_DIM.1 as f32) < avail_x {
    // height bound
    height = avail_y - 2.0 * margin;
    width = height / BOARD_DIM.1 as f32 * BOARD_DIM.0 as f32;
  } else {
    // width bound
    width = avail_x - 2.0 * margin;
    height = width * BOARD_DIM.1 as f32 / BOARD_DIM.0 as f32;
  }

  Quad::new(Vec2::new(width, height))
  // Rect::from_center_size(Vec2::ZERO, Vec2::new(width, height))
  // Transform::default()
}

pub fn get_board_transform(
  players: usize,
  player: Player,
  resolution: &WindowResolution,
) -> Transform {
  let avail_x = resolution.width();
  // let avail_x = if players == 1 {
  //   resolution.width()
  // } else {
  //   resolution.width() / 2.0
  // };
  let avail_y = if players == 1 {
    resolution.height()
  } else {
    resolution.height() / 2.0
  };

  let x: f32;
  let y: f32;
  let width: f32;
  let height: f32;
  let margin = BOARD_MARGIN.evaluate(avail_x).unwrap();
  if (avail_y * BOARD_DIM.0 as f32 / BOARD_DIM.1 as f32) < avail_x {
    // height bound
    height = avail_y - 2.0 * margin;
    width = height / BOARD_DIM.1 as f32 * BOARD_DIM.0 as f32;
    y = 0.0;
    if players == 2 {
      x = if player == Player::P1 {
        -avail_x / 4.0
      } else {
        avail_x / 4.0
      };
    } else {
      x = 0.0;
    }
  } else {
    // width bound
    width = avail_x - 2.0 * margin;
    height = width * BOARD_DIM.1 as f32 / BOARD_DIM.0 as f32;
    x = 0.0;
    if players == 2 {
      y = if player == Player::P1 {
        avail_y / 4.0
      } else {
        -avail_y / 4.0
      };
    } else {
      y = 0.0;
    }
  }
  Transform::from_translation(Vec3::new(x, y, BOARD_Z)) //.with_scale(Vec3::new(width, height, 1.0))
                                                        // Transform::default()
}
