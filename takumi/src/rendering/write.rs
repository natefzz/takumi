use std::io::Write;

use image::{
  ExtendedColorType, ImageEncoder, ImageFormat, RgbaImage,
  codecs::{
    jpeg::JpegEncoder,
    png::{CompressionType, FilterType, PngEncoder},
  },
};
use serde::{Deserialize, Serialize};

use image_webp::{ColorType, WebPEncoder};

#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::Error::IoError;

/// Output format for rendered images.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageOutputFormat {
  /// WebP image format, provides good compression and supports animation.
  /// It is useful for images in web contents.
  WebP,

  /// AVIF typically offers better compression than WebP/PNG but requires significant more CPU time.
  #[cfg(feature = "avif")]
  Avif,

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
      #[cfg(feature = "avif")]
      ImageOutputFormat::Avif => "image/avif",
      ImageOutputFormat::Png => "image/png",
      ImageOutputFormat::Jpeg => "image/jpeg",
    }
  }
}

impl From<ImageOutputFormat> for ImageFormat {
  fn from(format: ImageOutputFormat) -> Self {
    match format {
      ImageOutputFormat::WebP => Self::WebP,
      #[cfg(feature = "avif")]
      ImageOutputFormat::Avif => Self::Avif,
      ImageOutputFormat::Png => Self::Png,
      ImageOutputFormat::Jpeg => Self::Jpeg,
    }
  }
}

/// Writes a single rendered image to `destination` using `format`.
pub fn write_image<T: Write + std::io::Seek>(
  image: &RgbaImage,
  destination: &mut T,
  format: ImageOutputFormat,
  quality: Option<u8>,
) -> Result<(), image::ImageError> {
  match format {
    ImageOutputFormat::Jpeg => {
      // Strip alpha channel into a tightly packed RGB buffer
      let raw = image.as_raw();
      let mut rgb = Vec::with_capacity(raw.len() / 4 * 3);
      for px in raw.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
      }

      let encoder = JpegEncoder::new_with_quality(destination, quality.unwrap_or(75));
      encoder.write_image(&rgb, image.width(), image.height(), ExtendedColorType::Rgb8)?;
    }
    ImageOutputFormat::Avif => {
      let encoder = image::codecs::avif::AvifEncoder::new_with_speed_quality(
        destination,
        10,
        quality.unwrap_or(75),
      );

      encoder.write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        ExtendedColorType::Rgba8,
      )?;
    }
    ImageOutputFormat::Png => {
      let encoder =
        PngEncoder::new_with_quality(destination, CompressionType::Fast, FilterType::Sub);

      encoder.write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        ExtendedColorType::Rgba8,
      )?;
    }
    _ => {
      image.write_to(destination, format.into())?;
    }
  }

  Ok(())
}

/// Extracts VP8L/VP8 payload from a RIFF WEBP buffer produced by `WebPEncoder`.
fn extract_vp8_payload(buf: &[u8]) -> Result<Vec<u8>, crate::Error> {
  let mut i = 12usize; // skip RIFF header
  while i + 8 <= buf.len() {
    let name = &buf[i..i + 4];
    let len = u32::from_le_bytes(buf[i + 4..i + 8].try_into().unwrap()) as usize;
    let start = i + 8;
    let end = start + len;
    if end > buf.len() {
      break;
    }
    if name == b"VP8L" || name == b"VP8 " {
      return Ok(buf[start..end].to_vec());
    }
    i = end + (len % 2);
  }

  Err(IoError(std::io::Error::other(
    "failed to extract VP8 payload",
  )))
}

