use crate::Encoders;

use smallvec::SmallVec;

pub struct Clip<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Clip<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn start(mut self, start: &'a str) -> Self {
        self.args.push("-ss");
        self.args.push(start);
        self
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-i");
        self.args.push(path);
        self
    }

    pub fn end(mut self, end: &'a str) -> Self {
        self.args.push("-to");
        self.args.push(end);
        self.args.extend_from_slice(&["-c", "copy", "-copyts"]);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Merge<'a> {
    pub args: SmallVec<[&'a str; 32]>,
}

impl<'a> Merge<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn inputs(mut self, paths: &'a [String]) -> Self {
        for path in paths {
            self.args.push("-i");
            self.args.push(path);
        }
        self
    }

    pub fn filters(mut self, filters: &'a str) -> Self {
        self.args.push("-filter_complex");
        self.args.push(filters);
        // Filter options
        self.args.extend_from_slice(&["-map", "[v]", "-map", "[a]"]);
        // Encode options
        self.args
            .extend_from_slice(&["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac"]);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Video<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Video<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args
            .extend_from_slice(&["-f", "concat", "-safe", "0", "-i"]);
        self.args.push(path);
        self.args
            .extend_from_slice(&["-c:v", "libx264", "-r", "30", "-pix_fmt", "yuv420p"]);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Audio<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Audio<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-i");
        self.args.push(path);
        self.args
            .extend_from_slice(&["-vn", "-c:a", "mp3", "-b:a", "192k"]);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Encode<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Encode<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-i");
        self.args.push(path);
        self
    }

    pub fn encode(mut self, encoder: Encoders, crf: &'a str) -> Self {
        let (encoder, preset, flag, value) = match encoder {
            Encoders::H264 => ("libx264", "ultrafast", "-crf", crf),
            Encoders::H265 => ("hevc_nvenc", "p1", "-cq", "20"),
            Encoders::AV1 => ("av1_nvenc", "p1", "-cq", "20"),
        };
        self.args.push("-c:v");
        self.args.push(encoder);
        self.args.push(flag);
        self.args.push(value);
        self.args.push("-preset");
        self.args.push(preset);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Youtube<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Youtube<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-i");
        self.args.push(path);
        self.args.extend_from_slice(&[
            "-c:v",
            "libx264",
            "-crf",
            "18",
            "-preset",
            "ultrafast",
            "-c:a",
            "aac",
            "-b:a",
            "384k",
            "-pix_fmt",
            "yuv420p",
        ]);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Upscale<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Upscale<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-i");
        self.args.push(path);
        self.args.extend_from_slice(&[
            "-vf",
            "scale=iw*2:ih*2:flags=neighbor",
            "-c:v",
            "libx264",
            "-crf",
            "18",
            "-preset",
            "ultrafast",
        ]);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Flip<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Flip<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-display_rotation:v:0");
        self.args.push("-90.0");
        self.args.push("-i");
        self.args.push(path);
        self.args.push("-c");
        self.args.push("copy");
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Gif<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Gif<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-i");
        self.args.push(path);
        self
    }

    pub fn start(mut self, start: &'a str) -> Self {
        self.args.push("-ss");
        self.args.push(start);
        self
    }

    pub fn end(mut self, end: &'a str) -> Self {
        self.args.push("-to");
        self.args.push(end);
        self
    }

    pub fn flags(mut self) -> Self {
        self.args.extend_from_slice(&[
            "-vf",
            "fps=30,scale=1080:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
            "-loop",
            "0",
        ]);
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}

pub struct Remux<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Remux<'a> {
    pub fn new() -> Self {
        Self {
            args: SmallVec::new(),
        }
    }

    pub fn input(mut self, path: &'a str) -> Self {
        self.args.push("-i");
        self.args.push(path);
        self
    }

    pub fn encode(mut self, should_encode: bool) -> Self {
        if !should_encode {
            self.args.extend_from_slice(&["-c", "copy"]);
        }
        self
    }

    pub fn output(mut self, output: &'a str) -> Self {
        self.args.push(output);
        self
    }
}
