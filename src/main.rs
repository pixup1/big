// std
use std::{env, thread};
use std::io::{self, BufRead, IsTerminal, stdout};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::sync::mpsc;
use std::panic;

// crates
use getopts::Options;
use crossterm;
use rand::rngs::StdRng;
use rusttype::{point, Font, Scale};
use rand::{SeedableRng, Rng, thread_rng};

// source files
mod pixels;
mod effects;
mod color;
mod term_colors;

use effects::*;
use color::*;
use pixels::Pixels;

const MAX_FRAMERATE: u32 = 60;
const PUNCTUATION: &str = ".,!?;:";

// Map value to a character in mapping_chars
fn cacamap(mapping_chars: &str, value: f32) -> u8 {
	let mapping = mapping_chars.as_bytes();
	let mapping_scale = (mapping.len() - 1) as f32;
	let i = (value * mapping_scale + 0.5) as usize;
	mapping.get(i).cloned().unwrap_or(mapping[mapping.len() - 1]) // If there's an error we just use the maximum value
}

// See https://github.com/redox-os/rusttype/blob/master/dev/examples/ascii.rs //TODO: Do something with max_width, idk
fn render_word(word: &str, font: &Font, mapping_chars: &str, height: f32, max_width: Option<f32>) -> Pixels {
	// Compensate for the aspect ratio of monospace characters
	let scale = Scale {
		x: height * 2.0,
		y: height,
	};
	
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
	
	let mut pixels = Pixels::new((width, height as usize));
	
	for g in glyphs {
		if let Some(bb) = g.pixel_bounding_box() {
			g.draw(|x, y, v| {
				// v should be in the range 0.0 to 1.0
				let c = cacamap(mapping_chars, v);
				let x = x as i32 + bb.min.x;
				let y = y as i32 + bb.min.y;
				// There's still a possibility that the glyph clips the boundaries of the bitmap
				if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
					let x = x as usize;
					let y = y as usize;
					pixels.set_char((x,y), c);
				}
			})
		}
	}
	
	pixels
}

fn scroll(pixels: &mut Pixels, to_comp: &Pixels, frame_progress: f32, tsize: (u16, u16)) {
	let x = (1.0-frame_progress) * (0.3 * tsize.0 as f32 + to_comp.size.0 as f32 / 2.0) + frame_progress * (tsize.0 as f32 * 0.7 - to_comp.size.0 as f32 / 2.0);
	pixels.comp(& to_comp, (x as i32, tsize.1 as i32 / 2));
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} TEXT [options]", program);
	print!("{}", opts.usage(&brief));
	print!("\n{}", list_effects());
}

fn read_stdin(tx: &    mpsc::Sender<String>) {
	let stdin = io::stdin();
	let reader = stdin.lock();
	
	for line in reader.lines() {
		match line {
			Ok(content) => tx.send(content).unwrap(),
			Err(e) => panic!("Error: {}", e),
		}
	}
}

