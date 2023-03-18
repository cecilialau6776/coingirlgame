use std::cmp::Ordering;
use std::time::Duration;

use bevy::prelude::shape::Quad;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::Stopwatch;
use bevy::transform::commands;
use bevy::utils::HashSet;
use bevy::{prelude::*, window::WindowResolution};
use devcaders::{DevcadeControls, Player};
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::{consts::*, GameInfo, RenderInfo};

#[derive(PartialEq, Eq)]
enum ActionType {
  CoinPull,
  CoinPush,
  NewRow,
}

struct MergeEvent {
  player: Player,
  position: Position,
}

struct GameActionEvent {
  player: Player,
  action_type: ActionType,
}

#[derive(Component, Clone)]
struct InputTimer {
  timer: Timer,
}

#[derive(Component, Clone)]
struct Inventory {
  obj_count: i32,
  obj_type: ObjType,
}

#[derive(Component, Clone)]
struct CoinGirl;
impl CoinGirl {
  fn spawn(commands: &mut Commands, player: Player, render_info: &Res<RenderInfo>) -> Entity {
    let mut timer = InputTimer {
      timer: Timer::from_seconds(1.0, TimerMode::Once),
    };
    timer.timer.pause();
    commands
      .spawn((
        CoinGirl,
        player,
        Position {
          col: BOARD_DIM.0 / 2,
          row: BOARD_DIM.1 - 1,
        },
        timer,
        Inventory {
          obj_count: 0,
          obj_type: ObjType::One,
        },
        SpriteBundle {
          transform: Transform {
            translation: render_info
              .obj_translate(player, BOARD_DIM.0 / 2, BOARD_DIM.1 - 1)
              .extend(GIRL_Z),
            scale: Vec3::splat(render_info.coin_size * GIRL_SIZE_FACTOR),
            ..default()
          },
          sprite: Sprite {
            color: Color::rgb(50.0 / 255.0, 168.0 / 255.0, 82.0 / 255.0),
            ..default()
          },
          ..default()
        },
      ))
      .id()
  }
}

#[derive(PartialOrd, Hash, PartialEq, Eq, Debug, Component, Clone, Copy)]
struct Position {
  pub col: i32,
  pub row: i32,
}
impl Ord for Position {
  fn cmp(&self, other: &Self) -> Ordering {
    if self == other {
      return Ordering::Equal;
    }
    if self.col < other.col {
      return Ordering::Less;
    } else if self.row > other.row {
      return Ordering::Less;
    }
    return Ordering::Greater;
  }
}

#[derive(Component, Clone, PartialEq, Eq)]
struct Owned(bool);

#[derive(Component, Clone)]
struct BoardObj;

impl BoardObj {
  fn spawn(
    commands: &mut Commands,
    obj_type: ObjType,
    col: i32,
    row: i32,
    player: Player,
    asset_server: &Res<AssetServer>,
    render_info: &Res<RenderInfo>,
  ) -> Entity {
    commands
      .spawn((
        BoardObj,
        Position { col, row },
        obj_type,
        player,
        SpriteBundle {
          transform: Transform {
            translation: render_info
              .obj_translate(player, col, row) //(top_left_coin_pos + Vec2::new(col as f32, -(row as f32)) * coin_size)
              .extend(COIN_Z),
            scale: Vec3::splat(render_info.coin_size / COIN_SIZE_PX),
            ..default()
          },
          texture: asset_server.load(obj_type.get_path()),
          ..default()
        },
        Owned(false),
      ))
      .id()
  }

  //   fn other_player(&self, render_info: &Res<RenderInfo>) -> Self {
  //     let mut obj = self.clone();
  //     obj.player = Player::P2;
  //     obj.sprite_bundle.transform.translation = render_info
  //       .obj_translate(Player::P2, obj.position.col, obj.position.row)
  //       .extend(COIN_Z);
  //     obj
  //   }
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
      .add_event::<MergeEvent>()
      .add_systems((
        setup_game.in_schedule(OnEnter(AppState::Game)),
        time.in_set(OnUpdate(AppState::Game)),
        render.in_set(OnUpdate(AppState::Game)),
        // cleanup_game.in_schedule(OnExit(AppState::Game)),
      ))
      .add_systems((
        game_input.before(render).in_set(OnUpdate(AppState::Game)),
        coin_pull_handler
          .after(game_input)
          .in_set(OnUpdate(AppState::Game)),
        coin_push_handler
          .after(game_input)
          .in_set(OnUpdate(AppState::Game)),
        new_row_handler
          .after(game_input)
          .in_set(OnUpdate(AppState::Game)),
        merge_handler
          .after(coin_push_handler)
          .before(render)
          .in_set(OnUpdate(AppState::Game)),
      ));
  }
}

