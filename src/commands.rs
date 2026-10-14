mod audio;
mod clip;
mod encode;
mod merge;
mod video;

use std::process::Command;

pub use crate::args::{DEFAULT_CQ_AV1, DEFAULT_CRF, DEFAULT_QP_HEVC};
pub use crate::push_args;
pub use audio::audio;
pub use clip::{clip, clip_gif};
pub use encode::{encode, encode_upscale, encode_youtube};
pub use merge::merge;
pub use video::video;

use crate::args::{AudioCodec, CropCommand, OffSet, VideoCodec, VideoFormat};

pub const FFMPEG: &str = "ffmpeg";

pub fn global_flags(cmd: &mut Command) {
    push_args![cmd => ["-y", "-hide_banner", "-loglevel", "error", "-progress", "pipe:1"]];
}

pub fn audio_flags(cmd: &mut Command, audio_codec: AudioCodec) {
    match audio_codec {
        enc @ AudioCodec::Copy => {
            push_args![cmd => ["-c:a", enc.to_string()]];
        }
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
        VideoCodec::Copy => 0,
    });

    match video_codec {
        codec @ VideoCodec::Copy => {
            push_args![cmd => ["-c:v", codec.to_string()]];
        }
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
                "-b:v", "0",
                "-preset", "p5",
                "-level", "4.1",
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

pub fn map_flags(cmd: &mut Command, indexes: &[u64]) {
    push_args![cmd => ["-map", "0:v", "-map", "0:a"]];
    for idx in indexes {
        push_args![cmd => ["-map", format!("0:s:{idx}")]];
    }
    push_args![cmd => ["-map", "-0:t"]];
}

pub fn subtitles_flags(cmd: &mut Command, video_format: &VideoFormat) {
    let flags_for_mp4_or_mkv = match video_format {
        VideoFormat::Mkv => "copy",
        VideoFormat::Mp4 => "mov_text",
    };
    push_args![cmd => ["-c:s", flags_for_mp4_or_mkv]];
}

pub fn crop_flags(cmd: &mut Command, crop_opts: &CropCommand) {
    let CropCommand::Crop {
        width,
        height,
        offset,
    } = crop_opts;

    if let Some(offset) = offset {
        let OffSet { x, y } = offset;

        let (Some(x), Some(y)) = (x, y) else {
            unreachable!()
        };

        let crop = format!("crop={width}:{height}:{x}:{y}");
        push_args![cmd => ["-vf", crop]];
    } else {
        let crop = format!("crop={width}:{height}:(in_w-{width})/2:(in_h-{height})/2");
        push_args![cmd => ["-vf", crop]];
    }
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
