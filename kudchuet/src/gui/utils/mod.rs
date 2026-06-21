#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use eframe::NativeOptions;
#[cfg(not(target_arch = "wasm32"))]
use egui::IconData;
#[cfg(not(target_arch = "wasm32"))]
pub fn get_native_default_option() -> NativeOptions {
	let mut options = eframe::NativeOptions::default();
	options.viewport.icon = Some(Arc::new(IconData {
		rgba: image::load_from_memory(include_bytes!("../../../../kudchuet.png").to_vec().as_slice())
			.unwrap()
			.to_rgba8()
			.to_vec(),
		width: 256,
		height: 256,
	}));
	options
}
