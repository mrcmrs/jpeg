mod parsing;
mod huffman;
mod scan;
mod transf;

use std::fs;
use image::{ImageBuffer, RgbImage};

pub fn get(img_path: &str) -> Vec<Vec<[u8; 3]>> {
    let data: Vec<u8> = fs::read(img_path).unwrap();
    let mut segments = parsing::parse(&data);

    let width = segments.start_of_frame.as_ref().unwrap().width;
    let height = segments.start_of_frame.as_ref().unwrap().height;

    let res = transf::get_mcus(&mut segments);
    transf::mcus_to_img(res, height, width)
}

pub fn save(pic: Vec<Vec<[u8; 3]>>) {
    let height = pic.len();
    let width = pic[0].len();

    let mut img: RgbImage = ImageBuffer::new(width as u32, height as u32);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let rgb = pic[y as usize][x as usize];
        *pixel = image::Rgb(rgb);
    }
    img.save("img/output.png").expect("Failed to save image");
}

