use cssparser::{Parser, ParserInput, Token, match_ignore_ascii_case};
use image::{
  Pixel, RgbaImage,
  imageops::colorops::{contrast_in_place, huerotate_in_place},
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use ts_rs::TS;

use crate::layout::style::{Angle, FromCss, ParseResult, parse_length_percentage};

/// Represents a single CSS filter operation
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Filter {
  /// Brightness multiplier (1 = unchanged). Accepts number or percentage
  Brightness(f32),
  /// Contrast multiplier (1 = unchanged). Accepts number or percentage
  Contrast(f32),
  /// Grayscale amount (0..1). Accepts number or percentage
  Grayscale(f32),
  /// Saturate multiplier (1 = unchanged). Accepts number or percentage
  Saturate(f32),
  /// Hue rotation in degrees
  HueRotate(Angle),
  /// Invert amount (0..1). Accepts number or percentage
  Invert(f32),
  /// Opacity amount (0..1). Accepts number or percentage
  Opacity(f32),
}

/// A list of filters
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TS)]
#[serde(untagged)]
pub(crate) enum FiltersValue {
  /// Structured set of filters
  #[ts(as = "Vec<Filter>")]
  Structured(SmallVec<[Filter; 4]>),
  /// Raw CSS string to be parsed
  Css(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TS, Default)]
#[ts(as = "FiltersValue")]
#[serde(try_from = "FiltersValue")]
/// A list of filter operations
pub struct Filters(pub SmallVec<[Filter; 4]>);

impl Filters {
  pub(crate) fn apply_to(&self, image: &mut RgbaImage) {
    for filter in self.0.iter() {
      match *filter {
        Filter::Brightness(value) => {
          for pixel in image.pixels_mut() {
            for channel in pixel.0.iter_mut().take(3) {
              *channel = ((*channel) as f32 * value).clamp(0.0, 255.0) as u8;
            }
          }
        }
        Filter::Contrast(value) => {
          let amount = value * 100.0 - 100.0;
          contrast_in_place(image, amount);
        }
        Filter::Grayscale(amount) => {
          for pixel in image.pixels_mut() {
            let lum = pixel.to_luma().0[0] as f32;

            for channel in pixel.0.iter_mut().take(3) {
              *channel =
                ((*channel as f32 * (1.0 - amount)) + (lum * amount)).clamp(0.0, 255.0) as u8;
            }
          }
        }
        Filter::HueRotate(angle) => {
          huerotate_in_place(image, *angle as i32);
        }
        Filter::Saturate(value) => {
          for pixel in image.pixels_mut() {
            let lum = pixel.to_luma().0[0] as f32;

            for channel in pixel.0.iter_mut().take(3) {
              *channel = (lum * (1.0 - value) + *channel as f32 * value).clamp(0.0, 255.0) as u8;
            }
          }
        }
        Filter::Invert(amount) => {
          for pixel in image.pixels_mut() {
            for channel in pixel.0.iter_mut().take(3) {
              let inverted = u8::MAX.saturating_sub(*channel);
              *channel = ((*channel as f32 * (1.0 - amount)) + (inverted as f32 * amount))
                .clamp(0.0, 255.0) as u8;
            }
          }
        }
        Filter::Opacity(value) => {
          for alpha in image.as_mut().iter_mut().skip(3).step_by(4) {
            *alpha = ((*alpha) as f32 * value).clamp(0.0, 255.0) as u8;
          }
        }
      }
    }
  }
}

impl TryFrom<FiltersValue> for Filters {
  type Error = String;

  fn try_from(value: FiltersValue) -> Result<Self, Self::Error> {
    match value {
      FiltersValue::Structured(filters) => Ok(Filters(filters)),
      FiltersValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        let mut filters = SmallVec::new();

        while !parser.is_exhausted() {
          let filter = Filter::from_css(&mut parser).map_err(|e| e.to_string())?;
          filters.push(filter);
        }

        Ok(Filters(filters))
      }
    }
  }
}

impl<'i> FromCss<'i> for Filter {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = parser.current_source_location();
    let token = parser.next()?;

    let Token::Function(function) = token else {
      return Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      );
    };

    match_ignore_ascii_case! {function,
      "brightness" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Brightness(value))
      }),
      "opacity" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Opacity(value))
      }),
      "contrast" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Contrast(value))
      }),
      "grayscale" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Grayscale(value))
      }),
      "hue-rotate" => parser.parse_nested_block(|input| {
        Ok(Filter::HueRotate(Angle::from_css(input)?))
      }),
      "invert" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Invert(value))
      }),
      "saturate" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Saturate(value))
      }),
      _ => Err(location.new_basic_unexpected_token_error(Token::Function(function.clone())).into()),
    }
  }
}
