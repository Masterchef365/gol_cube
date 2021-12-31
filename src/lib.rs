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

/// Return the index of the pixel at the given face and coordinates
pub fn cube_pixel_idx_in_bounds(u: usize, v: usize, sign: bool, dim: usize, width: usize) -> usize {
    let face_stride = width * width;
    let dim_base = dim * face_stride * 2;
    let face_base = dim_base + if sign { face_stride } else { 0 };
    let row_base = face_base + v * width;
    row_base + u
}

/// Return the index of the pixel at the given face and coordinates, handling boundary cases
/// Returns None when the given index does not exist
pub fn cube_pixel_idx_out_bounds(u: isize, v: isize, sign: bool, dim: usize, width: usize) -> Option<usize> {
    let is_in_bounds = |x| x < 0 || x >= width as isize;

    match (is_in_bounds(u), is_in_bounds(v)) {
        // In bounds
        (false, false) => Some(cube_pixel_idx_in_bounds(u as _, v as _, sign, dim, width)),

        // Out of bounds on U and V
        (true, true) => None,

        // Out of bounds in just one dimension
        (u_or_v, _) => {
            let repr = [u, v, if sign { width - 1 } else { 0 } as isize];
            let off = if u_or_v { 1 } else { 2 };

            let mut unbiased = [0; 3];
            for i in 0..3 {
                unbiased[i] = repr[(i + dim + off) % 3];
            }

            let [u, v, sign] = unbiased;
            Some(cube_pixel_idx_in_bounds(u as _, v as _, sign > 0, (off + dim) % 3, width))
        }
    }
}
