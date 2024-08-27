// std
use std::env;
use std::io::stdout;
use std::time::Instant;
use std::thread::sleep;

// crates
use getopts::Options;
use crossterm;
use rusttype::{point, Font, Scale};

const MAX_FRAMERATE: u32 = 60;
const PUNCTUATION: &str = ".,!?;:";

fn render_word(word: &str, font: &Font, height: f32, mapping_chars: &str) -> Vec<Vec<u8>> { // See https://github.com/redox-os/rusttype/blob/master/dev/examples/ascii.rs
	// Compensate for the aspect ratio of monospace characters
	let scale = Scale {
		x: height * 2.0,
		y: height,
	};
	
	let height = height.ceil() as usize;
	
	let v_metrics = font.v_metrics(scale);
	let offset = point(0.0, v_metrics.ascent);
	let glyphs: Vec<_> = font.layout(word, scale, offset).collect();
	
	// Find the most visually pleasing width to display, imma be real idk what this does
	let width = glyphs
		.iter()
		.rev()
		.map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
		.next()
		.unwrap_or(0.0)
		.ceil() as usize;
	
	let mut pixels = vec![vec![b' '; width]; height]; // 2D array of characters, each row is a line of text
	let mapping = mapping_chars.as_bytes();
	let mapping_scale = (mapping.len() - 1) as f32;
	
	for g in glyphs {
		if let Some(bb) = g.pixel_bounding_box() {
			g.draw(|x, y, v| {
				// v should be in the range 0.0 to 1.0
				let i = (v * mapping_scale + 0.5) as usize;
				let c = mapping.get(i).cloned().unwrap_or(mapping[mapping.len() - 1]); // If there's an error we just use the maximum value
				let x = x as i32 + bb.min.x;
				let y = y as i32 + bb.min.y;
				// There's still a possibility that the glyph clips the boundaries of the bitmap
				if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
					let x = x as usize;
					let y = y as usize;
					pixels[y][x] = c;
				}
			})
		}
	}
	
	return pixels;
}

fn display(pixels: &mut Vec<Vec<u8>>, todraw: Vec<Vec<u8>>, x: i32, y: i32) { // Display pixels centered at x,y
	let origin = (x - todraw[0].len() as i32 / 2, y - todraw.len() as i32 / 2);    
	
	for i in 0..todraw.len() {
		for j in 0..todraw[i].len() {
			let x = origin.0 + j as i32;
			let y = origin.1 + i as i32;
			if x >= 0 && x < pixels[0].len() as i32 && y >= 0 && y < pixels.len() as i32 {
				pixels[y as usize][x as usize] = todraw[i][j];
			}
		}
	}
}

fn scroll(pixels: &mut Vec<Vec<u8>>, todraw: Vec<Vec<u8>>, progress: f32) {
	let tsize = crossterm::terminal::size().unwrap();
	let x = (1.0-progress) * (0.25 * tsize.0 as f32 + todraw[0].len() as f32 / 2.0) + progress * (tsize.0 as f32 * 0.75 - todraw[0].len() as f32 / 2.0);
	
	display(pixels, todraw, x as i32, tsize.1 as i32 / 2);
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} TEXT [options]", program);
	print!("{}", opts.usage(&brief));
}


fn main() {
	// getopts things
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	
	let mut opts = Options::new();
	opts.optopt("f", "font", "set font", "PATH");
	opts.optopt("s", "speed", "set text speed (default: 10)", "INTEGER");
	opts.optflag("l", "loop", "loop text");
	opts.optflag("h", "help", "print this help menu");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => { panic!("{}", f.to_string()) }
	};

	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	}
	let do_loop = matches.opt_present("l");
	let font = match matches.opt_str("f") {
		Some(path) => Font::try_from_vec(std::fs::read(&path).expect("Error reading font")).expect("Error reading font"),
		None => Font::try_from_bytes(include_bytes!("../fonts/Montserrat/Montserrat-Black.ttf") as &[u8]).expect("Error reading default font")
	};
	let speed = match matches.opt_str("s") {
		Some(inverse_speed) => 10.0 / inverse_speed.parse::<i32>().unwrap_or(10) as f32,
		None => 1.0
	};
	let text = if !matches.free.is_empty() {
		matches.free[0].clone()
	} else {
		print_usage(&program, opts);
		return;
	};
	
	let min_frametime = 1000 / MAX_FRAMERATE;
	let mut stdout = stdout();
	
	crossterm::terminal::enable_raw_mode().unwrap();
	crossterm::execute!(stdout, crossterm::cursor::Hide).unwrap();
	crossterm::execute!(stdout, crossterm::terminal::DisableLineWrap).unwrap();
	crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();
	
	'outer: loop {
		for word in text.split_whitespace() { // Main loop
			let time = ((200.0 + 60.0 * word.len() as f32 + if PUNCTUATION.contains(word.chars().last().unwrap()) {350.0} else {0.0}) * speed) as i32; // Time to show current word for in milliseconds
			let timer = Instant::now();
			while timer.elapsed().as_millis() < time as u128 {
				let progress = timer.elapsed().as_millis() as f32 / time as f32;
				let frametime = Instant::now();    
				let tsize = crossterm::terminal::size().unwrap();
				let mut pixels: Vec<Vec<u8>> = vec![vec![b' '; tsize.0 as usize]; tsize.1 as usize];
				
				let render = render_word(word, &font, ((tsize.1 as f32)/2.0).max(10.0 as f32), " .:-=+*#%@"); 
				
				if render[0].len() as u16 > tsize.0 {
					scroll(&mut pixels, render, progress);    
				} else {
					display(&mut pixels, render, tsize.0 as i32 / 2, tsize.1 as i32 / 2);
				}
				
				for i in 0..pixels.len() {
					crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, i as u16)).unwrap();
					let mut line = String::new();
					for j in 0..pixels[i].len() {
						line.push(pixels[i][j] as char);
					}
					print!("{}", line);
				}
				
				if crossterm::event::poll(std::time::Duration::from_secs(0)).unwrap() {
					if let crossterm::event::Event::Key(_key_event) = crossterm::event::read().unwrap() {
						break 'outer;
					}
				}
				
				if frametime.elapsed().as_millis() < min_frametime as u128 {
					sleep(std::time::Duration::from_millis(min_frametime as u64) - frametime.elapsed());
				}
			}
		}
		
		if !do_loop {
			break;
		}
	}
	
	crossterm::execute!(stdout, crossterm::terminal::EnableLineWrap).unwrap();
	crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen).unwrap();
	crossterm::execute!(stdout, crossterm::cursor::Show).unwrap();
}
