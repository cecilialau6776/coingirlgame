use bevy::prelude::*;

use crate::consts::AppState;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems((
      setup_menu.in_schedule(OnEnter(AppState::Menu)),
      menu.in_set(OnUpdate(AppState::Menu)),
      cleanup_menu.in_schedule(OnExit(AppState::Menu)),
    ));
  }
}

#[derive(Resource)]
struct MenuData {
  singleplayer_button_entity: Entity,
}

fn menu(
  mut next_state: ResMut<NextState<AppState>>,
  mut interaction_query: Query<
    (&Interaction, &mut BackgroundColor),
    (Changed<Interaction>, With<bevy::ui::widget::Button>),
  >,
) {
  for (interaction, mut color) in &mut interaction_query {
    match *interaction {
      Interaction::Clicked => {
        *color = Color::rgb(0.35, 0.35, 0.35).into();
        next_state.set(AppState::Game);
      }
      Interaction::Hovered => {
        *color = Color::rgb(0.25, 0.25, 0.25).into();
      }
      Interaction::None => {
        *color = Color::rgb(0.15, 0.15, 0.15).into();
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
  let singleplayer_button_entity = commands
    .spawn(NodeBundle {
      style: Style {
        position: UiRect {
          left: Val::Percent(-20.0),
          right: Val::Percent(30.0),
          ..default() // top: Val::Percent(30.0),
                      // bottom: Val::Percent(40.0),
        },
        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
      },
      background_color: Color::rgb(1.0, 1.0, 0.0).into(),
      ..default()
    })
    .with_children(|parent| {
      parent
        .spawn(ButtonBundle {
          style: Style {
            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
          },
          background_color: Color::rgb(0.15, 0.15, 0.15).into(),
          ..default()
        })
        .with_children(|parent| {
          parent.spawn(TextBundle::from_section(
            "Play",
            TextStyle {
              // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
              font_size: 40.0,
              color: Color::rgb(0.9, 0.9, 0.9),
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
