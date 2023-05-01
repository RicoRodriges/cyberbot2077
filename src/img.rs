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

/// Clear image colors. Target `color` transforms to about black, other colors are white
pub fn filter_img(img: &mut Image, color: &Pixel, range: u8) {
    for (x, y) in img.coordinates() {
        let src = img.get_pixel(x, y);
        let dest = Pixel::new(dif(src.r, color.r), dif(src.g, color.g), dif(src.b, color.b));

        let result = if dest.r <= range && dest.g <= range && dest.b <= range {
            dest // about BLACK
        } else {
            bmp::consts::WHITE
        };
        img.set_pixel(x, y, result);
    }
}

#[inline]
fn dif(a: u8, b: u8) -> u8 {
    return (a as i16 - b as i16).abs() as u8;
}

pub fn crop_img(img: &Image, left: u32, top: u32, right: u32, bottom: u32) -> Image {
    let mut dest = Image::new(right - left, bottom - top);
    for (x, y) in dest.coordinates() {
        dest.set_pixel(x, y, img.get_pixel(x + left, y + top));
    }
    return dest;
}


#[cfg(test)]
mod tests {
    use bmp::{Pixel, px};

    use crate::img::{crop_img, dif, filter_img};

    #[test]
    fn test_dif() {
        assert_eq!(0, dif(0, 0));
        assert_eq!(1, dif(0, 1));
        assert_eq!(1, dif(1, 0));
        assert_eq!(0, dif(255, 255));
        assert_eq!(1, dif(255, 254));
        assert_eq!(1, dif(254, 255));
        assert_eq!(255, dif(0, 255));
        assert_eq!(255, dif(255, 0));
    }

    #[test]
    fn test_filter_img() {
        let mut img = bmp::Image::new(4, 4);
        img.set_pixel(0, 0, px!(0, 10, 20));
        img.set_pixel(1, 0, px!(10, 20, 30));
        img.set_pixel(0, 1, px!(15, 25, 35));
        img.set_pixel(1, 1, px!(20, 30, 40));

        filter_img(&mut img, &px!(10, 20, 30), 5);

        assert_eq!(px!(255, 255, 255), img.get_pixel(0, 0));
        assert_eq!(px!(0, 0, 0), img.get_pixel(1, 0));
        assert_eq!(px!(5, 5, 5), img.get_pixel(0, 1));
        assert_eq!(px!(255, 255, 255), img.get_pixel(1, 1));

        assert_eq!(4, img.get_width());
        assert_eq!(4, img.get_height());
    }

    #[test]
    fn test_crop_img() {
        let mut img = bmp::Image::new(4, 4);
        img.set_pixel(0, 0, px!(0, 0, 0));
        img.set_pixel(1, 0, px!(10, 20, 30));
        img.set_pixel(0, 1, px!(0, 0, 0));
        img.set_pixel(1, 1, px!(0, 0, 0));

        let crop = crop_img(&img, 1, 0, 2, 1);

        assert_eq!(1, crop.get_width());
        assert_eq!(1, crop.get_height());
        assert_eq!(px!(10, 20, 30), crop.get_pixel(0, 0));
    }
}