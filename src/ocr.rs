use bmp::{Pixel, px};

use crate::img::GrayImage;

// max space interval in px between 2 characters in same matrix item
const MAX_CHARACTER_SPACING: u32 = 15;

#[allow(non_snake_case)]
pub struct MatrixTemplates {
    T_1C: GrayImage,
    T_55: GrayImage,
    T_7A: GrayImage,
    T_BD: GrayImage,
    T_E9: GrayImage,
    T_FF: GrayImage,
}

impl MatrixTemplates {
    pub fn load_templates() -> Self {
        let bytes = include_bytes!("template.bmp");
        let bmp = bmp::from_reader(&mut bytes.as_ref()).unwrap();

        let color = px!(255, 255, 255);
        return Self {
            T_1C: GrayImage::filter(&bmp, &color, 1, 000, 0, 023, 19),
            T_55: GrayImage::filter(&bmp, &color, 1, 023, 0, 049, 20),
            T_7A: GrayImage::filter(&bmp, &color, 1, 049, 0, 075, 19),
            T_BD: GrayImage::filter(&bmp, &color, 1, 075, 0, 104, 20),
            T_E9: GrayImage::filter(&bmp, &color, 1, 104, 0, 130, 20),
            T_FF: GrayImage::filter(&bmp, &color, 1, 130, 0, 155, 20),
        };
    }
}

struct Location {
    start: u32,
    end: u32,
}

/// Returns 2 vectors:
/// - columns (`x`)
/// - rows (`y`)
fn locate_matrix_regions(img: &GrayImage) -> Result<(Vec<Location>, Vec<Location>), String> {
    // will think that usual matrix/conditions is not greater than 10x10.
    // matrix_table_* stores x/y coordinates in format (start1, end1, start2, end2, ...)
    let mut matrix_table_x: Vec<u32> = Vec::with_capacity(10 * 2);
    let mut matrix_table_y: Vec<u32> = Vec::with_capacity(10 * 2);

    let mut was_space = true;
    for (x, non_blank) in img.columns_usage().into_iter().enumerate() {
        if non_blank {
            if was_space {
                // new character

                // each item consists of 2 characters
                let same_item = if let Some(&prev) = matrix_table_x.last() {
                    x as u32 - prev <= MAX_CHARACTER_SPACING
                } else {
                    false
                };
                if same_item {
                    // item is continued. wait for new ending
                    matrix_table_x.pop();
                } else {
                    // new item
                    matrix_table_x.push(x as _);
                }
            } else {
                // do nothing. same character
            }
            was_space = false;
        } else {
            if !was_space {
                // end of character
                matrix_table_x.push(x as _);
            }
            was_space = true;
        }
    }

    let mut was_space = true;
    for (y, non_blank) in img.rows_usage().into_iter().enumerate() {
        if non_blank {
            if was_space {
                // new character
                matrix_table_y.push(y as _);
            } else {
                // do nothing. same character
            }
            was_space = false;
        } else {
            if !was_space {
                // end of character
                matrix_table_y.push(y as _);
            }
            was_space = true;
        }
    }

    if matrix_table_x.is_empty() || matrix_table_y.is_empty() {
        return Err("Matrix items not found".to_owned());
    }
    if matrix_table_x.len() % 2 != 0 || matrix_table_y.len() % 2 != 0 {
        return Err("Matrix items were recognized incorrectly".to_owned());
    }

    let columns = (0..(matrix_table_x.len() / 2))
        .map(|i| Location { start: matrix_table_x[i * 2], end: matrix_table_x[i * 2 + 1] })
        .collect();
    let rows = (0..(matrix_table_y.len() / 2))
        .map(|i| Location { start: matrix_table_y[i * 2], end: matrix_table_y[i * 2 + 1] })
        .collect();
    return Ok((columns, rows));
}

fn ocr_matrix_item(img: &GrayImage, templates: &MatrixTemplates, column: &Location, row: &Location) -> Option<u8> {

    let (x_start, y_start, x_end, y_end) = match img.rect_hull(column.start, row.start, column.end - 1, row.end - 1) {
        Some(v) => v,
        None => return None,
    };

    return Some([
        (0x1Cu8, img.template_match_error_score(x_start, y_start, x_end, y_end, &templates.T_1C)),
        (0x55u8, img.template_match_error_score(x_start, y_start, x_end, y_end, &templates.T_55)),
        (0x7Au8, img.template_match_error_score(x_start, y_start, x_end, y_end, &templates.T_7A)),
        (0xBDu8, img.template_match_error_score(x_start, y_start, x_end, y_end, &templates.T_BD)),
        (0xE9u8, img.template_match_error_score(x_start, y_start, x_end, y_end, &templates.T_E9)),
        (0xFFu8, img.template_match_error_score(x_start, y_start, x_end, y_end, &templates.T_FF)),
    ].into_iter()
        .min_by(|(_, error1), (_, error2)| error1.partial_cmp(error2).unwrap())
        .unwrap().0);
}

