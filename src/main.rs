use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};
use gol_cube::GolCube;
use structopt::StructOpt;

#[derive(StructOpt, Default)]
#[structopt(name = "Conway's Game of Life on da cube", about = "what do you think")]
struct Opt {
    /// Visualize in VR
    #[structopt(short, long)]
    vr: bool,

    /// Cube width
    #[structopt(short, long, default_value="100")]
    width: usize,

    /// Update interval
    #[structopt(short, long, default_value="1")]
    interval: usize,
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
        let vertices = golcube_vertices(opt.width);
        let indices = golcube_dummy_tri_indices(opt.width);

        let mut rng = rand::thread_rng();
        use rand::prelude::*;
        let mut front = GolCube::new(opt.width);
        for _ in 0..front.data.len() / 2 {
            *front.data.choose_mut(&mut rng).unwrap() = true;
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
            gol_cube::step(&self.back, &mut self.front);
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
        idek::close_when_asked(platform, &event);
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
