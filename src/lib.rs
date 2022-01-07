pub mod io;

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

// The cube is indexed as follows:
// for dim in 0..3
//     for sign in [low, hi]
//         for x in 0..width
//             for y in 0..width
// Each of the three dimensions contains two faces (positive and negative sign) and then x, y
// Dimensions:
// 0: [u, v, sign]
// 1: [sign, u, v]
// 2: [v, sign, u]

/// Return the index of the pixel at the given face and coordinates
pub fn cube_pixel_idx_in_bounds(u: usize, v: usize, sign: bool, dim: usize, width: usize) -> usize {
    assert!(u < width);
    assert!(v < width);
    assert!(dim < 3);
    let face_stride = width * width;
    let dim_base = dim * face_stride * 2;
    let face_base = dim_base + if sign { face_stride } else { 0 };
    let row_base = face_base + v * width;
    row_base + u
}

/// Return the index of the pixel at the given face and coordinates, handling boundary cases
/// Returns None when the given index does not exist
pub fn cube_pixel_idx_out_bounds(
    u: isize,
    v: isize,
    sign: bool,
    dim: usize,
    width: usize,
) -> Option<usize> {
    let is_in_bounds = |x| x < 0 || x >= width as isize;

    match (is_in_bounds(u), is_in_bounds(v)) {
        // In bounds
        (false, false) => Some(cube_pixel_idx_in_bounds(u as _, v as _, sign, dim, width)),

        // Out of bounds on U and V
        (true, true) => None,

        // Out of bounds in just one dimension
        (u_in_bounds, _) => {
            // The new orientation has sign in the same place as the out of bounds dimension
            let off = if u_in_bounds { 2 } else { 1 };
            let new_dim = (off + dim) % 3;

            // Create an array representation of our input coordinates
            let repr = [u, v, if sign { width - 1 } else { 0 } as isize];

            // Rotate the coordinates based off of where the new sign bit goes
            let mut unbiased = [0; 3];
            for i in 0..3 {
                unbiased[(i + off) % 3] = repr[i];
            }

            // Interpret the rotated coordinates
            let [u, v, sign] = unbiased;
            Some(cube_pixel_idx_in_bounds(
                u as _,
                v as _,
                sign > 0,
                new_dim,
                width,
            ))
        }
    }
}

pub fn step(read: &GolCube, write: &mut GolCube, corner_val: bool) {
    assert_eq!(read.width, write.width);
    let width = read.width;

    let mut neighborhood = vec![];
    for du in -1..=1 {
        for dv in -1..=1 {
            if !(du == 0 && dv == 0) {
                neighborhood.push((du, dv));
            }
        }
    }

    for dim in 0..3 {
        for sign in [false, true] {
            for u in 0..width {
                for v in 0..width {
                    let mut neighbors = 0;
                    for &(du, dv) in &neighborhood {
                        let iu = u as isize + du;
                        let iv = v as isize + dv;
                        if let Some(idx) = cube_pixel_idx_out_bounds(iu, iv, sign, dim, width) {
                            neighbors += read.data[idx] as usize;
                        } else {
                            neighbors += corner_val as usize;
                        }
                    }

                    let center_idx = cube_pixel_idx_in_bounds(u, v, sign, dim, width);
                    let center = read.data[center_idx];
                    write.data[center_idx] = match (center, neighbors) {
                        (true, n) if (n == 2 || n == 3) => true,
                        (false, n) if (n == 3) => true,
                        _ => false,
                    };
                }
            }
        }
    }
}
