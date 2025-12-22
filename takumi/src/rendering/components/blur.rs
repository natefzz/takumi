//! Stack blur implementation with O(1) complexity per pixel.
//!
//! Stack blur is a fast approximation of Gaussian blur that achieves O(1) complexity
//! by using a sliding window approach with weighted sums. Unlike Gaussian blur which
//! requires O(r) operations per pixel, stack blur incrementally updates sums as the
//! window slides.

use image::RgbaImage;

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
  let shr_val = 32;
  let mul_val = (1u64 << shr_val) / divisor;

  // Pre-allocate stack buffer (stores RGBA values for each position in the kernel)
  // u8 is sufficient because it stores individual pixel values
  let mut stack = vec![[0u8; 4]; div];

  // Temporary buffer for horizontal pass results (in premultiplied alpha)
  // u8 is sufficient because intermediate blurred values are still 0-255
  let mut temp = vec![0u8; pixels.len()];

  // Horizontal pass (convert to premultiplied alpha while reading, blur, keep as premultiplied)
  for y in 0..height {
    let row_start = y * width * 4;

    // Initialize sums
    // u32 is sufficient. Max sum = 255 * (255)^2 = ~16.5M << u32::MAX
    let mut sum = [0u32; 4];
    let mut sum_in = [0u32; 4];
    let mut sum_out = [0u32; 4];

    // Read first pixel and convert to premultiplied
    let first_pix = read_pixel_premultiplied(pixels, row_start);

    // Initialize the stack with edge-extended values
    for (i, slot) in stack.iter_mut().enumerate().take(radius + 1) {
      *slot = first_pix;
      let weight = (radius + 1 - i) as u32;
      for c in 0..4 {
        sum[c] += (first_pix[c] as u32) * weight;
        sum_out[c] += first_pix[c] as u32;
      }
    }

    for i in 1..=radius {
      let src_x = i.min(width - 1);
      let src_pix = read_pixel_premultiplied(pixels, row_start + src_x * 4);
      stack[i + radius] = src_pix;
      let weight = (radius + 1 - i) as u32;
      for c in 0..4 {
        sum[c] += (src_pix[c] as u32) * weight;
        sum_in[c] += src_pix[c] as u32;
      }
    }

    let mut stack_ptr = radius;

    for x in 0..width {
      let dst_idx = row_start + x * 4;

      // Write blurred value (still premultiplied)
      for c in 0..4 {
        temp[dst_idx + c] = ((sum[c] as u64 * mul_val) >> shr_val) as u8;
      }

      // Update sums
      for c in 0..4 {
        sum[c] -= sum_out[c];
      }

      let stack_start = (stack_ptr + div - radius) % div;
      for c in 0..4 {
        sum_out[c] -= stack[stack_start][c] as u32;
      }

      let src_x = (x + radius + 1).min(width - 1);
      let src_pix = read_pixel_premultiplied(pixels, row_start + src_x * 4);
      stack[stack_start] = src_pix;

      for c in 0..4 {
        sum_in[c] += src_pix[c] as u32;
        sum[c] += sum_in[c];
      }

      stack_ptr = (stack_ptr + 1) % div;

      for c in 0..4 {
        sum_out[c] += stack[stack_ptr][c] as u32;
        sum_in[c] -= stack[stack_ptr][c] as u32;
      }
    }
  }

  // Vertical pass (blur and convert back to straight alpha while writing)
  for x in 0..width {
    let mut sum = [0u32; 4];
    let mut sum_in = [0u32; 4];
    let mut sum_out = [0u32; 4];

    // Read first pixel from temp (already premultiplied)
    let first_pix = read_temp_pixel(&temp, x * 4);

    for (i, slot) in stack.iter_mut().enumerate().take(radius + 1) {
      *slot = first_pix;
      let weight = (radius + 1 - i) as u32;
      for c in 0..4 {
        sum[c] += (first_pix[c] as u32) * weight;
        sum_out[c] += first_pix[c] as u32;
      }
    }

    for i in 1..=radius {
      let src_y = i.min(height - 1);
      let src_idx = src_y * width * 4 + x * 4;
      let src_pix = read_temp_pixel(&temp, src_idx);
      stack[i + radius] = src_pix;
      let weight = (radius + 1 - i) as u32;
      for c in 0..4 {
        sum[c] += (src_pix[c] as u32) * weight;
        sum_in[c] += src_pix[c] as u32;
      }
    }

    let mut stack_ptr = radius;

    for y in 0..height {
      let dst_idx = y * width * 4 + x * 4;

      // Compute blurred premultiplied values
      let r = ((sum[0] as u64 * mul_val) >> shr_val).min(255) as u32;
      let g = ((sum[1] as u64 * mul_val) >> shr_val).min(255) as u32;
      let b = ((sum[2] as u64 * mul_val) >> shr_val).min(255) as u32;
      let a = ((sum[3] as u64 * mul_val) >> shr_val).min(255) as u32;

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

      // Update sums
      for c in 0..4 {
        sum[c] -= sum_out[c];
      }

      let stack_start = (stack_ptr + div - radius) % div;
      for c in 0..4 {
        sum_out[c] -= stack[stack_start][c] as u32;
      }

      let src_y = (y + radius + 1).min(height - 1);
      let src_idx = src_y * width * 4 + x * 4;
      let src_pix = read_temp_pixel(&temp, src_idx);
      stack[stack_start] = src_pix;

      for c in 0..4 {
        sum_in[c] += src_pix[c] as u32;
        sum[c] += sum_in[c];
      }

      stack_ptr = (stack_ptr + 1) % div;

      for c in 0..4 {
        sum_out[c] += stack[stack_ptr][c] as u32;
        sum_in[c] -= stack[stack_ptr][c] as u32;
      }
    }
  }
}

/// Reads a pixel from the source buffer and converts to premultiplied alpha.
#[inline(always)]
fn read_pixel_premultiplied(pixels: &[u8], idx: usize) -> [u8; 4] {
  let r = pixels[idx];
  let g = pixels[idx + 1];
  let b = pixels[idx + 2];
  let a = pixels[idx + 3];

  if a == 0 || a == 255 {
    // No conversion needed for fully transparent or fully opaque
    [r, g, b, a]
  } else {
    // Premultiply: color = color * alpha / 255
    [
      fast_div_255(r as u16 * a as u16),
      fast_div_255(g as u16 * a as u16),
      fast_div_255(b as u16 * a as u16),
      a,
    ]
  }
}

/// Fast approximation of integer division by 255.
#[inline(always)]
pub(crate) fn fast_div_255(v: u16) -> u8 {
  ((v + 128 + (v >> 8)) >> 8) as u8
}

/// Reads a pixel from the temporary buffer.
#[inline(always)]
fn read_temp_pixel(temp: &[u8], idx: usize) -> [u8; 4] {
  [temp[idx], temp[idx + 1], temp[idx + 2], temp[idx + 3]]
}
