use std::{str::FromStr, time::Instant};

use rand::{Rng, rngs::StdRng};

use crate::{cacamap, Color, Pixels};

#[derive(PartialEq, Eq)]
pub enum EffectType {
	Background,
	Text,
}

struct Effect {
	name: &'static str,
	r#type: EffectType,
	function: fn(&mut Pixels, f32, Instant, f32, &mut StdRng),
}

static EFFECTS: [Effect;7] = [
	Effect {
		name: "empty",
		r#type: EffectType::Background,
		function: |_: &mut Pixels, _: f32, _: Instant, _: f32, _: &mut StdRng| {}
	},
	Effect {
		name: "wave",
		r#type: EffectType::Background,
		function: |pixels: &mut Pixels, _: f32, timer: Instant, speed: f32, rng: &mut StdRng| {
			let hue = rng.gen_range(0.0..360.0);
			
			for i in 0..pixels.size.1 {
				let c = cacamap(" .-:=+*#%@", ((i as f32 + timer.elapsed().as_secs_f32() * if i < pixels.size.1/2 {14.0} else {-14.0} /speed).sin()+1.0)/2.0);
				
				for j in 0..pixels.size.0 {
					pixels.set_char((j,i), c);
					pixels.set_color((j, i), Color::new_hsv(hue, 1.0, 0.8));
				}
			}
		}
	},
	Effect {
		name: "spiral",
		r#type: EffectType::Background,
		function: |pixels: &mut Pixels, _: f32, timer: Instant, speed: f32, _: &mut StdRng| {
			for i in 0..pixels.size.1 {
				let y = (i as f32 - pixels.size.1 as f32 / 2.0) * 10.0;
				
				for j in 0..pixels.size.0 {
					let x = j as f32 - pixels.size.0 as f32 / 2.0;
					let angle = (y/x).atan();
					let distance = (x*x + y*y).sqrt();
					pixels.set_char((j,i), cacamap(" .-:=+*#%@", ((angle*16.0 + timer.elapsed().as_secs_f32()/speed * 5.0 + distance/100.0).cos()+1.0)/2.0));
					pixels.set_color((j,i), Color::new_hsv((distance + timer.elapsed().as_secs_f32() * 200.0)%360.0, 0.6, 1.0))
				}
			}
		}
	},
	Effect {
		name: "normal",
		r#type: EffectType::Text,
		function: |_: &mut Pixels, _: f32, _: Instant, _: f32, _: &mut StdRng| {}
	},
	Effect {
		name: "rainbow",
		r#type: EffectType::Text,
		function: |pixels: &mut Pixels, _: f32, timer: Instant, speed: f32, _: &mut StdRng| {
			for i in 0..pixels.size.1 {
				for j in 0..pixels.size.0 {
					pixels.set_color((j, i), Color::new_hsv((timer.elapsed().as_secs_f32()/speed*400.0 + ((i*i + j*j) as f32).sqrt()*2.0)%360.0, 1.0, 1.0));
				}
			}
		}
	},
	Effect {
		name: "split",
		r#type: EffectType::Text,
		function: |pixels: &mut Pixels, frame_progress: f32, _: Instant, _: f32, rng: &mut StdRng| {
			let direction: i32;
			if rng.gen::<bool>() {
				direction = 1;
			} else {
				direction = -1;
			}
			let c1 = Color::new_hsv(rng.gen_range(0.0..360.0), 1.0, 1.0);
			let c2 = Color::new_hsv(rng.gen_range(0.0..360.0), 1.0, 1.0);
			
			let mut new_pixels = Pixels::new((pixels.size.0 + (frame_progress * 8.0) as usize, pixels.size.1 + (frame_progress * 4.0) as usize));
			pixels.color_all(c1);
			new_pixels.comp(pixels, (new_pixels.size.0 as i32 / 2 - (frame_progress * 4.0) as i32, new_pixels.size.1 as i32 / 2 - direction * (frame_progress * 2.0) as i32));
			pixels.color_all(c2);
			new_pixels.comp(pixels, (new_pixels.size.0 as i32 / 2 + (frame_progress * 4.0) as i32, new_pixels.size.1 as i32 / 2 + direction * (frame_progress * 2.0) as i32));
			pixels.color_all(Color::new_rgb(255, 255, 255));
			new_pixels.comp(pixels, (new_pixels.size.0 as i32 / 2, new_pixels.size.1 as i32 / 2));
			*pixels = new_pixels;
		}
	},
	Effect {
		name: "worm",
		r#type: EffectType::Text,
		function: |pixels: &mut Pixels, _: f32, timer: Instant, speed: f32, _: &mut StdRng| {
			for i in 0..pixels.size.1 {
				for j in 0..pixels.size.0 {
					let sample_y = (i as f32 + ((j as f32 / 20.0 + timer.elapsed().as_secs_f32() / speed * 6.0).sin().powf(2.0) * 3.0)) as i32;
					if let Some((character, color)) = pixels.get_pixel((j, sample_y as usize)) {
						pixels.set_char((j, i), character);
						pixels.set_color((j, i), color);
					}
				}
			}
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

pub fn apply_effect(r#type: EffectType, pixels: &mut Pixels, frame_progress: f32, timer: Instant, speed: f32, rng: &mut StdRng, selected_effects: &Vec<String>) {
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
			panic!("Effect \"{}\" does not exist.\n\nUse --help flag for a list of available effects.", sel_e);
		}
	}
	
	if real_selection.is_empty() {
		for e in EFFECTS.iter() {
			if e.r#type == r#type {
				real_selection.push(String::from(e.name));
			}
		}
	}
	
	let pick = rng.gen::<usize>()  % real_selection.len() as usize;
	
	for e in EFFECTS.iter() {
		if e.name == real_selection[pick] {
			(e.function)(pixels, frame_progress, timer, speed, rng);
		}
	}
}