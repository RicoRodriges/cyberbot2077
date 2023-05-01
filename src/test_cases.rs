use std::collections::HashSet;
use crate::img::{crop_img, filter_img, load_img_from_file};
use crate::ocr::ocr_matrix;
use crate::{recognize, solver};
use crate::recognize::{CONDITION_COLOR, MATRIX_COLOR};
use crate::solver::{Solution, Step};

pub const FILE1: &str = "test/test1.bmp";

pub fn matrix1() -> Vec<Vec<u8>> {
    vec![
        vec![0xBD, 0x55, 0x55, 0x7A, 0xE9, 0x7A, 0x55],
        vec![0x55, 0xE9, 0x55, 0xBD, 0x55, 0x55, 0xE9],
        vec![0xE9, 0x55, 0xBD, 0x7A, 0x1C, 0x55, 0x7A],
        vec![0x55, 0x1C, 0x55, 0x55, 0x7A, 0x1C, 0xFF],
        vec![0x1C, 0x7A, 0x7A, 0x1C, 0xBD, 0x1C, 0xBD],
        vec![0x1C, 0x7A, 0xE9, 0xFF, 0x1C, 0xE9, 0xFF],
        vec![0x1C, 0x7A, 0x7A, 0xBD, 0x7A, 0x55, 0xBD],
    ]
}

pub fn conditions1() -> Vec<Vec<u8>> {
    vec![
        vec![0x1C, 0x7A],
        vec![0x7A, 0x1C, 0x1C],
        vec![0x7A, 0x7A, 0xBD, 0x7A],
    ]
}

pub const BLOCKS1: usize = 6;

pub fn solutions1() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(5, 0), Step::new(5, 3), Step::new(1, 3), Step::new(1, 4)],
            conditions: vec![true, true, false]
        },
        Solution {
            steps: vec![Step::new(3, 0), Step::new(3, 2), Step::new(2, 2), Step::new(2, 4), Step::new(5, 4), Step::new(5, 0)],
            conditions: vec![true, false, true]
        },
    ]
}

pub const HAS_FULL_SOLUTION1: bool = false;



pub const FILE2: &str = "test/test2.bmp";

pub fn matrix2() -> Vec<Vec<u8>> {
    vec![
        vec![0x55, 0xBD, 0xBD, 0xBD, 0x55],
        vec![0xBD, 0x1C, 0x55, 0xE9, 0x1C],
        vec![0xBD, 0xBD, 0x1C, 0x1C, 0x55],
        vec![0x55, 0xE9, 0xE9, 0x55, 0xE9],
        vec![0x1C, 0x55, 0x55, 0x1C, 0x1C],
    ]
}

pub fn conditions2() -> Vec<Vec<u8>> {
    vec![
        vec![0x1C, 0x55],
        vec![0xBD, 0x55],
        vec![0x55, 0x55, 0x1C],
    ]
}

pub fn solutions2() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(2, 0), Step::new(2, 4), Step::new(1, 4), Step::new(1, 1), Step::new(2, 1)],
            conditions: vec![true, true, true]
        },
        Solution {
            steps: vec![Step::new(3, 0), Step::new(3, 3), Step::new(0, 3), Step::new(0, 4), Step::new(1, 4)],
            conditions: vec![true, true, true]
        },
    ]
}

pub const BLOCKS2: usize = 6;

pub const HAS_FULL_SOLUTION2: bool = true;



pub const FILE3: &str = "test/test3.bmp";

pub fn matrix3() -> Vec<Vec<u8>> {
    vec![
        vec![0x7A, 0x55, 0xE9, 0xBD, 0x1C, 0x55, 0x55],
        vec![0xFF, 0x55, 0x55, 0xBD, 0xE9, 0x7A, 0x1C],
        vec![0x55, 0x1C, 0x55, 0x55, 0x1C, 0x55, 0xE9],
        vec![0xE9, 0x1C, 0xFF, 0x55, 0x1C, 0x1C, 0x55],
        vec![0xBD, 0x1C, 0x1C, 0x1C, 0xE9, 0x55, 0x1C],
        vec![0x1C, 0xFF, 0x55, 0x7A, 0xBD, 0x55, 0x1C],
        vec![0x55, 0x55, 0x7A, 0xBD, 0x7A, 0x55, 0x55],
    ]
}

