use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};

pub const DEFAULT_CRF: u64 = 23;
pub const DEFAULT_CQ: u64 = 19;

pub struct Config {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub dry_run: bool,
    pub verbose: bool,
}

#[derive(Parser, Debug)]
#[command(
    name = "pepega",
    version,
    author,
    about = "A command line wrapper for common `FFmpeg` tasks.",
    max_term_width = 80
)]
pub struct Opts {
    /// Don't do anything, only print the arguments without running `FFmpeg`
    #[arg(long, aliases = ["dry", "test"], default_value_t = false, global = true)]
    pub dry_run: bool,

    /// Set ffmpeg log level to verbose. Defaults to -loglevel error.
    #[arg(long, default_value_t = false, global = true)]
    pub verbose: bool,

    /// Input: either a single file or a directory
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Extract the audio stream from a video file.
    Audio(AudioArgs),

    /// Common tasks on video: Encode, Clip, Merge, Youtube, Upscale, Gif.
    /// Run pepega.exe video --help for more information
    Video(VideoArgs),

    /// Create a video from images inside a directory.
    ToVideo {
        /// Set the framerate (duration), between 1s and 15s.
        #[arg(
            long,
            alias = "duration",
            value_names = ["FRAMERATE, DURATION"],
            default_value_t = 5,
            value_parser = clap::value_parser!(u64).range(1..=15)
        )]
        framerate: u64,
    },
}

#[derive(Args, Debug)]
pub struct AudioArgs {
    /// Options to encode the audio stream if encoding.
    #[arg(short = 'A', long, value_enum, default_value_t = AudioCodec::Mp3)]
    pub audio_codec: AudioCodec,
}

#[derive(Args, Debug)]
pub struct VideoArgs {
    #[command(subcommand)]
    pub video_commands: VideoCommands,
}

#[derive(Subcommand, Debug)]
pub enum VideoCommands {
    /// Encode the video
    Encode {
        #[command(flatten)]
        encode_opt: EncodeOpt,

        /// Flip (rotate) video clockwise 90 degrees
        #[arg(long)]
        flip: bool,
    },

    /// Clip (trim) a video with START and END timestamps.
    /// If Encode Options are not present, the stream will be copied
    Clip {
        #[command(flatten)]
        encode_opt: Option<EncodeOpt>,

        /// The start of the clip
        start: String,

        /// The end of the clip
        end: String,
    },

    /// Merge two or more videos inside a directory
    #[command(alias = "concat")]
    Merge {
        #[command(flatten)]
        encode_opt: EncodeOpt,
    },

    /// Transcode optimized for `Youtube`
    Youtube {
        /// Flip (rotate) video clockwise 90 degrees
        #[arg(long)]
        flip: bool,
    },

    /// Upscale and transcode video for higher peak quality on platforms like `Youtube`.
    /// Uses `FFmpeg`'s recommended settings for upscalling
    ///
    /// See for more detail `<https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality>`
    Upscale {
        /// Flip (rotate) video clockwise 90 degrees
        #[arg(long)]
        flip: bool,
    },

    /// Clip (trim) a video and save it as `gif`
    /// Note: `gif` file is large, even for short clip
    Gif {
        /// The start of the gif
        start: String,

        /// The end of the gif
        end: String,
    },
}

#[derive(Args, Debug, Clone)]
pub struct EncodeOpt {
    /// Video codec options
    #[arg(short = 'V', long, value_enum)]
    pub video_codec: Option<VideoCodec>,

    /// Audio codec options
    #[arg(short = 'A', long, value_enum, default_value_t = AudioCodec::Aac)]
    pub audio_codec: AudioCodec,

    /// Video format options
    #[arg(short = 'F', long, value_enum, default_value_t = VideoFormat::Mp4)]
    pub video_format: VideoFormat,

    /// Constant Rate Factor (CRF): [0..=51] [default: 23]
    /// Only works with H264 and H265
    #[arg(
        long,
        value_name = "CRF",
        value_parser = clap::value_parser!(u64).range(0..=51),
        conflicts_with = "cq"
    )]
    pub crf: Option<u64>,

    /// Constant Quality: [1..=63] [default: 19]
    /// Only works with AV1 and HVEC
    #[arg(long,
          value_name = "CQ",
          value_parser = clap::value_parser!(u64).range(1..=63),
    )]
    pub cq: Option<u64>,
}

impl Default for EncodeOpt {
    fn default() -> Self {
        Self {
            video_codec: Some(VideoCodec::H264),
            audio_codec: AudioCodec::Aac,
            video_format: VideoFormat::Mp4,
            crf: Some(DEFAULT_CRF),
            cq: Some(DEFAULT_CQ),
        }
    }
}

impl EncodeOpt {
    pub fn check_quality_flags(&self) -> Result<()> {
        if let Some(ref video_codec) = self.video_codec {
            match video_codec {
                VideoCodec::H264 | VideoCodec::H265 => {
                    if self.cq.is_some() {
                        bail!("--cq is not used with {video_codec}");
                    }
                }

                VideoCodec::Av1 | VideoCodec::Hevc => {
                    if self.crf.is_some() {
                        bail!("--crf is not used with {video_codec}");
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(ValueEnum, Debug, Clone, strum::Display)]
pub enum VideoCodec {
    /// `av1_nvenc` NVIDIA av1 codec
    #[strum(to_string = "av1_nvenc")]
    Av1,
    /// `libx264` H.264 codec
    #[strum(to_string = "libx264")]
    H264,
    /// `libx265` H.265 codec
    #[strum(to_string = "libx265")]
    H265,
    /// `hevc_nvenc` HEVC codec
    #[strum(to_string = "hevc_nvenc")]
    Hevc,
}

#[derive(ValueEnum, Debug, Clone, strum::Display)]
pub enum AudioCodec {
    /// `aac` Advanced Audio Coding codec
    #[strum(to_string = "aac")]
    Aac,
    /// `mp3` MP3 codec
    #[strum(to_string = "mp3")]
    Mp3,
    /// `libopus` Opus codec
    #[strum(to_string = "libopus")]
    Opus,
}

#[derive(ValueEnum, Debug, Clone, strum::Display)]
pub enum VideoFormat {
    /// `mp4` MP4 container
    #[strum(to_string = "mp4")]
    Mp4,
    /// `mkv` Matroska container
    #[strum(to_string = "mkv")]
    Mkv,
}

#[test]
fn test_cli() {
    use clap::CommandFactory;
    Opts::command().debug_assert();
}
