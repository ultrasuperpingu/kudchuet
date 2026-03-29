use kudchuet::gomoku::gui::create_board;


#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
	eframe::run_native(
		"Gomoku",
		eframe::NativeOptions::default(),
		Box::new(|_cc| Ok(Box::new(create_board()))),
	)
}

#[cfg(target_arch = "wasm32")]
use eframe::web_sys;
#[cfg(target_arch = "wasm32")]
fn main() {
	use wasm_bindgen::JsCast;

	let window = web_sys::window().expect("no global `window` exists");
	let document = window.document().expect("should have a document");
	let canvas = document
		.get_element_by_id("canvas_id")
		.expect("canvas not found")
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.expect("element is not a canvas");

	wasm_bindgen_futures::spawn_local(async move {
		eframe::WebRunner::new()
			.start(
				canvas,
				eframe::WebOptions::default(),
				Box::new(|_cc| Ok(Box::new(create_board()))),
			)
			.await
			.expect("failed to start eframe");
	});
}
