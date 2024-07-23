
# tinypng [![Crates.io](https://img.shields.io/crates/v/tinypng.svg?style=flat-square)](https://crates.io/crates/tinypng)

Command line tool for compressing images using the TinyPNG API

## Different

1. No terminal display or no output display on Windows

2. Replacing Printing with Windows Toast Notifications

3. Windows toast notifications support click to open the image

4. Send different emojis according to the ratio

🥰: 80% < ratio

🥳: 60% < ratio <= 80%

😋: 40% < ratio <= 60%

😚: 30% < ratio <= 40%

🙂: 20% < ratio <= 30%

😧: 10% < ratio <= 20%

😨: 5% < ratio <= 10%

🤡: ratio <= 1%

## Usage

1. Register a KEY using your email at [link](https://tinypng.com/developers)

2. Set TinyPNG API KEY

```sh
tinypng -k <KEY>
# Set API KEY successfully
# Your key is stored in ~/.config/tinypng/config.toml
```

3. Compress images

```sh
tinypng ./test.png
# compress by TinyPNG
# test.png
# 1004.7 KB => 245.4 KB (75.6%) 🥳

# Glob
tinypng ./images/*.png
# compress by TinyPNG
# test1.png
# 1 MB => 100 KB (90.0%) 🥰

# compress by TinyPNG
# test2.png
# 1004.7 KB => 245.4 KB (75.6%) 🥳

# compress by TinyPNG
# test3.png
# 1.4 MB => 174.5 KB (87.8%) 😋
...
```
