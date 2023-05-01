use std::thread::sleep;
use std::time::Duration;
use winput::{Action, Button, Input, Mouse};

pub fn click(dx: i32, dy: i32) {
    Mouse::move_relative(dx, dy);
    sleep(Duration::from_millis(300));

    let input = Input::from_button(Button::Left, Action::Press);
    winput::send_inputs(&[input]);

    sleep(Duration::from_millis(30));

    let input = Input::from_button(Button::Left, Action::Release);
    winput::send_inputs(&[input]);
    sleep(Duration::from_millis(200));
}
