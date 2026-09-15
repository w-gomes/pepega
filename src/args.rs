use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// A command line tool to encode videos with the most common options using `FFmpeg`.
#[derive(Parser, Debug)]
#[command(version, author)]
pub struct Cli {
    /// Only print actions, without running ffmpeg
    #[arg(long, default_value_t = false)]
    pub dry: bool,

    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Extract the audio stream of a video
    Audio {
        /// Input
        input: PathBuf,

        /// Output
        output: Option<PathBuf>,

        /// Encode the ouput instead of copying stream from input
        #[arg(long, default_value_t = AudioCodec::Mp3)]
        audio_codec: AudioCodec,
    },

    /// Clip a video.
    /// Both the audio and video streams are copied by default
    #[command(aliases = ["trim"])]
    Clip {
        /// Input
        input: PathBuf,

        /// Output
        output: Option<PathBuf>,

        /// The START of the clip.
        start: String,
        /// The END of the clip.
        end: String,

        /// Saves output as `GIF`.
        /// Note: `GIF` files are large even for short clips.
        #[arg(long, default_value_t = false)]
        gif: bool,

        /// Encode options for encoding
        #[command(flatten)]
        encode_option: Option<EncodeOption>,
    },

    /// Encode
    Encode {
        /// Encode options for encoding
        #[command(flatten)]
        encode_option: EncodeOption,

        /// Constant Rate Factor (CRF):
        /// 0 is lossless, 51 is the worst quality possible.
        #[arg(long,
              value_name = "CRF",
              default_value_t = 23,
              value_parser = clap::value_parser!(u64).range(0..=51),
              conflicts_with = "cq"
        )]
        crf: u64,

        /// Constant Quality:
        /// 1 is lossless, 63 is the worst quality possible.
        #[arg(long,
              value_name = "CQ",
              default_value_t = 19,
              value_parser = clap::value_parser!(u64).range(1..=63),
              conflicts_with = "crf"
        )]
        cq: u64,

        /// Rotate clockwise 90 degrees
        #[arg(long)]
        rotate: bool,
    },

    /// Flip one or more videos.
    Flip {},

    /// Merge two or more videos.
    Merge {},

    /// Upscale one or more videos for higher peak quality on platforms like `Youtube`.
    /// Uses `FFmpeg`'s recommended settings for upscalling.
    /// See for more detail `<https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality>`
    Upscale {},

    /// Create a video slideshow from images.
    Video {
        /// Set a custom framerate 1..15.
        #[arg(short,
              long,
              value_name = "FRAMERATE",
              default_value_t = 5, value_parser = clap::value_parser!(u64).range(1..=15)
        )]
        framerate: u64,
    },

    /// Encode one or more videos with settings optimized for `Youtube`.
    Youtube {},
}

#[derive(Args, Debug, Clone)]
pub struct EncodeOption {
    #[arg(long, value_enum, default_value_t = VideoCodec::H264)]
    video_codec: VideoCodec,

    #[arg(long, value_enum, default_value_t = AudioCodec::Aac)]
    audio_codec: AudioCodec,

    #[arg(long, value_enum, default_value_t = VideoExtension::Mp4)]
    video_extension: VideoExtension,
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
    /// `aac` Adcanced Audio Coding codec
    #[strum(to_string = "aac")]
    Aac,
    /// `flac` Free Lossless Audio codec
    #[strum(to_string = "flac")]
    Flac,
    /// `mp3` MP3 codec
    #[strum(to_string = "mp3")]
    Mp3,
    /// `libopus` Opus codec
    #[strum(to_string = "opus")]
    Opus,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum VideoExtension {
    Mp4,
    Mkv,
}

#[test]
fn test_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
