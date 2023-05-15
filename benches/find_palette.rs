#[macro_use]
extern crate bencher;
extern crate image;
extern crate color_thief;

use std::path::Path;

use bencher::Bencher;

use color_thief::ColorFormat;

fn get_image_buffer(img: image::DynamicImage) -> Vec<u8> {
    match img {
        image::DynamicImage::ImageRgb8(buffer) => buffer.to_vec(),
        _ => unreachable!(),
    }
}

fn q1(bencher: &mut Bencher) {
    let img = image::open(&Path::new("images/photo1.jpg")).unwrap();
    let pixels = get_image_buffer(img);
    bencher.iter(|| color_thief::get_palette(&pixels, ColorFormat::Rgb, 1, 10))
}

fn q10(bencher: &mut Bencher) {
    let img = image::open(&Path::new("images/photo1.jpg")).unwrap();
    let pixels = get_image_buffer(img);
    bencher.iter(|| color_thief::get_palette(&pixels, ColorFormat::Rgb, 10, 10))
}

benchmark_group!(benches, q1, q10);
benchmark_main!(benches);
