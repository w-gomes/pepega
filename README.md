# Pepega
This tool is a simple FFmpeg cli wrapper for my personal common usages.

I like to play games with my friends and I will often record my gameplay.
So, I built this for fun and to quickly encode and clip my videos for sharing.


## Installation
Requires Rust and FFmpeg installed. FFmpeg must be in PATH.

`git clone https://github.com/w-gomes/pepega.git`

`cd pepega`

`cargo install --path . --locked`


## Examples
`pepega --help`

`pepega -i input.mp4 -o output.mp4 encode`

`pepega -i input.mp4 encode // output is optional`

`pepega -i input.mp4 clip 00:10:00.000 00:30:00.999`
