use std::path::PathBuf;

use anyhow::{Context, Result};
use smallvec::SmallVec;

use crate::InOutOption;

pub struct Audio<'a> {
    pub args: SmallVec<[&'a str; 16]>,
}

impl<'a> Audio<'a> {
    pub fn new(in_out: InOutOption) -> Result<Self> {
        let (inputs, outputs) = in_out.split();

        if inputs.len() > 1 {
            println!(
                "WARN: More than one input entered, skipping [{}]. Taking only the last one.",
                inputs.len() - 1
            );
        }
        let input = inputs.pop()?.context("Error taking input");

        // TODO:
        // Check if outputs is_some()
        // Some(outputs) => check if outputs.len() == inputs.len()
        //                        if true continue
        //                        else warn, we only need 1 input for audio
        // None => generate a name based off input
        //         e.g.:
        //         let input = "helloworld.mp4"
        //         let output = format!("OUT_{input}_audio.mp3");
        // NOTE: we should pick the handle the extension instead
        //       of hard-coding in the format!

        let output = if let Some(outputs) = outputs {
            if outputs.len() > 1 {
                println!(
                    "More than one output entered, skipping... [{}]",
                    outputs.len() - 1
                );
            }
            outputs.pop()
        } else {
            let output = PathBuf::new();
            output.push(format!("OUT_{}_audio", input.clone()));
            output
        };

        Ok(Self {
            args: SmallVec::new(),
        })
    }

    pub fn run(self) {
        todo!();
    }
}