fn main() {
	// getopts things
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	
	let mut opts = Options::new();
	opts.optopt("f", "font", "set font", "PATH");
	opts.optopt("s", "speed", "set text speed (default: 10)", "INT");
	opts.optopt("S", "size", "set text size (default: 10)", "INT");
	opts.optopt("e", "effects", "pick only some effects", "EFFECT1 EFFECT2 ...");
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
	let size = match matches.opt_str("S") {
		Some(size) => size.parse::<f32>().unwrap_or(10.0)/10.0,
		None => 1.0
	};
	
	let mut free_matches = matches.free.clone(); // This one will take into account all the effects instead of just the first one, as I'm pretty sure getopts can't do that.
	
	let selected_effects: Vec<String> = if matches.opt_present("e") {
		let mut e_opt_positions:Vec<usize> = Vec::new(); // Of course getopts has an opt_positions() function but that function would be useful if it actually worked
		
		for i in 0..env::args().len() {
			if env::args().nth(i).unwrap() == "-e" {
				e_opt_positions.push(i);
			}
		}
				
		let mut fx: Vec<String> = Vec::new();
		
		'outer: for p in e_opt_positions {
			let mut i = p + 1;
			loop {
				if let Some(effect) = env::args().nth(i) {
					if effect.chars().nth(0).unwrap() == '-' {
						break;
					}
					for i in 0..free_matches.len() {
						if free_matches[i] == effect {
							free_matches.remove(i);
							break;
						}
					}
					fx.push(effect);
					i += 1;
				} else {
					break 'outer;
				}
			}
		}
		
		fx
	} else {
		Vec::new()
	};
	
	let mut text = String::new();
	
	let mut piped = false;
	let (tx, rx) = mpsc::channel();
	if !io::stdin().is_terminal() { // We are being piped to
		piped = true;
		 // Ignore user input
		thread::spawn(move || read_stdin(&tx));
	} else {
		if !free_matches.is_empty() {
			for word in free_matches {
				text = [text, word].join(" ");
			}
		} else {
			println!("Please either provide text as an argument or pipe it in.\n");
			print_usage(&program, opts);
			return;
		};
	}
	
	// This is used by the color functions to determine what escape sequences can be used
	let term_color_support = get_term_color_support();
	//let term_color_support = TermColorSupport::Ansi256;
	
	let min_frametime = 1000 / MAX_FRAMERATE;
	let mut stdout = stdout();
	
	crossterm::terminal::enable_raw_mode().unwrap();
	crossterm::execute!(stdout, crossterm::cursor::Hide).unwrap();
	crossterm::execute!(stdout, crossterm::terminal::DisableLineWrap).unwrap();
	crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();
	
	// This will be called on a panic so the terminal doesn't stay all messed up
	panic::set_hook(Box::new(|info| {
		let mut stdout = std::io::stdout();
		
		crossterm::terminal::disable_raw_mode().unwrap();
		crossterm::execute!(stdout, crossterm::terminal::EnableLineWrap).unwrap();
		crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen).unwrap();
		crossterm::execute!(stdout, crossterm::cursor::Show).unwrap();
		
		println!("{}", info);
	}));
	
	let full_timer = Instant::now();
	
	'outer: loop {
		for word in text.split_whitespace() { // Main loop
			let mut seed: <StdRng as SeedableRng>::Seed = Default::default();
			thread_rng().fill(&mut seed);

			// Time to show current word for in milliseconds
			let time = ((200.0 + 60.0 * word.len() as f32
				+ (word.len() as f32 - 10.0).max(0.0) * 25.0
				+ (word.len() as f32 - 20.0).max(0.0) * 15.0
				+ if PUNCTUATION.contains(word.chars().last().unwrap()) {350.0} else {0.0})
				* speed) as i32;
			
			let timer = Instant::now();
			
			while timer.elapsed().as_millis() < time as u128 {
				let frame_progress = timer.elapsed().as_millis() as f32 / time as f32;
				let frametime = Instant::now();
				let tsize = crossterm::terminal::size().unwrap();
				let mut pixels = Pixels::new((tsize.0 as usize, tsize.1 as usize));
				
				let mut rng = StdRng::from_seed(seed);
				
				apply_effect(EffectType::Background, &mut pixels, frame_progress, full_timer, speed, &mut rng, &selected_effects);
				
				let scroll_fit = rng.gen::<bool>(); // If false words will be shrunk to fit
				
				let max_width = match scroll_fit {
					true => None,
					false => Some((tsize.0 - 6) as f32)
				};
				
				let mut word = render_word(word, &font, " .:-=+*#%@", (size*(tsize.1 as f32)/2.0).max(10.0 as f32), max_width); 
				
				let size_before_effect = word.size;
				
				apply_effect(EffectType::Text, &mut word, frame_progress, full_timer, speed, &mut rng, &selected_effects);
				
				if size_before_effect.0 as u16 > tsize.0 /*&& scroll_fit*/ {
					scroll(&mut pixels, &word, frame_progress, tsize);
				} else {
					pixels.comp(&word, (tsize.0 as i32 / 2, tsize.1 as i32 / 2));
				}
				
				pixels.render(&term_color_support);
				
				if cfg!(debug_assertions) { //TODO: Fix this it doesn't do anything
					crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveTo(0, 0)).unwrap();
					print!("{}ms", frametime.elapsed().as_millis());
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
		
		if !do_loop && !piped {
			break;
		} else if piped {
			text = String::new();
			
			loop {
				match rx.try_recv() {
					Ok(line) => text.push_str(&line),
					Err(error) => {
						match error {
							mpsc::TryRecvError::Empty => {
								if text.is_empty() {
									sleep(Duration::from_millis(100)) // We haven't received anything yet but the thread is still running
								} else {
									break // We've received something and the thread is all good, we can print it without further waiting
								}
							},
							mpsc::TryRecvError::Disconnected => { // The io thread is done
								if text.is_empty() {
									break 'outer; // And has not sent anything
								} else {
									break; // And has sent something, we will stop next 'outer loop
								}
							} 
						};
					}
				};
			}
		}
	}
	
	crossterm::terminal::disable_raw_mode().unwrap();
	crossterm::execute!(stdout, crossterm::terminal::EnableLineWrap).unwrap();
	crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen).unwrap();
	crossterm::execute!(stdout, crossterm::cursor::Show).unwrap();
}