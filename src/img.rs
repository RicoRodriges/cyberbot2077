use std::path::Path;

use bmp::{Image, Pixel, px};
use clipboard_win::{formats, get_clipboard};

pub fn load_img_from_clipboard() -> Option<Image> {
    return if let Ok(bytes) = get_clipboard(formats::Bitmap) {
        if bytes[0x1C] == 32 {
            // 32 bits per pixel :(
            // `bmp` crate does not support it. I don't want to include other crates.
            // convert it to 24 bits per pixel, ignore alpha channel
            let width = (bytes[0x12] as u32) | ((bytes[0x13] as u32) << 8);
            let height = (bytes[0x16] as u32) | ((bytes[0x17] as u32) << 8);

            let mut img = Image::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    let b = bytes[(0x36 + (x + y * width) * 4 + 0) as usize];
                    let g = bytes[(0x36 + (x + y * width) * 4 + 1) as usize];
                    let r = bytes[(0x36 + (x + y * width) * 4 + 2) as usize];
                    img.set_pixel(x, height - y - 1, px![r, g, b]);
                }
            }
            Some(img)
        } else {
            Some(bmp::from_reader(&mut &bytes[..]).unwrap())
        }
    } else {
        None
    };
}

pub fn load_img_from_file<P: AsRef<Path>>(path: P) -> Image {
    return bmp::open(path).unwrap();
}

pub struct GrayImage {
    w: u32,
    h: u32,
    data: Vec<u8>,
}

impl GrayImage {
    /// Target `color` transforms to 255, other colors are 0
    pub fn filter(img: &Image, color: &Pixel, threshold: u8, left: u32, top: u32, right: u32, bottom: u32) -> Self {
        debug_assert!(right > left);
        debug_assert!(bottom > top);

        let r_min = i16::max(color.r as i16 - threshold as i16, 0) as u8;
        let g_min = i16::max(color.g as i16 - threshold as i16, 0) as u8;
        let b_min = i16::max(color.b as i16 - threshold as i16, 0) as u8;
        let color_min = px!(r_min, g_min, b_min);

        let r_max = i16::min(color.r as i16 + threshold as i16, 255) as u8;
        let g_max = i16::min(color.g as i16 + threshold as i16, 255) as u8;
        let b_max = i16::min(color.b as i16 + threshold as i16, 255) as u8;
        let color_max = px!(r_max, g_max, b_max);

        let w = right - left;
        let h = bottom - top;
        let mut data = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                let p1 = img.get_pixel(x + left, y + top);

                let v = if p1.r >= color_min.r && p1.r <= color_max.r &&
                    p1.g >= color_min.g && p1.g <= color_max.g &&
                    p1.b >= color_min.b && p1.b <= color_max.b {
                    255u8
                } else {
                    0u8
                };
                data.push(v);
            }
        }
        debug_assert_eq!(w * h, data.len() as u32);
        return Self { w, h, data };
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.w
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.h
    }

    #[inline]
    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        self.data[(y * self.w + x) as usize]
    }

    /// Tries to find filled rectangle in reverse mode (the most bottom-right)
    pub fn rfind_rect(&self, rect_width: u32, rect_height: u32) -> Option<(u32, u32)> {
        if self.h < rect_height || self.w < rect_width || rect_width == 0 || rect_height == 0 {
            return None;
        }
        (0..=(self.h - rect_height)).rev().find_map(|y_start| {
            (0..=(self.w - rect_width)).rev().find(|x_start| {
                let is_rect = (0..rect_height).all(|dy| {
                    (0..rect_width).all(|dx|
                        self.pixel(x_start + dx, y_start + dy) != 0
                    )
                });
                is_rect
            }).map(|x_start| (x_start, y_start))
        })
    }

    /// `result[x] == true` means column `x` has at least 1 non-zero pixel
    pub fn columns_usage(&self) -> Vec<bool> {
        (0..self.w).map(|x| {
            (0..self.h).any(|y|
                self.pixel(x, y) != 0
            )
        }).collect()
    }

    /// `result[y] == true` means row `y` has at least 1 non-zero pixel
    pub fn rows_usage(&self) -> Vec<bool> {
        (0..self.h).map(|y| {
            (0..self.w).any(|x|
                self.pixel(x, y) != 0
            )
        }).collect()
    }

    /// Returns the smallest rectangle, which contains all non-zero pixels in `self_*` area
    pub fn rect_hull(&self, self_left: u32, self_top: u32, self_right: u32, self_bottom: u32) -> Option<(u32, u32, u32, u32)> {
        debug_assert!(self_right >= self_left);
        debug_assert!(self_bottom >= self_top);
        debug_assert!(self_right < self.w);
        debug_assert!(self_bottom < self.h);

        let mut result: Option<(u32, u32, u32, u32)> = None;

        for y in self_top..=self_bottom {
            for x in self_left..=self_right {
                if self.pixel(x, y) != 0 {
                    let prev = result.take()
                        .unwrap_or((u32::max_value(), u32::max_value(), u32::min_value(), u32::min_value()));
                    result = Some((
                        u32::min(x, prev.0),
                        u32::min(y, prev.1),
                        u32::max(x, prev.2),
                        y,
                    ));
                }
            }
        }

        return result;
    }

    pub fn template_match_error_score(&self, self_left: u32, self_top: u32, self_right: u32, self_bottom: u32, template: &GrayImage) -> f64 {
        debug_assert!(self_right >= self_left);
        debug_assert!(self_bottom >= self_top);
        debug_assert!(self_right < self.w);
        debug_assert!(self_bottom < self.h);

        let self_width = self_right - self_left;
        let self_height = self_bottom - self_top;
        let self_pixel = |x: u32, y: u32| -> u8 {
            self.pixel(self_left + x, self_top + y)
        };

        let x_norm = (template.w as f64) / (self_width as f64);
        let y_norm = (template.h as f64) / (self_height as f64);
        let x_max = template.w - 1;
        let y_max = template.h - 1;
        let template_pixel = |x: u32, y: u32| -> u8 {
            template.pixel(u32::min((x as f64 * x_norm) as _, x_max), u32::min((y as f64 * y_norm) as _, y_max))
        };

        let error: u32 = (0..self_height).map(|y| {
            (0..self_width)
                .filter(|x| self_pixel(*x, y) != template_pixel(*x, y))
                .count() as u32
        }).sum();
        return error as f64 / (self_width * self_height) as f64;
    }
}

