
use egui::{Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use egui_field_editor::EguiInspect;
use serde::{Deserialize, Serialize};

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
	#[inspect(slider(min=0.0, max=1.0))]
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

#[derive(Clone, Debug, EguiInspect, PartialEq, Serialize, Deserialize)]
pub struct StripData {
	pub color: Color32,
	#[inspect(slider(min=0.0, max=1.0))]
	pub weight: f32
}
impl Default for StripData {
	fn default() -> Self {
		Self { color: Color32::from_rgb(255, 255, 255), weight: 1.0 }
	}
}
#[derive(Clone, Debug, EguiInspect, PartialEq, Serialize, Deserialize)]
pub enum Shape {
	Circle {
		fill_color: Option<Color32>,
		#[inspect(slider(min=0.0, max=1.0))]
		size: f32,
		text:Option<TextData>,
		stroke: Option<StrokeData>,
	},
	StrippedCircle {
		strips: Vec<StripData>,
		#[inspect(slider(min=0.0, max=1.0))]
		angle: f32,
		#[inspect(slider(min=0.0, max=1.0))]
		size: f32,
		text:Option<TextData>,
		stroke: Option<StrokeData>,
	},
	Rect {
		fill_color: Option<Color32>,
		#[inspect(slider(min=0.0, max=1.0))]
		size: f32,
		text: Option<TextData>,
		stroke: Option<StrokeData>
	},
	String {
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
fn draw_weighted_stripped_circle(
	painter: &egui::Painter,
	center: egui::Pos2,
	radius: f32,
	colors: &[Color32],
	weights: &[f32], // sum = 1.0
	angle: f32,      // 0..1
) {
	let steps = 64;
	let epsilon = 2.0;

	let dir = egui::Vec2::angled(angle * std::f32::consts::TAU);
	let normal = egui::vec2(-dir.y, dir.x);

	let total_width = radius * 2.0;

	let mut offset = -total_width / 2.0;

	for (color, weight) in colors.iter().zip(weights.iter()) {
		let band_width = weight * total_width;

		let min = offset;
		let max = offset + band_width;

		let mut points = Vec::new();

		for i in 0..=steps {
			let theta = i as f32 / steps as f32 * std::f32::consts::TAU;
			let p = center + egui::Vec2::new(theta.cos(), theta.sin()) * radius;

			let proj = (p - center).dot(normal);

			if proj >= min - epsilon && proj <= max + epsilon {
				points.push(p);
			}
		}

		if points.len() >= 3 {
			painter.add(egui::Shape::convex_polygon(
				points,
				*color,
				egui::Stroke::NONE,
			));
		}

		offset += band_width;
	}
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
			Shape::StrippedCircle { strips, angle, size, text, stroke} => {
				let mut colors=vec![Color32::default();strips.len()];
				let mut weights=vec![0.0;strips.len()];
				let mut sum=0.0;
				for (i, s) in strips.iter().enumerate() {
					colors[i] = s.color;
					weights[i] = s.weight;
					sum += s.weight;
				}
				for w in &mut weights {
					if sum > 0.0 {
						*w /= sum;
					} else {
						*w = 1.0 / (strips.len() as f32);
					}
				}
				draw_weighted_stripped_circle(painter, center, cell_size * size/2.0, &colors, &weights, *angle);
				
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