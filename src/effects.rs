use crate::{Color, TermColorSupport, Pixels};

pub enum EffectType {
	Background,
	Text,
}

struct Effect {
	name: &'static str,
	r#type: EffectType,
	function: fn(&mut Pixels, f32, i32, &TermColorSupport),
}

static EFFECTS: [Effect;2] = [
	Effect {
		name: "none",
		r#type: EffectType::Background,
		function: |pixels: &mut Pixels, progress: f32, rand: i32, term_color_support: &TermColorSupport| {
			
		}
	},
	Effect {
		name: "rainbow",
		r#type: EffectType::Text,
		function: |pixels: &mut Pixels, progress: f32, rand: i32, term_color_support: &TermColorSupport| {
			for i in 0..pixels.size.1 {
				for j in 0..pixels.size.0 {
					pixels.set_color((j, i), Color::new_hsv((progress*360.0 + ((i*i + j*j) as f32).sqrt())%360.0, 1.0, 1.0));
				}
			}
		}
	},
];

pub fn apply_effect(r#type: EffectType, pixels: &mut Pixels, progress: f32, rand: i32, selected_effects: &Option<Vec<String>>, term_color_support: &TermColorSupport) {
	//(EFFECTS[1].function)(pixels, progress, rand, term_color_support);
}