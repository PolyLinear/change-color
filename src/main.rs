mod image;

use image::thumbnail::Thumbnail;

use jpeg_decoder::Decoder;
use lofty::{config::ParseOptions, file::AudioFile, flac::FlacFile, ogg::OggPictureStorage};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

const SIZE: usize = 128;
fn main() -> Result<(), Box<dyn Error>> {
    //NOTE: test flac is not included in the repo
    let mut file = File::open("src/test.flac")?;
    let flac_file = FlacFile::read_from(&mut file, ParseOptions::new())?;

    let flac_image = flac_file
        .pictures()
        .iter()
        .next()
        .ok_or("Failed to get image")?;

    //TODO: implement support for png (trivial)
    let image_pixels = Decoder::new(flac_image.0.data())
        .decode()
        .expect("Failed to read byte array");

    let icon = Thumbnail::from_byte_arr(
        flac_image.1.width as usize,
        flac_image.1.height as usize,
        image_pixels,
    )
    .color_map(0xae98b5, 0x0)
    .resize_image(SIZE, SIZE);

    let writer = BufWriter::new(File::create("bilinear.png")?);
    let mut encoder = png::Encoder::new(writer, SIZE as u32, SIZE as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(icon.get_bytes()))
        .unwrap();

    Ok(())
}