fn time(time: Res<Time>, mut girl_query: Query<&mut InputTimer>) {
  for mut girl in girl_query.iter_mut() {
    girl.timer.tick(time.delta());
  }
}

fn render(
  mut query: Query<(&mut Transform, &Position, &Player)>,
  game_state: Res<GameInfo>,
  render_info: Res<RenderInfo>,
) {
  for (mut transform, position, &player) in &mut query {
    let z_index = transform.translation.z;
    transform.translation = render_info
      .obj_translate(player, position.col, position.row)
      .extend(z_index);
  }
  // // render girl
  // for (mut girl_transform, girl) in set.p1().iter_mut() {
  //   girl_transform.translation = render_info
  //     .obj_translate(girl.player, girl.col, BOARD_DIM.1 - 1)
  //     .extend(GIRL_Z);
  // }

  // // render coins
  // for (mut transform, obj) in set.p0().iter_mut() {
  //   transform.translation = render_info
  //     .obj_translate(obj.player, obj.col, obj.row)
  //     .extend(GIRL_Z);
  // }
}

fn game_input(
  input: DevcadeControls,
  mut girl_query: Query<
    (&Player, &mut Position, &mut InputTimer),
    (With<CoinGirl>, Without<BoardObj>),
  >,
  mut obj_query: Query<(&Player, &mut Position, &Owned), With<BoardObj>>,
  mut action_writer: EventWriter<GameActionEvent>,
  time_step: Res<FixedTime>,
) {
  for (&player, mut position, mut timer) in &mut girl_query {
    if input.just_pressed(player, devcaders::Button::StickLeft) {
      if position.col > 0 {
        position.col -= 1;
        for (&coin_player, mut coin_pos, owned) in &mut obj_query {
          if player == coin_player && owned.0 {
            coin_pos.col -= 1;
          }
        }
      }
    }
    if input.just_pressed(player, devcaders::Button::StickRight) {
      if position.col < 6 {
        position.col += 1;
        for (&coin_player, mut coin_pos, owned) in &mut obj_query {
          if player == coin_player && owned.0 {
            coin_pos.col += 1;
          }
        }
      }
    }

    if input.just_pressed(player, devcaders::Button::StickDown) {
      // timer is running and finished
      if !timer.timer.paused() && !timer.timer.finished() {
        // new row
        action_writer.send(GameActionEvent {
          player,
          action_type: ActionType::NewRow,
        });
        timer.timer.pause();
      } else {
        // timer is paused or finished
        timer.timer.reset();
        timer.timer.unpause();
      }
    }

    if input.just_pressed(player, devcaders::Button::A1) {
      action_writer.send(GameActionEvent {
        player,
        action_type: ActionType::CoinPull,
      });
    }
    if input.just_pressed(player, devcaders::Button::A2) {
      action_writer.send(GameActionEvent {
        player,
        action_type: ActionType::CoinPush,
      });
    }
  }
}

fn coin_push_handler(
  mut events: EventReader<GameActionEvent>,
  mut coin_query: Query<(Entity, &mut Position, &Player, &ObjType, &mut Owned), With<BoardObj>>,
  mut girl_query: Query<(&mut Inventory, &Player, &Position), (With<CoinGirl>, Without<BoardObj>)>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  render_info: Res<RenderInfo>,
  mut merge_event_writer: EventWriter<MergeEvent>,
) {
  for ev in events.iter() {
    if ev.action_type != ActionType::CoinPush {
      return;
    }
    for (mut inventory, &girl_player, girl_pos) in &mut girl_query {
      if girl_player != ev.player {
        continue;
      }
      if inventory.obj_count == 0 {
        continue;
      }

      let mut max_row = 0;
      let mut same_coin_types = Vec::new();
      for (entity, coin_pos, &coin_player, &obj_type, owned) in &coin_query {
        if coin_player != ev.player || owned.0 {
          continue;
        }
        if obj_type == inventory.obj_type {
          same_coin_types.push((entity, coin_pos.clone()));
        }
        if coin_pos.col != girl_pos.col {
          continue;
        }
        if coin_pos.row + 1 > max_row {
          max_row = coin_pos.row + 1;
        }
      }

      let placed_coin_pos = Position {
        col: girl_pos.col,
        row: girl_pos.row - (BOARD_DIM.1 - max_row - inventory.obj_count),
      };
      // move inv back to board
      for (entity, mut coin_pos, &coin_player, &obj_type, mut owned) in &mut coin_query {
        if coin_player != ev.player || !owned.0 {
          continue;
        }
        coin_pos.row -= BOARD_DIM.1 - max_row - inventory.obj_count;
        owned.0 = false;
        same_coin_types.push((entity, coin_pos.to_owned().clone()));
      }
      inventory.obj_count = 0;
      merge_event_writer.send(MergeEvent {
        player: ev.player,
        position: placed_coin_pos,
      });
    }
  }
}

