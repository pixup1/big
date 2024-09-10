use std::{str::FromStr, time::Instant};

use crate::{Color, TermColorSupport, Pixels};

#[derive(PartialEq, Eq)]
pub enum EffectType {
	Background,
	Text,
}

struct Effect {
	name: &'static str,
	r#type: EffectType,
	function: fn(&mut Pixels, f32, Instant, f32, i32),
}

static EFFECTS: [Effect;4] = [
	Effect {
		name: "empty",
		r#type: EffectType::Background,
		function: |_: &mut Pixels, _: f32, _: Instant, _: f32, _: i32| {}
	},
	Effect {
		name: "normal",
		r#type: EffectType::Text,
		function: |_: &mut Pixels, _: f32, _: Instant, _: f32, _: i32| {}
	},
	Effect {
		name: "rainbow",
		r#type: EffectType::Text,
		function: |pixels: &mut Pixels, _: f32, timer: Instant, speed: f32, _: i32| {
			for i in 0..pixels.size.1 {
				for j in 0..pixels.size.0 {
					pixels.set_color((j, i), Color::new_hsv((speed*timer.elapsed().as_secs_f32()*500.0 + ((i*i + j*j) as f32).sqrt())%360.0, 1.0, 1.0));
				}
			}
		}
	},
	Effect {
		name: "split",
		r#type: EffectType::Text,
		function: |pixels: &mut Pixels, frame_progress: f32, timer: Instant, speed: f32, rand: i32| {
			let mut new_pixels = Pixels::new((pixels.size.0 + (frame_progress * 12.0) as usize, pixels.size.1 + (frame_progress * 6.0) as usize));
			pixels.color_all(Color::new_rgb(255, 0, 0));
			new_pixels.comp(pixels, (new_pixels.size.0 as i32 / 2 - (frame_progress * 6.0) as i32, new_pixels.size.1 as i32 / 2 - (frame_progress * 3.0) as i32));
			pixels.color_all(Color::new_rgb(0, 255, 0));
			new_pixels.comp(pixels, (new_pixels.size.0 as i32 / 2 + (frame_progress * 6.0) as i32, new_pixels.size.1 as i32 / 2 + (frame_progress * 3.0) as i32));
			pixels.color_all(Color::new_rgb(255, 255, 255));
			new_pixels.comp(pixels, (new_pixels.size.0 as i32 / 2, new_pixels.size.1 as i32 / 2));
			*pixels = new_pixels;
		}
	},
];

pub fn list_effects() -> String {
	let mut fx = String::from_str("Background effects :\n").unwrap();
	for e in EFFECTS.iter() {
		if matches!(e.r#type,EffectType::Background) {
			fx.push_str(&format!(" - {}\n", e.name))
		}
	}
	
	fx.push_str("\nText effects :\n");
	for e in EFFECTS.iter() {
		if matches!(e.r#type,EffectType::Text) {
			fx.push_str(&format!(" - {}\n", e.name))
		}
	}
	
	fx
}

pub fn apply_effect(r#type: EffectType, pixels: &mut Pixels, frame_progress: f32, timer: Instant, speed: f32, rand: i32, selected_effects: &Vec<String>) {
	let mut real_selection: Vec<String> = Vec::new();
	
	for sel_e in selected_effects {
		let mut found = false;
		
		for e in EFFECTS.iter() {
			if e.name == sel_e {
				found = true;
				if e.r#type == r#type {
					real_selection.push(String::from(sel_e));
				}
			}
		}
		
		if !found {
			panic!("Effect \"{}\" does not exist", sel_e);	
		}
	}
	
	if real_selection.is_empty() {
		for e in EFFECTS.iter() {
			if e.r#type == r#type {
				real_selection.push(String::from(e.name));
			}
		}
	}
	
	let pick = rand as usize % real_selection.len() as usize;
	
	for e in EFFECTS.iter() {
		if e.name == real_selection[pick] {
			(e.function)(pixels, frame_progress, timer, speed, rand);
		}
	}
}