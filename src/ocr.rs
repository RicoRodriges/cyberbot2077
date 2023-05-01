use std::process::{Command, Stdio};

use bmp::Image;

// TODO: migrate to dll or `tesseract` crate
const TESSERACT_PATH: &str = "tesseract/tesseract.exe";

const ALL_MATRIX_ITEMS: [&str; 6] = ["1C", "55", "7A", "BD", "E9", "FF"];
const WHITELIST: &str = "1579ABCDEF ";

fn ocr(img: &Image, conf: &[(&str, &str)]) -> Result<String, String> {
    let m = conf.iter().flat_map(|v| ["-c".to_owned(), format!("{}={}", v.0, v.1)]);
    let mut child = Command::new(TESSERACT_PATH)
        .args(["stdin", "stdout", /*"-l", "eng", "--tessdata-dir", "/",*/ "--dpi", "72"])
        .args(m)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().map_err(|e| e.to_string())?;

    let mut child_stdin = child.stdin.take().unwrap();
    img.to_writer(&mut child_stdin).map_err(|e| e.to_string())?;
    // Close stdin
    drop(child_stdin);

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        return Ok(std::str::from_utf8(&output.stdout).map_err(|e| e.to_string())?.trim().to_owned());
    }
    return Ok("".to_owned());
}

pub fn ocr_text(img: &Image) -> Result<String, String> {
    return ocr(&img, &[("tessedit_char_whitelist", WHITELIST), ("tessedit_create_txt", "1")]);
}

pub fn ocr_tsv(img: &Image) -> Result<Vec<(u32, u32, u32, u32, String)>, String> {

    let str = ocr(&img, &[("tessedit_char_whitelist", WHITELIST), ("tessedit_create_tsv", "1")])?;

    let mut result = Vec::new();
    for x in str.lines().skip(1) {
        let mut line = x.split("\t").skip(6);
        let x = u32::from_str_radix(line.next().unwrap(), 10).unwrap();
        let y = u32::from_str_radix(line.next().unwrap(), 10).unwrap();
        let w = u32::from_str_radix(line.next().unwrap(), 10).unwrap();
        let h = u32::from_str_radix(line.next().unwrap(), 10).unwrap();
        line.next();
        let str = line.next().unwrap().trim();
        if !str.is_empty() {
            result.push((x, y, w, h, str.to_owned()));
        }
    }
    return Ok(result);
}

pub fn ocr_matrix(img: &Image) -> Result<(u32, u32, u32, u32, Vec<Vec<u8>>), String> {
    let ocr = ocr_tsv(&img)?;

    let mut left = u32::MAX;
    let mut top = u32::MAX;
    let mut right = u32::MIN;
    let mut bottom = u32::MIN;
    for (x, y, _, _, _) in ocr.iter() {
        left = u32::min(left, *x);
        right = u32::max(right, *x);
        top = u32::min(top, *y);
        bottom = u32::max(bottom, *y);
    }

    let line_offset = 30;

    let mut result: Vec<Vec<String>> = Vec::new();
    let mut current_line = top as i32;
    loop {
        let mut line: Vec<_> = ocr.iter()
            .filter(|v| (v.1 as i32 - current_line).abs() <= line_offset)
            .collect();
        if line.is_empty() {
            break;
        }

        line.sort_by_key(|v| v.0);
        result.push(line.iter().map(|v| v.4.to_owned()).collect());

        current_line = ocr.iter()
            .map(|v| v.1 as i32)
            .filter(|v| (v - current_line) > line_offset)
            .min().unwrap_or(i32::MAX / 2);
    }

    // Corrections
    // Convert "1" and "C" to "1C", ...
    for v in result.iter_mut() {
        for hex in v.iter_mut() {
            if hex.len() != 2 {
                let candidate = ALL_MATRIX_ITEMS.iter().find(|h| h.contains(&hex[..]));
                if let Some(found) = candidate {
                    *hex = (*found).to_owned();
                } else {
                    return Err(format!("'{}' is not a hex value", hex));
                }
            } else if ALL_MATRIX_ITEMS.iter().all(|h| *h != hex) {
                return Err(format!("'{}' is not expected hex value", hex));
            }
        }
    }

    let result = result.iter()
        .map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect())
        .collect();
    return Ok((left, top, right, bottom, result));
}