/// Encode a sequence of RGBA frames into an animated WebP and write to `destination`.
pub fn encode_animated_webp<W: Write>(
  frames: &[RgbaImage],
  duration_ms: u16,
  destination: &mut W,
  blend: bool,
  dispose: bool,
  loop_count: Option<u16>,
) -> Result<(), crate::Error> {
  assert_ne!(frames.len(), 0);

  // encode frames losslessly and collect VP8L/VP8 payloads
  #[cfg(feature = "rayon")]
  let frames_payloads: Vec<Vec<u8>> = frames
    .par_iter()
    .map(|frame| {
      let mut buf = Vec::new();
      WebPEncoder::new(&mut buf)
        .encode(frame, frame.width(), frame.height(), ColorType::Rgba8)
        .map_err(|_| IoError(std::io::Error::other("WebP encode error")))?;

      extract_vp8_payload(&buf)
    })
    .collect::<Result<_, _>>()?;

  #[cfg(not(feature = "rayon"))]
  let frames_payloads: Vec<Vec<u8>> = frames
    .iter()
    .map(|frame| {
      let mut buf = Vec::new();
      WebPEncoder::new(&mut buf)
        .encode(frame, frame.width(), frame.height(), ColorType::Rgba8)
        .map_err(|_| IoError(std::io::Error::other("WebP encode error")))?;

      extract_vp8_payload(&buf)
    })
    .collect::<Result<_, _>>()?;

  // assemble RIFF WEBP with VP8X + ANIM + ANMF frames
  let mut chunks: Vec<u8> = Vec::new();

  // VP8X: set animation bit and alpha bit
  let mut vp8x = Vec::new();
  let mut flags: u8 = 0;
  flags |= 1 << 1; // animation
  flags |= 1 << 4; // alpha assumed
  vp8x.push(flags);
  vp8x.extend_from_slice(&[0u8; 3]);
  // canvas width/height (24-bit little-endian, stored as width-1 / height-1)
  let cw = frames[0].width() - 1;
  let ch = frames[0].height() - 1;
  let cw = cw.to_le_bytes();
  let ch = ch.to_le_bytes();
  vp8x.extend_from_slice(&cw[..3]);
  vp8x.extend_from_slice(&ch[..3]);
  // write VP8X chunk
  chunks.extend_from_slice(b"VP8X");
  chunks.extend_from_slice(&(vp8x.len() as u32).to_le_bytes());
  chunks.extend_from_slice(&vp8x);
  if vp8x.len() % 2 == 1 {
    chunks.push(0);
  }

  // ANIM chunk: background color (24-bit), loop count (u16)
  let mut anim = Vec::new();
  anim.extend_from_slice(&[0u8, 0u8, 0u8]); // bgcolor
  let loop_value: u16 = loop_count.unwrap_or(0);
  anim.extend_from_slice(&loop_value.to_le_bytes());
  chunks.extend_from_slice(b"ANIM");
  chunks.extend_from_slice(&(anim.len() as u32).to_le_bytes());
  chunks.extend_from_slice(&anim);
  if anim.len() % 2 == 1 {
    chunks.push(0);
  }

  // ANMF frames
  let frames_len = frames_payloads.len() as u32;
  for frame_payload in frames_payloads.into_iter() {
    let mut anmf = Vec::new();
    // frame rect: x(24) y(24) w(24) h(24) - x/y = 0
    anmf.extend_from_slice(&[0u8, 0u8, 0u8]); // x
    anmf.extend_from_slice(&[0u8, 0u8, 0u8]); // y
    let w_bytes = (frames[0].width() - 1).to_le_bytes();
    let h_bytes = (frames[0].height() - 1).to_le_bytes();
    anmf.extend_from_slice(&w_bytes[..3]);
    anmf.extend_from_slice(&h_bytes[..3]);
    // frame duration as 24-bit little-endian (milliseconds per spec)
    let per_frame_ms = (duration_ms as u32) / frames_len;
    let delay_bytes = per_frame_ms.to_le_bytes();
    anmf.extend_from_slice(&delay_bytes[..3]);
    // flags: 1 bit for dispose, 1 bit for blend in LSBs of a single byte
    let mut f: u8 = 0;
    if !blend {
      f |= 1 << 1;
    }
    if dispose {
      f |= 1 << 0;
    }
    anmf.push(f);

    // append frame payload as nested chunk (VP8L or VP8)
    anmf.extend_from_slice(b"VP8L");
    anmf.extend_from_slice(&(frame_payload.len() as u32).to_le_bytes());
    anmf.extend_from_slice(&frame_payload);
    if frame_payload.len() % 2 == 1 {
      anmf.push(0);
    }

    chunks.extend_from_slice(b"ANMF");
    chunks.extend_from_slice(&(anmf.len() as u32).to_le_bytes());
    chunks.extend_from_slice(&anmf);
    if anmf.len() % 2 == 1 {
      chunks.push(0);
    }
  }

  // write RIFF header
  destination.write_all(b"RIFF")?;
  let total_size = (4 + chunks.len()) as u32; // 'WEBP' + chunks
  destination.write_all(&total_size.to_le_bytes())?;
  destination.write_all(b"WEBP")?;
  destination.write_all(&chunks)?;

  Ok(())
}
