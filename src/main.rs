use gol_cube::GolCube;
use std::fs::File;
use anyhow::{Context as AnyhowContext, Result, ensure, bail};
use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};
use rand::prelude::*;
use structopt::StructOpt;
use std::path::{PathBuf, Path};

#[derive(StructOpt, Default)]
#[structopt(name = "Conway's Game of Life on da cube", about = "what do you think")]
struct Opt {
    /// Visualize in VR
    #[structopt(short, long)]
    vr: bool,

    /// Cube width
    #[structopt(short, long, default_value = "100")]
    width: usize,

    /// update interval
    #[structopt(short, long, default_value = "1")]
    interval: usize,

    /// Fill percentage for the initial value
    #[structopt(short, long, default_value = "0.25")]
    rand_p: f64,

    /// Seed. If unspecified, random seed
    #[structopt(short, long)]
    seed: Option<u64>,

    /// Seed. If unspecified, random seed
    #[structopt(long)]
    sphere: bool,

    /// The missing values on corners are true instead of false if this is set
    #[structopt(long)]
    corner_true: bool,

    /// Import a PNG image, supercedes width
    #[structopt(long)]
    import: Option<PathBuf>,

    /// Export a PNG image on quit
    #[structopt(long)]
    export: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    launch::<Opt, GolCubeVisualizer>(Settings::default().vr(opt.vr).args(opt))
}

struct GolCubeVisualizer {
    verts: VertexBuffer,
    indices: IndexBuffer,
    points_shader: Shader,
    camera: MultiPlatformCamera,

    front: GolCube,
    back: GolCube,

    opt: Opt,
    frame: usize,
}