fn get_connected(
  coin_list: &Vec<(Entity, Position, &Player, ObjType)>,
  position: Position,
  player: Player,
) -> Option<(ObjType, Vec<(Entity, Position)>)> {
  // println!("{:?}", position);
  // for c in coin_query {
  //   if *c.2 == player {//     println!("{:?} {:?}", c.1, c.3);
  //   }
  // }
  let coin = coin_list
    .iter()
    .find(|&(_, pos, &q_player, _)| *pos == position && q_player == player)?
    .to_owned();
  let init_obj_type = coin.3.to_owned();
  let mut set = HashSet::new();
  let mut stack = Vec::new();
  stack.push((coin.0, position));
  while !stack.is_empty() {
    let (coin_entity, coin_pos) = stack.pop().unwrap();
    if !set.contains(&(coin_entity, coin_pos)) {
      if let Some(adj_coin) = coin_list.iter().find(|(_, pos, _, obj_type)| {
        *obj_type == init_obj_type && pos.col == coin_pos.col - 1 && pos.row == coin_pos.row
      }) {
        stack.push((adj_coin.0.to_owned(), adj_coin.1.to_owned()));
      }
      if let Some(adj_coin) = coin_list.iter().find(|(_, pos, _, obj_type)| {
        *obj_type == init_obj_type && pos.col == coin_pos.col + 1 && pos.row == coin_pos.row
      }) {
        stack.push((adj_coin.0.to_owned(), adj_coin.1.to_owned()));
      }
      if let Some(adj_coin) = coin_list.iter().find(|(_, pos, _, obj_type)| {
        *obj_type == init_obj_type && pos.col == coin_pos.col && pos.row == coin_pos.row - 1
      }) {
        stack.push((adj_coin.0.to_owned(), adj_coin.1.to_owned()));
      }
      if let Some(adj_coin) = coin_list.iter().find(|(_, pos, _, obj_type)| {
        *obj_type == init_obj_type && pos.col == coin_pos.col && pos.row == coin_pos.row + 1
      }) {
        stack.push((adj_coin.0.to_owned(), adj_coin.1.to_owned()));
      }
      set.insert((coin_entity, coin_pos));
    }
  }
  Some((init_obj_type, set.into_iter().collect()))
}

fn merge_handler(
  mut events: EventReader<MergeEvent>,
  coin_query: Query<(Entity, &Position, &Player, &ObjType), With<BoardObj>>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  render_info: Res<RenderInfo>,
) {
  // let mut entities_to_remove: Vec<Entity> = Vec::new();
  // let mut entities_to_spawn = Vec::new();
  for ev in events.iter() {
    let coin_list: Vec<(Entity, Position, &Player, ObjType)> = coin_query
      .iter()
      .filter(|&(_, _, &player, _)| player == ev.player)
      .map(|x| (x.0, x.1.to_owned(), x.2, x.3.to_owned()))
      .collect();
    let mut pos = ev.position.clone();
    let (obj_type, coins) = get_connected(&coin_list, pos, ev.player).unwrap();
    let mut coin_count = coins.len();
    println!("{:?}, {}\n{:?}\n", obj_type, coins.len(), coins);
    // while coin_count >= obj_type.get_merge_count() {
    if coin_count >= obj_type.get_merge_count() {
      // actually merge
      // println!(
      //   "\nmerging {coin_count} {:?} coins that are at {:?}\n",
      //   obj_type, coins
      // );

      // create new coin
      let new_pos = coins
        .iter()
        .max_by(|&(_, pos_a), &(_, pos_b)| pos_a.cmp(pos_b))
        .unwrap()
        .to_owned()
        .1;
      if let Some(new_type) = obj_type.get_upgrade() {
        println!("new {:?} at: {:?}", new_type, new_pos);
        BoardObj::spawn(
          &mut commands,
          new_type,
          new_pos.col,
          new_pos.row,
          ev.player,
          &asset_server,
          &render_info,
        );
      }
      // entities_to_spawn.push((new_type, new_pos, ev.player));
      // coin_list.push((None, new_pos, &ev.player, new_type));
      // remove existing coins
      for (entity, _) in coins {
        commands.entity(entity).despawn();
      }
    }
  }
}

