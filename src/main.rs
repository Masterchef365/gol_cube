use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};

fn main() -> Result<()> {
    launch::<_, GolCubeVisualizer>(Settings::default().vr_if_any_args())
}

struct GolCubeVisualizer {
    verts: VertexBuffer,
    indices: IndexBuffer,
    points_shader: Shader,
    camera: MultiPlatformCamera,
}

impl App for GolCubeVisualizer {
    fn init(ctx: &mut Context, platform: &mut Platform, _: ()) -> Result<Self> {
        let width = 20;
        let vertices = golcube_vertices(width);
        //let indices: Vec<u32> = (0..vertices.len() as u32).collect();
        let indices = golcube_dummy_tri_indices(width);

        Ok(Self {
            points_shader: ctx.shader(
                DEFAULT_VERTEX_SHADER,
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Triangles,
            )?,
            verts: ctx.vertices(&vertices, false)?,
            indices: ctx.indices(&indices, true)?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        let mut cube = GolCube::new(20);
        cube.data.fill(true);
        let indices = golcube_tri_indices(&cube);
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

struct GolCube {
    pub data: Vec<bool>,
    pub width: usize,
}

impl GolCube {
    pub fn new(width: usize) -> Self {
        Self {
            data: vec![false; 6 * width * width],
            width,
        }
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
                        color: pos.map(|v| if v > 0. { v } else { -v * 0.3 }),
                        //color: [1.; 3],
                    });
                }
            }
        }
    }

    vertices
}

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

fn golcube_tri_indices(cube: &GolCube) -> Vec<u32> {
    let mut indices = vec![];
    let idx_stride = cube.width as u32 + 1;

    let face_data_stride = cube.width * cube.width;
    let face_idx_stride = idx_stride * idx_stride;

    for (face_idx, face) in cube.data.chunks_exact(face_data_stride).enumerate() {
        let face_base = face_idx as u32 * face_idx_stride;
        for (y, row) in face.chunks_exact(cube.width).enumerate() {
            let row_base = face_base + y as u32 * idx_stride;
            for (x, &elem) in row.iter().enumerate() {
                let elem_idx = row_base + x as u32;
                if elem {
                    indices.push(elem_idx + face_idx_stride);
                    indices.push(elem_idx + 1);
                    indices.push(elem_idx);

                    indices.push(elem_idx + face_idx_stride);
                    indices.push(elem_idx + face_idx_stride + 1);
                    indices.push(elem_idx + 1);
                }
            }
        }
    }

    indices
}

fn golcube_dummy_tri_indices(width: usize) -> Vec<u32> {
    (0..width * width * 6 * 3 * 2).map(|_| 0).collect()
}
