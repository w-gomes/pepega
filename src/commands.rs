mod audio;
mod clip;
mod encode;
mod merge;
mod video;

use std::path::Path;

pub use audio::audio;
pub use clip::{clip, clip_gif};
pub use encode::{encode, encode_upscale, encode_youtube};
pub use merge::merge;
pub use video::video;

use crate::args::{AudioCodec, VideoCodec};

pub const DEFAULT_CRF: u64 = 23;
pub const DEFAULT_CQ_AV1: u64 = 28;
pub const DEFAULT_QP_HEVC: u64 = 24;

pub fn loglevel_flag(verbose: bool) -> Vec<String> {
    if verbose {
        vec!["-loglevel".to_string(), "info".to_string()]
    } else {
        vec!["-loglevel".to_string(), "error".to_string()]
    }
}

pub fn audio_flag(audio_codec: AudioCodec) -> Vec<String> {
    match audio_codec {
        enc @ AudioCodec::Aac => {
            vec![
                "-c:a".to_string(),
                enc.to_string(),
                "-b:a".to_string(),
                "256k".to_string(),
            ]
        }
        enc @ AudioCodec::Mp3 => {
            vec![
                "-c:a".to_string(),
                enc.to_string(),
                "-b:a".to_string(),
                "320k".to_string(),
            ]
        }
        enc @ AudioCodec::Opus => {
            vec![
                "-c:a".to_string(),
                enc.to_string(),
                "-b:a".to_string(),
                "192k".to_string(),
            ]
        }
    }
}

pub fn video_flag(video_codec: VideoCodec, quality: Option<u64>) -> Vec<String> {
    let quality = quality.unwrap_or(match video_codec {
        VideoCodec::H264 | VideoCodec::H265 => DEFAULT_CRF,
        VideoCodec::Av1 => DEFAULT_CQ_AV1,
        VideoCodec::Hevc => DEFAULT_QP_HEVC,
    });

    match video_codec {
        codec @ (VideoCodec::H264 | VideoCodec::H265) => {
            vec![
                "-c:v".to_string(),
                codec.to_string(),
                "-crf".to_string(),
                quality.to_string(),
                "-preset".to_string(),
                "veryfast".to_string(),
            ]
        }
        codec @ VideoCodec::Av1 => {
            vec![
                "-c:v".to_string(),
                codec.to_string(),
                "-rc".to_string(),
                "vbr".to_string(),
                "-cq".to_string(),
                quality.to_string(),
                "-b:v".to_string(),
                0.to_string(),
                "-preset".to_string(),
                "p5".to_string(),
                "-level".to_string(),
                4.1.to_string(),
            ]
        }
        codec @ VideoCodec::Hevc => {
            vec![
                "-c:v".to_string(),
                codec.to_string(),
                "-rc".to_string(),
                "constqp".to_string(),
                "-qp".to_string(),
                quality.to_string(),
                "-preset".to_string(),
                "p5".to_string(),
            ]
        }
    }
}

pub fn flip_flag(input: &Path, flip: bool) -> Vec<String> {
    if flip {
        vec![
            "-display_rotation:v:0".to_string(),
            "-90.0".to_string(),
            "-i".to_string(),
            input.display().to_string(),
        ]
    } else {
        vec!["-i".to_string(), input.display().to_string()]
    }
}
