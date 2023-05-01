# CyberBot 2077

Bot, who solves breach protocol games from Cyberpunk 2077.

[![Preview](assets/preview.gif)](assets/preview.mp4)

## Usage

- [Download](#download) or [build](#build) the bot
- Launch `cyberbot2077.exe`
- Launch a breach protocol game
- Move mouse cursor away. It must not obstruct the game field
- Press `PrintScreen` keyboard button
- Wait a second
- ???
- PROFIT

> Bot may consume `bmp` image path as last command line argument to print all possible solutions
> ```
> cyberbot2077.exe path/to/image.bmp
> ```

## How it works

This bot uses [Tesseract](https://github.com/tesseract-ocr/tesseract) library as OCR engine.

`PrintScreen` keyboard button triggers the bot, and it grabs screenshot
to recognize game field (matrix, conditions, max step count and screen coordinates).

![Recognize example](assets/recognize.jpg)

```
Matrix:
0xbd 0x55 0x55 0x7a 0xe9 0x7a 0x55
0x55 0xe9 0x55 0xbd 0x55 0x55 0xe9
0xe9 0x55 0xbd 0x7a 0x1c 0x55 0x7a
0x55 0x1c 0x55 0x55 0x7a 0x1c 0xff
0x1c 0x7a 0x7a 0x1c 0xbd 0x1c 0xbd
0x1c 0x7a 0xe9 0xff 0x1c 0xe9 0xff
0x1c 0x7a 0x7a 0xbd 0x7a 0x55 0xbd

Conditions:
0x1c 0x7a
0x7a 0x1c 0x1c
0x7a 0x7a 0xbd 0x7a

Steps: 6
```

Then bot finds all solutions for each condition

![All solution](assets/all.jpg)

Tries to merge solutions together to cover as many conditions as possible:
- One solution may be small piece of other
- Ending of one solution may be beginning of other
- 1-3 additional steps are required to merge them
- ...

Then all merged solutions must be finalized:
first step is always vertical and contains any top item.

Current example has 303 completed solutions.
Bot filters the shortest solutions and applies solution,
which covers as many conditions as possible (last conditions have greater score).

![Best solutions](assets/best.jpg)

```
Found 303 solutions
6 best solutions:
Solution #1, conditions: ✔ ✖ ✖, steps: 0xe9 0x1c 0x7a
Solution #2, conditions: ✖ ✔ ✖, steps: 0x7a 0x1c 0x1c
Solution #3, conditions: ✖ ✖ ✔, steps: 0x7a 0x7a 0xbd 0x7a
Solution #4, conditions: ✔ ✔ ✖, steps: 0x7a 0x1c 0x1c 0x7a
Solution #5, conditions: ✔ ✖ ✔, steps: 0x55 0x1c 0x7a 0x7a 0xbd 0x7a
Solution #6, conditions: ✖ ✔ ✔, steps: 0x7a 0x7a 0xbd 0x7a 0x1c 0x1c
```

Last solution `#6` will be applied.

## Download

Download bot [here](https://github.com/ricorodriges/cyberbot2077/releases).

Bot uses `tesseract/tesseract.exe` binary. So please [download binaries](https://github.com/UB-Mannheim/tesseract/wiki) and extract to `tesseract` directory
```
|- tesseract
|  |- tesseract.exe
|- cyberbot2077.exe
```

Then run `cyberbot2077.exe` and enjoy!

## Build

Since this project uses [Tesseract](https://github.com/tesseract-ocr/tesseract),
you need to build it yourself or [download binaries](https://github.com/UB-Mannheim/tesseract/wiki).

Bot uses `tesseract/tesseract.exe` binary. So please extract binaries to `tesseract` directory
```
|- tesseract
|  |- tesseract.exe
|- src
|- test
|- Cargo.toml
```

Then you may run tests to make sure everything is ok and build
```sh
cargo test
cargo build --release

./target/release/cyberbot2077.exe
```