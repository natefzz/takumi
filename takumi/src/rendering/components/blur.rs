//! Stack blur implementation with O(1) complexity per pixel.
//!
//! Stack blur is a fast approximation of Gaussian blur that achieves O(1) complexity
//! by using a sliding window approach with weighted sums. Unlike Gaussian blur which
//! requires O(r) operations per pixel, stack blur incrementally updates sums as the
//! window slides.
//!
//! This implementation uses SIMD via the `wide` crate for accelerated 4-channel operations.

use image::RgbaImage;
use wide::{u32x4, u64x4};

/// Applies a stack blur to an image.
pub(crate) fn apply_blur(image: &mut RgbaImage, radius: f32) {
  if radius <= 0.0 {
    return;
  }

  // Convert standard deviation (radius) to stack blur radius.
  // Approximation: stack_radius ~= sigma * 1.225
  let blur_radius = (radius * 3.0).round().max(1.0) as u32;

  let (width, height) = image.dimensions();
  stack_blur_with_premultiply(image.as_mut(), width, height, blur_radius);
}

/// Applies stack blur with integrated premultiplied alpha conversion.
/// This merges the premultiply, blur, and unpremultiply operations into fewer passes.
fn stack_blur_with_premultiply(pixels: &mut [u8], width: u32, height: u32, radius: u32) {
  if radius == 0 || width == 0 || height == 0 {
    return;
  }

  let radius = radius.min(254) as usize;
  let width = width as usize;
  let height = height as usize;

  let div = (radius * 2) + 1;

  // The sum of weights in stack blur is: sum(1..=radius+1) + sum(1..=radius) = (radius+1)^2
  let divisor = ((radius + 1) * (radius + 1)) as u64;

  // Fixed-point multiplier for the divisor
  // We want to replace x / divisor with (x * mul) >> shift
  let mul_val = (1u64 << 32) / divisor;
  let mul_vec = u64x4::splat(mul_val);

  // Pre-allocate stack buffer (stores RGBA values for each position in the kernel)
  // Using u32x4 for smaller memory footprint and better cache efficiency
  let mut stack = vec![u32x4::ZERO; div];

  // Temporary buffer for horizontal pass results (in premultiplied alpha)
  let mut temp = vec![0u8; pixels.len()];

  // Horizontal pass (convert to premultiplied alpha while reading, blur, keep as premultiplied)
  for y in 0..height {
    let row_start = y * width * 4;

    // Initialize sums using SIMD (u32x4 is sufficient for accumulation)
    // Max sum = 255 * 255^2 = ~16.5M << u32::MAX (~4.3B)
    let mut sum = u32x4::ZERO;
    let mut sum_in = u32x4::ZERO;
    let mut sum_out = u32x4::ZERO;

    // Read first pixel and convert to premultiplied
    let first_pix = read_pixel_premultiplied_simd(pixels, row_start);

    // Initialize the stack with edge-extended values
    for (i, slot) in stack.iter_mut().enumerate().take(radius + 1) {
      *slot = first_pix;
      let weight = u32x4::splat((radius + 1 - i) as u32);
      sum += first_pix * weight;
      sum_out += first_pix;
    }

    for (i, slot) in stack.iter_mut().enumerate().skip(radius + 1).take(radius) {
      let src_x = (i - radius).min(width - 1);
      let src_pix = read_pixel_premultiplied_simd(pixels, row_start + src_x * 4);
      *slot = src_pix;
      let weight = u32x4::splat((i - radius) as u32);
      sum += src_pix * weight;
      sum_in += src_pix;
    }

    let mut stack_ptr = radius;

    for x in 0..width {
      let dst_idx = row_start + x * 4;

      // Convert to u64x4 for the multiply+shift division (to avoid overflow)
      let sum_u64 = u32x4_to_u64x4(sum);
      let blurred: u64x4 = (sum_u64 * mul_vec) >> 32;
      let arr = blurred.to_array();
      temp[dst_idx] = arr[0] as u8;
      temp[dst_idx + 1] = arr[1] as u8;
      temp[dst_idx + 2] = arr[2] as u8;
      temp[dst_idx + 3] = arr[3] as u8;

      // Update sums (all SIMD operations on u32x4)
      sum -= sum_out;

      let stack_start = (stack_ptr + div - radius) % div;
      sum_out -= stack[stack_start];

      let src_x = (x + radius + 1).min(width - 1);
      let src_pix = read_pixel_premultiplied_simd(pixels, row_start + src_x * 4);
      stack[stack_start] = src_pix;

      sum_in += src_pix;
      sum += sum_in;

      stack_ptr = (stack_ptr + 1) % div;

      sum_out += stack[stack_ptr];
      sum_in -= stack[stack_ptr];
    }
  }

  // Vertical pass (blur and convert back to straight alpha while writing)
  for x in 0..width {
    let mut sum = u32x4::ZERO;
    let mut sum_in = u32x4::ZERO;
    let mut sum_out = u32x4::ZERO;

    // Read first pixel from temp (already premultiplied)
    let first_pix = read_temp_pixel_simd(&temp, x * 4);

    for (i, slot) in stack.iter_mut().enumerate().take(radius + 1) {
      *slot = first_pix;
      let weight = u32x4::splat((radius + 1 - i) as u32);
      sum += first_pix * weight;
      sum_out += first_pix;
    }

    for (i, slot) in stack.iter_mut().enumerate().skip(radius + 1).take(radius) {
      let src_y = (i - radius).min(height - 1);
      let src_idx = src_y * width * 4 + x * 4;
      let src_pix = read_temp_pixel_simd(&temp, src_idx);
      *slot = src_pix;
      let weight = u32x4::splat((i - radius) as u32);
      sum += src_pix * weight;
      sum_in += src_pix;
    }

    let mut stack_ptr = radius;

    for y in 0..height {
      let dst_idx = y * width * 4 + x * 4;

      // Convert to u64x4 for the multiply+shift division (to avoid overflow)
      let sum_u64 = u32x4_to_u64x4(sum);
      let blurred: u64x4 = (sum_u64 * mul_vec) >> 32;
      let arr = blurred.to_array();
      let r = (arr[0] as u32).min(255);
      let g = (arr[1] as u32).min(255);
      let b = (arr[2] as u32).min(255);
      let a = (arr[3] as u32).min(255);

      // Convert back to straight alpha and write
      if a == 0 {
        pixels[dst_idx] = 0;
        pixels[dst_idx + 1] = 0;
        pixels[dst_idx + 2] = 0;
        pixels[dst_idx + 3] = 0;
      } else if a == 255 {
        pixels[dst_idx] = r as u8;
        pixels[dst_idx + 1] = g as u8;
        pixels[dst_idx + 2] = b as u8;
        pixels[dst_idx + 3] = 255;
      } else {
        // Unpremultiply: color = premultiplied_color * 255 / alpha
        pixels[dst_idx] = ((r * 255) / a).min(255) as u8;
        pixels[dst_idx + 1] = ((g * 255) / a).min(255) as u8;
        pixels[dst_idx + 2] = ((b * 255) / a).min(255) as u8;
        pixels[dst_idx + 3] = a as u8;
      }

      // Update sums (all SIMD operations on u32x4)
      sum -= sum_out;

      let stack_start = (stack_ptr + div - radius) % div;
      sum_out -= stack[stack_start];

      let src_y = (y + radius + 1).min(height - 1);
      let src_idx = src_y * width * 4 + x * 4;
      let src_pix = read_temp_pixel_simd(&temp, src_idx);
      stack[stack_start] = src_pix;

      sum_in += src_pix;
      sum += sum_in;

      stack_ptr = (stack_ptr + 1) % div;

      sum_out += stack[stack_ptr];
      sum_in -= stack[stack_ptr];
    }
  }
}

