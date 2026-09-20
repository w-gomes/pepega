use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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

    /// Common tasks on video.
    Video(VideoArgs),
}

#[derive(Args, Debug)]
pub struct AudioArgs {
    /// Audio codec options
    #[arg(short = 'A', long, value_enum, default_value_t = AudioCodec::Mp3)]
    pub audio_codec: AudioCodec,
}

#[derive(Args, Debug)]
pub struct VideoArgs {
    /// Flip (rotate) video clockwise 90 degrees
    #[arg(
        short,
        long,
        alias = "rotate",
        default_value_t = false,
        conflicts_with_all = ["youtube", "upscale"]
    )]
    pub flip: bool,

    /// Encoding options
    #[command(flatten)]
    pub encode_opt: EncodeOpt,

    /// Transcode optimized for `Youtube`
    #[arg(short = 'Y', long, default_value_t = false, conflicts_with = "upscale")]
    pub youtube: bool,

    /// Upscale and transcode video for higher peak quality on platforms like `Youtube`.
    /// Uses `FFmpeg`'s recommended settings for upscalling
    ///
    /// See for more detail `<https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality>`
    #[arg(short = 'U', long, default_value_t = false, conflicts_with = "youtube")]
    pub upscale: bool,

    /// Extra options for video
    #[command(subcommand)]
    pub video_cmd: Option<VideoCmd>,
}

#[derive(Subcommand, Debug)]
pub enum VideoCmd {
    /// Clip (trim) a video with START and END timestamps
    #[command(alias = "trim")]
    Clip {
        /// The start of the clip
        start: String,

        /// The end of the clip
        end: String,

        /// Output as `GIF`
        /// Note: `GIF` file is large even for short clips.
        #[arg(long, default_value_t = false)]
        gif: bool,
    },

    /// Merge two or more videos
    #[command(alias = "concat")]
    Merge,

    /// Create a video from images
    Create {
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

#[derive(Args, Debug, Clone)]
pub struct EncodeOpt {
    /// Video codec options
    #[arg(short = 'V', long, value_enum)]
    pub video_codec: VideoCodec,

    /// Audio codec options
    #[arg(short = 'A', long, value_enum, default_value_t = AudioCodec::Aac)]
    pub audio_codec: AudioCodec,

    /// Video format options
    #[arg(short = 'F', long, value_enum, default_value_t = VideoFormat::Mp4)]
    pub video_format: VideoFormat,

    /// Constant Rate Factor (CRF): [0..=51]
    /// 0 is lossless, 51 is the worst quality possible.
    /// Only works with H264
    #[arg(
        long,
        value_name = "CRF",
        default_value_t = 23,
        value_parser = clap::value_parser!(u64).range(0..=51),
        conflicts_with = "cq"
    )]
    pub crf: u64,

    /// Constant Quality: [1..=63]
    /// 1 is lossless, 63 is the worst quality possible.
    /// Only works with H265, AV1 and HVEC
    #[arg(long,
          value_name = "CQ",
          default_value_t = 19,
          value_parser = clap::value_parser!(u64).range(1..=63),
    )]
    pub cq: u64,
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
    /// `mp3` MP3 codec
    #[strum(to_string = "mp3")]
    Mp3,
    /// `libopus` Opus codec
    #[strum(to_string = "libopus")]
    Opus,
}

#[derive(ValueEnum, Debug, Clone, strum::Display)]
pub enum VideoFormat {
    #[strum(to_string = "mp4")]
    Mp4,
    #[strum(to_string = "mkv")]
    Mkv,
}

#[test]
fn test_cli() {
    use clap::CommandFactory;
    Opts::command().debug_assert();
}
