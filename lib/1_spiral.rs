#[unsafe(no_mangle)]
pub extern "C-unwind" fn transform(x: f32, y: f32, t: f32, fft: &[f32]) -> (f32, f32, f32) {
	let t = (t.sin() + 1.0) / 2.0;
	let m = fft[((1.0 - y) * 16.0 * 16.0 + x * 32.0) as usize] / 10.0;

	if m < 0.2 { return (0.0, 0.0, 0.0); }

	(x * m, (1.0 - x) * m * (1.0 - t), m * t)
}
