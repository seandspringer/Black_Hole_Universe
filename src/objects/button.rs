//! button.rs
//!
//! This module is basically a helper wrapper around creating a pressable-btn.
//! It is used by plugins.rs to generate the Restart button.
//!
//! Please note that the majority of this code was adapted directly from the "UI (User Interface) / Button"
//! Bevy example. See https://bevy.org/examples/ui-user-interface/button/

use bevy::{input_focus::InputFocus, prelude::*};

/// constants that define the color of the button during different events
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.55, 0.35);

/// BtnState enum: Component
///
/// Contains the mouse-over state of the button
#[derive(Component)]
pub enum BtnState {
    Hovered,
    Pressed,
    None,
}

/// GameOverBtn struct: Component
///
/// Used to identify the GameOverBtn from possible future buttons
#[derive(Component)]
pub struct GameOverBtn;

/// fn update_btn
///
/// Given an input BtnState and parameters needed to change the appearance of the button,
/// updates the button's color to indicate actionablity to the user
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

/// fn gen_button
///
/// Wrapper Constructor-like function which returns a Bevy bundle containing the button.
/// Sets the buttons Text to text and size parameters to width and height.
///
/// Adapted directly from the "UI (User Interface) / Button" Bevy example. please see:
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
