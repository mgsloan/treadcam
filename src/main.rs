#![feature(type_ascription)]

extern crate rscam;
extern crate image;
extern crate serde;
extern crate serde_json;
extern crate rustfft;
extern crate goertzel;

#[macro_use]
extern crate serde_derive;

use std::fs;
use std::io::Write;
use std::str;
use std::time;

use rustfft::FFTplanner;
use rustfft::num_complex::Complex;

#[derive(Serialize, Deserialize, Debug)]
struct DataPoint {
    x: u32,
    y: u32,
    time: time::SystemTime,
}

const OFFSET: usize = 0;
const COUNT: usize = 100;

fn main() {
    let mut input_bytes = fs::read("data/walking/out.json").unwrap();
    let data: Vec<DataPoint> =
        serde_json::from_str(str::from_utf8(&input_bytes).unwrap()).unwrap();

    let run_time = data.last().unwrap().time.duration_since(data.first().unwrap().time).unwrap();
    let run_time_float = (run_time.as_secs() as f32) + (run_time.subsec_millis() as f32) / 1000.;
    let sample_rate = (data.len() as f32) / run_time_float;

    println!("{:?}", sample_rate);

    let target_freq = 1. / 1.2;

    let params = goertzel::Parameters::new(target_freq, sample_rate as u32, COUNT);

    let mut to_add = [0: i16; COUNT];
    for (ix, dp) in data.iter().skip(OFFSET).take(COUNT).enumerate() {
        to_add[ix] = dp.x as i16;
    }

    //    .iter().map(|dp| dp.x as i16).collect();
    println!("{:?}", params.start().add(&to_add).finish());

    // goertzel::Parameters::new()

    // ~ 1.2 seconds for X axis

    /*
    let start_time = data[0].time;
    for dp in data {
        println!("{:?} {:?} {:?}", dp.x, dp.y, dp.time.duration_since(start_time));
    }
    */

    /*
    // println!("{:?}", data);

    let mut input: Vec<Complex<f32>> = data.iter().map(|p| Complex::new(p.y as f32, 0.)).collect();
    let mut output: Vec<Complex<f32>> = data.iter().map(|p| Complex::new(0., 0.)).collect();

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(data.len());
    fft.process(&mut input, &mut output);

    let amps:Vec<f32> = output.iter().map(|x| x.re).collect();
    println!("amplitudes = {:?}", amps);

    */
}

fn old_main() {
    let mut camera = rscam::new("/dev/video0").unwrap();

    for wformat in camera.formats() {
        let format = wformat.unwrap();
        println!("{:?}", format);

        let resolutions = camera.resolutions(&format.format).unwrap();

        if let rscam::ResolutionInfo::Discretes(d) = resolutions {
            for resol in &d {
                println!("  {}x{}  {:?}", resol.0, resol.1,
                         camera.intervals(&format.format, *resol).unwrap());
            }
        } else {
            println!("  {:?}", resolutions);
        }
    }

    /*
    for wformat in camera.formats() {
        let format = wformat.unwrap();
        println!("{:?}", format);
        println!("  {:?}", camera.resolutions(&format.format).unwrap());
    } */

    println!("Controls:");
    for control in camera.controls() {
        println!("{:?}", control.unwrap().name);
    }

    camera.start(&rscam::Config {
        interval: (1, 10),
        resolution: (1280, 720),
        format: b"RGB3",
        ..Default::default()
    }).unwrap();

    let out_width = 200;
    let mut out_img = image::ImageBuffer::new(out_width, 720);

    let mut data = Vec::new();

    for i in 0..out_width {
        let frame = camera.capture().unwrap();

        println!("Frame {} of length {}", i, frame.len());

        let img0: image::ImageBuffer<image::Rgb<u8>, rscam::Frame> =
          image::ImageBuffer::from_raw(frame.resolution.0, frame.resolution.1, frame).unwrap();

        // TODO: avoid copying.  Issue is that rscam::Frame is immutable.
        let mut img = image::ImageBuffer::from_fn(img0.width(), img0.height(), |x, y| img0[(x, y)]);

        let mut salient_x_sum = 0;
        let mut salient_x_count = 0;
        for x in 0..img.width() {
            let y = img.height() - 1;
            if rgb8_to_gray(img[(x, y)]) > 25000 {
                // img.put_pixel(x, y, image::Rgb { data: [255, 0, 0] });
                salient_x_sum += x;
                salient_x_count += 1;
            }
        }
        let salient_x_average =
            if salient_x_count == 0 { img.width() / 2 } else { salient_x_sum / salient_x_count };

        let mut last_top = img.height() - 1;
        for y in (0..img.height()).rev() {
            let x = salient_x_average;
            let gray = rgb8_to_gray(img[(x, y)]);
            if gray > 25000 {
                last_top = y;
            } else if last_top - y > 20 {
                // If more than 20 pixels haven't been salient, abort.
                break;
            }
            // img.put_pixel(x, y, image::Rgb { data: [0, 255, 0] });
        }

        for x in 0..(img.width() - 1) {
            // img.put_pixel(x, last_top, image::Rgb { data: [0, 0, 255] });
        }

        out_img.put_pixel(i, last_top, image::Rgb { data: [255, 0, 0] });
        out_img.put_pixel(salient_x_average * out_width / img.width(), i, image::Rgb { data: [0, 255, 0] });

        // FIXME: use frame.timestamp instead
        let now = time::SystemTime::now();
        data.push(DataPoint { x: salient_x_average, y: last_top, time: now });
        // println!("{}", last_top);

        println!("{:?}", now)

        // img.save(&format!("frame-{}.jpg", i));
    }

    out_img.save("out.jpg");

    let mut file = fs::File::create("out.json").unwrap();
    file.write_all(serde_json::to_string(&data).unwrap().as_bytes());

}

pub fn rgb8_to_gray(pixel: image::Rgb<u8>) -> u32 {
    let r = pixel[0];
    let g = pixel[1];
    let b = pixel[2];
    return (r as u32 * 77) + (g as u32 * 151) + (b as u32 * 28);
}
