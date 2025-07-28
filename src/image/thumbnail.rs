macro_rules! interpolation {
    ($a:expr, $b:expr, $t:expr) => {
        $a * (1.0 - $t) + $b * $t
    };
}

macro_rules! get_rgb {
    ($num:expr) => {
        (
            (($num >> 16) & 0xff) as f32,
            (($num >> 8) & 0xff) as f32,
            ($num & 0xff) as f32,
        )
    };
}

/*
* Thumbnail follows the builder paradaigm, methods are meant to be chained to perform manipulations
* to the image source
*/

#[derive(Default)]
pub struct Thumbnail {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl Thumbnail {
    pub fn new(width: usize, height: usize) -> Self {
        Thumbnail {
            data: vec![0; width * height * 3],
            width,
            height,
        }
    }

    pub fn from_byte_arr(width: usize, height: usize, data: Vec<u8>) -> Self {
        Thumbnail {
            data,
            width,
            height,
        }
    }

    //TODO: possible optimization, original image can be used in place if the downscaling instead
    //of upscaling -> can reuse the original buffer, update width and heigh, and return self
    pub fn resize_image(&self, width: usize, height: usize) -> Self {
        let w_factor: f32 = ((self.width - 1) as f32) / ((width - 1) as f32);
        let h_factor: f32 = ((self.height - 1) as f32) / ((height - 1) as f32);

        let mut result = Thumbnail::new(width, height);

        for i in 0..width * height {
            let x: f32 = (i % width) as f32 * w_factor;
            let y: f32 = (i / width) as f32 * h_factor;

            //neighboring pixels in 2d space
            let x_0 = x as usize;
            let y_0 = y as usize;
            let x_1 = (x_0 + 1).clamp(0, self.width - 1);
            let y_1 = (y_0 + 1).clamp(0, self.height - 1);

            //distance between neighboring pixels
            let dx = x - x_0 as f32;
            let dy = y - y_0 as f32;

            //absolute position of neighboring pixels in array [topleft, topright, bottomleft,
            //bottomright]
            let tl = (y_0 * self.width + x_0) * 3;
            let tr = (y_0 * self.width + x_1) * 3;
            let bl = (y_1 * self.width + x_0) * 3;
            let br = (y_1 * self.width + x_1) * 3;

            //loop through RGB channels and apply bilinear interpolation
            for c in 0..3 {
                let tl_p: f32 = self.data[tl + c] as f32;
                let tr_p: f32 = self.data[tr + c] as f32;
                let bl_p: f32 = self.data[bl + c] as f32;
                let br_p: f32 = self.data[br + c] as f32;

                let top = interpolation!(tl_p, tr_p, dx);
                let bottom = interpolation!(bl_p, br_p, dx);
                result.data[i * 3 + c] = interpolation!(top, bottom, dy) as u8;
            }
        }
        result
    }

    pub fn color_map(mut self, upper: u32, lower: u32) -> Self {
        let (ru, gu, bu) = get_rgb!(upper);
        let (rl, gl, bl) = get_rgb!(lower);
        self.data.chunks_mut(3).for_each(|rgb| {
            let lum =
                ((0.30 * rgb[0] as f32) + (0.59 * rgb[1] as f32) + (0.11 * rgb[2] as f32)) / 255.0;
            rgb[0] = interpolation!(rl, ru, lum) as u8;
            rgb[1] = interpolation!(gl, gu, lum) as u8;
            rgb[2] = interpolation!(bl, bu, lum) as u8;
        });
        self
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.data
    }
}
