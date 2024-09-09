use crate::term_colors::TERM_COLORS;

pub enum TermColorSupport {
	TrueColor,
	Ansi256,
	Ansi16,
	None,
}

pub fn get_term_color_support() -> TermColorSupport {
	match std::env::var("COLORTERM") {
		Ok(val) => match val.as_str() {
			"truecolor" | "24bit" => TermColorSupport::TrueColor,
			"256color" | "8bit" => TermColorSupport::Ansi256,
			"ansi" | "standard" => TermColorSupport::Ansi16,
			_ => TermColorSupport::Ansi16 // I wasn't able to find a list of all possible values for $COLORTERM
		},
		Err(_) => match std::env::var("TERM") {
			Ok(val) => match val.as_str() {
				"xterm-256color" | "screen-256color" | "tmux-256color" | "rxvt-unicode-256color" | "linux" => TermColorSupport::Ansi256,
				_ => TermColorSupport::Ansi16
			},
			Err(_) => TermColorSupport::None
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Color {
	r: u8,
	g: u8,
	b: u8,
}

impl Color {
	// New color from RGB
	pub const fn new_rgb(red: u8, green: u8, blue: u8) -> Color {
		Color {
			r: red,
			g: green,
			b: blue
		}
	}
	
	// New color from HSV (Hue is in degrees, saturation and value range from 0.0 to 1.0) (https://en.wikipedia.org/wiki/HSL_and_HSV#Color_conversion_formulae)
	pub fn new_hsv(hue: f32, saturation: f32, value: f32) -> Color {
		let c = value * saturation;
		let x = c * (1.0 - (((hue / 60.0) % 2.0) - 1.0).abs());
		let m = value - c;
		
		let nc = ((c + m) * 255.0) as u8;
		let nx = ((x + m) * 255.0) as u8;
		let no = (m * 255.0) as u8;
		
		match hue {
			h if (0.0..60.0).contains(&h) => Color{r: nc, g: nx, b: no},
			h if (60.0..120.0).contains(&h) => Color{r: nx, g: nc, b: no},
			h if (120.0..180.0).contains(&h) => Color{r: no, g: nc, b: nx},
			h if (180.0..240.0).contains(&h) => Color{r: no, g: nx, b: nc},
			h if (240.0..300.0).contains(&h) => Color{r: nx, g: no, b: nc},
			h if (300.0..360.0).contains(&h) => Color{r: nc, g: no, b: nx},
			_ => panic!("Hue must be between 0 and 360"),
		}
	}
	
	pub fn new_hex(hex: &str) -> Color {
		let hex = hex.trim_start_matches("#");
		Color {
			r: u8::from_str_radix(&hex[0..2], 16).unwrap(),
			g: u8::from_str_radix(&hex[2..4], 16).unwrap(),
			b: u8::from_str_radix(&hex[4..6], 16).unwrap()
		}
	}
	
	// Convert color to hex
	pub fn to_hex(&self) -> String {
		format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
	}
	
	// Find the closest color from a list of colors
	fn closest_color(&self, colors: &[Color]) -> (Color, usize) {
		let mut best = 0;
		let mut best_score = std::f32::MAX;
		for i in 1..colors.len() {
			// Distance in RGB space (lower = better match)
			let score = ((colors[i].r - self.r) as f32).powf(2.0) + ((colors[i].g - self.g) as f32).powf(2.0) + ((colors[i].b - self.b) as f32).powf(2.0);
			if score < best_score {
				best = i;
				best_score = score;
			}
		}
		(colors[best].clone(), best)
	}
	
	// Convert color to escape sequence
	pub fn to_escape(&self, term_color_support: &TermColorSupport) -> Option<String> {
		match term_color_support {
			TermColorSupport::TrueColor => Some(format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)),
			TermColorSupport::Ansi256 => {
				let c_index = self.closest_color(&TERM_COLORS).1;
				Some(format!("\x1b[38;5;{}m", c_index))
			},
			TermColorSupport::Ansi16 => {
				let c_index = self.closest_color(&TERM_COLORS[0..16]).1;
				let prefix = if c_index > 7 {3} else {9};
				Some(format!("\x1b[{}{}m", prefix, c_index % 8))
			},
			TermColorSupport::None => None
		}
	}
}