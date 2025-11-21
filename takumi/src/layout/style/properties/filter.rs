use std::ops::Deref;

use cssparser::{Parser, Token, match_ignore_ascii_case};
use image::{
  Pixel, RgbaImage,
  imageops::colorops::{contrast_in_place, huerotate_in_place},
};
use smallvec::SmallVec;

use crate::layout::style::{Angle, FromCss, ParseResult, PercentageNumber};

/// Represents a single CSS filter operation
#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Default)]
/// A list of filter operations
pub struct Filters(SmallVec<[Filter; 4]>);

impl Deref for Filters {
  type Target = SmallVec<[Filter; 4]>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

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

impl<'i> FromCss<'i> for Filters {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut filters = SmallVec::new();

    while !input.is_exhausted() {
      let filter = Filter::from_css(input)?;
      filters.push(filter);
    }

    Ok(Filters(filters))
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
        let PercentageNumber(value) = PercentageNumber::from_css(input)?;
        Ok(Filter::Brightness(value))
      }),
      "opacity" => parser.parse_nested_block(|input| {
        let PercentageNumber(value) = PercentageNumber::from_css(input)?;
        Ok(Filter::Opacity(value))
      }),
      "contrast" => parser.parse_nested_block(|input| {
        let PercentageNumber(value) = PercentageNumber::from_css(input)?;
        Ok(Filter::Contrast(value))
      }),
      "grayscale" => parser.parse_nested_block(|input| {
        let PercentageNumber(value) = PercentageNumber::from_css(input)?;
        Ok(Filter::Grayscale(value))
      }),
      "hue-rotate" => parser.parse_nested_block(|input| {
        Ok(Filter::HueRotate(Angle::from_css(input)?))
      }),
      "invert" => parser.parse_nested_block(|input| {
        let PercentageNumber(value) = PercentageNumber::from_css(input)?;
        Ok(Filter::Invert(value))
      }),
      "saturate" => parser.parse_nested_block(|input| {
        let PercentageNumber(value) = PercentageNumber::from_css(input)?;
        Ok(Filter::Saturate(value))
      }),
      _ => Err(location.new_basic_unexpected_token_error(Token::Function(function.clone())).into()),
    }
  }
}
