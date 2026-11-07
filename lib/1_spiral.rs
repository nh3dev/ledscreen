#[unsafe(no_mangle)]
pub extern "C-unwind" fn transform(x: f32, y: f32, t: f32, fft: &[f32]) -> (f32, f32, f32) {
	let m = fft[((1.0 - y) * 32.0 * 32.0 + x * 32.0) as usize];

	(x * m, (1.0 - x) * m * (1.0 - t), m * t)
}
