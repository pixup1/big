use crate::color::*;

pub struct Pixels {
	pub size: (usize, usize),
	chars: Vec<u8>,
	colors: Vec<Color>,
}

impl Pixels {
	pub fn new(size: (usize, usize)) -> Pixels {
		Pixels {
			size,
			chars: vec![b' '; size.0 * size.1],
			colors: vec![Color::new_rgb(255, 255, 255); size.0 * size.1],
		}
	}
	
	pub fn set_char(&mut self, position: (usize, usize), character: u8) {
		self.chars[position.1 * self.size.0 + position.0] = character;
	}
	
	pub fn set_color(&mut self, position: (usize, usize), color: Color) {
		self.colors[position.1 * self.size.0 + position.0] = color;
	}
	
	pub fn get_pixel(&self, position: (usize, usize)) -> Option<(u8, Color)> {
		if position.0 < self.size.0 && position.1 < self.size.1 {
			Some((self.chars[position.1 * self.size.0 + position.0], self.colors[position.1 * self.size.0 + position.0]))
		} else {
			None
		}
	}
	
	pub fn color_all(&mut self, color: Color) {
		for i in 0..self.size.1 {
			for j in 0..self.size.0 {
				self.set_color((j, i), color);
			}
		}
	}
	
	// Composite to_comp onto self centered at position (0,0 = top left corner of self)
	pub fn comp(&mut self, to_comp: &Pixels, position: (i32, i32)) {
		let origin = (position.0 - to_comp.size.0  as i32 / 2, position.1 - to_comp.size.1 as i32 / 2);
		
		for i in 0..to_comp.size.1 {
			let y = origin.1 + i as i32;
			
			for j in 0..to_comp.size.0 {
				let x = origin.0 + j as i32;
				
				if x >= 0 && x < self.size.0 as i32 && y >= 0 && y < self.size.1 as i32 && to_comp.chars[i * to_comp.size.0 + j] != b' ' {
					self.set_char((x as usize, y as usize), to_comp.chars[i * to_comp.size.0 + j]);
					self.set_color((x as usize, y as usize), to_comp.colors[i * to_comp.size.0 + j]);
				}
			}
		}
	}
	
	pub fn render (&self, term_color_support: &TermColorSupport) {
		for i in 0..self.size.1 {
			crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveTo(0, i as u16)).unwrap();
			let mut line = String::new();
			for j in 0..self.size.0 {
				if j == 0 {
					match self.colors[i * self.size.0 + j].to_escape(&term_color_support) {
						Some(e) => line.push_str(e.as_str()),
						None => {}
					}
				} else {
					if self.colors[i * self.size.0 + j] != self.colors[i * self.size.0 + j - 1] {
						match self.colors[i * self.size.0 + j].to_escape(&term_color_support) {
							Some(e) => line.push_str(e.as_str()),
							None => {}
						}
					}
				}
				
				line.push(self.chars[i * self.size.0 + j] as char);
			}
			line.push_str("\x1b[0m\n");
			print!("{}", line);
		}
	}
}