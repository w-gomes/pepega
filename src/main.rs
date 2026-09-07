use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

mod commands;

use crate::commands::Audio;

#[derive(Parser, Debug)]
struct Cli {
    /// Print the args only.
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Args, Debug)]
struct InOutOption {
    /// Inputs files.
    #[arg(short, long, num_args = 1.., required = true)]
    inputs: Vec<PathBuf>,

    /// Output files.
    #[arg(short, long, num_args = 1..)]
    outputs: Option<Vec<PathBuf>>,
}

impl InOutOption {
    fn split(self) -> (Vec<PathBuf>, Option<Vec<PathBuf>>) {
        (self.inputs, self.outputs)
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract the audio stream of a video.
    Audio {
        #[command(flatten)]
        in_out: InOutOption,
    },

    /// Clip a video.
    Clip {
        #[command(flatten)]
        in_out: InOutOption,

        /// The START of the clip.
        start: String,
        /// The END of the clip.
        end: String,

        /// Saves output as `GIF`.
        /// Note: `GIF` files are large even for short clips.
        #[arg(long, default_value_t = false)]
        gif: bool,
    },

    /// Convert one or more videos.
    Convert {
        #[command(flatten)]
        in_out: InOutOption,

        /// Encode or Remux.
        #[command(subcommand)]
        subcmd: ConvertSubcmd,
    },

    /// Flip one or more videos.
    Flip {
        #[command(flatten)]
        in_out: InOutOption,
    },

    /// Merge two or more videos.
    Merge {
        #[command(flatten)]
        in_out: InOutOption,
    },

    /// Upscale one or more videos for higher peak quality on platforms like `Youtube`.
    /// Uses `FFmpeg`'s recommended settings for upscalling.
    /// See for more detail `<https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality>`
    Upscale {
        #[command(flatten)]
        in_out: InOutOption,
    },

    /// Create a video slideshow from images.
    Video {
        #[command(flatten)]
        in_out: InOutOption,

        /// Set a custom framerate 1..15.
        #[arg(short,
              long,
              value_name = "FRAMERATE",
              default_value_t = 5, value_parser = clap::value_parser!(u64).range(1..=15)
        )]
        framerate: u64,
    },

    /// Encode one or more videos with settings optimized for `Youtube`.
    Youtube {
        #[command(flatten)]
        in_out: InOutOption,
    },
}

#[derive(Subcommand, Debug)]
enum ConvertSubcmd {
    /// Encode using H264, H265, AV1 or HEVC.
    Encode {
        /// Codec options for encoding.
        #[command(flatten)]
        codec_option: CodecOption,

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
    },

    /// Convert to a different format. The streams are copied.
    Remux,
}

#[derive(Args, Debug, Clone)]
struct CodecOption {
    #[arg(short, long, value_enum, default_value_t = VideoCodec::H264)]
    video_codec: VideoCodec,

    #[arg(short, long, value_enum, default_value_t = AudioCodec::Aac)]
    audio_codec: AudioCodec,
}

#[derive(ValueEnum, Debug, Clone)]
enum VideoCodec {
    /// av1_nvenc NVIDIA av1 codec
    Av1,
    /// libx264 H.264 codec
    H264,
    /// libx265 H.265 codec
    H265,
    /// hevc_nvenc HEVC codec
    Hevc,
}

impl VideoCodec {
    const fn to_string(self) -> &'static str {
        match self {
            VideoCodec::Av1 => "av1_nvenc",
            VideoCodec::H264 => "libx264",
            VideoCodec::H265 => "libx265",
            VideoCodec::Hevc => "hevc_nvenc",
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
enum AudioCodec {
    /// aac Adcanced Audio Coding codec
    Aac,
    /// flac Free Lossless Audio codec
    Flac,
    /// mp3_mf MP3 codec
    Mp3,
    /// libopus Opus codec
    Opus,
}

impl AudioCodec {
    const fn to_string(self) -> &'static str {
        match self {
            AudioCodec::Aac => "aac",
            AudioCodec::Flac => "flac",
            AudioCodec::Mp3 => "mp3_mf",
            AudioCodec::Opus => "opus",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{cli:?}");

    match cli.commands {
        Commands::Audio { in_out } => {
            let audio = Audio::new(in_out)?;

            audio.run();
        }
        Commands::Clip {
            in_out,
            start,
            end,
            gif,
        } => {
            todo!()
        }
        Commands::Convert { in_out, subcmd } => {
            todo!()
        }
        Commands::Flip { in_out } => {
            todo!()
        }
        Commands::Merge { in_out } => {
            todo!()
        }
        Commands::Upscale { in_out } => {
            todo!()
        }
        Commands::Video { in_out, framerate } => {
            todo!()
        }
        Commands::Youtube { in_out } => {
            todo!()
        }
    }

    Ok(())
}
