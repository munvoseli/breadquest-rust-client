

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
use crate::player::Player;

mod qc;
mod apio;
mod chunk;
mod player;

pub struct Enemy { x: i32, y: i32, }


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
			walks_left: 0, play_mode: 0,
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
					players[act_pli].try_walk(x % 480, y % 480, &world_tiles);
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
			player.game_step(apio, &mut canvas, &mut world_tiles);
		}
		i = i + 1;
		::std::thread::sleep(Duration::new(0, 1000000000u32 / 32));
	}
	world_tiles.save_all();
}

