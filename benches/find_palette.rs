#[macro_use]
extern crate bencher;
extern crate image;
extern crate color_thief;

use std::path::Path;

use bencher::Bencher;

use color_thief::ColorFormat;

fn q1(bencher: &mut Bencher) {
    let img = image::open(&Path::new("images/photo1.jpg")).unwrap();
    let pixels = img.raw_pixels();
    bencher.iter(|| color_thief::get_palette(&pixels, ColorFormat::Rgb, 1, 10))
}

fn q10(bencher: &mut Bencher) {
    let img = image::open(&Path::new("images/photo1.jpg")).unwrap();
    let pixels = img.raw_pixels();
    bencher.iter(|| color_thief::get_palette(&pixels, ColorFormat::Rgb, 10, 10))
}

benchmark_group!(benches, q1, q10);
benchmark_main!(benches);
