

#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

// you know what a good name for this is?
// crust
// breadquest and rust
use std::fs::File;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::io::{Write, Read, stdout};
use crate::apio::Apioform;

mod qc;
mod apio;

struct Enemy { x: i32, y: i32, }

struct Player {
	pindex: i32,
	x: i32,
	y: i32,
	user: String,
	comque: Vec<String>,
	enemies: Vec<Enemy>,
	tile_arr: [u8; 128*128],
	walks_to: [u8; 128*128]
}


/* i am not entirely convinced
   that the song sunny came home
   is not about death note
   think about it
   especially that list of names stanza
   */

#[tokio::main]
async fn main() {
	println!("Hello, world!"); // this is literally my hello cargo project and i am not removing this line
	lop();
}

fn get_login_name() -> Vec<String> {
	let path = std::path::Path::new("config.txt");
	let mut file = match File::open(&path) {
		Err(e) => panic!("Couldn't open config file: {}", e),
		Ok(file) => file
	};
	let mut s = String::new();
	match file.read_to_string(&mut s) {
		Err(e) => panic!("Couldn't read config file: {}", e),
		Ok(file) => file
	};
	let infvec: Vec<&str> = s.split("\n").collect();
	//(String::from(infvec[0]), String::from(infvec[1]))
	infvec.iter().map(|&x| String::from(x)).collect()
}

fn lop() {
//	let infostr = get_login_name();
//	println!("Login info: {} {}", infostr.0, infostr.1);
//	let tcpclient = login_socket(infostr.0, infostr.1);
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("h", 1440, 960)
	.position_centered().build().unwrap();
	let mut canvas = window.into_canvas().build().unwrap();
	let mut i:i32 = 0;
	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut players: Vec<Player> = Vec::new();
	let mut player_wss: Vec<String> = Vec::new();
	let infvec = get_login_name();
	let mut player_apio: Vec<Apioform> = Vec::new();
	for i in 0..(infvec.len() / 2) {
		let user = infvec[i * 2].to_string();
		let pass = infvec[i * 2 + 1].to_string();
		let mut apio = Apioform::new(user, pass);
		apio.build();
		player_apio.push(apio);
		let mut player = Player {
			pindex: i as i32, x: 0, y: 0,
			user: infvec[i * 2].to_string(),
			walks_to: [255; 128*128],
			tile_arr: [60; 128*128],
			enemies: Vec::new(),
			comque: Vec::new()
		};
		qc::initial_commands(&mut player.comque);
		println!("Set up player {}", player.user);
		players.push(player);
	}
	println!("hhh");
	let mut pindex: i32 = 0;
	'running: loop {
		canvas.present();
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				Event::KeyDown { keycode: Some(Keycode::W), .. } => { qc::remove_tile(&mut players[0].comque, 0); },
				Event::KeyDown { keycode: Some(Keycode::D), .. } => { qc::remove_tile(&mut players[0].comque, 1); },
				Event::KeyDown { keycode: Some(Keycode::S), .. } => { qc::remove_tile(&mut players[0].comque, 2); },
				Event::KeyDown { keycode: Some(Keycode::A), .. } => { qc::remove_tile(&mut players[0].comque, 3); },
				_ => {}
			}
		}

		// now do things for each player
		for mut player in &mut players {
			draw(&mut canvas, &player);
			if i % 64 == 0 {
				qc::get_entities(&mut player.comque);
				qc::get_tiles(&mut player.comque);
				qc::add_chat_message(&mut player.comque, "test".to_string());
			}
			let apio = &mut player_apio[player.pindex as usize];
			'message_loop: loop {
				let vecstr = match apio.poll_next() {
					Some(str) => str,
					None => { break 'message_loop; }
				};
				println!("{:?}", vecstr);
				let respdata = json::parse(&vecstr).unwrap();
				for command in respdata["commandList"].members() {
					let typ:&str = command["commandName"].as_str().unwrap();
					println!("{} {}", player.user, typ);
					println!("{}", command.dump());
					let ty = String::from(typ);
					if ty.eq("setTiles") {
						cq_set_tiles(command, &mut player.tile_arr);
						generate_pathing(&mut player);
					} else if ty.eq("setLocalPlayerPos") {
						player.x = command["pos"]["x"].as_i32().unwrap();
						player.y = command["pos"]["y"].as_i32().unwrap();
					} else if ty.eq("addEntity") {
						cq_add_entity(command, &mut player.enemies);
					} else if ty.eq("removeAllEntities") {
						player.enemies = Vec::new();
					} else if ty.eq("setLocalPlayerInfo") {
						player.user = command["username"].as_str().unwrap().to_string();
					}
				}
			}
			qc::send_commands(apio, &player.comque);
			player.comque = Vec::new();
		}
		i = i + 1;
		::std::thread::sleep(Duration::new(0, 1000000000u32 / 32));
	}
}

fn set_tile_at_pos(x_world: i32, y_world: i32, tile_arr: &mut [u8; 128*128], tile: u8) {
	let x = x_world & 127;
	let y = y_world & 127;
	let i = (x + y * 128) as usize;
	tile_arr[i] = tile;
}

