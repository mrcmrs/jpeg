mod parsing;
mod huffman;
mod scan;
mod transf;

use std::fs;
use image::{ImageBuffer, RgbImage};

fn main() {
    // let img_path = "img/white_square.jpg";
    // let img_path = "img/white_square_16x16.jpg";
    let img_path = "img/sq16do.jpg";
    let data: Vec<u8> = fs::read(img_path).unwrap();
    let mut segments = parsing::parse(&data);

    // dbg!(&segments.start_of_frame);
    // dbg!(&segments.quantization_tables[0]);
    // dbg!(&segments.quantization_tables[1]);
    
    let res = transf::get_mcus(&mut segments);

    for array in res {
        let mut img: RgbImage = ImageBuffer::new(16, 16);

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let rgb = array[y as usize][x as usize]; // Access pixel data
            *pixel = image::Rgb(rgb);
        }

        img.save("img/output.png").expect("Failed to save image");

        println!("{:?}", array);
        println!();
    }
    

    // Create a new RgbImage (16x16)
    

    // dbg!(&segments.huffman_tables[0].tree);

}

