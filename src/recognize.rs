use bmp::{Image, Pixel, px};

use crate::img::GrayImage;

pub const MATRIX_COLOR: Pixel = px!(0xD0, 0xED, 0x57);

const CONDITION_BORDER_COLOR: Pixel = px!(0x81, 0x96, 0x38);
pub const CONDITION_COLOR: Pixel = px!(0xF0, 0xF0, 0xF0);

const BUFFER_COLOR: Pixel = px!(0x4F, 0x5A, 0x25);

pub fn find_matrix_area(img: &Image) -> Option<(u32, u32, u32, u32)> {
    // matrix is on left part of image
    let img = GrayImage::filter(img, &MATRIX_COLOR, 50, 0, 0, img.get_width() / 2, img.get_height());

    // ██████████████████
    // █ matrix caption █
    // ██████████████████
    // │ matrix content │
    // └────────────────┘

    // tries to find the most bottom-right rectangle
    let rect_width = 300;
    let rect_height = 5;
    let (x_right, y_top) = match img.rfind_rect(rect_width, rect_height) {
        Some((x_start, y_start)) => (x_start + rect_width - 1, y_start + rect_height - 1),
        None => return None,
    };

    // right-bottom corner of matrix caption. Matrix content is below
    // ███████████████│
    // ───────────────┤  <- (x_right, y_top) is here
    // matrix content │
    debug_assert_eq!(img.pixel(x_right - 2, y_top + 0), 255);
    debug_assert_eq!(img.pixel(x_right - 1, y_top + 0), 255);
    debug_assert_eq!(img.pixel(x_right + 0, y_top + 0), 255);
    debug_assert_eq!(img.pixel(x_right + 1, y_top + 0), 0);
    debug_assert_eq!(img.pixel(x_right + 2, y_top + 0), 0);
    debug_assert_eq!(img.pixel(x_right - 2, y_top + 1), 0);
    debug_assert_eq!(img.pixel(x_right - 1, y_top + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 0, y_top + 1), 255);
    debug_assert_eq!(img.pixel(x_right + 1, y_top + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 2, y_top + 1), 0);

    let y_bottom = match (100..(img.height() - y_top - 1)).find(|&dy| img.pixel(x_right, y_top + dy + 1) == 0) {
        Some(height) => y_top + height,
        None => return None,
    };

    // matrix content │
    // ───────────────┘ <- (x_right, y_bottom) is here
    debug_assert_eq!(img.pixel(x_right - 2, y_bottom + 0), 255);
    debug_assert_eq!(img.pixel(x_right - 1, y_bottom + 0), 255);
    debug_assert_eq!(img.pixel(x_right + 0, y_bottom + 0), 255);
    debug_assert_eq!(img.pixel(x_right + 1, y_bottom + 0), 0);
    debug_assert_eq!(img.pixel(x_right + 2, y_bottom + 0), 0);
    debug_assert_eq!(img.pixel(x_right - 2, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right - 1, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 0, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 1, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 2, y_bottom + 1), 0);

    let x_left = match (300..=x_right).find(|&dx| img.pixel(x_right - dx, y_bottom - 1) != 0) {
        Some(width) => x_right - width,
        None => return None,
    };
    return Some((x_left + 1, y_top + 1, x_right - 1, y_bottom - 1));
}

pub fn find_condition_area(img: &Image, matrix_area: &(u32, u32, u32, u32)) -> Option<(u32, u32, u32, u32)> {
    // conditions are near matrix
    let (_, matrix_top, matrix_right, matrix_bottom) = *matrix_area;

    let img = GrayImage::filter(img, &CONDITION_BORDER_COLOR, 30, matrix_right, matrix_top, img.get_width(), matrix_bottom);

    // │ condition content    descriptions │
    // └───────────────────────────────────┘

    // tries to find the most bottom-right horizontal line
    let rect_width = 300;
    let rect_height = 1;
    let (x_right, y_bottom) = match img.rfind_rect(rect_width, rect_height) {
        Some((x_start, y_start)) => (x_start + rect_width - 1, y_start + rect_height - 1),
        None => return None,
    };

    // │ condition content    descriptions │
    // └───────────────────────────────────┘ <- (x_right, y_bottom) is here
    debug_assert_eq!(img.pixel(x_right - 2, y_bottom + 0), 255);
    debug_assert_eq!(img.pixel(x_right - 1, y_bottom + 0), 255);
    debug_assert_eq!(img.pixel(x_right + 0, y_bottom + 0), 255);
    debug_assert_eq!(img.pixel(x_right + 1, y_bottom + 0), 0);
    debug_assert_eq!(img.pixel(x_right + 2, y_bottom + 0), 0);
    debug_assert_eq!(img.pixel(x_right - 2, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right - 1, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 0, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 1, y_bottom + 1), 0);
    debug_assert_eq!(img.pixel(x_right + 2, y_bottom + 1), 0);

    let x_left = match (300..x_right).find(|dx| img.pixel(x_right - dx - 1, y_bottom) == 0) {
        Some(width) => x_right - width,
        None => return None,
    };

    // │ condition content    descriptions │
    // └───────────────────────────────────┘ <- (x_right, y_bottom) is here
    // ^
    // (x_left, y_bottom) is here
    // need to filter condition descriptions. Searching for description images

    let width = img.rect_hull(x_left + 1, 0, x_right - 1, y_bottom - 1)
        .map(|(desc_start_x, _, _, _)| desc_start_x)
        .unwrap_or(x_right) - x_left;

    return Some((matrix_right + x_left + 1, matrix_top, matrix_right + width - 1, matrix_top + y_bottom - 1));
}

