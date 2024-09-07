use crate::{color, color_text, Color, TermColorSupport};

pub enum EffectType {
	Background,
	Text,
}

struct Effect {
	name: &'static str,
	r#type: EffectType,
	function: fn(&mut Vec<Vec<u8>>, f32, i32, &TermColorSupport),
}

static EFFECTS: [Effect;2] = [
	Effect {
		name: "none",
		r#type: EffectType::Background,
		function: |pixels: &mut Vec<Vec<u8>>, progress: f32, rand: i32, term_color_support: &TermColorSupport| {
			
		}
	},
	Effect {
		name: "rainbow",
		r#type: EffectType::Text,
		function: |pixels: &mut Vec<Vec<u8>>, progress: f32, rand: i32, term_color_support: &TermColorSupport| {
			for line in pixels {
				color_text(line, Color::new_hsv((progress*720.0)%360.0, 1.0, 1.0), term_color_support, None, None)}
		}
	},
];

pub fn apply_effect(r#type: EffectType, pixels: &mut Vec<Vec<u8>>, progress: f32, rand: i32, selected_effects: &Option<Vec<String>>, term_color_support: &TermColorSupport) {
	//(EFFECTS[1].function)(pixels, progress, rand, term_color_support);
}