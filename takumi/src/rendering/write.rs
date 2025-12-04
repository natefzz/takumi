use std::{borrow::Cow, io::Write};

use image::{ExtendedColorType, ImageEncoder, ImageFormat, RgbaImage, codecs::jpeg::JpegEncoder};
use png::{ColorType, Compression, Filter};
use serde::Deserialize;

use image_webp::WebPEncoder;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::Error::IoError;

/// Output format for rendered images.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageOutputFormat {
  /// WebP image format, provides good compression and supports animation.
  /// It is useful for images in web contents.
  WebP,

  /// PNG image format, lossless and widely supported, and its the fastest format to encode.
  Png,

  /// JPEG image format, lossy and does not support transparency.
  Jpeg,
}

impl ImageOutputFormat {
  /// Returns the MIME type for the image output format.
  pub fn content_type(&self) -> &'static str {
    match self {
      ImageOutputFormat::WebP => "image/webp",
      ImageOutputFormat::Png => "image/png",
      ImageOutputFormat::Jpeg => "image/jpeg",
    }
  }
}

impl From<ImageOutputFormat> for ImageFormat {
  fn from(format: ImageOutputFormat) -> Self {
    match format {
      ImageOutputFormat::WebP => Self::WebP,
      ImageOutputFormat::Png => Self::Png,
      ImageOutputFormat::Jpeg => Self::Jpeg,
    }
  }
}

/// Represents a single frame of an animated image.
#[derive(Debug, Clone)]
pub struct AnimationFrame {
  /// The image data for the frame.
  pub image: RgbaImage,
  /// The duration of the frame in milliseconds.
  /// Maximum value is 0xffffff (24-bit), overflow will be clamped.
  pub duration_ms: u32,
}

impl AnimationFrame {
  /// Creates a new animation frame.
  pub fn new(image: RgbaImage, duration_ms: u32) -> Self {
    Self { image, duration_ms }
  }
}

const U24_MAX: u32 = 0xffffff;

// Strip alpha channel into a tightly packed RGB buffer
fn strip_alpha_channel(image: &RgbaImage) -> Vec<u8> {
  let raw = image.as_raw();

  let pixel_count = raw.len() / 4 * 3;
  let mut rgb = Vec::with_capacity(pixel_count);

  for chunk in raw.chunks_exact(4) {
    rgb.extend_from_slice(&chunk[..3]);
  }

  rgb
}

fn has_any_alpha_pixel(image: &RgbaImage) -> bool {
  let raw = image.as_raw();

  #[cfg(feature = "rayon")]
  {
    raw
      .par_chunks_exact(4)
      .with_min_len(1024)
      .any(|chunk| chunk[3] != u8::MAX)
  }

  #[cfg(not(feature = "rayon"))]
  {
    raw.chunks_exact(4).any(|chunk| chunk[3] != u8::MAX)
  }
}

/// Writes a single rendered image to `destination` using `format`.
pub fn write_image<T: Write>(
  image: &RgbaImage,
  destination: &mut T,
  format: ImageOutputFormat,
  quality: Option<u8>,
) -> Result<(), crate::Error> {
  match format {
    ImageOutputFormat::Jpeg => {
      let rgb = strip_alpha_channel(image);

      let encoder = JpegEncoder::new_with_quality(destination, quality.unwrap_or(75));
      encoder.write_image(&rgb, image.width(), image.height(), ExtendedColorType::Rgb8)?;
    }
    ImageOutputFormat::Png => {
      let has_alpha = has_any_alpha_pixel(image);

      let image_data = if has_alpha {
        Cow::Borrowed(image.as_raw())
      } else {
        Cow::Owned(strip_alpha_channel(image))
      };

      let mut encoder = png::Encoder::new(destination, image.width(), image.height());

      encoder.set_color(if has_alpha {
        ColorType::Rgba
      } else {
        ColorType::Rgb
      });

      encoder.set_compression(Compression::Fast);
      encoder.set_filter(Filter::Sub);

      let mut writer = encoder.write_header()?;

      writer.write_image_data(&image_data)?;

      writer.finish()?;
    }
    ImageOutputFormat::WebP => {
      let encoder = WebPEncoder::new(destination);

      encoder.encode(
        image.as_raw(),
        image.width(),
        image.height(),
        image_webp::ColorType::Rgba8,
      )?;
    }
  }

  Ok(())
}

/// Extracts VP8L/VP8 payload from a RIFF WEBP buffer produced by `WebPEncoder`.
fn extract_vp8_payload(buf: &[u8]) -> Result<&[u8], crate::Error> {
  let mut i = 12usize; // skip RIFF header
  while i + 8 <= buf.len() {
    let len = u32::from_le_bytes(
      buf[i + 4..i + 8]
        .try_into()
        .map_err(|_| IoError(std::io::Error::other("Invalid buffer size")))?,
    ) as usize;

    let start = i + 8;
    let end = start + len;

    if end > buf.len() {
      break;
    }

    if &buf[i..i + 4] == b"VP8L" || &buf[i..i + 4] == b"VP8 " {
      return Ok(&buf[start..end]);
    }

    i = end + (len & 1);
  }

  Err(IoError(std::io::Error::other(
    "failed to extract VP8 payload",
  )))
}

