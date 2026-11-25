mod loading;
mod renderer;
mod listener;

const PLUGINS_DIR: &str = "lib";

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let (tx, rx) = std::sync::mpsc::channel();

	let _listener = listener::Listener::new(tx)?;
	let plugins = loading::PluginManager::new(PLUGINS_DIR);
	let mut renderer = renderer::Renderer::new(rx);

	eframe::run_ui_native("ledscreen", Default::default(),
		move |ui, _| renderer.render_blocking(&plugins, ui))?;
	
	Ok(())
}