fn coin_pull_handler(
  mut events: EventReader<GameActionEvent>,
  mut girl_query: Query<(&Position, &Player, &mut Inventory), (With<CoinGirl>, Without<BoardObj>)>,
  mut coin_query: Query<
    (Entity, &mut Position, &Player, &ObjType, &mut Owned),
    (With<BoardObj>, Without<CoinGirl>),
  >,
  asset_server: Res<AssetServer>,
  render_info: Res<RenderInfo>,
  mut commands: Commands,
) {
  for ev in events.iter() {
    if ev.action_type != ActionType::CoinPull {
      return;
    }
    for (girl_pos, &girl_player, mut inventory) in &mut girl_query {
      if ev.player != girl_player {
        continue;
      }
      if inventory.obj_count != 0 {
        return;
      }
      let mut coins = Vec::new();
      for (entity, coin_pos, &coin_player, &obj, owned) in &mut coin_query {
        if !owned.0 && girl_player == coin_player && girl_pos.col == coin_pos.col {
          coins.push((coin_pos, obj, entity, owned));
        }
      }
      coins.sort_by(|(a, _, _, _), (b, _, _, _)| b.row.cmp(&a.row));
      let first_coin = coins.first();
      let mut valid_coin_pos = Vec::new();
      if let Some(&(_, coin_type, _, _)) = first_coin {
        for (coin_pos, obj_type, entity, mut owned) in coins {
          if coin_type != obj_type {
            break;
          }
          valid_coin_pos.push((coin_pos, entity));
          owned.0 = true;
        }
        let move_down_by = 12 - 1 - valid_coin_pos.first().unwrap().0.row;
        inventory.obj_type = coin_type;
        inventory.obj_count = valid_coin_pos.len() as i32;
        for (mut pos, entity) in valid_coin_pos {
          pos.row += move_down_by;
        }
      }
    }
  }
}

fn new_row_handler(
  mut events: EventReader<GameActionEvent>,
  mut query: Query<(&mut Position, &Player, &Owned), With<BoardObj>>,
  asset_server: Res<AssetServer>,
  render_info: Res<RenderInfo>,
  mut commands: Commands,
) {
  for ev in events.iter() {
    if ev.action_type == ActionType::NewRow {
      // move existing coins down one
      for (mut position, &player, owned) in &mut query {
        if !owned.0 && player == ev.player {
          position.row += 1;
        }
      }
      // spawn a new row
      let mut rng = thread_rng();
      let dist = WeightedIndex::new(BOARD_OBJS.iter().map(|item| item.weight)).unwrap();
      for col in 0..BOARD_DIM.0 {
        let item = BOARD_OBJS[dist.sample(&mut rng)].clone();
        BoardObj::spawn(
          &mut commands,
          item.obj_type,
          col,
          0,
          ev.player,
          &asset_server,
          &render_info,
        );
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

  let board_quad = get_board_quad(game_state.players, Player::P1, resolution);

  // Board
  commands.spawn(MaterialMesh2dBundle {
    mesh: meshes.add(board_quad.into()).into(),
    material: materials.add(ColorMaterial::from(Color::rgb(0.5, 0.5, 0.5))),
    transform: render_info.transform_p1,
    ..default()
  });
  if game_state.players == 2 {
    commands.spawn(MaterialMesh2dBundle {
      mesh: meshes.add(board_quad.into()).into(),
      material: materials.add(ColorMaterial::from(Color::rgb(0.5, 0.5, 0.5))),
      transform: render_info.transform_p2,
      ..default()
    });
  }

  // Initial Coins
  let dist = WeightedIndex::new(BOARD_OBJS.iter().map(|item| item.weight)).unwrap();
  for row in 0..3 {
    for col in 0..BOARD_DIM.0 {
      let item = BOARD_OBJS[dist.sample(&mut rng)].clone();
      BoardObj::spawn(
        &mut commands,
        item.obj_type,
        col,
        row,
        Player::P1,
        &asset_server,
        &render_info,
      );
      if game_state.players == 2 {
        BoardObj::spawn(
          &mut commands,
          item.obj_type,
          col,
          row,
          Player::P2,
          &asset_server,
          &render_info,
        );
      }
    }
  }

  // Coin Girl
  CoinGirl::spawn(&mut commands, Player::P1, &render_info);
  if game_state.players == 2 {
    CoinGirl::spawn(&mut commands, Player::P2, &render_info);
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