/// Encode a sequence of RGBA frames into an animated WebP and write to `destination`.
pub fn encode_animated_webp<W: Write>(
  frames: &[AnimationFrame],
  destination: &mut W,
  blend: bool,
  dispose: bool,
  loop_count: Option<u16>,
) -> Result<(), crate::Error> {
  assert_ne!(frames.len(), 0);

  // encode frames losslessly and collect VP8L/VP8 payloads
  #[cfg(feature = "rayon")]
  let frames_payloads: Vec<(&AnimationFrame, Vec<u8>)> = frames
    .par_iter()
    .map(|frame| {
      let mut buf = Vec::new();
      WebPEncoder::new(&mut buf).encode(
        &frame.image,
        frame.image.width(),
        frame.image.height(),
        image_webp::ColorType::Rgba8,
      )?;

      Ok((frame, buf))
    })
    .collect::<Result<Vec<(&AnimationFrame, Vec<u8>)>, crate::Error>>()?;

  #[cfg(not(feature = "rayon"))]
  let frames_payloads: Vec<(&AnimationFrame, Vec<u8>)> = frames
    .iter()
    .map(|frame| {
      let mut buf = Vec::new();
      WebPEncoder::new(&mut buf)
        .encode(
          &frame.image,
          frame.image.width(),
          frame.image.height(),
          image_webp::ColorType::Rgba8,
        )
        .map_err(|_| IoError(std::io::Error::other("WebP encode error")))?;

      Ok((frame, buf))
    })
    .collect::<Result<Vec<(&AnimationFrame, Vec<u8>)>, crate::Error>>()?;

  // assemble RIFF WEBP with VP8X + ANIM + ANMF frames
  let mut output: Vec<u8> = Vec::new();

  // VP8X chunk
  let vp8x_flags: u8 = (1 << 1) | (1 << 4); // animation + alpha
  let cw = (frames[0].image.width() - 1).to_le_bytes();
  let ch = (frames[0].image.height() - 1).to_le_bytes();

  output.extend_from_slice(b"VP8X");
  output.extend_from_slice(&10u32.to_le_bytes());
  output.push(vp8x_flags);
  output.extend_from_slice(&[0u8; 3]);
  output.extend_from_slice(&cw[..3]);
  output.extend_from_slice(&ch[..3]);

  // ANIM chunk
  output.extend_from_slice(b"ANIM");
  output.extend_from_slice(&6u32.to_le_bytes());
  output.extend_from_slice(&[0u8; 4]); // bgcolor (4 bytes)
  output.extend_from_slice(&loop_count.unwrap_or(0).to_le_bytes());

  let frame_flags = ((blend as u8) << 1) | (dispose as u8);

  // ANMF frames
  for (frame, vp8_data) in frames_payloads.into_iter() {
    let w_bytes = (frame.image.width() - 1).to_le_bytes();
    let h_bytes = (frame.image.height() - 1).to_le_bytes();
    let vp8_payload = extract_vp8_payload(&vp8_data)?;

    let payload_padded = vp8_payload.len() + (vp8_payload.len() & 1);
    let anmf_size = 16 + 4 + 4 + payload_padded; // x, y, w, h, duration, flags, payload

    output.extend_from_slice(b"ANMF");
    output.extend_from_slice(&(anmf_size as u32).to_le_bytes());

    // frame header (16 bytes)
    output.extend_from_slice(&[0u8; 6]); // x, y (3 bytes each)
    output.extend_from_slice(&w_bytes[..3]); // w (3 bytes)
    output.extend_from_slice(&h_bytes[..3]); // h (3 bytes)
    output.extend_from_slice(&frame.duration_ms.clamp(0, U24_MAX).to_le_bytes()[..3]); // duration (3 bytes)
    output.push(frame_flags); // flags (1 byte)

    // VP8L chunk: VP8L payload
    output.extend_from_slice(b"VP8L");
    output.extend_from_slice(&(vp8_payload.len() as u32).to_le_bytes());
    output.extend_from_slice(vp8_payload);

    if vp8_payload.len() & 1 == 1 {
      output.push(0);
    }
  }

  // write RIFF header
  destination.write_all(b"RIFF")?;
  let total_size = (4 + output.len()) as u32; // 'WEBP' + chunks

  destination.write_all(&total_size.to_le_bytes())?;
  destination.write_all(b"WEBP")?;
  destination.write_all(&output)?;

  Ok(())
}

/// Encode a sequence of RGBA frames into an animated PNG and write to `destination`.
pub fn encode_animated_png<W: Write>(
  frames: &[AnimationFrame],
  destination: &mut W,
  loop_count: Option<u16>,
) -> Result<(), crate::Error> {
  assert_ne!(frames.len(), 0);

  let mut encoder = png::Encoder::new(
    destination,
    frames[0].image.width(),
    frames[0].image.height(),
  );

  encoder.set_color(ColorType::Rgba);
  encoder.set_compression(png::Compression::Fastest);
  encoder.set_animated(frames.len() as u32, loop_count.unwrap_or(0) as u32)?;

  // Since APNG doesn't support variable frame duration, we use the minimum duration of all frames.
  let min_duration_ms = frames
    .iter()
    .map(|frame| frame.duration_ms)
    .min()
    .unwrap_or(0);

  encoder.set_frame_delay(min_duration_ms.clamp(0, u16::MAX as u32) as u16, 1000)?;

  let mut writer = encoder.write_header()?;

  for frame in frames {
    writer.write_image_data(frame.image.as_raw())?;
  }

  writer.finish()?;

  Ok(())
}
