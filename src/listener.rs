use std::sync::mpsc::Sender;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct Listener {
	_stream: <cpal::Device as DeviceTrait>::Stream,
}

impl Listener {
	pub fn new(tx: Sender<f32>) -> Result<Self, Box<dyn std::error::Error>> {
		let host = cpal::default_host();
		let dev = host.default_input_device().unwrap();
		let config = dev.default_input_config()?.config();

		let stream = dev.build_input_stream(config,
			move |d: &[f32], _: &cpal::InputCallbackInfo|
				d.chunks(config.channels as usize).for_each(|f| { let _ = tx.send(f.iter().sum::<f32>() / config.channels as f32); }),
			|e| println!("audio err: {e}"), None)?;

		stream.play()?;

		Ok(Self { _stream: stream })
	}
}
