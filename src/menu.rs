use bevy::{app::AppExit, prelude::*};
use devcaders::DevcadeControls;

use crate::{consts::AppState, GameInfo};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
  fn build(&self, app: &mut App) {
    app.add_event::<SelectEvent>().add_systems((
      setup_menu.in_schedule(OnEnter(AppState::Menu)),
      menu.in_set(OnUpdate(AppState::Menu)),
      menu_input.in_set(OnUpdate(AppState::Menu)),
      cleanup_menu.in_schedule(OnExit(AppState::Menu)),
      select_handler
        .after(menu_input)
        .in_set(OnUpdate(AppState::Menu)),
    ));
  }
}

#[derive(Resource)]
struct MenuData {
  singleplayer_button_entity: Entity,
}

fn menu(
  mut query: Query<
    (&Selected, &mut BackgroundColor),
    (
      Changed<Selected>,
      With<bevy::ui::widget::Button>,
      With<MenuButton>,
    ),
  >,
) {
  for (selected, mut color) in &mut query {
    if selected.0 {
      *color = Color::rgb(0.75, 0.75, 0.75).into();
    } else {
      *color = Color::rgb(0.15, 0.15, 0.15).into();
    }
  }
}

#[derive(Debug, Component, PartialEq, Eq)]
enum MenuButton {
  OnePlayer,
  TwoPlayer,
  Quit,
}

#[derive(Component)]
struct Selected(bool);

struct SelectEvent(MenuButton);

fn select_handler(
  mut events: EventReader<SelectEvent>,
  mut query: Query<
    (&mut Selected, &MenuButton),
    (With<bevy::ui::widget::Button>, With<MenuButton>),
  >,
) {
  for ev in events.iter() {
    println!("{:?}", ev.0);
    for (mut selected, button) in &mut query {
      println!("{:?}", button);
      if *button != ev.0 {
        selected.0 = false;
      } else {
        selected.0 = true;
      }
    }
  }
}

fn menu_input(
  input: DevcadeControls,
  mut game_info: ResMut<GameInfo>,
  mut next_state: ResMut<NextState<AppState>>,
  mut select_writer: EventWriter<SelectEvent>,
  mut exit: EventWriter<AppExit>,
  selected_query: Query<(&Selected, &MenuButton), With<bevy::ui::widget::Button>>,
) {
  for (selected, menu_button) in &selected_query {
    if !selected.0 {
      continue;
    }
    if input.just_released(devcaders::Player::P1, devcaders::Button::StickUp) {
      match menu_button {
        MenuButton::OnePlayer => (),
        MenuButton::TwoPlayer => select_writer.send(SelectEvent(MenuButton::OnePlayer)),
        MenuButton::Quit => select_writer.send(SelectEvent(MenuButton::TwoPlayer)),
      }
    } else if input.just_released(devcaders::Player::P1, devcaders::Button::StickDown) {
      match menu_button {
        MenuButton::OnePlayer => select_writer.send(SelectEvent(MenuButton::TwoPlayer)),
        MenuButton::TwoPlayer => select_writer.send(SelectEvent(MenuButton::Quit)),
        MenuButton::Quit => (),
      }
    } else if input.just_released(devcaders::Player::P1, devcaders::Button::A1) {
      match menu_button {
        MenuButton::OnePlayer => {
          game_info.players = 1;
          next_state.set(AppState::Game);
        }
        MenuButton::TwoPlayer => {
          game_info.players = 2;
          next_state.set(AppState::Game);
        }
        MenuButton::Quit => exit.send(AppExit),
      }
    }
  }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
  commands
    .entity(menu_data.singleplayer_button_entity)
    .despawn_recursive();
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
  let font = asset_server.load("Evogria.otf");
  let singleplayer_button_entity = commands
    .spawn(NodeBundle {
      style: Style {
        position: UiRect {
          left: Val::Percent(5.0),
          right: Val::Percent(5.0),
          top: Val::Percent(5.0),
          bottom: Val::Percent(5.0),
        },
        display: Display::Flex,
        size: Size::new(Val::Percent(90.0), Val::Percent(90.0)),
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        align_content: AlignContent::Center,
        // margin: UiRect::top(Val::Percent(5.0)),
        ..default()
      },
      background_color: Color::rgb(1.0, 1.0, 0.0).into(),
      ..default()
    })
    .with_children(|parent| {
      parent.spawn(
        TextBundle::from_section(
          "Coin Girl Game",
          TextStyle {
            font: font.clone(),
            font_size: 80.0,
            color: Color::BLACK,
            ..default()
          },
        )
        .with_style(Style {
          flex_grow: 0.2,
          ..default()
        }),
      );
      parent
        .spawn((
          ButtonBundle {
            style: Style {
              size: Size::new(Val::Px(150.0), Val::Px(65.0)),
              justify_content: JustifyContent::Center,
              align_items: AlignItems::Center,
              // margin: UiRect::all(Val::Percent(5.0)),
              ..default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..default()
          },
          MenuButton::OnePlayer,
          Selected(true),
        ))
        .with_children(|parent| {
          parent.spawn(TextBundle::from_section(
            "1 Player",
            TextStyle {
              font: font.clone(),
              font_size: 40.0,
              color: Color::WHITE,
              ..default()
            },
          ));
        });
      parent
        .spawn((
          ButtonBundle {
            style: Style {
              size: Size::new(Val::Px(150.0), Val::Px(65.0)),
              justify_content: JustifyContent::Center,
              align_items: AlignItems::Center,
              // margin: UiRect::all(Val::Percent(5.0)),
              ..default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..default()
          },
          MenuButton::TwoPlayer,
          Selected(false),
        ))
        .with_children(|parent| {
          parent.spawn(TextBundle::from_section(
            "2 Player",
            TextStyle {
              font: asset_server.load("Evogria.otf"),
              font_size: 40.0,
              color: Color::WHITE,
              ..default()
            },
          ));
        });
      parent
        .spawn((
          ButtonBundle {
            style: Style {
              size: Size::new(Val::Px(150.0), Val::Px(65.0)),
              justify_content: JustifyContent::Center,
              align_items: AlignItems::Center,
              // margin: UiRect::all(Val::Percent(5.0)),
              ..default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..default()
          },
          MenuButton::Quit,
          Selected(false),
        ))
        .with_children(|parent| {
          parent.spawn(TextBundle::from_section(
            "Exit",
            TextStyle {
              font: asset_server.load("Evogria.otf"),
              font_size: 40.0,
              color: Color::WHITE,
              ..default()
            },
          ));
        });
    })
    .id();
  commands.insert_resource(MenuData {
    singleplayer_button_entity,
  });
}
