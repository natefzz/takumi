use serde::{Deserialize, Serialize};
use taffy::{AvailableSpace, Layout, Size};

#[cfg(feature = "image_data_uri")]
use crate::resources::image::ImageResult;
use crate::{
  GlobalContext,
  layout::{
    inline::InlineContentKind,
    node::Node,
    style::{InheritedStyle, Style},
  },
  rendering::{Canvas, RenderContext, draw_image},
  resources::image::{ImageResourceError, ImageSource, is_svg},
};

/// A node that renders image content.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageNode {
  /// The styling properties for this image node
  pub style: Option<Style>,
  /// The source URL or path to the image
  pub src: String,
  /// The width of the image
  pub width: Option<f32>,
  /// The height of the image
  pub height: Option<f32>,
}

impl<Nodes: Node<Nodes>> Node<Nodes> for ImageNode {
  fn create_inherited_style(&mut self, parent_style: &InheritedStyle) -> InheritedStyle {
    self.style.take().unwrap_or_default().inherit(parent_style)
  }

  fn inline_content(&self, _context: &RenderContext) -> Option<InlineContentKind> {
    Some(InlineContentKind::Box)
  }

  fn measure(
    &self,
    context: &RenderContext,
    _available_space: Size<AvailableSpace>,
    known_dimensions: Size<Option<f32>>,
    style: &taffy::Style,
  ) -> Size<f32> {
    let Ok(image) = resolve_image(&self.src, context.global) else {
      return Size::zero();
    };

    let image_size = match &*image {
      #[cfg(feature = "svg")]
      ImageSource::Svg(svg) => Size {
        width: svg.size().width(),
        height: svg.size().height(),
      },
      ImageSource::Bitmap(bitmap) => Size {
        width: bitmap.width() as f32,
        height: bitmap.height() as f32,
      },
    };

    let overridden_size = Size {
      width: self.width.unwrap_or(image_size.width),
      height: self.height.unwrap_or(image_size.height),
    };

    let aspect_ratio = style
      .aspect_ratio
      .unwrap_or(overridden_size.width / overridden_size.height);

    if let Size {
      width: Some(width),
      height: Some(height),
    } = known_dimensions.maybe_apply_aspect_ratio(Some(aspect_ratio))
    {
      return Size { width, height };
    }

    overridden_size
  }

  fn draw_content(&self, context: &RenderContext, canvas: &Canvas, layout: Layout) {
    let Ok(image) = resolve_image(&self.src, context.global) else {
      return;
    };

    draw_image(&image, context, canvas, layout);
  }
}

const DATA_URI_PREFIX: &str = "data:";

fn is_data_uri(src: &str) -> bool {
  src.starts_with(DATA_URI_PREFIX)
}

#[cfg(feature = "image_data_uri")]
fn parse_data_uri_image(src: &str) -> ImageResult {
  use crate::resources::image::load_image_source_from_bytes;
  use base64::{Engine as _, engine::general_purpose};

  let comma_pos = src
    .find(',')
    .ok_or(ImageResourceError::InvalidDataUriFormat)?;

  let metadata = &src[DATA_URI_PREFIX.len()..comma_pos];
  let data = &src[comma_pos + 1..];

  if !metadata.contains("base64") {
    return Err(ImageResourceError::InvalidDataUriFormat);
  }

  let image_bytes = general_purpose::STANDARD
    .decode(data)
    .map_err(|_| ImageResourceError::MalformedDataUri)?;

  load_image_source_from_bytes(&image_bytes)
}

fn resolve_image(src: &str, context: &GlobalContext) -> ImageResult {
  if is_data_uri(src) {
    #[cfg(feature = "image_data_uri")]
    return parse_data_uri_image(src);
    #[cfg(not(feature = "image_data_uri"))]
    return Err(ImageResourceError::DataUriParseNotSupported);
  }

  if is_svg(src) {
    #[cfg(feature = "svg")]
    return crate::resources::image::parse_svg(src);
    #[cfg(not(feature = "svg"))]
    return Err(ImageResourceError::SvgParseNotSupported);
  }

  if let Some(img) = context.persistent_image_store.get(src) {
    return Ok(img);
  }

  Err(ImageResourceError::Unknown)
}