/// Converts u32x4 to u64x4 for overflow-safe multiplication.
#[inline(always)]
fn u32x4_to_u64x4(v: u32x4) -> u64x4 {
  let arr = v.to_array();
  u64x4::from([arr[0] as u64, arr[1] as u64, arr[2] as u64, arr[3] as u64])
}

/// Reads a pixel from the source buffer, converts to premultiplied alpha, and returns as SIMD vector.
#[inline(always)]
fn read_pixel_premultiplied_simd(pixels: &[u8], idx: usize) -> u32x4 {
  let r = pixels[idx] as u32;
  let g = pixels[idx + 1] as u32;
  let b = pixels[idx + 2] as u32;
  let a = pixels[idx + 3] as u32;

  if a == 0 || a == 255 {
    // No conversion needed for fully transparent or fully opaque
    u32x4::from([r, g, b, a])
  } else {
    // Premultiply: color = color * alpha / 255
    u32x4::from([
      fast_div_255((r * a) as u16) as u32,
      fast_div_255((g * a) as u16) as u32,
      fast_div_255((b * a) as u16) as u32,
      a,
    ])
  }
}

/// Fast approximation of integer division by 255.
#[inline(always)]
pub(crate) fn fast_div_255(v: u16) -> u8 {
  ((v + 128 + (v >> 8)) >> 8) as u8
}

/// Reads a pixel from the temporary buffer as SIMD vector.
#[inline(always)]
fn read_temp_pixel_simd(temp: &[u8], idx: usize) -> u32x4 {
  u32x4::from([
    temp[idx] as u32,
    temp[idx + 1] as u32,
    temp[idx + 2] as u32,
    temp[idx + 3] as u32,
  ])
}