impl App<Opt> for GolCubeVisualizer {
    fn init(ctx: &mut Context, platform: &mut Platform, opt: Opt) -> Result<Self> {
        let mut vertices = golcube_vertices(opt.width);

        if opt.sphere {
            vertices.iter_mut().for_each(|v| {
                let len = v.pos.iter().map(|x| x * x).sum::<f32>().sqrt();
                v.pos = v.pos.map(|x| x / len);
            });
        }


        let indices = golcube_dummy_tri_indices(opt.width);

        let seed = opt.seed.unwrap_or_else(|| rand::thread_rng().gen());
        println!("Using seed {}", seed);
        let mut rng = SmallRng::seed_from_u64(seed);

        let mut front;
        if let Some(import_path) = opt.import {
            //import_golcube_png(import_path)?
            unimplemented!("Imports currently unsupported!")
        } else {
            front = GolCube::new(opt.width);
            front
                .data
                .iter_mut()
                .for_each(|px| *px = rng.gen_bool(opt.rand_p));
        }

        Ok(Self {
            front,
            back: GolCube::new(opt.width),
            points_shader: ctx.shader(
                DEFAULT_VERTEX_SHADER,
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Triangles,
            )?,
            verts: ctx.vertices(&vertices, false)?,
            indices: ctx.indices(&indices, true)?,
            camera: MultiPlatformCamera::new(platform),
            opt,
            frame: 0,
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        if self.frame % self.opt.interval == 0 {
            std::mem::swap(&mut self.front, &mut self.back);
            gol_cube::step(&self.back, &mut self.front, self.opt.corner_true);
        }
        self.frame += 1;

        let indices = golcube_tri_indices(&self.front);
        ctx.update_indices(self.indices, &indices)?;

        Ok(vec![DrawCmd::new(self.verts)
            .limit(indices.len() as _)
            .indices(self.indices)
            .shader(self.points_shader)])
    }

    fn event(
        &mut self,
        ctx: &mut Context,
        platform: &mut Platform,
        mut event: Event,
    ) -> Result<()> {
        if self.camera.handle_event(&mut event) {
            ctx.set_camera_prefix(self.camera.get_prefix())
        }
        match (event, platform) {
        (
            Event::Winit(idek::winit::event::Event::WindowEvent {
                event: idek::winit::event::WindowEvent::CloseRequested,
                ..
            }),
            Platform::Winit { control_flow, .. },
        ) => {
            if let Some(export_path) = self.opt.export.as_ref() {
                write_png_mono(export_path, &self.front.data, self.front.width)?;
            }

            **control_flow = idek::winit::event_loop::ControlFlow::Exit
        }
        _ => (),
    }
        Ok(())
    }
}

fn golcube_vertices(width: usize) -> Vec<Vertex> {
    let mut vertices = vec![];
    const MIN: f32 = -1.0;
    const MAX: f32 = 1.0;

    let idx_to_world = |i| (i as f32 / width as f32) * (MAX - MIN) + MIN;

    for dim in 0..3 {
        for z in [MIN, MAX] {
            for x in 0..=width {
                let x = idx_to_world(x);
                for y in 0..=width {
                    let y = idx_to_world(y);
                    let mut pos = [0.0; 3];
                    for i in 0..3 {
                        pos[(dim + i) % 3] = [x, y, z][i];
                    }
                    vertices.push(Vertex {
                        pos,
                        color: pos.map(|v| if v > 0. { v } else { -v * 0.05 }),
                        //color: [1.; 3],
                    });
                }
            }
        }
    }

    vertices
}

/*
fn golcube_dense_line_indices(width: usize) -> Vec<u32> {
    let mut indices = vec![];
    let width = width as u32;
    let stride = width + 1;

    let face_stride = stride * stride;
    for face in 0..6 {
        let base = face * face_stride;
        for x in 0..stride {
            indices.push(base + x);
            indices.push(base + x + face_stride - stride);
            indices.push(base + x * stride);
            indices.push(base + x * stride + width);
        }
    }

    indices
}
*/

fn golcube_tri_indices(cube: &GolCube) -> Vec<u32> {
    let mut indices = vec![];
    let idx_stride = cube.width as u32 + 1;

    let face_data_stride = cube.width * cube.width;
    let face_idx_stride = idx_stride * idx_stride;

    for (face_idx, face) in cube.data.chunks_exact(face_data_stride).enumerate() {
        let mut backface = |[a, b, c]: [u32; 3]| {
            indices.extend_from_slice(&[a, b, c]);
            indices.extend_from_slice(&[c, b, a]);
            /*
            if face_idx % 2 != 0 {
                indices.extend_from_slice(&[a, b, c]);
            } else {
                indices.extend_from_slice(&[c, b, a]);
            }
            */
        };

        let face_base = face_idx as u32 * face_idx_stride;
        for (y, row) in face.chunks_exact(cube.width).enumerate() {
            let row_base = face_base + y as u32 * idx_stride;
            for (x, &elem) in row.iter().enumerate() {
                let elem_idx = row_base + x as u32;
                if elem {
                    backface([elem_idx + idx_stride, elem_idx + 1, elem_idx]);

                    backface([
                        elem_idx + idx_stride,
                        elem_idx + idx_stride + 1,
                        elem_idx + 1,
                    ]);
                }
            }
        }
    }

    indices
}

fn golcube_dummy_tri_indices(width: usize) -> Vec<u32> {
    //(0..width * width * 6 * 3 * 2).map(|_| 0).collect()
    (0..width * width * 6 * 3 * 2 * 2).map(|_| 0).collect()
}

fn import_golcube_png() {
}

/// Returns (width, rgb data) for the given PNG image reader
/*
fn load_png_mono(path: impl AsRef<Path>) -> Result<(usize, Vec<bool>)> {
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info().context("Creating reader")?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).context("Reading frame")?;

    if info.bit_depth != png::BitDepth::Eight {
        bail!("Bit depth {:?} unsupported!", info.bit_depth);
    }

    buf.truncate(info.buffer_size());

    let buf: Vec<u8> = match info.color_type {
        png::ColorType::Rgb => buf,
        png::ColorType::Rgba => buf
            .chunks_exact(4)
            .map(|px| [px[0], px[1], px[2]])
            .flatten()
            .collect(),
        png::ColorType::Grayscale => buf.iter().map(|&px| [px; 3]).flatten().collect(),
        png::ColorType::GrayscaleAlpha => {
            buf.chunks_exact(2).map(|px| [px[0]; 3]).flatten().collect()
        }
        other => bail!("Images with color type {:?} are unsupported", other),
    };

    Ok((info.width as usize, buf))
}
*/

/// Writes the given RGB data to a PNG file
fn write_png_mono(path: impl AsRef<Path>, buf: &[bool], width: usize) -> Result<()> {
    ensure!(buf.len() % width == 0, "Image data must be divisible by width");
    let height = buf.len() / width;

    let file = std::fs::File::create(path)?;
    let ref mut w = std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width as _, height as _);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let buf: Vec<u8> = buf.iter().copied().map(|v| if v { 0xff } else { 0x00 }).collect();

    writer.write_image_data(&buf)?;

    Ok(())
}
