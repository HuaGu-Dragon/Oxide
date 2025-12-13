use std::io::Read;

use anyhow::Context;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() -> anyhow::Result<()> {
    enable_raw_mode().context("enable raw mode in terminal")?;

    for input in std::io::stdin().lock().bytes() {
        let input = input.context("read input")?;
        println!("Input: {}", input as char);
        if input == b'q' {
            disable_raw_mode().context("disable raw mode in terminal")?;
            break;
        }
    }

    Ok(())
}
