use std::mem::DropGuard;
use std::sync::Arc;
use std::path::Path;

use libloading::{Symbol, Library};
use parking_lot::{RwLock, RwLockReadGuard};

const TARGET_DIR: &str = "target/libs";

// TODO: switch to our arcswap implemented with AtomicU128 in the future perhaps
pub struct PluginManager {
	plugs: Arc<RwLock<Box<[Plugin]>>>,
}

impl PluginManager {
	pub fn new(dir: impl AsRef<Path>) -> Self {
		let path = dir.as_ref().to_path_buf();
		let plugs = Arc::new(RwLock::<Box<[Plugin]>>::new(Box::new([])));

		let plugs_ = plugs.clone();

		// TODO: use inotify if inefficient
		std::thread::spawn(move || {
			let mut old = Vec::new();

			loop {
				let mut new = std::fs::read_dir(&path).unwrap()
					.filter_map(Result::ok)
					.filter(|entry| entry.file_type().unwrap().is_file())
					.map(|e| e.path())
					.filter(|p| p.extension().is_some_and(|e| e == "rs"))
					.map(|p| (std::fs::metadata(&p).unwrap().modified().unwrap(), p))
					.collect::<Vec<_>>();

				new.sort_by(|a, b| a.1.file_name().cmp(&b.1.file_name()));

				if new != old {
					let _ = std::fs::create_dir(TARGET_DIR);

					let mut plugs = plugs_.write();
					std::mem::drop(std::mem::take(&mut *plugs));

					*plugs = new.iter().map(|(_, file)| {
						let out = Path::new(TARGET_DIR).join(file.file_name().unwrap())
							.with_extension("so");

						dbg!(&file);

						let status = std::process::Command::new("rustc")
							.args(["--color=always", "--crate-type=dylib", "-o"])
							.args([&out, file]).output().unwrap();

						if !status.status.success() {
							println!("{}", std::str::from_utf8(&status.stdout).unwrap());
							println!("{}", std::str::from_utf8(&status.stderr).unwrap());
						}

						unsafe { Plugin::new(&out) }
					}).collect::<Vec<_>>().into_boxed_slice();

					old = new;
				}

				std::thread::sleep(std::time::Duration::from_secs(2));
			}
		});

		// perhaps unnecessary but makes sure plugins load before they can be accessed
		std::thread::sleep(std::time::Duration::from_millis(200));

		Self { plugs }
	}

	#[allow(clippy::type_complexity)]
	pub fn get<'a>(&'a self) -> DropGuard<RwLockReadGuard<'a, Box<[Plugin]>>, impl FnOnce(RwLockReadGuard<'a, Box<[Plugin]>>)> {
		DropGuard::new(self.plugs.read(), RwLockReadGuard::unlock_fair)
	}
}

#[derive(Debug)]
pub struct Plugin {
	_lib: Library,
	pub time_divisor:         f32,
	pub time_dialation_range: f32,
	pub intensity_range:      f32,
	pub lum_mod:              f32,
	#[allow(improper_ctypes_definitions)]
	transform: unsafe extern "C-unwind" fn(x: f32, y: f32, t: f32, fft: &[f32]) -> (f32, f32, f32),
}

impl Plugin {
	pub unsafe fn new(path: &Path) -> Self {
		unsafe {
			let lib = Library::new(path).unwrap();

			Self {
				transform:           *lib.get(b"transform").unwrap(),
				time_divisor:         lib.get(b"TIME_DIVISOR").map_or(1000000000.0,  |s: Symbol<*const f32>| **s),
				time_dialation_range: lib.get(b"TIME_DIALATION_RANGE").map_or(100.0, |s: Symbol<*const f32>| **s),
				intensity_range:      lib.get(b"INTENSITY_RANGE").map_or(0.01,       |s: Symbol<*const f32>| **s),
				lum_mod:              lib.get(b"LUM_MOD").map_or(100.0,              |s: Symbol<*const f32>| **s),
				_lib: lib,
			}
		}
	}

	pub fn call(&self, x: f32, y: f32, t: f32, fft: &[f32]) -> (f32, f32, f32) {
		unsafe { (self.transform)(x, y, t, fft) }
	}
}