pub fn conditions3() -> Vec<Vec<u8>> {
    vec![
        vec![0x1C, 0xE9, 0xBD],
        vec![0x1C, 0x55, 0x1C],
        vec![0x7A, 0xE9, 0x1C],
    ]
}

pub fn solutions3() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(0, 0), Step::new(0, 3), Step::new(1, 3), Step::new(1, 1), Step::new(6, 1)],
            conditions: vec![false, true, true]
        },
        Solution {
            steps: vec![Step::new(0, 0), Step::new(0, 3), Step::new(4, 3), Step::new(4, 1), Step::new(3, 1)],
            conditions: vec![true, false, true]
        },
        Solution {
            steps: vec![Step::new(4, 0), Step::new(4, 1), Step::new(3, 1), Step::new(3, 4), Step::new(5, 4), Step::new(5, 3)],
            conditions: vec![true, true, false]
        },
    ]
}

pub const BLOCKS3: usize = 6;

pub const HAS_FULL_SOLUTION3: bool = false;



pub const FILE4: &str = "test/test4.bmp";

pub fn matrix4() -> Vec<Vec<u8>> {
    vec![
        vec![0x7A, 0x55, 0xFF, 0xBD, 0xBD, 0xBD, 0xE9],
        vec![0xBD, 0xFF, 0xFF, 0x55, 0xE9, 0xE9, 0x7A],
        vec![0x55, 0x7A, 0x7A, 0xBD, 0xE9, 0xFF, 0xBD],
        vec![0x7A, 0x1C, 0xFF, 0xFF, 0x7A, 0x1C, 0x1C],
        vec![0x7A, 0xBD, 0x55, 0x55, 0xE9, 0x55, 0x55],
        vec![0x7A, 0xE9, 0x55, 0xBD, 0xBD, 0x1C, 0x1C],
        vec![0x55, 0x7A, 0x7A, 0x7A, 0x7A, 0x7A, 0x1C],
    ]
}

pub fn conditions4() -> Vec<Vec<u8>> {
    vec![
        vec![0xFF, 0x7A],
        vec![0x1C, 0x7A, 0xBD],
        vec![0x7A, 0x7A, 0x55, 0x1C],
    ]
}

pub fn solutions4() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(0, 0), Step::new(0, 4), Step::new(6, 4), Step::new(6, 3), Step::new(4, 3), Step::new(4, 0)],
            conditions: vec![false, true, true]
        },
        Solution {
            steps: vec![Step::new(0, 0), Step::new(0, 4), Step::new(6, 4), Step::new(6, 3), Step::new(2, 3), Step::new(2, 6)],
            conditions: vec![true, false, true]
        },
        Solution {
            steps: vec![Step::new(6, 0), Step::new(6, 3), Step::new(4, 3), Step::new(4, 0), Step::new(2, 0), Step::new(2, 2)],
            conditions: vec![true, true, false]
        },
    ]
}

pub const BLOCKS4: usize = 6;

pub const HAS_FULL_SOLUTION4: bool = false;



pub const FILE5: &str = "test/test5.bmp";

pub fn matrix5() -> Vec<Vec<u8>> {
    vec![
        vec![0x7A, 0xFF, 0xE9, 0x55, 0x7A, 0x7A, 0x7A],
        vec![0x7A, 0xE9, 0x1C, 0x55, 0xFF, 0x55, 0x1C],
        vec![0x7A, 0x1C, 0x7A, 0xE9, 0x7A, 0x7A, 0x55],
        vec![0x7A, 0x55, 0x55, 0xBD, 0x1C, 0x55, 0x1C],
        vec![0xBD, 0x7A, 0xE9, 0xE9, 0x1C, 0xFF, 0x55],
        vec![0x55, 0xFF, 0xBD, 0xBD, 0x1C, 0x7A, 0x1C],
        vec![0x55, 0x1C, 0xBD, 0xBD, 0x7A, 0xFF, 0xBD],
    ]
}

