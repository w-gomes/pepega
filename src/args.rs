use std::path::PathBuf;

use clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
};
use clap::{Args, Parser, Subcommand, ValueEnum};

const DEFAULT_FRAMERATE: u64 = 5;

const ABOUT: &str = "A command line wrapper for common `FFmpeg` tasks.";

const QUALITY_HELP_TEXT: &str = "The quality flag.

Set the quality with value in [0..=51]
For H264 and H265, the flag crf is used. \
Whereas for AV1 and HEVC, the flag cq and qp is used respectively.

Reasonable values for crf: [17..28]
Reasonable values for qp using hevc: [22..34]
Reasonable values for cq using av1: [26..36]
";

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .error(AnsiColor::Red.on_default().effects(Effects::UNDERLINE));

pub struct Config {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub dry_run: bool,
    pub threads: Option<usize>,
}

#[derive(Debug, Parser)]
#[command(about = ABOUT, author, version, max_term_width = 80, styles = STYLES)]
pub struct Opts {
    #[command(subcommand)]
    pub command: Command,

    /// The input file or directory.
    #[arg(short, long)]
    pub input: PathBuf,

    /// The output file.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Set the number of threads for rayon's thread pool.
    #[arg(short = 'j', long, alias = "jobs", global = true)]
    pub threads: Option<usize>,

    /// Don't do anything, only print the arguments.
    #[arg(short, long, alias = "test", default_value_t = false, global = true)]
    pub dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Extract the audio.
    Audio {
        /// The audio codec.
        #[arg(short = 'A', long, value_enum, default_value_t)]
        audio_codec: AudioCodec,
    },

    /// Create a video from images.
    Create {
        /// Set the framerate.
        /// E.g. --framerate 1 each image becomes one second of video.
        #[arg(long,
              default_value_t = {DEFAULT_FRAMERATE},
              value_parser = clap::value_parser!(u64).range(1..=15))]
        framerate: u64,
    },

    /// Create a clip.
    Clip {
        #[command(subcommand)]
        encode_command: ClipCommand,

        /// The start of the clip.
        start: String,

        /// The end of the clip.
        end: String,
    },

    /// Encode a video.
    Encode {
        #[command(flatten)]
        encode_opts: EncodeOpts,

        /// Flip (rotate) the output clockwise 90 degrees.
        #[arg(short, long)]
        flip: bool,
    },

    /// Merge two or more videos.
    Merge {
        #[command(flatten)]
        encode_opts: EncodeOpts,
    },

    /// Encode a video with specific flags for `Youtube`.
    Youtube {
        /// Flip (rotate) the output clockwise 90 degrees.
        #[arg(short, long)]
        flip: bool,
    },

    /// Upscale and encode video for higher peak quality.
    ///
    /// See for more detail `<https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality>`
    Upscale {
        /// Flip (rotate) the output clockwise 90 degrees.
        #[arg(short, long)]
        flip: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ClipCommand {
    /// Don't encode the output.
    Copy,

    /// Encode the output.
    Encode(EncodeOpts),

    /// Encode the ouput as `gif`.
    ///
    /// Note: `gif` file is large, even for short clip.
    Gif,
}

#[derive(Args, Debug)]
pub struct EncodeOpts {
    /// The video codec.
    #[arg(short = 'V', value_enum, default_value_t)]
    pub video_codec: VideoCodec,

    /// The audio codec.
    #[arg(short = 'A', value_enum, default_value_t = AudioCodec::Opus)]
    pub audio_codec: AudioCodec,

    /// The video format.
    #[arg(short = 'F', value_enum, default_value_t)]
    pub video_format: VideoFormat,

    #[arg(help(QUALITY_HELP_TEXT),
          short = 'Q',
          value_parser = clap::value_parser!(u64).range(0..=63))]
    pub quality: Option<u64>,
}

#[derive(Clone, Debug, Default, strum::Display, ValueEnum)]
pub enum VideoCodec {
    /// `libx264` codec.
    #[strum(to_string = "libx264")]
    #[value(aliases = ["H264", "264"])]
    H264,
    /// `libx265` codec.
    #[value(aliases = ["H265", "265"])]
    #[strum(to_string = "libx265")]
    H265,

    // NVidia
    /// `av1_nvenc` codec.
    #[strum(to_string = "av1_nvenc")]
    Av1,
    /// `hevc_nvenc` codec.
    #[strum(to_string = "hevc_nvenc")]
    #[default]
    Hevc,
}

#[derive(Clone, Debug, Default, strum::Display, ValueEnum)]
pub enum AudioCodec {
    /// `aac` Advanced Audio Coding codec.
    #[strum(to_string = "aac")]
    Aac,
    /// `mp3` MP3 codec.
    #[strum(to_string = "mp3")]
    #[default]
    Mp3,
    /// `libopus` Opus codec.
    #[strum(to_string = "libopus")]
    Opus,
}

#[derive(Clone, Debug, Default, strum::Display, ValueEnum)]
pub enum VideoFormat {
    /// `mp4` MP4 format.
    #[strum(to_string = "mp4")]
    #[default]
    Mp4,
    /// `mkv` Matroska format.
    #[strum(to_string = "mkv")]
    Mkv,
}

#[test]
fn test_cli() {
    use clap::CommandFactory;
    Opts::command().debug_assert();
}