pub fn ocr_matrix(img: &GrayImage, templates: &MatrixTemplates) -> Result<(u32, u32, u32, u32, Vec<Vec<u8>>), String> {

    let (columns, rows) = match locate_matrix_regions(&img) {
        Ok(x) => x,
        Err(e) => return Err(e),
    };

    if columns.len() < 3 || columns.len() != rows.len() {
        return Err("Bad matrix dimension".to_owned());
    }

    let matrix_dimension = columns.len();
    let left = columns.first().unwrap().start;
    let right = columns.last().unwrap().start;
    let top = rows.first().unwrap().start;
    let bottom = rows.last().unwrap().start;
    let mut result: Vec<Vec<u8>> = Vec::with_capacity(matrix_dimension);

    for row in rows.iter() {
        let mut matrix_row = Vec::with_capacity(matrix_dimension);
        for column in columns.iter() {
            let matrix_item = match ocr_matrix_item(&img, &templates, &column, &row) {
                Some(v) => v,
                None => return Err("One of matrix items was not recognized".to_owned()),
            };
            matrix_row.push(matrix_item);
        }
        result.push(matrix_row);
    }

    debug_assert_eq!(matrix_dimension, result.len());
    debug_assert_eq!(matrix_dimension, result[0].len());
    debug_assert_eq!(matrix_dimension, result[1].len());
    return Ok((left, top, right, bottom, result));
}

pub fn ocr_conditions(img: &GrayImage, templates: &MatrixTemplates) -> Result<Vec<Vec<u8>>, String> {

    let (columns, rows) = match locate_matrix_regions(&img) {
        Ok(x) => x,
        Err(e) => return Err(e),
    };

    if columns.is_empty() || rows.is_empty() {
        return Err("Bad condition dimension".to_owned());
    }

    let mut result: Vec<Vec<u8>> = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        let mut condition_row = Vec::with_capacity(columns.len());
        for column in columns.iter() {
            let matrix_item = match ocr_matrix_item(&img, &templates, &column, &row) {
                Some(v) => v,
                None => break, // short condition. Goto next row
            };
            condition_row.push(matrix_item);
        }
        result.push(condition_row);
    }

    debug_assert_eq!(rows.len(), result.len());
    return Ok(result);
}


#[cfg(test)]
mod tests {
    use crate::img::{GrayImage, load_img_from_file};
    use crate::ocr::{MatrixTemplates, ocr_conditions, ocr_matrix};
    use crate::recognize::{CONDITION_COLOR, MATRIX_COLOR};
    use crate::test_cases::{CONDITION_AREA1, CONDITION_AREA2, CONDITION_AREA3, CONDITION_AREA4, CONDITION_AREA5, conditions1, conditions2, conditions3, conditions4, conditions5, FILE1, FILE2, FILE3, FILE4, FILE5, matrix1, matrix2, matrix3, matrix4, matrix5, MATRIX_AREA1, MATRIX_AREA2, MATRIX_AREA3, MATRIX_AREA4, MATRIX_AREA5};

    #[test]
    fn test_ocr_matrix1() {
        test_ocr_matrix(FILE1, MATRIX_AREA1, (143, 29, 526, 414, matrix1()));
    }

    #[test]
    fn test_ocr_matrix2() {
        test_ocr_matrix(FILE2, MATRIX_AREA2, (142, 30, 400, 286, matrix2()));
    }

    #[test]
    fn test_ocr_matrix3() {
        test_ocr_matrix(FILE3, MATRIX_AREA3, (142, 29, 528, 414, matrix3()));
    }

    #[test]
    fn test_ocr_matrix4() {
        test_ocr_matrix(FILE4, MATRIX_AREA4, (142, 29, 526, 414, matrix4()));
    }

    #[test]
    fn test_ocr_matrix5() {
        test_ocr_matrix(FILE5, MATRIX_AREA5, (142, 29, 526, 414, matrix5()));
    }

    fn test_ocr_matrix(filename: &str, matrix_area: (u32, u32, u32, u32), expected: (u32, u32, u32, u32, Vec<Vec<u8>>)) {
        let templates = MatrixTemplates::load_templates();
        let img = load_img_from_file(filename);
        let img = GrayImage::filter(&img, &MATRIX_COLOR, 50, matrix_area.0, matrix_area.1, matrix_area.2, matrix_area.3);

        let result = ocr_matrix(&img, &templates);
        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn test_ocr_conditions1() {
        test_ocr_conditions(FILE1, CONDITION_AREA1, conditions1());
    }

    #[test]
    fn test_ocr_conditions2() {
        test_ocr_conditions(FILE2, CONDITION_AREA2, conditions2());
    }

    #[test]
    fn test_ocr_conditions3() {
        test_ocr_conditions(FILE3, CONDITION_AREA3, conditions3());
    }

    #[test]
    fn test_ocr_conditions4() {
        test_ocr_conditions(FILE4, CONDITION_AREA4, conditions4());
    }

    #[test]
    fn test_ocr_conditions5() {
        test_ocr_conditions(FILE5, CONDITION_AREA5, conditions5());
    }

    fn test_ocr_conditions(filename: &str, condition_area: (u32, u32, u32, u32), expected: Vec<Vec<u8>>) {
        let templates = MatrixTemplates::load_templates();
        let img = load_img_from_file(filename);
        let img = GrayImage::filter(&img, &CONDITION_COLOR, 50, condition_area.0, condition_area.1, condition_area.2, condition_area.3);

        let result = ocr_conditions(&img, &templates);
        assert_eq!(Ok(expected), result);
    }
}