pub fn conditions5() -> Vec<Vec<u8>> {
    vec![
        vec![0x7A, 0xE9, 0xFF],
        vec![0x1C, 0x55, 0xE9],
        vec![0xBD, 0x7A, 0x55, 0xE9],
    ]
}

pub fn solutions5() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(0, 0), Step::new(0, 1), Step::new(1, 1), Step::new(1, 5)],
            conditions: vec![true, false, false]
        },
        Solution {
            steps: vec![Step::new(2, 0), Step::new(2, 1), Step::new(3, 1), Step::new(3, 2)],
            conditions: vec![false, true, false]
        },
    ]
}

pub const BLOCKS5: usize = 6;

pub const HAS_FULL_SOLUTION5: bool = false;



pub const FILE6: &str = "test/test6.bmp";

pub fn matrix6() -> Vec<Vec<u8>> {
    vec![
        vec![0x1C, 0x1C, 0x1C, 0xBD, 0x1C, 0x1C],
        vec![0xBD, 0xBD, 0xE9, 0x55, 0x1C, 0x7A],
        vec![0x7A, 0x1C, 0x1C, 0xBD, 0x55, 0x1C],
        vec![0x7A, 0x7A, 0x7A, 0x7A, 0xE9, 0x55],
        vec![0x7A, 0x7A, 0xE9, 0x1C, 0x55, 0x55],
        vec![0x1C, 0x7A, 0x1C, 0xE9, 0x1C, 0x55],
    ]
}

pub fn conditions6() -> Vec<Vec<u8>> {
    vec![
        vec![0x1C, 0xBD, 0xE9, 0x1C],
    ]
}

pub fn solutions6() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(0, 0), Step::new(0, 1), Step::new(2, 1), Step::new(2, 0)],
            conditions: vec![true]
        },
        Solution {
            steps: vec![Step::new(1, 0), Step::new(1, 1), Step::new(2, 1), Step::new(2, 0)],
            conditions: vec![true]
        },
    ]
}

pub const BLOCKS6: usize = 6;

pub const HAS_FULL_SOLUTION6: bool = true;



pub const FILE7: &str = "test/test7.bmp";

pub fn matrix7() -> Vec<Vec<u8>> {
    vec![
        vec![0x7A, 0xFF, 0x7A, 0xFF, 0x55, 0x1C, 0x7A],
        vec![0x7A, 0xBD, 0x7A, 0xFF, 0x7A, 0x1C, 0xE9],
        vec![0x7A, 0x7A, 0xE9, 0x1C, 0x55, 0x1C, 0xFF],
        vec![0xE9, 0x7A, 0xE9, 0xBD, 0xBD, 0xE9, 0xE9],
        vec![0x1C, 0xBD, 0xE9, 0xE9, 0xE9, 0x1C, 0x7A],
        vec![0x1C, 0x1C, 0x7A, 0xE9, 0x55, 0x7A, 0x7A],
        vec![0xFF, 0x1C, 0x1C, 0xFF, 0xE9, 0xBD, 0x1C],
    ]
}

pub fn conditions7() -> Vec<Vec<u8>> {
    vec![
        vec![0xFF, 0x55, 0xE9],
        vec![0x1C, 0x7A, 0xE9, 0xFF],
        vec![0xE9, 0xE9, 0xFF],
    ]
}

pub fn solutions7() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(5, 0), Step::new(5, 5), Step::new(3, 5), Step::new(3, 0), Step::new(4, 0), Step::new(4, 4)],
            conditions: vec![true, true, false]
        },
        Solution {
            steps: vec![Step::new(6, 0), Step::new(6, 2), Step::new(4, 2), Step::new(4, 4), Step::new(3, 4), Step::new(3, 0)],
            conditions: vec![true, false, true]
        },
    ]
}

pub const BLOCKS7: usize = 6;

pub const HAS_FULL_SOLUTION7: bool = false;



