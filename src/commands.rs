mod audio;
mod clip;
mod encode;
mod merge;
mod video;

use std::process::Command;

pub use crate::push_args;
pub use audio::audio;
pub use clip::{clip, clip_gif};
pub use encode::{encode, encode_upscale, encode_youtube};
pub use merge::merge;
pub use video::video;

use crate::args::{AudioCodec, VideoCodec};

pub const FFMPEG: &str = "ffmpeg";

pub const DEFAULT_CRF: u64 = 23;
pub const DEFAULT_CQ_AV1: u64 = 28;
pub const DEFAULT_QP_HEVC: u64 = 24;

pub fn loglevel_flags(cmd: &mut Command, verbose: bool) {
    if verbose {
        push_args![cmd => ["-loglevel", "info"]];
    } else {
        push_args![cmd => ["-loglevel", "error"]];
    }
}

pub fn audio_flags(cmd: &mut Command, audio_codec: AudioCodec) {
    match audio_codec {
        enc @ AudioCodec::Aac => {
            push_args![cmd => ["-c:a", enc.to_string(), "-b:a", "256k"]];
        }
        enc @ AudioCodec::Mp3 => {
            push_args![cmd => ["-c:a", enc.to_string(), "-b:a", "320k"]];
        }
        enc @ AudioCodec::Opus => {
            push_args![cmd => ["-c:a", enc.to_string(), "-b:a", "192k"]];
        }
    }
}

pub fn video_flags(cmd: &mut Command, video_codec: VideoCodec, quality: Option<u64>) {
    let quality = quality.unwrap_or(match video_codec {
        VideoCodec::H264 | VideoCodec::H265 => DEFAULT_CRF,
        VideoCodec::Av1 => DEFAULT_CQ_AV1,
        VideoCodec::Hevc => DEFAULT_QP_HEVC,
    });

    match video_codec {
        codec @ (VideoCodec::H264 | VideoCodec::H265) => {
            push_args![cmd => [
                "-c:v", codec.to_string(),
                "-crf", quality.to_string(),
                "-preset", "veryfast",
            ]];
        }
        codec @ VideoCodec::Av1 => {
            push_args![cmd => [
                "-c:v", codec.to_string(),
                "-rc", "vbr",
                "-cq", quality.to_string(),
                "-b:v", 0.to_string(),
                "-preset", "p5",
                "-level", 4.1.to_string(),
            ]];
        }
        codec @ VideoCodec::Hevc => {
            push_args![cmd => [
                "-c:v", codec.to_string(),
                "-rc", "constqp",
                "-qp", quality.to_string(),
                "-preset", "p5",
            ]];
        }
    }
}

pub fn flip_flags(cmd: &mut Command) {
    push_args![cmd => ["-display_rotation:v:0", "-90.0"]];
}

// macro helper for cmd.arg(..);
//
// push_args! [ cmd =>
//   [
//      "c:v", "libx264",
//      "-crf", "20",
//      "-preset", "veryfast",
//   ]
// ]
//
#[macro_export]
macro_rules! push_args [
    ($cmd:ident => [ $( $flag:expr ),+ $(,)? ]) => {
        {
            $(
                $cmd.arg($flag);
            )+
        }
    }
];
