use bevy::prelude::*;

pub const SLIDERWIDTH: f32 = 100.0; //pixels
pub const POSSTDEVMIN: f32 = 0.1; //position stdev min: tightest grouping allowed
pub const VELSTDEVMIN: f32 = 0.01; //velocity stdev min

#[derive(Component)]
pub struct Range<T> {
    pub lower: T,
    pub upper: T,
}

pub const BLACKHOLE_COUNT_RNG: Range<u32> = Range {
    lower: 2,
    upper: 100,
};

pub const BLACKHOLE_MASS_RNG: Range<f32> = Range {
    lower: 2.0,
    upper: 20.0,
};

pub const BLACKHOLE_VEL_RNG: Range<f32> = Range {
    lower: -1_000.0,
    upper: 1_000.0,
};

#[derive(Component, Debug)]
pub struct SliderValue {
    pub value: f32,
    pub prev_value: f32,
}
impl Default for SliderValue {
    fn default() -> Self {
        SliderValue {
            value: 0.5,
            prev_value: 0.5,
        }
    }
}

#[derive(Component)]
pub enum SliderType {
    Count,
    Mass,
    Density,
    Velocity,
}

#[derive(Component)]
pub struct SliderBkg;

#[derive(Bundle)]
pub struct SliderBase {
    node: Node,
    bordercolor: BorderColor,
    outline: Outline,
    pub bevy_identifier: SliderType,
}

#[derive(Bundle)]
pub struct SliderText {
    text: Text,
    font: TextFont,
    color: TextColor,
    layout: TextLayout,
}

#[derive(Bundle)]
pub struct SliderBackground {
    node: Node,
    color: BackgroundColor,
}

#[derive(Bundle)]
pub struct SliderGraphic {
    pub base: SliderBase,
    pub text: SliderText,
    pub bkg: SliderBackground,
}

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
