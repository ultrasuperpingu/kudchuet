
use egui::{Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use egui_field_editor::EguiInspect;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Clone, Debug, EguiInspect, PartialEq, Serialize, Deserialize)]
pub struct StrokeData {
	pub stroke: Stroke,
	pub kind: StrokeKind,
}
impl Default for StrokeData {
	fn default() -> Self {
		Self { stroke: Stroke::new(3.0, Color32::BLACK), kind: StrokeKind::Inside }
	}
}
#[derive(Clone, Debug, EguiInspect, PartialEq, Serialize, Deserialize)]
pub struct TextData {
	pub text:String,
	pub color: Color32,
	pub size: f32,
}
impl Default for TextData {
	fn default() -> Self {
		Self { text: String::new(), color: Color32::BLACK, size: 0.5 }
	}
}
impl TextData {
	pub fn from_string(text: &str) -> Self {
		Self {
			text: text.to_owned(),
			color: Color32::BLACK,
			size: 1.0,
		}
	}
}

#[derive(Clone, Debug, EguiInspect, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum Shape {
	Circle {
		fill_color: Option<Color32>,
		size: f32,
		text:Option<TextData>,
		stroke: Option<StrokeData>,
	},
	Rect {
		fill_color: Option<Color32>,
		size: f32,
		text: Option<TextData>,
		stroke: Option<StrokeData>
	},
	String {
		#[inspect(transparent)]
		text:TextData
	},
	Composed(Vec<Shape>)
}
impl Default for Shape {
	fn default() -> Self {
		Shape::Circle {
			fill_color: Some(Color32::WHITE),
			text: None,
			size: 0.7,
			stroke: None
		}
	}
}
fn draw_text(painter: &egui::Painter, center: Pos2, cell_size: f32, text: &TextData) {
	painter.text(
		center,
		egui::Align2::CENTER_CENTER,
			&text.text,
			egui::FontId::proportional(cell_size * text.size),
			text.color
	);
}
impl Shape {
	pub fn draw(&self, painter: &egui::Painter, center: Pos2, cell_size: f32) {
		match self {
			Shape::Circle { fill_color: color, size, text, stroke} => {
				if let Some(color) = color {
					painter.circle_filled(center, cell_size * size/2.0, *color);
				}
				if let Some(c) = stroke.as_ref() {
					painter.circle_stroke(center, cell_size * size/2.0, c.stroke);
				}
				if let Some(text) = text {
					draw_text(painter, center, cell_size, text);
				}
			},
			Shape::String { text } => {
				draw_text(painter, center, cell_size, text);
			},
			Shape::Rect { fill_color: color, size, text, stroke } => {
				let size_vec = Vec2::new(cell_size * size, cell_size * size);
				let rect = Rect::from_center_size(center, size_vec);
				if let Some(color) = color {
					painter.rect_filled(rect, 0.0, *color);
				}
				if let Some(stroke) = stroke.as_ref() {
					painter.rect_stroke(rect, 0.0, stroke.stroke, stroke.kind);
				}
				if let Some(text) = text {
					draw_text(painter, center, cell_size, text);
				}
			},
			Shape::Composed(shapes) => {
				for s in shapes {
					Self::draw(s, painter, center, cell_size);
				}
			}
		}
	}
}