fn get_tile_at_pos(x_world: i32, y_world: i32, tile_arr: [u8; 128*128]) -> u8 {
	let x = x_world & 127;
	let y = y_world & 127;
	let i = (x + y * 128) as usize;
	return tile_arr[i];
}

fn cq_set_tiles(command: &json::JsonValue, tile_arr: &mut [u8; 128*128]) {
	let slen: u32 = command["size"].as_u32().unwrap();
	let mut tilei = 0;
	let x0 = command["pos"]["x"].as_i32().unwrap();
	let y0 = command["pos"]["y"].as_i32().unwrap();
//	let hslen: i32 = i32::try_from(slen >> 1).unwrap();
	for y in 0..slen {
		for x in 0..slen {
			let x_world = (x as i32) + x0;
			let y_world = (y as i32) + y0;
			let x_wrap = x_world & 127;
			let y_wrap = y_world & 127;
			let bufi = (x_wrap + y_wrap * 128) as usize;
			tile_arr[bufi] = command["tileList"][tilei].as_u8().unwrap();
//			set_tile_at_pos(x_world, y_world, tile_arr, command["tileList"][tilei].as_u8().unwrap());
			tilei = tilei + 1;
		}
	}
}

fn cq_add_entity(command: &json::JsonValue, enemies: &mut Vec<Enemy>) {
	enemies.push(Enemy {
		x: command["entityInfo"]["pos"]["x"].as_i32().unwrap(),
		y: command["entityInfo"]["pos"]["y"].as_i32().unwrap(),
		});
}

fn get_tile_at_relpos(player: &Player, xrel: i32, yrel: i32, tile_arr: [u8; 128*128]) -> u8 {
	let x = (player.x + xrel) & 127;
	let y = (player.y + yrel) & 127;
	let i = (x + y * 128) as usize;
	return tile_arr[i];
}

fn get_tilbufi(x: i32, y: i32) -> usize {
	((x & 127) + (y & 127) * 128) as usize
}

fn generate_pathing(player: &mut Player) {
	for i in 0..player.tile_arr.len() {
		let tile = player.tile_arr[i];
		player.walks_to[i] = if tile == 0x80 ||
			(tile >= 0x89 && tile <= 0x94)
			{ 255 } else { 254 };
		// 254 for determined unreachable / don't evaluate
	}
	player.walks_to[get_tilbufi(player.x, player.y)] = 0;
	let mut coords: Vec<(i32, i32)> = Vec::new();
	coords.push((player.x, player.y));
	loop {
		let mut newcoords: Vec<(i32, i32)> = Vec::new();
		for coord in coords {
			let bufi = get_tilbufi(coord.0, coord.1);
			let tile = player.walks_to[bufi];
			// if tile > 32 { continue; }
			for tup in [(0,-1),(1,0),(0,1),(-1,0)] {
				let nbufi = get_tilbufi(coord.0 + tup.0, coord.1 + tup.1);
				let ntile = player.walks_to[nbufi];
				if ntile != 255 { continue; }
				player.walks_to[nbufi] = tile + 1;
				newcoords.push((coord.0 + tup.0, coord.1 + tup.1));
			}
		}
		if newcoords.len() == 0 {
			break;
		}
		coords = newcoords;
	}
}

fn draw(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, player: &Player) {
	let ssize: i32 = 60;
	let shsize: i32 = ssize >> 1;
	for y in 0..ssize {
		for x in 0..ssize {
			let tile = get_tile_at_pos(player.x + x - shsize, player.y + y - shsize, player.tile_arr);
			let mut r:u8;
			let mut g:u8;
			let mut b:u8;
			let mul: f32 = 5.0 / (get_tile_at_pos(player.x + x - shsize, player.y + y - shsize, player.walks_to) as f32 + 6.0) + 1.0 / 6.0;
			if tile >= 0x80 && tile <= 0x89 {
				r = [255,255,255,255,  0,  0,  0,255,170][tile as usize - 0x80];
				g = [255,  0,170,255,255,255,  0,  0,170][tile as usize - 0x80];
				b = [255,  0,  0,  0,  0,255,255,255,170][tile as usize - 0x80];
			} else {
				r = tile;
				b = tile;
				g = tile;
			}
			r = (r as f32 * mul) as u8;
			g = (g as f32 * mul) as u8;
			b = (b as f32 * mul) as u8;
			canvas.set_draw_color(Color::RGB(r,g,b));
			let r = sdl2::rect::Rect::new((x + 60 * (player.pindex % 3)) * 8, (y + 60 * (player.pindex / 3)) * 8, 8, 8);
			canvas.fill_rect(Some(r)).ok();
		}
	}
	canvas.set_draw_color(Color::RGB(255,0,85));
	for enemy in &player.enemies {
		let relx = enemy.x - player.x;
		let rely = enemy.y - player.y;
		if relx.abs() < shsize && rely.abs() < shsize {
			let r = sdl2::rect::Rect::new((relx + shsize + 60 * (player.pindex % 3)) * 8 + 1, ((rely + shsize + 60 * (player.pindex / 3))) * 8 + 1, 6, 6);
			canvas.fill_rect(Some(r)).ok();
		}
	}
}