#[cfg(test)]
pub fn into_image(src: &GrayImage) -> Image {
    let mut dest = Image::new(src.w, src.h);
    for (x, y) in dest.coordinates() {
        let v = src.data[(y * src.w + x) as usize];
        dest.set_pixel(x, y, px!(v, v, v));
    }
    return dest;
}

#[cfg(test)]
mod tests {
    use bmp::{Pixel, px};

    use crate::img::{GrayImage, into_image};

    #[test]
    fn test_filter() {
        let mut img = bmp::Image::new(4, 4);
        img.set_pixel(0, 0, px!(0, 10, 20));
        img.set_pixel(1, 0, px!(10, 20, 30));
        img.set_pixel(0, 1, px!(15, 25, 35));
        img.set_pixel(1, 1, px!(20, 30, 40));

        let filtered = GrayImage::filter(&img, &px!(10, 20, 30), 5, 0, 0, 2, 2);
        assert_eq!(filtered.width(), 2);
        assert_eq!(filtered.height(), 2);
        assert_eq!(filtered.data, vec![0, 255, 255, 0]);
        assert_eq!(filtered.pixel(0, 0), 0);
        assert_eq!(filtered.pixel(1, 0), 255);
        assert_eq!(filtered.pixel(0, 1), 255);
        assert_eq!(filtered.pixel(1, 1), 0);
    }

    #[test]
    fn test_rfind_rect() {
        let pixels = vec![
            255, 255, 000,
            255, 255, 000,
        ];

        let filtered = into_img(pixels, 3, 2);
        assert_eq!(None, filtered.rfind_rect(10, 10));
        assert_eq!(None, filtered.rfind_rect(3, 2));
        assert_eq!(None, filtered.rfind_rect(3, 1));
        assert_eq!(Some((0, 1)), filtered.rfind_rect(2, 1));
        assert_eq!(Some((0, 0)), filtered.rfind_rect(2, 2));
    }

    #[test]
    fn test_columns_usage() {
        let pixels = vec![
            255, 000, 000,
            000, 000, 255,
        ];

        let filtered = into_img(pixels, 3, 2);
        let usage = filtered.columns_usage();
        assert_eq!(vec![true, false, true], usage);
        assert_eq!(3, usage.capacity());
    }

