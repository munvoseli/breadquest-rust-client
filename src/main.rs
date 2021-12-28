

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
use crate::apio::Apioform as Apioform;
use crate::chunk::WorldTiles as WorldTiles;

mod qc;
mod apio;
mod chunk;

struct Enemy { x: i32, y: i32, }

pub struct Player {
	pindex: i32,
	x: i32,
	y: i32,
	rx: i32,
	ry: i32,
	health: u8,
	user: String,
	comque: Vec<String>,
	enemies: Vec<Enemy>,
	walks_to: [u8; 67*67]
}

impl Player {
	fn get_walk_relpos(&self, x: i32, y: i32) -> u8 {
		if x.abs() > 33 || y.abs() > 33 {
			return 255;
		}
		let i = (x + 33) + (y + 33) * 67;
		self.walks_to[i as usize]
	}
	fn set_walk_relpos(&mut self, x: i32, y: i32, tile: u8) {
		let i = (x + 33) + (y + 33) * 67;
		self.walks_to[i as usize] = tile;
	}
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
	let window = video_subsystem.window("h", 1440, 480)
	.position(480, 0).build().unwrap();
	let mut canvas = window.into_canvas().build().unwrap();
	let mut i:i32 = 0;
	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut players: Vec<Player> = Vec::new();
	let mut player_wss: Vec<String> = Vec::new();
	let infvec = get_login_name();
	let mut player_apio: Vec<Apioform> = Vec::new();
	let mut world_tiles = WorldTiles::new();
	//world_tiles.load_all_file();
	for i in 0..(infvec.len() / 2) {
		let user = infvec[i * 2].to_string();
		let pass = infvec[i * 2 + 1].to_string();
		let mut apio = Apioform::new(user, pass);
		apio.build();
		player_apio.push(apio);
		let mut player = Player {
			pindex: i as i32, x: 0, y: 0, health: 5,
			rx: 0, ry: 0,
			user: infvec[i * 2].to_string(),
			walks_to: [255; 67*67],
			enemies: Vec::new(),
			comque: Vec::new()
		};
		qc::initial_commands(&mut player.comque);
		println!("Set up player {}", player.user);
		players.push(player);
	}
	println!("hhh");
	let mut pindex: i32 = 0;
	let mut act_pli: usize = 0;
	'running: loop {
		canvas.present();
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				Event::KeyDown { keycode: Some(Keycode::H), .. } => { players[act_pli].rx -= 8; },
				Event::KeyDown { keycode: Some(Keycode::J), .. } => { players[act_pli].ry += 8; },
				Event::KeyDown { keycode: Some(Keycode::K), .. } => { players[act_pli].ry -= 8; },
				Event::KeyDown { keycode: Some(Keycode::L), .. } => { players[act_pli].rx += 8; },
				Event::KeyDown { keycode: Some(Keycode::R), .. } => { players[act_pli].rx = 0; players[act_pli].ry = 0; },
				Event::KeyDown { keycode: Some(Keycode::W), .. } => { qc::remove_tile(&mut players[act_pli].comque, 0); },
				Event::KeyDown { keycode: Some(Keycode::D), .. } => { qc::remove_tile(&mut players[act_pli].comque, 1); },
				Event::KeyDown { keycode: Some(Keycode::S), .. } => { qc::remove_tile(&mut players[act_pli].comque, 2); },
				Event::KeyDown { keycode: Some(Keycode::A), .. } => { qc::remove_tile(&mut players[act_pli].comque, 3); },
				Event::MouseMotion { x, y, .. } => {
					act_pli = ((x / 480) as usize) + ((y / 480) as usize) * 3;
				},
				Event::MouseButtonDown { x, y, .. } => {
					try_walk(&mut players[act_pli], x % 480, y % 480, &world_tiles);
				},
				_ => {}
			}
		}
		if i % 64 == 32 {
			world_tiles.unload_unused(&players);
		}

		// now do things for each player
		for mut player in &mut players {
			if i % 64 == 0 {
				qc::get_entities(&mut player.comque);
				qc::get_tiles(&mut player.comque);
				qc::assert_pos(&mut player.comque);
				qc::get_stats(&mut player.comque);
//				qc::add_chat_message(&mut player.comque, "test".to_string());
			}
			let apio = &mut player_apio[player.pindex as usize];
			let mut recvcom: String = player.user.to_string();
			let mut has_recv = false;
			'message_loop: loop {
				let vecstr = match apio.poll_next() {
					Some(str) => str,
					None => { break 'message_loop; }
				};
				has_recv = false;
				//println!("{:?}", vecstr);
				let respdata = json::parse(&vecstr).unwrap();
				for command in respdata["commandList"].members() {
					let typ:&str = command["commandName"].as_str().unwrap();
					recvcom.push(' ');
					recvcom.push_str(typ);
					//println!("{} {}", player.user, typ);
					//println!("{}", command.dump());
					let ty = String::from(typ);
					if ty.eq("setTiles") {
						cq_set_tiles(command, &mut world_tiles);
						generate_pathing(&mut player, &world_tiles);
					} else if ty.eq("setLocalPlayerPos") {
						player.x = command["pos"]["x"].as_i32().unwrap();
						player.y = command["pos"]["y"].as_i32().unwrap();
					} else if ty.eq("addEntity") {
						cq_add_entity(command, &mut player.enemies);
					} else if ty.eq("removeAllEntities") {
						player.enemies = Vec::new();
					} else if ty.eq("setLocalPlayerInfo") {
						player.user = command["username"].as_str().unwrap().to_string();
					} else if ty.eq("setStats") {
						//println!("{}", command.dump());
						player.health = command["health"].as_u8().unwrap();
					} else if ty.eq("addChatMessage") {
						println!("{}", command.dump());
					}
				}
			}
			if has_recv {
				println!("{}", recvcom);
			}
			qc::send_commands(apio, &player.comque);
			player.comque = Vec::new();
			draw(&mut canvas, &player, &world_tiles);
		}
		i = i + 1;
		::std::thread::sleep(Duration::new(0, 1000000000u32 / 32));
	}
	world_tiles.save_all();
}

