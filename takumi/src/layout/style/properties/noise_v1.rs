use cssparser::{Parser, Token, match_ignore_ascii_case};
use fastnoise_lite::{FastNoiseLite, FractalType};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
  layout::style::{Color, FromCss, Gradient, ParseResult},
  rendering::RenderContext,
};

/// Procedural noise gradient that generates organic, natural-looking patterns using fractal Brownian motion.
/// This creates dynamic textures that can be used as backgrounds or overlays with customizable parameters
/// for controlling the noise characteristics and visual appearance.
#[derive(Debug, Clone, PartialEq, TS, Deserialize, Serialize, Default)]
#[ts(optional_fields)]
pub struct NoiseV1 {
  /// Controls the scale of the noise pattern. Higher values create finer, more detailed patterns
  pub frequency: Option<f32>,
  /// Random seed value that determines the unique noise pattern generated
  pub seed: Option<i32>,
  /// Number of noise layers combined to create complex patterns. More octaves add detail
  pub octaves: Option<i32>,
  /// Controls how much each octave contributes to the final pattern. Lower values create smoother patterns
  pub persistence: Option<f32>,
  /// Controls the frequency multiplier between octaves. Higher values create more varied patterns
  pub lacunarity: Option<f32>,
  /// Controls the opacity of the noise pattern. 0.0 is fully transparent, 1.0 is fully opaque
  pub opacity: Option<f32>,
}

impl Gradient for NoiseV1 {
  type DrawContext = (FastNoiseLite, f32);

  fn at(&self, x: u32, y: u32, (fnl, opacity): &Self::DrawContext) -> Color {
    // let (x, y) = fnl.domain_warp_2d(x as f32, y as f32);
    // range [-1.0, 1.0]
    let noise = fnl.get_noise_2d(x as f32, y as f32);

    let color = ((noise + 1.0) * 128.0).clamp(0.0, 255.0) as u8;
    let alpha = (color as f32 * opacity).clamp(0.0, 255.0) as u8;

    Color([color, color, color, alpha])
  }

  fn to_draw_context(
    &self,
    _width: f32,
    _height: f32,
    _context: &RenderContext,
  ) -> Self::DrawContext {
    let mut fnl = FastNoiseLite::with_seed(self.seed.unwrap_or(0));
    fnl.fractal_type = FractalType::FBm;
    fnl.set_frequency(self.frequency);
    fnl.set_fractal_gain(self.persistence);

    if let Some(octaves) = self.octaves {
      fnl.octaves = octaves;
    }

    if let Some(lacunarity) = self.lacunarity {
      fnl.lacunarity = lacunarity;
    }

    (fnl, self.opacity.unwrap_or(1.0).clamp(0.0, 1.0))
  }
}

impl<'i> FromCss<'i> for NoiseV1 {
  /// Example: noise-v1(frequency(0.01) octaves(4) persistence(0.7) lacunarity(2.0) seed(42) opacity(0.5))
  /// Syntax: noise-v1([<frequency>] | [<octaves>] | [<persistence>] | [<lacunarity>] | [<seed>] | [<opacity>])
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, NoiseV1> {
    input.expect_function_matching("noise-v1")?;

    input.parse_nested_block(|input| {
      let mut instance = NoiseV1::default();

      while !input.is_exhausted() {
        let location = input.current_source_location();
        let token = input.next()?;

        let Token::Function(key) = token else {
          return Err(
            location
              .new_basic_unexpected_token_error(token.clone())
              .into(),
          );
        };

        match_ignore_ascii_case! {key,
          "frequency" => instance.frequency = Some(input.parse_nested_block(|input| Ok(input.expect_number()?))?),
          "octaves" => instance.octaves = Some(input.parse_nested_block(|input| Ok(input.expect_integer()?))?),
          "persistence" => instance.persistence = Some(input.parse_nested_block(|input| Ok(input.expect_number()?))?),
          "lacunarity" => instance.lacunarity = Some(input.parse_nested_block(|input| Ok(input.expect_number()?))?),
          "seed" => instance.seed = Some(input.parse_nested_block(|input| Ok(input.expect_integer()?))?),
          "opacity" => instance.opacity = Some(input.parse_nested_block(|input| Ok(input.expect_number()?))?),
          _ => return Err(location.new_basic_unexpected_token_error(token.clone()).into()),
        }
      }

      Ok(instance)
    })
  }
}
