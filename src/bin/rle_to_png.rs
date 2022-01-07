use gol_cube::io::*;
use std::path::PathBuf;
use anyhow::{Result, Context};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().context("Requires path")?;
    let (data, width) = load_rle(&path)?;

    let mut out_path = path;
    out_path.push_str(".png");
    write_png_binary(out_path, &data, width)
}
