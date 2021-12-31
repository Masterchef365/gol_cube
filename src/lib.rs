pub struct GolCube {
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

fn step(cube: &mut GolCube) {}

// The cube is indexed as follows:
// for dim in 0..3
//     for sign in [low, hi]
//         for x in 0..width
//             for y in 0..width
// Each of the three dimensions contains two faces (positive and negative sign) and then x, y

/// Return the index of the pixel at the given
pub fn cube_pixel_idx_in_bounds(u: usize, v: usize, sign: bool, dim: usize, width: usize) -> usize {
    let face_stride = width * width;
    let dim_base = dim * face_stride * 2;
    let face_base = dim_base + if sign { face_stride } else { 0 };
    let row_base = face_base + v * width;
    row_base + u
}
