use crate::GolCube;
use anyhow::{bail, ensure, format_err, Context as AnyhowContext, Result};
use std::fs::File;
use std::path::Path;
use std::iter::repeat;

/// Load a GolCube from a file.
pub fn import_golcube_png(path: impl AsRef<Path>) -> Result<GolCube> {
    let (width, data) = load_png_binary(path)?;
    ensure!(data.len() == width * width * 6);
    Ok(GolCube { data, width })
}

/// Export a GolCube to a file
pub fn export_golcube_png(path: impl AsRef<Path>, cube: &GolCube) -> Result<()> {
    write_png_binary(path, &cube.data, cube.width)
}

/// Imports the given RLE onto one face of a GolCube
pub fn import_golcube_rle(path: impl AsRef<Path>) -> Result<GolCube> {
    let (rle_data, width) = load_rle(path)?;
    let height = rle_data.len() / width;

    // Fit the whole pattern on one face
    let side_len = width.max(height);

    // Populate the face
    let mut out_data = vec![];
    for row in rle_data.chunks_exact(width) {
        let row_len = row.len(); 
        out_data.extend_from_slice(&row);
        out_data.extend(repeat(false).take(side_len - row_len));
    }

    // Fill the rest with zeroes
    let out_len = out_data.len();
    out_data.extend(repeat(false).take(side_len * side_len * 6 - out_len));

    Ok(GolCube { 
        data: out_data,
        width: side_len,
    })
}

/// Import an RLE file
pub fn load_rle(path: impl AsRef<Path>) -> Result<(Vec<bool>, usize)> {
    let text = std::fs::read_to_string(path)?;
    let mut lines = text.lines();

    // Find header
    let header_line = loop {
        let line = lines.next().ok_or(format_err!("Missing header"))?;
        if line.starts_with('#') {
            continue;
        } else {
            break line;
        }
    };

    // Parse header
    let header_err = || format_err!("Header failed to parse");
    let mut sections = header_line.split(',');

    let mut parse_section = |prefix: &str| -> Result<usize> {
        let sec = sections.next().ok_or_else(header_err)?;
        let mut halves = sec.split('=');

        let var_name = halves.next().ok_or_else(header_err)?.trim();
        let value = halves.next().ok_or_else(header_err)?.trim();

        if var_name != prefix {
            return Err(header_err());
        } else {
            Ok(value.parse()?)
        }
    };

    let (width, height) = (parse_section("x")?, parse_section("y")?);


    // Load data
    let mut data = vec![];

    let mut digits: String = "".into();
    let mut x = 0;

    'lines: for line in lines {
        for c in line.trim().chars() {
            match c {
                'b' | 'o' => {
                    let n = digits.parse().unwrap_or(1);
                    digits.clear();
                    x += n;
                    if x > width {
                    }
                    data.extend(repeat(c == 'o').take(n));
                }
                '$' | '!' => {
                    match width.checked_sub(x) {
                        None => bail!("Pattern exceeds width!"),
                        Some(filler) => data.extend(repeat(false).take(filler)),
                    }
                    digits.clear();
                    x = 0;
                    if c == '!' {
                        break 'lines;
                    }
                }
                c if c.is_digit(10) => digits.push(c),
                c if c.is_whitespace() => (),
                _ => bail!("Unrecognized character {}", c),
            }
        }
    }

    let len = data.len();
    data.extend(repeat(false).take((width * height).checked_sub(len).unwrap()));

    /*if data.len() !=  {
        bail!(
            "Data length does not match header! {} vs {} * {} = {}",
            data.len(),
            width,
            height,
            width * height
        );
    }*/

    Ok((data, width))
}

/// Returns (width, mono data) for the given PNG image reader
pub fn load_png_binary(path: impl AsRef<Path>) -> Result<(usize, Vec<bool>)> {
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info().context("Creating reader")?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).context("Reading frame")?;

    if info.bit_depth != png::BitDepth::Eight {
        bail!("Bit depth {:?} unsupported!", info.bit_depth);
    }

    buf.truncate(info.buffer_size());

    // Check if the first component of each pixel is > 0
    let buf = buf
        .into_iter()
        .step_by(info.color_type.samples())
        .map(|v| v > 0)
        .collect();

    Ok((info.width as usize, buf))
}

/// Writes the given RGB data to a PNG file
pub fn write_png_binary(path: impl AsRef<Path>, buf: &[bool], width: usize) -> Result<()> {
    ensure!(
        buf.len() % width == 0,
        "Image data must be divisible by width"
    );
    let height = buf.len() / width;

    let file = std::fs::File::create(path)?;
    let ref mut w = std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width as _, height as _);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let buf: Vec<u8> = buf
        .iter()
        .copied()
        .map(|v| if v { 0xff } else { 0x00 })
        .collect();

    writer.write_image_data(&buf)?;

    Ok(())
}
