# Pepega

A command line wrapper for `FFmpeg`.

I often encode, clip and extract audio streams from video as well as upload to youtube.
This tool tries to simplify some of the flags, making it less verbose.

## Installation

Requires Rust and FFmpeg.
FFmpeg should be easy to install with scoop or choco.

```
$ git clone https://github.com/w-gomes/pepega.git
$ cd pepega
$ cargo install --path . --locked
```

## Examples

```
$ pepega -i input.mp4 -o output.mp4 encode

$ pepega -i input.mp4 -o output.mp4 encode -V av1 -A opus

// the output will be created automatically
$ pepega -i input.mp4 encode

// 00:10:00 00:30:00 also works
$ pepega -i input.mp4 clip 00:10:00.000 00:30:00.999 copy

$ pepega -i input.mp4 clip 00:10:00.000 00:30:00.999 encode

// create a gif
$ pepega -i input.mp4 clip 00:10:00.000 00:30:00.999 gif

// specific settings for youtube
$ pepega -i input.mp4 youtube

// extract the audio stream and encode
$ pepega -i input.mp4 audio
```
