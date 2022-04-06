// Copyright 2017, Reizner Evgeniy <razrfalcon@gmail.com>.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

/*!
*color-thief-rs* is a [color-thief](https://github.com/lokesh/color-thief)
algorithm reimplementation in Rust.

The implementation itself is a heavily modified
[Swift version](https://github.com/yamoridon/ColorThiefSwift) of the same algorithm.
*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate rgb;

use std::cmp;
use std::fmt;
use std::error;
use std::u8;

pub use rgb::RGB8 as Color;

const SIGNAL_BITS: i32              = 5; // Use only upper 5 bits of 8 bits.
const RIGHT_SHIFT: i32              = 8 - SIGNAL_BITS;
const MULTIPLIER: i32               = 1 << RIGHT_SHIFT;
const MULTIPLIER_64: f64            = MULTIPLIER as f64;
const HISTOGRAM_SIZE: usize         = 1 << (3 * SIGNAL_BITS);
const VBOX_LENGTH: usize            = 1 << SIGNAL_BITS;
const FRACTION_BY_POPULATION: f64   = 0.75;
const MAX_ITERATIONS: i32           = 1000;

/// Represent a color format of an underlying image data.
#[allow(missing_docs)]
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum ColorFormat {
    Rgb,
    Rgba,
    Argb,
    Bgr,
    Bgra,
}

/// List of all errors.
#[allow(missing_docs)]
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum Error {
    InvalidVBox,
    VBoxCutFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::InvalidVBox => "an invalid VBox",
            Error::VBoxCutFailed => "failed to cut a VBox",
        };

        write!(f, "{}", msg)
    }
}

impl error::Error for Error {}

/// Returns a representative color palette of an image.
///
/// * `pixels` - A raw image data.
///
///   We do not use any existing image representing crate for a better portability.
/// * `color_format` - Represent a color format of an underlying image data.
/// * `quality` - Quality of an output colors.
///
///   Basically, a step in pixels to improve performance.
///
///   Range: 1..10.
/// * `max_colors` - A number of colors in the output palette.
///   Actual colors count can be lower depending on the image.
///
///   Range: 2..255.
pub fn get_palette(
    pixels: &[u8],
    color_format: ColorFormat,
    quality: u8,
    max_colors: u8,
) -> Result<Vec<Color>, Error> {
    assert!(quality > 0 && quality <= 10);
    assert!(max_colors > 1);

    quantize(&pixels, color_format, quality, max_colors)
}

enum ColorChannel {
    Red,
    Green,
    Blue,
}

#[derive(Clone)]
struct VBox {
    r_min: u8,
    r_max: u8,
    g_min: u8,
    g_max: u8,
    b_min: u8,
    b_max: u8,
    average: Color,
    volume: i32,
    count: i32,
}

impl VBox {
    fn new(
        r_min: u8, r_max: u8,
        g_min: u8, g_max: u8,
        b_min: u8, b_max: u8,
    ) -> VBox {
        VBox {
            r_min: r_min,
            r_max: r_max,
            g_min: g_min,
            g_max: g_max,
            b_min: b_min,
            b_max: b_max,
            average: Color::new(0, 0, 0),
            volume: 0,
            count: 0,
        }

        // `recalc()` should be called right after `new()`.
    }

    fn recalc(&mut self, histogram: &[i32]) {
        self.average = self.calc_average(histogram);
        self.count = self.calc_count(histogram);
        self.volume = self.calc_volume();
    }

    /// Get 3 dimensional volume of the color space.
    fn calc_volume(&self) -> i32 {
          (self.r_max as i32 - self.r_min as i32 + 1)
        * (self.g_max as i32 - self.g_min as i32 + 1)
        * (self.b_max as i32 - self.b_min as i32 + 1)
    }

    /// Get total count of histogram samples.
    fn calc_count(&self, histogram: &[i32]) -> i32 {
        let mut count = 0;
        for i in self.r_min..(self.r_max + 1) {
            for j in self.g_min..(self.g_max + 1) {
                for k in self.b_min..(self.b_max + 1) {
                    let index = make_color_index_of(i, j, k);
                    count += histogram[index];
                }
            }
        }

        count
    }

    fn calc_average(&self, histogram: &[i32]) -> Color {
        let mut ntot = 0;

        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;

        for i in self.r_min..(self.r_max + 1) {
            for j in self.g_min..(self.g_max + 1) {
                for k in self.b_min..(self.b_max + 1) {
                    let index = make_color_index_of(i, j, k);
                    let hval = histogram[index] as f64;
                    ntot += hval as i32;
                    r_sum += (hval * (i as f64 + 0.5) * MULTIPLIER_64) as i32;
                    g_sum += (hval * (j as f64 + 0.5) * MULTIPLIER_64) as i32;
                    b_sum += (hval * (k as f64 + 0.5) * MULTIPLIER_64) as i32;
                }
            }
        }

        if ntot > 0 {
            let r = r_sum / ntot;
            let g = g_sum / ntot;
            let b = b_sum / ntot;
            Color::new(r as u8, g as u8, b as u8)
        } else {
            let r = MULTIPLIER * (self.r_min as i32 + self.r_max as i32 + 1) / 2;
            let g = MULTIPLIER * (self.g_min as i32 + self.g_max as i32 + 1) / 2;
            let b = MULTIPLIER * (self.b_min as i32 + self.b_max as i32 + 1) / 2;
            Color::new(cmp::min(r, 255) as u8,
                       cmp::min(g, 255) as u8,
                       cmp::min(b, 255) as u8)
        }
    }

    fn widest_color_channel(&self) -> ColorChannel {
        let r_width = self.r_max - self.r_min;
        let g_width = self.g_max - self.g_min;
        let b_width = self.b_max - self.b_min;

        let max = cmp::max(cmp::max(r_width, g_width), b_width);

        if max == r_width {
            ColorChannel::Red
        } else if max == g_width {
            ColorChannel::Green
        } else {
            ColorChannel::Blue
        }
    }
}

fn make_histogram_and_vbox(
    pixels: &[u8],
    color_format: ColorFormat,
    step: u8,
) -> (VBox, Vec<i32>) {
    let mut histogram: Vec<i32> = (0..HISTOGRAM_SIZE).map(|_| 0).collect();

    let mut r_min = u8::MAX;
    let mut r_max = u8::MIN;
    let mut g_min = u8::MAX;
    let mut g_max = u8::MIN;
    let mut b_min = u8::MAX;
    let mut b_max = u8::MIN;

    let colors_count = match color_format {
        ColorFormat::Rgb => 3,
        ColorFormat::Rgba => 4,
        ColorFormat::Argb => 4,
        ColorFormat::Bgr => 3,
        ColorFormat::Bgra => 4,
    };

    let pixel_count = pixels.len() / colors_count;
    let mut i = 0;
    while i < pixel_count {
        let pos = i * colors_count;

        let (r, g, b, a) = color_parts(pixels, color_format, pos);

        i += colors_count * step as usize;

        // If pixel is mostly opaque or white.
        if a < 125 || (r > 250 && g > 250 && b > 250) {
            continue;
        }

        let shifted_r = r >> RIGHT_SHIFT as u8;
        let shifted_b = b >> RIGHT_SHIFT as u8;
        let shifted_g = g >> RIGHT_SHIFT as u8;

        r_min = cmp::min(r_min, shifted_r);
        r_max = cmp::max(r_max, shifted_r);
        g_min = cmp::min(g_min, shifted_g);
        g_max = cmp::max(g_max, shifted_g);
        b_min = cmp::min(b_min, shifted_b);
        b_max = cmp::max(b_max, shifted_b);

        // Increment histogram.
        let index = make_color_index_of(shifted_r, shifted_g, shifted_b);
        histogram[index] += 1;
    }

    let mut vbox = VBox::new(r_min, r_max, g_min, g_max, b_min, b_max);
    vbox.recalc(&histogram);

    (vbox, histogram)
}


/// Extracts r, g, b, a color parts.
fn color_parts(
    pixels: &[u8],
    color_format: ColorFormat,
    pos: usize,
) -> (u8, u8, u8, u8) {
    match color_format {
        ColorFormat::Rgb => {
            (pixels[pos + 0],
             pixels[pos + 1],
             pixels[pos + 2],
             255)
        }
        ColorFormat::Rgba => {
            (pixels[pos + 0],
             pixels[pos + 1],
             pixels[pos + 2],
             pixels[pos + 3])
        }
        ColorFormat::Argb => {
            (pixels[pos + 1],
             pixels[pos + 2],
             pixels[pos + 3],
             pixels[pos + 0])
        },
        ColorFormat::Bgr => {
            (pixels[pos + 2],
             pixels[pos + 1],
             pixels[pos + 0],
             255)
        }
        ColorFormat::Bgra => {
            (pixels[pos + 2],
             pixels[pos + 1],
             pixels[pos + 0],
             pixels[pos + 3])
        }
    }
}

fn apply_median_cut(
    histogram: &[i32],
    vbox: &mut VBox,
) -> Result<(VBox, Option<VBox>), Error> {
    if vbox.count == 0 {
        return Err(Error::InvalidVBox);
    }

    // Only one pixel, no split.
    if vbox.count == 1 {
        return Ok((vbox.clone(), None));
    }

    // Find the partial sum arrays along the selected axis.
    let mut total = 0;
    let mut partial_sum: Vec<i32> = (0..VBOX_LENGTH).map(|_| -1).collect();

    let axis = vbox.widest_color_channel();
    match axis {
        ColorChannel::Red => {
            for i in vbox.r_min..(vbox.r_max + 1) {
                let mut sum = 0;
                for j in vbox.g_min..(vbox.g_max + 1) {
                    for k in vbox.b_min..(vbox.b_max + 1) {
                        let index = make_color_index_of(i, j, k);
                        sum += histogram[index];
                    }
                }
                total += sum;
                partial_sum[i as usize] = total;
            }
        }
        ColorChannel::Green => {
            for i in vbox.g_min..(vbox.g_max + 1) {
                let mut sum = 0;
                for j in vbox.r_min..(vbox.r_max + 1) {
                    for k in vbox.b_min..(vbox.b_max + 1) {
                        let index = make_color_index_of(j, i, k);
                        sum += histogram[index];
                    }
                }
                total += sum;
                partial_sum[i as usize] = total;
            }
        }
        ColorChannel::Blue => {
            for i in vbox.b_min..(vbox.b_max + 1) {
                let mut sum = 0;
                for j in vbox.r_min..(vbox.r_max + 1) {
                    for k in vbox.g_min..(vbox.g_max + 1) {
                        let index = make_color_index_of(j, k, i);
                        sum += histogram[index];
                    }
                }
                total += sum;
                partial_sum[i as usize] = total;
            }
        }
    }

    let mut look_ahead_sum: Vec<i32> = (0..VBOX_LENGTH).map(|_| -1).collect();
    for (i, sum) in partial_sum.iter().enumerate().filter(|&(_, sum)| *sum != -1) {
        look_ahead_sum[i] = total - sum;
    }

    cut(axis, vbox, histogram, &partial_sum, &look_ahead_sum, total)
}

fn cut(
    axis: ColorChannel,
    vbox: &VBox,
    histogram: &[i32],
    partial_sum: &[i32],
    look_ahead_sum: &[i32],
    total: i32,
) -> Result<(VBox, Option<VBox>), Error> {
    let (vbox_min, vbox_max) = match axis {
        ColorChannel::Red =>   (vbox.r_min as i32, vbox.r_max as i32),
        ColorChannel::Green => (vbox.g_min as i32, vbox.g_max as i32),
        ColorChannel::Blue =>  (vbox.b_min as i32, vbox.b_max as i32),
    };

    for i in vbox_min..vbox_max + 1 {
        if partial_sum[i as usize] <= total / 2 {
            continue;
        }

        let mut vbox1 = vbox.clone();
        let mut vbox2 = vbox.clone();

        let left = i - vbox_min;
        let right = vbox_max - i;

        let mut d2 = if left <= right {
            cmp::min(vbox_max - 1, i + right / 2)
        } else {
            // 2.0 and cast to int is necessary to have the same
            // behavior as in JavaScript.
            cmp::max(vbox_min, ((i - 1) as f64 - left as f64 / 2.0) as i32)
        };

        // Avoid 0-count.
        while d2 < 0 || partial_sum[d2 as usize] <= 0 {
            d2 += 1;
        }
        let mut count2 = look_ahead_sum[d2 as usize];
        while count2 == 0 && d2 > 0 && partial_sum[d2 as usize - 1] > 0 {
            d2 -= 1;
            count2 = look_ahead_sum[d2 as usize];
        }

        // Set dimensions.
        match axis {
            ColorChannel::Red => {
                vbox1.r_max = d2 as u8;
                vbox2.r_min = (d2 + 1) as u8;
            }
            ColorChannel::Green => {
                vbox1.g_max = d2 as u8;
                vbox2.g_min = (d2 + 1) as u8;
            }
            ColorChannel::Blue => {
                vbox1.b_max = d2 as u8;
                vbox2.b_min = (d2 + 1) as u8;
            }
        }

        vbox1.recalc(histogram);
        vbox2.recalc(histogram);

        return Ok((vbox1, Some(vbox2)));
    }

    Err(Error::VBoxCutFailed)
}

fn quantize(
    pixels: &[u8],
    color_format: ColorFormat,
    quality: u8,
    max_colors: u8,
) -> Result<Vec<Color>, Error> {
    // Get the histogram and the beginning vbox from the colors.
    let (vbox, histogram) = make_histogram_and_vbox(pixels, color_format, quality);

    // Priority queue.
    let mut pq = vec![vbox.clone()];

    // Round up to have the same behavior as in JavaScript
    let target = (FRACTION_BY_POPULATION * max_colors as f64).ceil() as u8;

    // First set of colors, sorted by population.
    iterate(&mut pq, compare_by_count, target, &histogram)?;

    // Re-sort by the product of pixel occupancy times the size in color space.
    pq.sort_by(compare_by_product);

    // next set - generate the median cuts using the (npix * vol) sorting.
    let len = pq.len() as u8;
    iterate(&mut pq, compare_by_product, max_colors - len, &histogram)?;

    // Reverse to put the highest elements first into the color map.
    pq.reverse();

    // Keep at most `max_colors` in the resulting vector.
    let mut colors: Vec<Color> = pq.iter().map(|v| v.average).collect();
    colors.truncate(max_colors as usize);

    Ok(colors)
}

// Inner function to do the iteration.
fn iterate<P>(
    queue: &mut Vec<VBox>,
    comparator: P,
    target: u8,
    histogram: &[i32],
) -> Result<(), Error>
    where P: FnMut(&VBox, &VBox) -> cmp::Ordering + Copy
{
    let mut color = 1;

    for _ in 0..MAX_ITERATIONS {
        if let Some(mut vbox) = queue.last().cloned() {
            if vbox.count == 0 {
                queue.sort_by(comparator);
                continue;
            }
            queue.pop();

            // Do the cut.
            let vboxes = apply_median_cut(histogram, &mut vbox)?;
            queue.push(vboxes.0.clone());
            if let Some(ref vb) = vboxes.1 {
                queue.push(vb.clone());
                color += 1;
            }

            queue.sort_by(comparator);

            if color >= target {
               break;
            }
        }
    }

    Ok(())
}

fn compare_by_count(a: &VBox, b: &VBox) -> cmp::Ordering {
    a.count.cmp(&b.count)
}

fn compare_by_product(a: &VBox, b: &VBox) -> cmp::Ordering {
    if a.count == b.count {
        // If count is 0 for both (or the same), sort by volume.
        a.volume.cmp(&b.volume)
    } else {
        // Otherwise sort by products.
        let a_product = a.count as i64 * a.volume as i64;
        let b_product = b.count as i64 * b.volume as i64;
        a_product.cmp(&b_product)
    }
}

/// Get reduced-space color index for a pixel.
fn make_color_index_of(red: u8, green: u8, blue: u8) -> usize {
    (   ((red as i32) << (2 * SIGNAL_BITS))
      + ((green as i32) << SIGNAL_BITS)
      +   blue as i32
    ) as usize
}
