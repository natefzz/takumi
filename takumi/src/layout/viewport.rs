use taffy::Size;

/// The default font size in pixels.
pub const DEFAULT_FONT_SIZE: f32 = 16.0;

/// The default line height multiplier.
pub const DEFAULT_LINE_HEIGHT_SCALER: f32 = 1.2;

/// The viewport for the image renderer.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
  /// The width of the viewport in pixels.
  pub width: u32,
  /// The height of the viewport in pixels.
  pub height: u32,
  /// The font size in pixels, used for em and rem units.
  pub font_size: f32,
}

impl From<Viewport> for Size<u32> {
  fn from(value: Viewport) -> Self {
    Self {
      width: value.width,
      height: value.height,
    }
  }
}

impl Viewport {
  /// Creates a new viewport with the default font size.
  #[must_use]
  pub fn new(width: u32, height: u32) -> Self {
    Self::new_with_font_size(width, height, DEFAULT_FONT_SIZE)
  }

  /// Creates a new viewport with the specified font size.
  #[must_use]
  pub fn new_with_font_size(width: u32, height: u32, font_size: f32) -> Self {
    Self {
      width,
      height,
      font_size,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_viewport_new_defaults() {
    let v = Viewport::new(800, 600);
    assert_eq!(v.width, 800);
    assert_eq!(v.height, 600);
    assert_eq!(v.font_size, DEFAULT_FONT_SIZE);
  }

  #[test]
  fn test_viewport_new_with_font_size() {
    let v = Viewport::new_with_font_size(1024, 768, 14.0);
    assert_eq!(v.width, 1024);
    assert_eq!(v.height, 768);
    assert_eq!(v.font_size, 14.0);
  }
}
