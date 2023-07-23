use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;
use std::time::Duration;

use bmp::Image;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::winuser;

use crate::img::{GrayImage, load_img_from_clipboard, load_img_from_file};
use crate::input::click;
use crate::ocr::{ocr_matrix, ocr_conditions, MatrixTemplates};
use crate::recognize::{CONDITION_COLOR, MATRIX_COLOR};

mod img;
mod ocr;
mod recognize;
mod solver;
mod util;
mod input;
#[cfg(test)]
mod test_cases;

static LOCK: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
unsafe extern "system" fn keyboard_hook(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == winuser::HC_ACTION && w_param == winuser::WM_KEYUP as _ {
        let info: winuser::PKBDLLHOOKSTRUCT = std::mem::transmute(l_param);
        if (*info).vkCode == winuser::VK_SNAPSHOT as _ {
            if LOCK.compare_exchange(false, true, Acquire, Acquire) == Ok(false) {
                thread::spawn(|| {
                    let templates = MatrixTemplates::load_templates();
                    // wait for clipboard buffer initialization
                    thread::sleep(Duration::from_millis(600));
                    match load_img_from_clipboard() {
                        None => {
                            eprintln!("Clipboard has no image data");
                        }
                        Some(img) => {
                            let result = execute(img, &templates, false);
                            if result.is_err() {
                                eprintln!("{}", result.unwrap_err());
                            }
                        }
                    };
                    LOCK.store(false, Release);
                });
            }
        }
    }
    winuser::CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param)
}

fn execute(img: Image, templates: &MatrixTemplates, solutions_only: bool) -> Result<(), String> {
    let matrix_area = recognize::find_matrix_area(&img).ok_or_else(|| "Matrix was not found".to_owned())?;
    let matrix_img = GrayImage::filter(&img, &MATRIX_COLOR, 50, matrix_area.0, matrix_area.1, matrix_area.2, matrix_area.3);
    let matrix = match ocr_matrix(&matrix_img, templates) {
        Ok(r) => r,
        Err(err) => Err(format!("Matrix was not recognized: {}", err))?,
    };
    drop(matrix_img);

    println!("Matrix:");
    for line in matrix.4.iter() {
        let hex = line.iter()
            .map(|v| format!("{:#04x} ", v))
            .collect::<String>();
        println!("{}", hex);
    }
    println!();

    let condition_area = recognize::find_condition_area(&img, &matrix_area).ok_or_else(|| "Conditions were not found".to_owned())?;
    let condition_img = GrayImage::filter(&img, &CONDITION_COLOR, 50, condition_area.0, condition_area.1, condition_area.2, condition_area.3);
    let conditions = match ocr_conditions(&condition_img, templates) {
        Ok(r) => r,
        Err(err) => Err(format!("Conditions were not recognized: {}", err))?,
    };
    drop(condition_img);

    println!("Conditions:");
    for line in conditions.iter() {
        let hex = line.iter()
            .map(|v| format!("{:#04x} ", v))
            .collect::<String>();
        println!("{}", hex);
    }
    println!();

    let steps = recognize::find_buffer_size(&img, &condition_area).ok_or_else(|| "Buffer size was not recognized".to_owned())?;
    println!("Steps: {}", steps);
    println!();

    let solutions = solver::solve(&matrix.4, &conditions, steps);
    println!("Found {} solutions", solutions.len());
    let best = solver::filter_best(&solutions);
    println!("{} best solutions:", best.len());
    for (i, s) in best.iter().enumerate() {
        let conditions = s.conditions.iter()
            .map(|&b| if b { "✔ " } else { "✖ " })
            .collect::<String>();
        let steps = s.steps.iter()
            .map(|step| matrix.4[step.y as usize][step.x as usize])
            .map(|item| format!("{:#04x} ", item))
            .collect::<String>();
        println!("Solution #{}, conditions: {}, steps: {}", i + 1, conditions, steps);
    }
    println!();

    if !solutions_only && !best.is_empty() {
        let left = matrix_area.0 + matrix.0;
        let top = matrix_area.1 + matrix.1;
        let item_width = (matrix.2 - matrix.0) / (matrix.4.len() - 1) as u32;
        let item_height = (matrix.3 - matrix.1) / (matrix.4.len() - 1) as u32;

        let mut cur = (0, 0);
        click(-5000, -5000);
        for s in best.last().unwrap().steps.iter() {
            let x = s.x as u32 * item_width + left + 15;
            let y = s.y as u32 * item_height + top + 10;

            click(x as i32 - cur.0, y as i32 - cur.1);
            cur = (x as i32, y as i32);
        }
    }
    Ok(())
}

fn main() {
    if std::env::args().len() > 1 {
        let bmp_path = std::env::args().last().unwrap();
        println!("Reading {} bmp file...", &bmp_path);
        let img = load_img_from_file(bmp_path);
        execute(img, &MatrixTemplates::load_templates(), true).expect("Error");
        return;
    }


    let hook = unsafe {
        winuser::SetWindowsHookExA(winuser::WH_KEYBOARD_LL, Some(keyboard_hook), std::ptr::null_mut(), 0)
    };
    if hook.is_null() {
        panic!("SetWindowsHookExA returns null");
    }

    println!("Press PrintScreen keyboard button to trigger the bot...");
    unsafe {
        let mut msg: winuser::MSG = std::mem::zeroed();
        while 0 == winuser::GetMessageA(&mut msg, std::ptr::null_mut(), 0, 0) {
            winuser::TranslateMessage(&msg);
            winuser::DispatchMessageA(&msg);
        }
    }

    unsafe { winuser::UnhookWindowsHookEx(hook) };
}
