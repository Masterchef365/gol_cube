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
        let vertices = golcube_vertices(20);
        let indices: Vec<u32> = (0..vertices.len() as u32).collect();
        dbg!(vertices.len());

        Ok(Self {
            points_shader: ctx.shader(
                DEFAULT_VERTEX_SHADER,
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Points,
            )?,
            verts: ctx.vertices(&vertices, false)?,
            indices: ctx.indices(&indices, false)?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)
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
                        color: pos.map(|v| if v > 0. { v } else { -v * 0.8 }),
                    });
                }
            }
        }
    }

    vertices
}

fn golcube_indices(cube: &GolCube) -> Vec<u32> {
    todo!()
}
