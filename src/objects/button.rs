//! button.rs
//!
//! https://bevy.org/examples/ui-user-interface/button/

use bevy::{input_focus::InputFocus, prelude::*};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.55, 0.35);

#[derive(Component)]
pub enum BtnState {
    Hovered,
    Pressed,
    None,
}

#[derive(Component)]
pub struct GameOverBtn;

pub fn update_btn(
    entity: Entity,
    input_focus: &mut ResMut<InputFocus>,
    background_color: &mut BackgroundColor,
    state: BtnState,
) {
    match state {
        BtnState::None => {
            input_focus.clear();
            *background_color = NORMAL_BUTTON.into();
        }
        BtnState::Hovered => {
            input_focus.set(entity);
            *background_color = HOVERED_BUTTON.into();
        }
        BtnState::Pressed => {
            input_focus.set(entity);
            *background_color = PRESSED_BUTTON.into();
        }
    };
}

/// https://bevy.org/examples/ui-user-interface/button/
pub fn gen_button(text: &str, width: u32, height: u32, state: Visibility) -> impl Bundle {
    (
        Button,
        state,
        Node {
            width: px(width),
            height: px(height),
            border: UiRect::all(px(5)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BorderRadius::MAX,
        BackgroundColor(Color::BLACK),
        children![(
            Text::new(text),
            TextFont { ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}