pub fn find_buffer_size(img: &Image, condition_area: &(u32, u32, u32, u32)) -> Option<usize> {
    let (condition_left, condition_top, condition_right, _) = *condition_area;

    let img = GrayImage::filter(img, &BUFFER_COLOR, 30, condition_left, condition_top / 2, condition_right, 3 * condition_top / 4);

    // ───────────────────┐
    //  ┌ ─ ┐ ┌ ─ ┐ ┌ ─ ┐ │
    //                    │
    //  │   │ │   │ │   │ │
    //                    │
    //  └ ─ ┘ └ ─ ┘ └ ─ ┘ │
    // ───────────────────┘

    // tries to find right border
    let rect_width = 1;
    let rect_height = 35;
    let (x_right, y_bottom) = match img.rfind_rect(rect_width, rect_height) {
        Some((x_start, y_start)) => (x_start + rect_width - 1, y_start + rect_height - 1),
        None => return None,
    };

    let height = match (30..y_bottom).find(|dy| img.pixel(x_right, y_bottom - dy - 1) == 0) {
        Some(height) => height,
        None => return None,
    };

    let y = y_bottom - height / 2;
    debug_assert_eq!(img.pixel(x_right + 2, y), 0);
    debug_assert_eq!(img.pixel(x_right + 1, y), 0);
    debug_assert_eq!(img.pixel(x_right + 0, y), 255);
    debug_assert_eq!(img.pixel(x_right - 1, y), 0);
    debug_assert_eq!(img.pixel(x_right - 2, y), 0);

    let count = (0..x_right).filter(|&x| img.pixel(x, y) != 0).count() / 2;
    return Some(count);
}


#[cfg(test)]
mod tests {
    use bmp::Image;

    use crate::img::load_img_from_file;
    use crate::recognize::{find_buffer_size, find_condition_area, find_matrix_area};
    use crate::test_cases::{BUFFER_SIZE1, BUFFER_SIZE2, BUFFER_SIZE3, BUFFER_SIZE4, BUFFER_SIZE5, CONDITION_AREA1, CONDITION_AREA2, CONDITION_AREA3, CONDITION_AREA4, CONDITION_AREA5, FILE1, FILE2, FILE3, FILE4, FILE5, MATRIX_AREA1, MATRIX_AREA2, MATRIX_AREA3, MATRIX_AREA4, MATRIX_AREA5};

    #[test]
    fn test_find_matrix_area1() {
        test_find_matrix_area(FILE1, MATRIX_AREA1);
    }

    #[test]
    fn test_find_matrix_area2() {
        test_find_matrix_area(FILE2, MATRIX_AREA2);
    }

    #[test]
    fn test_find_matrix_area3() {
        test_find_matrix_area(FILE3, MATRIX_AREA3);
    }

    #[test]
    fn test_find_matrix_area4() {
        test_find_matrix_area(FILE4, MATRIX_AREA4);
    }

    #[test]
    fn test_find_matrix_area5() {
        test_find_matrix_area(FILE5, MATRIX_AREA5);
    }

    fn test_find_matrix_area(filename: &str, expected: (u32, u32, u32, u32)) {
        let img = load_img_from_file(filename);
        let actual = find_matrix_area(&img);
        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn test_find_matrix_area_not_found() {
        let img = Image::new(2, 2);
        let area = find_matrix_area(&img);
        assert_eq!(None, area);
    }

    #[test]
    fn test_find_condition_area1() {
        test_find_condition_area(FILE1, &MATRIX_AREA1, CONDITION_AREA1);
    }

    #[test]
    fn test_find_condition_area2() {
        test_find_condition_area(FILE2, &MATRIX_AREA2, CONDITION_AREA2);
    }

    #[test]
    fn test_find_condition_area3() {
        test_find_condition_area(FILE3, &MATRIX_AREA3, CONDITION_AREA3);
    }

    #[test]
    fn test_find_condition_area4() {
        test_find_condition_area(FILE4, &MATRIX_AREA4, CONDITION_AREA4);
    }

    #[test]
    fn test_find_condition_area5() {
        test_find_condition_area(FILE5, &MATRIX_AREA5, CONDITION_AREA5);
    }

    fn test_find_condition_area(filename: &str, matrix_area: &(u32, u32, u32, u32), expected: (u32, u32, u32, u32)) {
        let img = load_img_from_file(filename);
        let actual = find_condition_area(&img, matrix_area);
        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn test_find_condition_area_not_found() {
        let img = Image::new(2, 2);
        let area = find_condition_area(&img, &(0, 0, 1, 1));
        assert_eq!(None, area);
    }

    #[test]
    fn test_find_buffer_size1() {
        test_find_buffer_size(FILE1, &CONDITION_AREA1, BUFFER_SIZE1);
    }

    #[test]
    fn test_find_buffer_size2() {
        test_find_buffer_size(FILE2, &CONDITION_AREA2, BUFFER_SIZE2);
    }

    #[test]
    fn test_find_buffer_size3() {
        test_find_buffer_size(FILE3, &CONDITION_AREA3, BUFFER_SIZE3);
    }

    #[test]
    fn test_find_buffer_size4() {
        test_find_buffer_size(FILE4, &CONDITION_AREA4, BUFFER_SIZE4);
    }

    #[test]
    fn test_find_buffer_size5() {
        test_find_buffer_size(FILE5, &CONDITION_AREA5, BUFFER_SIZE5);
    }

    fn test_find_buffer_size(filename: &str, condition_area: &(u32, u32, u32, u32), expected: usize) {
        let img = load_img_from_file(filename);
        let actual = find_buffer_size(&img, condition_area);
        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn test_find_buffer_size_not_found() {
        let img = Image::new(6, 6);
        let count = find_buffer_size(&img, &(4, 4, 6, 6));
        assert_eq!(None, count);
    }
}