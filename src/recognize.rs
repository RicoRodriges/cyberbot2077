use bmp::{Image, Pixel, px};

use crate::img::{crop_img, filter_img};

pub const MATRIX_COLOR: Pixel = px!(0xD0, 0xED, 0x57);

const CONDITION_BORDER_COLOR: Pixel = px!(0x81, 0x96, 0x38);
pub const CONDITION_COLOR: Pixel = px!(0xF0, 0xF0, 0xF0);

const BLOCK_COLOR: Pixel = px!(0x4F, 0x5A, 0x25);

pub fn find_matrix_area(img: &Image) -> Option<(u32, u32, u32, u32)> {
    let mut img = crop_img(&img, 0, 0, img.get_width() / 2, img.get_height());
    filter_img(&mut img, &MATRIX_COLOR, 50);

    let mut horizontal_area: Option<(u32, u32)> = None;
    {
        // tries to find second and third vertical lines in center from left
        let line_height = 150;

        let mut l: Option<u32> = None;
        let mut r: Option<u32> = None;
        let mut lines = 0;
        let center = img.get_height() / 2;
        for x in 0..img.get_width() {
            let vertical_line = (0..line_height).all(|v| img.get_pixel(x, center - v) != bmp::consts::WHITE);
            if vertical_line {
                lines += 1;
                if lines == 2 {
                    l = Some(x);
                } else if lines == 3 {
                    r = Some(x);
                }
            }
            if lines >= 3 {
                break;
            }
        }
        if l.is_some() && r.is_some() {
            horizontal_area = Some((l.unwrap(), r.unwrap()));
        }
    }

    if horizontal_area.is_none() {
        return None;
    }
    // now we know matrix x coordinates between `l..r`
    let (l, r) = horizontal_area.unwrap();

    let mut vertical_area: Option<(u32, u32)> = None;
    {
        // tries to find second and third horizontal lines in center from bottom
        let line_width = 150;

        let mut t: Option<u32> = None;
        let mut b: Option<u32> = None;
        let mut lines = 0;
        let center = (r - l) / 2 + l;
        let mut y = img.get_height() - 1;
        while y > 0 {
            let horizontal_line = (0..line_width).all(|v| img.get_pixel(center - v, y) != bmp::consts::WHITE || img.get_pixel(center - v, y - 1) != bmp::consts::WHITE);
            if horizontal_line {
                y -= 1;

                lines += 1;
                if lines == 2 {
                    b = Some(y);
                } else if lines == 3 {
                    t = Some(y);
                }
            }
            if lines >= 3 {
                break;
            }

            y -= 1;
        }
        if b.is_some() && t.is_some() {
            vertical_area = Some((t.unwrap(), b.unwrap()));
        }
    }
    if let Some((t, b)) = vertical_area {
        return Some((l + 1, t + 2, r - 1, b - 2));
    }
    return None;
}

pub fn find_condition_area(img: &Image, matrix_area: &(u32, u32, u32, u32)) -> Option<(u32, u32, u32, u32)> {
    let mut img = crop_img(&img, matrix_area.2, matrix_area.1, img.get_width(), matrix_area.3);
    filter_img(&mut img, &CONDITION_BORDER_COLOR, 15);

    let mut bottom: Option<u32> = None;
    {
        // tries to find first horizontal line in center from bottom
        let line_width = 50;

        let center = img.get_width() / 2;
        for y in (0..img.get_height()).rev() {
            let horizontal_line = (0..line_width).all(|v| img.get_pixel(center - v, y) != bmp::consts::WHITE);
            if horizontal_line {
                bottom = Some(y);
                break;
            }
        }
    }
    if bottom.is_none() {
        return None;
    }

    let mut horizontal_area: Option<(u32, u32)> = None;
    {
        // tries to find first and second vertical lines in top from left
        let line_height = 50;

        let mut l: Option<u32> = None;
        let mut r: Option<u32> = None;
        let mut lines = 0;
        let top = 0;
        for x in 0..img.get_width() {
            let vertical_line = (0..line_height).all(|v| img.get_pixel(x, top + v) != bmp::consts::WHITE);
            if vertical_line {
                lines += 1;
                if lines == 1 {
                    l = Some(x);
                } else if lines == 2 {
                    r = Some(x);
                }
            }
            if lines >= 2 {
                break;
            }
        }
        if l.is_some() && r.is_some() {
            horizontal_area = Some((l.unwrap(), r.unwrap()));
        }
    }

    if let Some((l, r)) = horizontal_area {
        return Some((matrix_area.2 + l + 1, matrix_area.1, matrix_area.2 + (l + r) / 2, matrix_area.1 + bottom.unwrap() - 1));
    }

    return None;
}