fn cq_set_tiles(command: &json::JsonValue, world_tiles: &mut WorldTiles) {
	let slen: i32 = command["size"].as_i32().unwrap();
	let mut tilei = 0;
	let x0 = command["pos"]["x"].as_i32().unwrap();
	let y0 = command["pos"]["y"].as_i32().unwrap();
	for y in 0..slen {
		for x in 0..slen {
			let tile = command["tileList"][tilei].as_u8().unwrap();
			world_tiles.set_tile_at(x + x0, y + y0, tile);
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

fn try_walk(player: &mut Player, xpix: i32, ypix: i32, world_tiles: &WorldTiles) {
	let sctx: i32 = xpix / 8;
	let scty: i32 = ypix / 8;
	let mut relx = sctx - 30;
	let mut rely = scty - 30;
	let crx = relx;
	let cry = rely;
	let steps = player.get_walk_relpos(relx, rely);//get_tile_at_relpos(player, relx, rely, player.walks_to);
	//println!("{} {} {}", relx, rely, steps);
	let mut walks: Vec<u8> = Vec::new();
	for i in 0..steps {
		for (x, y, d) in [(0,-1,2), (1,0,3), (0,1,0), (-1,0,1)] {
			let tsep = player.get_walk_relpos(relx+x, rely+y);
			if tsep < steps - i {
				relx += x;
				rely += y;
				walks.push(d);
				break;
			}
		}
	}
	//println!("{:?}", walks);
	for i in 0..steps {
		if let Some(d) = walks.pop() {
			qc::walk(&mut player.comque, d);
		}
	}
	player.x += crx;
	player.y += cry;
	generate_pathing(player, world_tiles);
	qc::assert_pos(&mut player.comque);
	qc::get_tiles(&mut player.comque);
}

fn generate_pathing(player: &mut Player, world_tiles: &WorldTiles) {
	let mut i = 0;
	for y in -33..=33 { for x in -33..=33 {
		let tile = world_tiles.get_tile_at(player.x + x, player.y + y);
		player.walks_to[i] = if
			(tile > 0 && tile <= 0x80) ||
			(tile >= 0x89 && tile <= 0x94)
			{ 255 } else { 254 };
		i = i + 1;
	}}
	player.set_walk_relpos(0, 0, 0);
	let mut coords: Vec<(i32, i32)> = Vec::new();
	coords.push((player.x, player.y));
	for j in 0..30 { // increase to 32 or 33 if possible
		let mut newcoords: Vec<(i32, i32)> = Vec::new();
		for coord in coords {
			let relx = coord.0 - player.x;
			let rely = coord.1 - player.y;
			let tile = player.get_walk_relpos(relx, rely);
			for tup in [(0i32,-1i32),(1,0),(0,1),(-1,0)] {
				let ntile = player.get_walk_relpos(relx+tup.0, rely+tup.1);
				if ntile != 255 { continue; }
				player.set_walk_relpos(relx+tup.0, rely+tup.1, tile + 1);
				newcoords.push((coord.0 + tup.0, coord.1 + tup.1));
			}
		}
		if newcoords.len() == 0 {
			break;
		}
		coords = newcoords;
	}
}

fn draw(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, player: &Player, world_tiles: &WorldTiles) {
	let ssize: i32 = 60;
	let shsize: i32 = ssize >> 1;
	for y in 0..ssize {
		for x in 0..ssize {
			let tile = world_tiles.get_tile_at(player.x + x - shsize + player.rx, player.y + y - shsize + player.ry);
			let mut r:u8;
			let mut g:u8;
			let mut b:u8;
			let mut mul: f32;
			if player.rx == 0 && player.ry == 0 {
				mul = 5.0 / (player.get_walk_relpos(x - shsize, y - shsize) as f32 + 6.0) + 1.0 / 6.0;
			} else {
				mul = 1.0;
			}
			// 0x00         ???
			// 0x21, 0x7f  +spri, reduced shading
			// 0x80        +rect, shading
			// 0x81  0x88   rect, no shading
			// 0x89  0x90  +rect, shading
			// 0x91  0x96  +spri, no shading
			let draw_wbg = tile <= 0x80 || tile >= 0x89;
			if tile == 0x80 {
				r = 255;
				g = 255;
				b = 255;
			} else if tile >= 0x81 && tile <= 0x88 {
				r = [255,255,255,  0,  0,  0,255,170][tile as usize - 0x81];
				g = [  0,170,255,255,255,  0,  0,170][tile as usize - 0x81];
				b = [  0,  0,  0,  0,255,255,255,170][tile as usize - 0x81];
				mul = 0.75;
			} else if tile >= 0x91 && tile <= 0x94 {
				r = 255;
				g = 255;
				b = 255;
				mul = 1.0;
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
	canvas.set_draw_color(Color::RGB(85,0,255));
	let r = sdl2::rect::Rect::new((shsize + 60 * (player.pindex % 3)) * 8 + 1, ((shsize + 60 * (player.pindex / 3))) * 8 + 1, 6, 6);
	canvas.fill_rect(Some(r)).ok();
}
