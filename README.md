# Pepega
A command line wrapper for `FFmpeg`.
This tool is a simple FFmpeg cli wrapper for my personal common usages.


## Installation
Requires Rust and FFmpeg installed and be in PATH.

```
$ git clone https://github.com/w-gomes/pepega.git
$ cd pepega
$ cargo install --path . --locked
```


## Examples
defaults to h264, aac and crf 23
```
$ pepega -i input.mp4 -o output.mp4 video encode
```

```
$ pepega -i input.mp4 -o output.mp4 video encode -V av1 -A opus
```

output is optional and it will be generated automatically next to the input
```
$ pepega -i input.mp4 video encode
```

```
$ pepega -i input.mp4 video clip 00:10:00.000 00:30:00.999
```

encode the video with settings specific for youtube
```
$ pepega -i input.mp4 video youtube
```

can also extract audio
```
$ pepega -i input.mp4 audio
```

for more usages:
`$ pepega --help`