    #[test]
    fn test_rows_usage() {
        let pixels = vec![
            255, 000,
            000, 000,
            000, 255,
        ];

        let filtered = into_img(pixels, 2, 3);
        let usage = filtered.rows_usage();
        assert_eq!(vec![true, false, true], usage);
        assert_eq!(3, usage.capacity());
    }

    #[test]
    fn test_rect_hull() {
        let pixels = vec![
            000, 000, 000, 000, 000,
            000, 255, 255, 255, 255,
            000, 255, 000, 000, 255,
            000, 255, 255, 000, 255,
            000, 255, 255, 255, 255,
        ];

        let img = into_img(pixels, 5, 5);
        assert_eq!(Some((1, 1, 4, 4)), img.rect_hull(0, 0, 4, 4));
        assert_eq!(Some((2, 3, 2, 3)), img.rect_hull(2, 2, 3, 3));
        assert_eq!(None, img.rect_hull(2, 2, 3, 2));
    }

    #[test]
    fn test_template_match_error_score() {
        let template_pixels = vec![
            000, 000, 000, 000,
            000, 000, 000, 000,
            000, 000, 255, 255,
            000, 000, 255, 255,
        ];
        let template = into_img(template_pixels, 4, 4);

        let exact_match = vec![
            255, 255, 255, 255, 255, 255,
            255, 000, 000, 000, 000, 255,
            255, 000, 000, 000, 000, 255,
            255, 000, 000, 255, 255, 255,
            255, 000, 000, 255, 255, 255,
            255, 255, 255, 255, 255, 255,
        ];
        assert_eq!(0f64, into_img(exact_match, 6, 6).template_match_error_score(1, 1, 4, 4, &template));

        let exact_small_match = vec![
            255, 255, 255, 255,
            255, 000, 000, 255,
            255, 000, 255, 255,
            255, 255, 255, 255,
        ];
        assert_eq!(0f64, into_img(exact_small_match, 4, 4).template_match_error_score(1, 1, 2, 2, &template));

        let exact_big_match = vec![
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 000, 000, 000, 000, 000, 000, 000, 000, 255,
            255, 000, 000, 000, 000, 000, 000, 000, 000, 255,
            255, 000, 000, 000, 000, 000, 000, 000, 000, 255,
            255, 000, 000, 000, 000, 000, 000, 000, 000, 255,
            255, 000, 000, 000, 000, 255, 255, 255, 255, 255,
            255, 000, 000, 000, 000, 255, 255, 255, 255, 255,
            255, 000, 000, 000, 000, 255, 255, 255, 255, 255,
            255, 000, 000, 000, 000, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ];
        assert_eq!(0f64, into_img(exact_big_match, 10, 10).template_match_error_score(1, 1, 8, 8, &template));

        let full_unmatch = vec![
            255, 255, 255, 255,
            255, 255, 255, 255,
            255, 255, 000, 000,
            255, 255, 000, 000,
        ];
        assert_eq!(1f64, into_img(full_unmatch, 4, 4).template_match_error_score(0, 0, 3, 3, &template));

        let half_match = vec![
            255, 255, 000, 000,
            255, 255, 000, 000,
            000, 000, 000, 000,
            000, 000, 000, 000,
        ];
        let half_error = into_img(half_match, 4, 4).template_match_error_score(0, 0, 3, 3, &template);
        assert!(0.43 < half_error);
        assert!(0.58 > half_error);
    }

    #[test]
    fn test_into_image() {
        let pixels = vec![
            255, 000,
            000, 255,
        ];

        let bmp: bmp::Image = into_image(&into_img(pixels, 2, 2));
        assert_eq!(bmp.get_width(), 2);
        assert_eq!(bmp.get_height(), 2);
        assert_eq!(px!(255, 255, 255), bmp.get_pixel(0, 0));
        assert_eq!(px!(000, 000, 000), bmp.get_pixel(1, 0));
        assert_eq!(px!(000, 000, 000), bmp.get_pixel(0, 1));
        assert_eq!(px!(255, 255, 255), bmp.get_pixel(1, 1));

    }

    fn into_img(data: Vec<u8>, w: u32, h: u32) -> GrayImage {
        assert_eq!((w * h) as usize, data.len());
        return GrayImage { data, w, h };
    }
}