use crate::scan;
use crate::parsing;


const ZIGZAG: [[usize; 8]; 8] = [
    [ 0,  1,  5,  6, 14, 15, 27, 28],
    [ 2,  4,  7, 13, 16, 26, 29, 42],
    [ 3,  8, 12, 17, 25, 30, 41, 43],
    [ 9, 11, 18, 24, 31, 40, 44, 53],
    [10, 19, 23, 32, 39, 45, 52, 54],
    [20, 22, 33, 38, 46, 51, 55, 60],
    [21, 34, 37, 47, 50, 56, 59, 61],
    [35, 36, 48, 49, 57, 58, 62, 63],
];

const PI: f32 = std::f32::consts::PI;

pub fn dequantize(values: &[i16; 64], table: &[u8; 64]) -> [[i32; 8]; 8] {
    let mut res = [[0; 8]; 8];

    for (v, row) in ZIGZAG.iter().enumerate() {
        for (u, &i) in row.iter().enumerate() {
            res[v][u] = values[i] as i32 * table[i] as i32;
        }
    }
    res
}

fn cu_cv(u: usize, v: usize) -> f32 {
    if u == 0 && v == 0 {
        0.5
    } else {
        1.
    }
}


fn cos(a: usize, b: usize) -> f32 {
    f32::cos((2. * a as f32 + 1.)*b as f32 * PI /16.)
}

fn syx(y: usize, x: usize, svu: [[i32; 8]; 8]) -> f32 {
    let mut res = 0.;
    for u in 0..8 {
        for v in 0..8 {
            res += cu_cv(u, v) * svu[v][u] as f32 * cos(x, u) * cos(y, v);
        }
    }
    res / 4.
}

pub fn idct(input: [[i32; 8]; 8]) -> [[f32; 8]; 8] {
    let mut res = [[0.; 8]; 8];

    for y in 0..8 {
        for x in 0..8 {
            res[y][x] = syx(y, x, input) + 128.;
        }
    } 
    res
}

pub fn ycbcr_to_rgb(y: f32, cb: f32, cr: f32) -> [u8; 3] {
    let cb = cb - 128.;
    let cr = cr - 128.;
    let red = y + 1.402 * cr;
    let green = y +  -0.344136 * cb - 0.714136 * cr;
    let blue = y +  1.772 * cb;

    let red = red.clamp(0., 255.) as u8;
    let green = green.clamp(0., 255.) as u8;
    let blue = blue.clamp(0., 255.) as u8;
    [red, green, blue]
}

fn lum_idx(i: usize, j: usize) -> usize {
    (j >> 3) + ((i >> 3) << 1)
}

pub fn mcu_to_rgb(mcu: Vec<[[f32; 8]; 8]>) -> [[[u8; 3]; 16]; 16] {
    let mut ycbcr = [[[0; 3]; 16]; 16];
    for i in 0..16 {
        for j in 0..16 {
            let lum = mcu[lum_idx(i, j)][i % 8][j % 8];
            let cb = mcu[4][i / 2][j / 2];
            let cr = mcu[5][i / 2][j / 2];
            ycbcr[i][j] = ycbcr_to_rgb(lum, cb, cr);
        }
    }
    ycbcr
}

pub fn get_mcus(segments: &mut parsing::Segments) -> Vec<[[[u8; 3]; 16]; 16]> {
    let mut res = Vec::new();

    let vec = scan::scan_blocks(segments);

    let mut count = 0;
    
    let mut mcu = Vec::new();
    let mut q_table;
    
    
    for array in vec {
        if count < 4 {
            q_table = &segments.quantization_tables[0].table;
        } else {
            q_table = &segments.quantization_tables[1].table;
        } 
        let mat = dequantize(&array, q_table);
        let mat = idct(mat);
        mcu.push(mat);
        count += 1;

        if count >= 6 {
            res.push(mcu_to_rgb(mcu));
            count = 0;
            mcu = Vec::new();
        }
    }

    res
}


pub fn mcus_to_img(mcus: Vec<[[[u8; 3]; 16]; 16]>, height: u16, width: u16) -> Vec<Vec<[u8; 3]>> {
    let mut img = Vec::new();  
    let h_mcus = usize::div_ceil(width as usize, 16);

    for i in 0..height as usize {
        let mut row = Vec::new();
        for j in 0..width as usize {
            let mcus_idx = (j >> 4) + (i >> 4) * h_mcus;
            let mcu_i = i % 16;
            let mcu_j = j % 16;
            let pixel = mcus[mcus_idx][mcu_i][mcu_j];
            row.push(pixel);
        }
        img.push(row);
    }
    img
}

#[cfg(test)]
mod test {
    // use super::*;

    #[test]
    fn test_n_mcus() {
        
    }
}