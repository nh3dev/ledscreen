use std::collections::VecDeque;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use eframe::egui::{self, Color32, Pos2, Vec2};
use rustfft::num_complex::Complex32;

mod loading;

const FFT_SIZE: usize = 1 << 12;

const WIDTH: usize = 64;
const HEIGHT: usize = 64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let host = cpal::default_host();
	let dev = host.default_input_device().unwrap();
	let config = dev.default_input_config()?.config();
	let (tx, rx) = std::sync::mpsc::channel();

	let stream = dev.build_input_stream(config,
		move |d: &[f32], _: &cpal::InputCallbackInfo|
			d.chunks(config.channels as usize).for_each(|f| { let _ = tx.send(f.iter().sum::<f32>() / config.channels as f32); }),
		|e| println!("audio err: {e}"), None)?;

	stream.play()?;

	let plugins = loading::Plugin::from_dir("target/libs");
	let time = std::time::Instant::now();

	let hann = (0..FFT_SIZE).map(|i|
		0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos()))
		.collect::<Vec<_>>();

	let mut buf = std::iter::repeat_n(0.0f32, FFT_SIZE).collect::<VecDeque<_>>();
	let mut fft_buf = vec![Complex32::new(0.0, 0.0); FFT_SIZE];
	let mut buckets_buf = [0.0f32; FFT_SIZE / 2];

	let mut planner = rustfft::FftPlanner::<f32>::new();
	let fft = planner.plan_fft_forward(FFT_SIZE);

	eframe::run_ui_native("poop", Default::default(), move |ui, _frame| {
		ui.ctx().request_repaint();

		rx.try_iter().for_each(|f| { buf.pop_front(); buf.push_back(f); });

		buf.iter().zip(hann.iter()).zip(fft_buf.iter_mut()).for_each(|((s, h), b)|
			*b = Complex32::new(s * h, 0.0));
		
		fft.process(&mut fft_buf);

		egui::CentralPanel::default().show(ui, |ui| {
			egui::Frame::canvas(&Default::default()).show(ui, |ui| {
				fft_buf.iter().zip(buckets_buf.iter_mut()).for_each(|(c, b)| *b = c.norm());
				let t = time.elapsed().as_secs_f32();
				let xs = ui.available_width() / WIDTH as f32;
				let ys = ui.available_height() / HEIGHT as f32;

				(0..WIDTH).flat_map(|x| (0..HEIGHT).map(move |y| (x as f32, y as f32))).for_each(|(x, y)| {
					let (r, g, b) = plugins[0].call(x / WIDTH as f32, y / HEIGHT as f32, t, &buckets_buf);
					ui.painter().rect_filled(egui::Rect::from_min_size(Pos2::new(x * xs, y * ys), Vec2::new(xs, ys)), 0,
						Color32::from_rgb((r * 256.0) as u8, (g * 256.0) as u8, (b * 256.0) as u8));
				});
			});

			ui.heading(format!("fps {:.0}", 1.0 / ui.ctx().input(|state| state.stable_dt)));
		});
	})?;
	
	Ok(())
}
