use std::sync::Arc;

use eframe::NativeOptions;
use egui::IconData;

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