pub fn find_blocks_count(img: &Image, condition_area: &(u32, u32, u32, u32)) -> Option<usize> {
    let mut img = crop_img(&img, condition_area.0, condition_area.1 / 2, img.get_width(), condition_area.1);
    filter_img(&mut img, &BLOCK_COLOR, 5);

    // tries to find first vertical line
    let line_height = 8;

    let mut start: Option<(u32, u32)> = None;
    for x in 0..img.get_width() {
        for y in 0..(img.get_height() - line_height) {
            let vertical_line = (0..line_height).all(|v| img.get_pixel(x, y + v) != bmp::consts::WHITE);
            if vertical_line {
                start = Some((x, y));
                break;
            }
        }
        if start.is_some() {
            break;
        }
    }

    if let Some((start_x, y)) = start {
        let block_width = ((start_x + 1)..img.get_width()).take_while(|x| img.get_pixel(*x, y) == bmp::consts::WHITE).count() + 1;
        let blocks = (start_x..img.get_width()).step_by(block_width).take_while(|x| img.get_pixel(*x, y) != bmp::consts::WHITE).count();
        return Some(blocks);
    }

    return None;
}


#[cfg(test)]
mod tests {
    use crate::img::load_img_from_file;
    use crate::recognize::{find_blocks_count, find_condition_area, find_matrix_area};

    const MATRIX_AREA1: (u32, u32, u32, u32) = (142, 337, 837, 802);
    const CONDITION_AREA1: (u32, u32, u32, u32) = (885, 337, 1346, 567);

    const MATRIX_AREA2: (u32, u32, u32, u32) = (206, 337, 773, 673);
    const CONDITION_AREA2: (u32, u32, u32, u32) = (821, 337, 1282, 567);

    #[test]
    fn test_find_matrix_area1() {
        let img = load_img_from_file("test/test1.bmp");
        let area = find_matrix_area(&img);
        assert_eq!(Some(MATRIX_AREA1), area);
    }

    #[test]
    fn test_find_matrix_area2() {
        let img = load_img_from_file("test/test2.bmp");
        let area = find_matrix_area(&img);
        assert_eq!(Some(MATRIX_AREA2), area);
    }

    #[test]
    fn test_find_matrix_area_not_found() {
        let img = load_img_from_file("test/ocr_simple_test.bmp");
        let area = find_matrix_area(&img);
        assert_eq!(None, area);
    }

    #[test]
    fn test_find_condition_area1() {
        let img = load_img_from_file("test/test1.bmp");
        let area = find_condition_area(&img, &MATRIX_AREA1);
        assert_eq!(Some(CONDITION_AREA1), area);
    }

    #[test]
    fn test_find_condition_area2() {
        let img = load_img_from_file("test/test2.bmp");
        let area = find_condition_area(&img, &MATRIX_AREA2);
        assert_eq!(Some(CONDITION_AREA2), area);
    }

    #[test]
    fn test_find_condition_area_not_found() {
        let img = load_img_from_file("test/ocr_simple_test.bmp");
        let area = find_condition_area(&img, &(0, 0, 0, 0));
        assert_eq!(None, area);
    }

    #[test]
    fn find_blocks_count1() {
        let img = load_img_from_file("test/test1.bmp");
        let count = find_blocks_count(&img, &CONDITION_AREA1);
        assert_eq!(Some(6), count);
    }

    #[test]
    fn find_blocks_count2() {
        let img = load_img_from_file("test/test2.bmp");
        let count = find_blocks_count(&img, &CONDITION_AREA2);
        assert_eq!(Some(6), count);
    }
}