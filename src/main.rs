use clap::{Parser, Subcommand};

use std::process::Command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Get {
        #[arg(required = true)]
        value: String,
    },
    Set {
        #[arg(required = true)]
        value: String,
    },
}

fn main() {
    //let args = Cli::parse();

    //let mut v = "hello".to_string();

    //match args.cmd {
    //    Commands::Get { ref value } => {
    //        println!("Getting {value}");
    //        if *value == v {
    //            println!("Here's the value {v}");
    //        } else {
    //            println!("This value doesn't exist.");
    //        }
    //    }

    //    Commands::Set { ref value } => {
    //        println!("Setting value to {value}");
    //        println!("Old value {v}");
    //        v = value.clone();
    //        println!("New value {v}");
    //    }
    //}

    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout))
}
