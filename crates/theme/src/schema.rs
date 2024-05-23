#![allow(missing_docs)]

use gpui::Hsla;
use palette::FromColor;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The appearance of a theme in serialized content.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AppearanceContent {
    Light,
    Dark,
}

/* TODO: where did theme_colors_refinement go?
        editor_jump_marker: this
            .editor_jump_marker
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_jump_marker_entered: this
            .editor_jump_marker_entered
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_jump_marker_first: this
            .editor_jump_marker_first
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
*/

/// Parses a color string into an [`Hsla`] value.
pub fn try_parse_color(color: &str) -> anyhow::Result<Hsla> {
    let rgba = gpui::Rgba::try_from(color)?;
    let rgba = palette::rgb::Srgba::from_components((rgba.r, rgba.g, rgba.b, rgba.a));
    let hsla = palette::Hsla::from_color(rgba);

    let hsla = gpui::hsla(
        hsla.hue.into_positive_degrees() / 360.,
        hsla.saturation,
        hsla.lightness,
        hsla.alpha,
    );

    Ok(hsla)
}