pub const FILE8: &str = "test/test8.bmp";

pub fn matrix8() -> Vec<Vec<u8>> {
    vec![
        vec![0xBD, 0x1C, 0x1C, 0x7A, 0x55, 0x1C],
        vec![0x1C, 0xE9, 0xE9, 0x55, 0x7A, 0x55],
        vec![0x55, 0x1C, 0x55, 0x7A, 0x55, 0x55],
        vec![0xE9, 0xE9, 0x1C, 0x55, 0x55, 0xBD],
        vec![0x1C, 0x7A, 0x7A, 0xE9, 0x1C, 0x1C],
        vec![0x1C, 0x1C, 0xBD, 0xBD, 0x1C, 0xBD],
    ]
}

pub fn conditions8() -> Vec<Vec<u8>> {
    vec![
        vec![0x55, 0x1C, 0xBD, 0xE9],
        vec![0x55, 0x7A, 0x55],
    ]
}

pub fn solutions8() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(4, 0), Step::new(4, 1), Step::new(5, 1), Step::new(5, 0), Step::new(0, 0), Step::new(0, 3)],
            conditions: vec![true, true]
        },
    ]
}

pub const BLOCKS8: usize = 6;

pub const HAS_FULL_SOLUTION8: bool = true;



pub const FILE9: &str = "test/test9.bmp";

pub fn matrix9() -> Vec<Vec<u8>> {
    vec![
        vec![0xE9, 0x55, 0x7A, 0xE9, 0xE9, 0x7A],
        vec![0xE9, 0xBD, 0xBD, 0x55, 0x1C, 0x7A],
        vec![0x7A, 0x1C, 0x1C, 0xE9, 0x55, 0x1C],
        vec![0x7A, 0x7A, 0x55, 0x1C, 0xBD, 0x7A],
        vec![0x7A, 0x1C, 0x1C, 0x1C, 0x7A, 0xE9],
        vec![0xBD, 0xBD, 0x55, 0x1C, 0x7A, 0x1C],
    ]
}

pub fn conditions9() -> Vec<Vec<u8>> {
    vec![
        vec![0x1C, 0x7A, 0xE9],
        vec![0xE9, 0x1C, 0xBD, 0xE9],
        vec![0xE9, 0x7A, 0xBD, 0x55],
    ]
}

pub fn solutions9() -> Vec<Solution> {
    vec![
        Solution {
            steps: vec![Step::new(1, 0), Step::new(1, 2), Step::new(0, 2), Step::new(0, 0)],
            conditions: vec![true, false, false]
        },
        Solution {
            steps: vec![Step::new(3, 0), Step::new(3, 3), Step::new(4, 3), Step::new(4, 0)],
            conditions: vec![false, true, false]
        },
        Solution {
            steps: vec![Step::new(0, 0), Step::new(0, 3), Step::new(4, 3), Step::new(4, 2)],
            conditions: vec![false, false, true]
        },
    ]
}

pub const BLOCKS9: usize = 6;

pub const HAS_FULL_SOLUTION9: bool = false;



#[test]
fn all_hex() {
    let mut hex = HashSet::new();
    for matrix in [matrix1(), matrix2(), matrix3(), matrix4(), matrix5(), matrix6(), matrix7(), matrix8(), matrix9()] {
        for line in matrix {
            for h in line {
                hex.insert(h);
            }
        }
    }
    assert_eq!(HashSet::from([0x1C, 0x55, 0x7A, 0xBD, 0xE9, 0xFF]), hex);
}

#[test]
fn test1() {
    test(FILE1, &matrix1(), &conditions1(), BLOCKS1, &solutions1(), HAS_FULL_SOLUTION1);
}

#[test]
fn test2() {
    test(FILE2, &matrix2(), &conditions2(), BLOCKS2, &solutions2(), HAS_FULL_SOLUTION2);
}

#[test]
fn test3() {
    test(FILE3, &matrix3(), &conditions3(), BLOCKS3, &solutions3(), HAS_FULL_SOLUTION3);
}

