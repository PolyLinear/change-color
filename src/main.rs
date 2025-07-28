use jpeg_decoder::Decoder;
use lofty::{config::ParseOptions, file::AudioFile, flac::FlacFile, ogg::OggPictureStorage};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

struct Thumbnail {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

macro_rules! interpolation {
    ($a:expr, $b:expr, $t:expr) => {
        $a * (1.0 - $t) + $b * $t
    };
}

fn color_map(data: &mut [u8]) {
    let rl = 0xae as f32;
    let gl = 0x98 as f32;
    let bl = 0xb5 as f32;
    let r = 0x0 as f32;
    data.chunks_mut(3).for_each(|rgb| {
        let lum =
            ((0.30 * rgb[0] as f32) + (0.59 * rgb[1] as f32) + (0.11 * rgb[2] as f32)) / 255.0;
        rgb[0] = interpolation!(r, rl, lum) as u8;
        rgb[1] = interpolation!(r, gl, lum) as u8;
        rgb[2] = interpolation!(r, bl, lum) as u8;
    });
}

fn resize_image(input: &Thumbnail, width: usize, height: usize) -> Vec<u8> {
    let w_factor: f32 = ((input.width - 1) as f32) / ((width - 1) as f32);
    let h_factor: f32 = ((input.height - 1) as f32) / ((height - 1) as f32);
    let mut pixel_data = vec![0; width * height * 3];

    for i in 0..width * height {
        //(y, x) position of input image
        let x: f32 = (i % width) as f32 * w_factor;
        let y: f32 = (i / width) as f32 * h_factor;

        //neighboring pixels in 2d space
        let x_0 = x as usize;
        let y_0 = y as usize;
        let x_1 = (x_0 + 1).clamp(0, input.width - 1);
        let y_1 = (y_0 + 1).clamp(0, input.height - 1);

        //distance between neighboring pixels
        let dx = x - x_0 as f32;
        let dy = y - y_0 as f32;

        //absolute position of neighboring pixels in array [topleft, topright, bottomleft,
        //bottomright]
        let tl = (y_0 * input.width + x_0) * 3;
        let tr = (y_0 * input.width + x_1) * 3;
        let bl = (y_1 * input.width + x_0) * 3;
        let br = (y_1 * input.width + x_1) * 3;

        //loop through RGB channels and apply bilinear interpolation
        for c in 0..3 {
            let tl_p: f32 = input.data[tl + c] as f32;
            let tr_p: f32 = input.data[tr + c] as f32;
            let bl_p: f32 = input.data[bl + c] as f32;
            let br_p: f32 = input.data[br + c] as f32;

            let top = interpolation!(tl_p, tr_p, dx);
            let bottom = interpolation!(bl_p, br_p, dx);
            pixel_data[i * 3 + c] = interpolation!(top, bottom, dy) as u8;
        }
    }

    pixel_data
}

//TODO: Fix the proof of concept
fn main() -> Result<(), Box<dyn Error>> {
    //NOTE: test flac is not included in the repo
    let mut file = File::open("src/test.flac")?;
    let flac_file = FlacFile::read_from(&mut file, ParseOptions::new())?;

    let image = flac_file
        .pictures()
        .iter()
        .next()
        .ok_or("Failed to get image")?;

    //TODO: implement support for png (trivial)
    let mut decoder = Decoder::new(image.0.data())
        .decode()
        .expect("Failed to read byte array");
    let (width, height, depth) = (image.1.width, image.1.height, image.1.color_depth);

    let mut data = Thumbnail {
        data: decoder,
        width: width as usize,
        height: height as usize,
    };

    color_map(&mut data.data);
    let mut vec = resize_image(&data, 256, 256);

    let writer = BufWriter::new(File::create("bilinear.png")?);

    let mut encoder = png::Encoder::new(writer, 256, 256);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&vec).unwrap();

    Ok(())
}
