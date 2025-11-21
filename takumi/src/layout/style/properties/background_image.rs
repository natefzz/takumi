use std::sync::Arc;

use cssparser::{Parser, match_ignore_ascii_case};
use smallvec::SmallVec;

use crate::layout::style::{
  FromCss, LinearGradient, NoiseV1, ParseResult, RadialGradient, tw::TailwindPropertyParser,
};

/// Background image variants supported by Takumi.
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundImage {
  /// CSS linear-gradient(...)
  Linear(LinearGradient),
  /// CSS radial-gradient(...)
  Radial(RadialGradient),
  /// Custom noise-v1(...)
  Noise(NoiseV1),
  /// Load external image resource.
  Url(Arc<str>),
}

impl TailwindPropertyParser for BackgroundImage {
  fn parse_tw(_token: &str) -> Option<Self> {
    // TODO: Implement
    None
  }
}

impl<'i> FromCss<'i> for BackgroundImage {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, BackgroundImage> {
    if let Ok(url) = input.try_parse(Parser::expect_url) {
      return Ok(BackgroundImage::Url((&*url).into()));
    }

    let start = input.state();
    let function = input.expect_function()?.to_owned();

    input.reset(&start);

    match_ignore_ascii_case! {&function,
      "linear-gradient" => Ok(BackgroundImage::Linear(LinearGradient::from_css(input)?)),
      "radial-gradient" => Ok(BackgroundImage::Radial(RadialGradient::from_css(input)?)),
      "noise-v1" => Ok(BackgroundImage::Noise(NoiseV1::from_css(input)?)),
      _ => Err(input.new_error_for_next_token()),
    }
  }
}

/// A collection of background images.
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundImages(pub SmallVec<[BackgroundImage; 4]>);

impl<'i> FromCss<'i> for BackgroundImages {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut images = SmallVec::new();
    images.push(BackgroundImage::from_css(input)?);

    while input.expect_comma().is_ok() {
      images.push(BackgroundImage::from_css(input)?);
    }

    Ok(Self(images))
  }
}