#[test]
fn test4() {
    test(FILE4, &matrix4(), &conditions4(), BLOCKS4, &solutions4(), HAS_FULL_SOLUTION4);
}

#[test]
fn test5() {
    test(FILE5, &matrix5(), &conditions5(), BLOCKS5, &solutions5(), HAS_FULL_SOLUTION5);
}

#[test]
fn test6() {
    test(FILE6, &matrix6(), &conditions6(), BLOCKS6, &solutions6(), HAS_FULL_SOLUTION6);
}

#[test]
fn test7() {
    test(FILE7, &matrix7(), &conditions7(), BLOCKS7, &solutions7(), HAS_FULL_SOLUTION7);
}

#[test]
fn test8() {
    test(FILE8, &matrix8(), &conditions8(), BLOCKS8, &solutions8(), HAS_FULL_SOLUTION8);
}

#[test]
fn test9() {
    test(FILE9, &matrix9(), &conditions9(), BLOCKS9, &solutions9(), HAS_FULL_SOLUTION9);
}

fn test(path: &str, expected_matrix: &Vec<Vec<u8>>, expected_conditions: &Vec<Vec<u8>>, expected_blocks: usize, expected_solutions: &Vec<Solution>, has_full_solution: bool) {
    let img = load_img_from_file(path);

    let matrix_area = recognize::find_matrix_area(&img).expect("Matrix was not found");
    let mut matrix_img = crop_img(&img, matrix_area.0, matrix_area.1, matrix_area.2, matrix_area.3);
    filter_img(&mut matrix_img, &MATRIX_COLOR, 60);
    let matrix = match ocr_matrix(&matrix_img) {
        Ok(r) => r,
        Err(err) => panic!("Matrix was not recognized: {}", err),
    };
    drop(matrix_img);
    assert_eq!(*expected_matrix, matrix.4);

    let condition_area = recognize::find_condition_area(&img, &matrix_area).expect("Conditions were not found");
    let mut condition_img = crop_img(&img, condition_area.0, condition_area.1, condition_area.2, condition_area.3);
    filter_img(&mut condition_img, &CONDITION_COLOR, 60);
    let conditions = match ocr_matrix(&condition_img) {
        Ok(r) => r,
        Err(err) => panic!("Conditions were not recognized: {}", err),
    };
    drop(condition_img);
    assert_eq!(*expected_conditions, conditions.4);

    let blocks = recognize::find_blocks_count(&img, &condition_area).expect("Blocks were not found");
    assert_eq!(expected_blocks, blocks);

    let solutions = solver::solve(&matrix.4, &conditions.4, blocks);
    for expected in expected_solutions.iter() {
        let found = solutions.iter().any(|actual| actual.conditions == expected.conditions && actual.steps == expected.steps);
        assert_eq!(true, found);
    }

    assert_eq!(true, solutions.iter().all(|s| s.conditions.contains(&true)), "solution covers nothing");
    for s in solutions.iter() {
        assert_eq!(true, !s.steps.is_empty() && s.steps.len() <= blocks, "solution is too long or empty");
        for step in s.steps.iter() {
            let found_steps = s.steps.iter().filter(|s| *s == step).count();
            assert_eq!(1, found_steps, "solution has same step 2 times");
        }

        assert_eq!(0, s.steps[0].y, "solution is not finalized");
        for i in (0..s.steps.len()).step_by(2) {
            let current = s.steps[i];
            if let Some(next) = s.steps.get(i + 1) {
                assert_eq!(current.x, next.x);
                assert_ne!(current.y, next.y);

                if let Some(next_next) = s.steps.get(i + 2) {
                    assert_ne!(next.x, next_next.x);
                    assert_eq!(next.y, next_next.y);
                }
            }
        }
    }

    let best = solver::filter_best(&solutions);
    for expected in expected_solutions.iter() {
        let found = best.iter().filter(|actual| actual.conditions == expected.conditions).count();
        assert_eq!(1, found);
    }

    let full_solution = best.iter().any(|s| s.conditions.iter().all(|s| *s));
    assert_eq!(has_full_solution, full_solution);
}