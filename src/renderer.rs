use std::sync::{Arc, mpsc::Receiver};
use std::time::Instant;
use std::collections::VecDeque;

use eframe::egui::{self, Color32, Pos2, Vec2, Key};
use rustfft::num_complex::Complex32;

const FFT_SIZE: usize = 1 << 12;

const WIDTH: usize = 64;
const HEIGHT: usize = 64;

pub struct Renderer {
	rx: Receiver<f32>,

	queue: VecDeque<f32>,
	buf: [f32; FFT_SIZE / 2],

	fft: FFt,
	time: Instant,

	func: usize,
}

impl Renderer {
	pub fn new(rx: Receiver<f32>) -> Self {
		Self {
			rx,
			time: Instant::now(),
			queue: std::iter::repeat_n(0.0f32, FFT_SIZE).collect::<VecDeque<_>>(),
			buf: [0.0f32; FFT_SIZE / 2],
			fft: FFt::new(),
			func: 0,
		}
	}

	pub fn render_blocking(&mut self, plugins: &crate::loading::PluginManager, ui: &mut egui::Ui) {
		ui.ctx().request_repaint();
		ui.ctx().input(|s| s.raw.events.iter()
			.filter_map(|e| match e { 
				egui::Event::Key { key, pressed: true, .. } => Some(key),
				_ => None })
			.for_each(|k| match k {
				Key::Num1 => self.func = 0,
				Key::Num2 => self.func = 1,
				Key::Num3 => self.func = 2,
				Key::Num4 => self.func = 3,
				Key::Num5 => self.func = 4,
				Key::Num6 => self.func = 5,
				Key::Num7 => self.func = 6,
				Key::Num8 => self.func = 7,
				Key::Num9 => self.func = 8,
				Key::Num0 => self.func = 9,
				_ => {},
			}));

		self.rx.try_iter().for_each(|f| { self.queue.pop_front(); self.queue.push_back(f); });

		self.fft.process(self.queue.iter()).iter()
			.zip(self.buf.iter_mut()).for_each(|(c, b)| *b = c.norm());

		let plugins = plugins.get();

		egui::CentralPanel::default().show(ui, |ui| {
			egui::Frame::canvas(&Default::default()).show(ui, |ui| {
				let t = self.time.elapsed().as_secs_f32();
				let xs = ui.available_width() / WIDTH as f32;
				let ys = ui.available_height() / HEIGHT as f32;

				(0..WIDTH).flat_map(|x| (0..HEIGHT).map(move |y| (x as f32, y as f32))).for_each(|(x, y)| {
					let (r, g, b) = plugins.get(self.func).map_or((0.0, 0.0, 0.0),
						|f| f.call(x / WIDTH as f32, y / HEIGHT as f32, t, &self.buf));

					ui.painter().rect_filled(egui::Rect::from_min_size(Pos2::new(x * xs, y * ys), Vec2::new(xs, ys)), 0,
						Color32::from_rgb((r * 256.0) as u8, (g * 256.0) as u8, (b * 256.0) as u8));
				});
			});

			ui.heading(format!("FPS {:.0}\nFunc: {}",
				(1.0 / ui.ctx().input(|s| s.stable_dt)).round_ties_even(),
				self.func));
		});
	}
}

struct FFt {
	fft: Arc<dyn rustfft::Fft<f32>>,
	hann: Vec<f32>,
	pub buf: Vec<Complex32>,
}

impl FFt {
	fn new() -> Self {
		Self {
			hann: (0..FFT_SIZE).map(|i|
				0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos()))
				.collect::<Vec<_>>(),
			fft: rustfft::FftPlanner::<f32>::new().plan_fft_forward(FFT_SIZE),
			buf: vec![Complex32::new(0.0, 0.0); FFT_SIZE],
		}
	}
	
	fn process<'a>(&'a mut self, input: impl IntoIterator<Item = &'a f32>) -> &'a [Complex32] {
		input.into_iter().zip(self.hann.iter()).zip(self.buf.iter_mut()).for_each(|((s, h), b)|
			*b = Complex32::new(s * h, 0.0));
		
		self.fft.process(&mut self.buf);

		&self.buf
	}
}
