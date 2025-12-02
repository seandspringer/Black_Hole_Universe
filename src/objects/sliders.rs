//! Sliders.rs
//!
//! These functions build the meat of the Slider UIs used in Plugins.rs.  
//! These functions act as helperfunctions to reduce repetitive code and
//! also contain some constants and structs that assign bounds to each of
//! the rendered sliders.

use bevy::prelude::*;

pub const SLIDERWIDTH: f32 = 100.0; //physical width of sliders, in pixels
pub const POSSTDEVMIN: f32 = 0.1; //position stdev min: tightest grouping allowed
pub const VELSTDEVMIN: f32 = 0.01; //velocity stdev min

/// Generic struct type used to represent a lower and upper bound on a range
/// of `Sized` types. It is up to the user to define whether inclusive or not.
///
/// Range<T> is mainly used to set boundaries on the allowed values that the
/// Slider bars can acheive.
#[derive(Component)]
pub struct Range<T>
where
    T: Sized,
{
    pub lower: T,
    pub upper: T,
}

/// Black Hole Count Range:
/// Slider.min = lower, Slider.max = upper
pub const BLACKHOLE_COUNT_RNG: Range<u32> = Range {
    lower: 2,
    upper: 100,
};

/// Black Hole Allowed Mass Range:
/// Slider.min = lower, Slider.max = upper
pub const BLACKHOLE_MASS_RNG: Range<f32> = Range {
    lower: 2.0,
    upper: 20.0,
};

/// Black Hole Allowed Velocity Range:
/// Slider.min = lower, Slider.max = upper
pub const BLACKHOLE_VEL_RNG: Range<f32> = Range {
    lower: -1_000.0,
    upper: 1_000.0,
};

/// SliderValue struct: Component
///
/// Stores the slider's current value and it's previous value.
/// Previous value is used to determine whether the slider bar  
/// has moved during the current frame
#[derive(Component, Debug)]
pub struct SliderValue {
    pub value: f32,
    pub prev_value: f32,
}

/// Implement the Default trait for SliderValue
///
/// Sets the Default slider position to be 0.5 - 1/2 full
impl Default for SliderValue {
    fn default() -> Self {
        SliderValue {
            value: 0.5,
            prev_value: 0.5,
        }
    }
}

/// SliderType Enum: Component
///
/// Tracks the metric which the Slider attached to it controls.
/// Used mostly for identifying the SliderValue within a Bevy Query
#[derive(Component)]
pub enum SliderType {
    Count,
    Mass,
    Density,
    Velocity,
}

/// SliderBkg Struct: Component
///
/// Used to identify and target the background graphic of the SLider bar.
/// The background is used to give the illusion of the slider moving during
/// a drag.
#[derive(Component)]
pub struct SliderBkg;

/// SliderBase Struct: Bundle
///
/// The SliderBase bundle is rendered together.
/// - node = physical size and alignment
/// - bordercolor = color of drawn border around the Slider
/// - outline = width and style of the border line
/// - bevy_identifier = SliderType enum giving it's purpose
#[derive(Bundle)]
pub struct SliderBase {
    node: Node,
    bordercolor: BorderColor,
    outline: Outline,
    pub bevy_identifier: SliderType,
}

/// SliderText Struct: Bundle
///
/// Components tied to the text rendered inside the Slider bar.
/// - text = physical letters to be displayed
/// - font = font style and size
/// - color = font color
/// - layout = how to render the text
#[derive(Bundle)]
pub struct SliderText {
    text: Text,
    font: TextFont,
    color: TextColor,
    layout: TextLayout,
}

/// SliderBackground Struct: Bundle
///
/// The background of the slider which moves with the user drag
/// - node = physical size and alignment
/// - color = background color which display the bar's state to the user
#[derive(Bundle)]
pub struct SliderBackground {
    node: Node,
    color: BackgroundColor,
}

/// SliderGraphic Struct: Bundle
///
/// Bevy bundle which contains the 3 structs above, describing the Slider's UI
/// - base = SliderBase struct
/// - text = SliderText struct
/// - bkg  = SliderBackground struct
#[derive(Bundle)]
pub struct SliderGraphic {
    pub base: SliderBase,
    pub text: SliderText,
    pub bkg: SliderBackground,
}

/// fn generate_slider returns a SliderGraphic Struct Bundle
///
/// Convience function for prepraing the Geometry, layout, text, graphics, and color  
/// for a slider bar. Called during `fn setup_hub` in plugins.rs
pub fn generate_slider(stype: SliderType, text: &str) -> SliderGraphic {
    let base = SliderBase {
        node: Node {
            height: px(50.0),
            width: px(SLIDERWIDTH),
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        bordercolor: BorderColor::all(Color::WHITE),
        outline: Outline::new(px(1), Val::ZERO, Color::WHITE),
        bevy_identifier: stype,
    };

    let text = SliderText {
        text: Text::new(text),
        font: TextFont {
            font_size: 16.0,
            ..default()
        },
        color: TextColor(Color::WHITE),
        layout: TextLayout::new_with_justify(Justify::Center),
    };

    let bkg = SliderBackground {
        node: Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            height: px(50.0),
            width: px(SLIDERWIDTH / 2.0),
            ..default()
        },
        color: BackgroundColor(Color::linear_rgba(0.0, 0.4, 0.0, 1.0)),
    };

    SliderGraphic { base, text, bkg }
}