#[cfg(test)]
mod tests {
    use crate::img::{crop_img, filter_img, load_img_from_file};
    use crate::ocr::{ocr_matrix, ocr_text, ocr_tsv};
    use crate::recognize::{CONDITION_COLOR, MATRIX_COLOR};

    #[test]
    fn test_ocr_text() {
        let img = load_img_from_file("test/ocr_simple_test.bmp");

        let string = ocr_text(&img).unwrap();
        assert_eq!("BD 55\r\nE9", string);
    }

    #[test]
    fn test_ocr_tsv() {
        let img = load_img_from_file("test/ocr_simple_test.bmp");

        let result = ocr_tsv(&img).unwrap();
        assert_eq!(vec![
            (5, 4, 40, 20, "BD".to_owned()),
            (69, 4, 34, 19, "55".to_owned()),
            (64, 31, 34, 20, "E9".to_owned()),
        ], result);
    }

    const MATRIX_AREA1: (u32, u32, u32, u32) = (142, 337, 837, 802);
    const MATRIX_AREA2: (u32, u32, u32, u32) = (206, 337, 773, 673);

    #[test]
    fn test_ocr_matrix1() {
        let img = load_img_from_file("test/test1.bmp");
        let mut img = crop_img(&img, MATRIX_AREA1.0, MATRIX_AREA1.1, MATRIX_AREA1.2, MATRIX_AREA1.3);
        filter_img(&mut img, &MATRIX_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["BD", "55", "55", "7A", "E9", "7A", "55"],
            vec!["55", "E9", "55", "BD", "55", "55", "E9"],
            vec!["E9", "55", "BD", "7A", "1C", "55", "7A"],
            vec!["55", "1C", "55", "55", "7A", "1C", "FF"],
            vec!["1C", "7A", "7A", "1C", "BD", "1C", "BD"],
            vec!["1C", "7A", "E9", "FF", "1C", "E9", "FF"],
            vec!["1C", "7A", "7A", "BD", "7A", "55", "BD"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((143, 29, 529, 415, expected)), result);
    }

    #[test]
    fn test_ocr_matrix2() {
        let img = load_img_from_file("test/test2.bmp");
        let mut img = crop_img(&img, MATRIX_AREA2.0, MATRIX_AREA2.1, MATRIX_AREA2.2, MATRIX_AREA2.3);
        filter_img(&mut img, &MATRIX_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["55", "BD", "BD", "BD", "55"],
            vec!["BD", "1C", "55", "E9", "1C"],
            vec!["BD", "BD", "1C", "1C", "55"],
            vec!["55", "E9", "E9", "55", "E9"],
            vec!["1C", "55", "55", "1C", "1C"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((142, 30, 401, 286, expected)), result);
    }

    #[test]
    fn test_ocr_matrix3() {
        let img = load_img_from_file("test/test3.bmp");
        let mut img = crop_img(&img, MATRIX_AREA1.0, MATRIX_AREA1.1, MATRIX_AREA1.2, MATRIX_AREA1.3);
        filter_img(&mut img, &MATRIX_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["7A", "55", "E9", "BD", "1C", "55", "55"],
            vec!["FF", "55", "55", "BD", "E9", "7A", "1C"],
            vec!["55", "1C", "55", "55", "1C", "55", "E9"],
            vec!["E9", "1C", "FF", "55", "1C", "1C", "55"],
            vec!["BD", "1C", "1C", "1C", "E9", "55", "1C"],
            vec!["1C", "FF", "55", "7A", "BD", "55", "1C"],
            vec!["55", "55", "7A", "BD", "7A", "55", "55"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((142, 29, 529, 415, expected)), result);
    }

    #[test]
    fn test_ocr_matrix4() {
        let img = load_img_from_file("test/test4.bmp");
        let mut img = crop_img(&img, MATRIX_AREA1.0, MATRIX_AREA1.1, MATRIX_AREA1.2, MATRIX_AREA1.3);
        filter_img(&mut img, &MATRIX_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["7A", "55", "FF", "BD", "BD", "BD", "E9"],
            vec!["BD", "FF", "FF", "55", "E9", "E9", "7A"],
            vec!["55", "7A", "7A", "BD", "E9", "FF", "BD"],
            vec!["7A", "1C", "FF", "FF", "7A", "1C", "1C"],
            vec!["7A", "BD", "55", "55", "E9", "55", "55"],
            vec!["7A", "E9", "55", "BD", "BD", "1C", "1C"],
            vec!["55", "7A", "7A", "7A", "7A", "7A", "1C"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((142, 29, 529, 415, expected)), result);
    }

    #[test]
    fn test_ocr_matrix5() {
        let img = load_img_from_file("test/test5.bmp");
        let mut img = crop_img(&img, MATRIX_AREA1.0, MATRIX_AREA1.1, MATRIX_AREA1.2, MATRIX_AREA1.3);
        filter_img(&mut img, &MATRIX_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["7A", "FF", "E9", "55", "7A", "7A", "7A"],
            vec!["7A", "E9", "1C", "55", "FF", "55", "1C"],
            vec!["7A", "1C", "7A", "E9", "7A", "7A", "55"],
            vec!["7A", "55", "55", "BD", "1C", "55", "1C"],
            vec!["BD", "7A", "E9", "E9", "1C", "FF", "55"],
            vec!["55", "FF", "BD", "BD", "1C", "7A", "1C"],
            vec!["55", "1C", "BD", "BD", "7A", "FF", "BD"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((142, 29, 529, 415, expected)), result);
    }

    const CONDITION_AREA1: (u32, u32, u32, u32) = (885, 337, 1346, 567);
    const CONDITION_AREA2: (u32, u32, u32, u32) = (821, 337, 1282, 567);

    #[test]
    fn test_ocr_conditions1() {
        let img = load_img_from_file("test/test1.bmp");
        let mut img = crop_img(&img, CONDITION_AREA1.0, CONDITION_AREA1.1, CONDITION_AREA1.2, CONDITION_AREA1.3);
        filter_img(&mut img, &CONDITION_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["1C", "7A"],
            vec!["7A", "1C", "1C"],
            vec!["7A", "7A", "BD", "7A"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((25, 12, 151, 147, expected)), result);
    }

    #[test]
    fn test_ocr_conditions2() {
        let img = load_img_from_file("test/test2.bmp");
        let mut img = crop_img(&img, CONDITION_AREA2.0, CONDITION_AREA2.1, CONDITION_AREA2.2, CONDITION_AREA2.3);
        filter_img(&mut img, &CONDITION_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["1C", "55"],
            vec!["BD", "55"],
            vec!["55", "55", "1C"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((24, 11, 110, 147, expected)), result);
    }

    #[test]
    fn test_ocr_conditions3() {
        let img = load_img_from_file("test/test3.bmp");
        let mut img = crop_img(&img, CONDITION_AREA1.0, CONDITION_AREA1.1, CONDITION_AREA1.2, CONDITION_AREA1.3);
        filter_img(&mut img, &CONDITION_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["1C", "E9", "BD"],
            vec!["1C", "55", "1C"],
            vec!["7A", "E9", "1C"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((25, 12, 110, 147, expected)), result);
    }

    #[test]
    fn test_ocr_conditions4() {
        let img = load_img_from_file("test/test4.bmp");
        let mut img = crop_img(&img, CONDITION_AREA1.0, CONDITION_AREA1.1, CONDITION_AREA1.2, CONDITION_AREA1.3);
        filter_img(&mut img, &CONDITION_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["FF", "7A"],
            vec!["1C", "7A", "BD"],
            vec!["7A", "7A", "55", "1C"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((25, 12, 152, 147, expected)), result);
    }

    #[test]
    fn test_ocr_conditions5() {
        let img = load_img_from_file("test/test5.bmp");
        let mut img = crop_img(&img, CONDITION_AREA1.0, CONDITION_AREA1.1, CONDITION_AREA1.2, CONDITION_AREA1.3);
        filter_img(&mut img, &CONDITION_COLOR, 50);

        let result = ocr_matrix(&img);
        let expected: Vec<Vec<u8>> = vec![
            vec!["7A", "E9", "FF"],
            vec!["1C", "55", "E9"],
            vec!["BD", "7A", "55", "E9"],
        ].iter().map(|c| c.iter().map(|v| u8::from_str_radix(v, 16).unwrap()).collect()).collect();
        assert_eq!(Ok((24, 12, 151, 147, expected)), result);
    }
}