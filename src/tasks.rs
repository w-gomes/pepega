use crate::Encoders;

pub(crate) struct Clip {
    pub args: Vec<String>,
}

impl Clip {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn start(mut self, start: &str) -> Self {
        self.args.push("-ss".to_string());
        self.args.push(start.to_string());
        self
    }

    pub fn input(mut self, path: &str) -> Self {
        self.args.push("-i".to_string());
        self.args.push(path.to_string());
        self
    }

    pub fn end(mut self, end: &str) -> Self {
        self.args.push("-to".to_string());
        self.args.push(end.to_string());
        self
    }

    pub fn encode(mut self, should_encode: bool) -> Self {
        if should_encode {
            self.args.push("-c:v".to_string());
            self.args.push("libx264".to_string());
            self.args.push("-c:a".to_string());
            self.args.push("aac".to_string());
        } else {
            self.args.push("-c".to_string());
            self.args.push("copy".to_string());
            self.args.push("-copyts".to_string());
        }
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}

pub(crate) struct Merge {
    pub args: Vec<String>,
}

impl Merge {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn input(mut self, path: &str) -> Self {
        self.args.push("-i".to_string());
        self.args.push(path.to_string());
        self
    }

    pub fn inputs(mut self, paths: &[String]) -> Self {
        for path in paths.iter() {
            self.args.push("-i".to_string());
            self.args.push(path.to_string());
        }
        self
    }

    pub fn filters(mut self, filters: &str) -> Self {
        self.args.push("-filter_complex".to_string());
        self.args.push(filters.to_string());
        // Filter options
        self.args
            .extend(["-map", "[v]", "-map", "[a]"].iter().map(|s| s.to_string()));
        // Encode options
        self.args.extend(
            ["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac"]
                .iter()
                .map(|s| s.to_string()),
        );
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}

pub(crate) struct Video {
    pub args: Vec<String>,
}

impl Video {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn input(mut self, path: &str) -> Self {
        self.args.extend(
            ["-f", "concat", "-safe", "0", "-i"]
                .iter()
                .map(|s| s.to_string()),
        );
        self.args.push(path.to_string());
        self.args.extend(
            ["-c:v", "libx264", "-r", "30", "-pix_fmt", "yuv420p"]
                .iter()
                .map(|s| s.to_string()),
        );
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}

pub(crate) struct Audio {
    pub args: Vec<String>,
}

impl Audio {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn input(mut self, path: &str) -> Self {
        self.args.push("-i".to_string());
        self.args.push(path.to_string());
        self.args
            .extend(["-vn", "mp3", "-b:a", "192k"].iter().map(|s| s.to_string()));
        self
    }

    #[allow(dead_code)]
    pub fn inputs(mut self, paths: &[&str]) -> Self {
        for path in paths.iter() {
            self.args.push("-i".to_string());
            self.args.push(path.to_string());
        }
        self.args
            .extend(["-vn", "mp3", "-b:a", "192k"].iter().map(|s| s.to_string()));
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}

pub(crate) struct Encode {
    pub args: Vec<String>,
}

impl Encode {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn input(mut self, path: &str) -> Self {
        self.args.push("-i".to_string());
        self.args.push(path.to_string());
        self
    }

    #[allow(dead_code)]
    fn inputs(mut self, paths: &[&str]) -> Self {
        for path in paths.iter() {
            self.args.push("-i".to_string());
            self.args.push(path.to_string());
        }
        self
    }

    pub fn encode(mut self, encoder: Encoders, crf: i16) -> Self {
        let (encoder, preset, flag, value) = match encoder {
            Encoders::H264 => ("libx264", "ultrafast", "-crf", crf),
            Encoders::H265 => ("hevc_nvenc", "p1", "-cq", 20),
            Encoders::AV1 => ("av1_nvenc", "p1", "-cq", 20),
        };
        self.args.push("-c:v".to_string());
        self.args.push(encoder.to_string());
        self.args.push(flag.to_string());
        self.args.push(value.to_string());
        self.args.push("-preset".to_string());
        self.args.push(preset.to_string());
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}

pub(crate) struct Youtube {
    pub args: Vec<String>,
}

impl Youtube {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn input(mut self, path: &str) -> Self {
        self.args.push("-i".to_string());
        self.args.push(path.to_string());
        self.args.extend(
            [
                "-c:v",
                "libx264",
                "-crf",
                "18",
                "-preset",
                "ultrapfast",
                "-c:a",
                "aac",
                "-b:a",
                "384k",
                "-pix_fmt",
                "yuv420p",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
        self
    }

    #[allow(dead_code)]
    pub fn inputs(mut self, paths: &[&str]) -> Self {
        for path in paths.iter() {
            self.args.push("-i".to_string());
            self.args.push(path.to_string());
        }
        self.args.extend(
            [
                "-c:v",
                "libx264",
                "-crf",
                "18",
                "-preset",
                "ultrapfast",
                "-c:a",
                "aac",
                "-b:a",
                "384k",
                "-pix_fmt",
                "yuv420p",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}

pub(crate) struct Upscale {
    pub args: Vec<String>,
}

impl Upscale {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn input(mut self, path: &str) -> Self {
        self.args.push("-i".to_string());
        self.args.push(path.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn inputs(mut self, paths: &[&str]) -> Self {
        for path in paths.iter() {
            self.args.push("-i".to_string());
            self.args.push(path.to_string());
        }
        self.args.extend(
            [
                "-vf",
                "scale=iw*2:ih*2:flags=neighbor",
                "-c:v",
                "libx264",
                "-crf",
                "18",
                "-preset",
                "ultrapfast",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}

pub(crate) struct Flip {
    pub args: Vec<String>,
}

impl Flip {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn input(mut self, path: &str) -> Self {
        self.args.push("-display_rotation:v:0".to_string());
        self.args.push("-90.0".to_string());
        self.args.push("-i".to_string());
        self.args.push(path.to_string());
        self.args.push("-c".to_string());
        self.args.push("copy".to_string());
        self
    }

    #[allow(dead_code)]
    pub fn inputs(mut self, paths: &[&str]) -> Self {
        self.args.push("-display_rotation:v:0".to_string());
        self.args.push("-90.0".to_string());
        for path in paths.iter() {
            self.args.push("-i".to_string());
            self.args.push(path.to_string());
        }
        self.args.push("-c".to_string());
        self.args.push("copy".to_string());
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.args.push(output.to_string());
        self
    }
}
