

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
use sdl2::image::{InitFlag, LoadTexture};
use std::time::Duration;
use std::io::{Write, Read, stdout};
use crate::apio::Apioform as Apioform;
use crate::chunk::WorldTiles as WorldTiles;
use crate::player::Player;
use std::sync::{Arc, Mutex};

mod qc;
mod apio;
mod chunk;
mod player;
mod statbox;

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
	lop().await;
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

fn set_login_state(infvec: &Vec<String>, states: &mut Vec<u8>, name: &str, state: u8) {
	for i in 0..states.len() {
		if infvec[2 * i] == name {
			states[i] = state;
			break;
		}
	}
}

async fn lop() {
//	let infostr = get_login_name();
//	println!("Login info: {} {}", infostr.0, infostr.1);
//	let tcpclient = login_socket(infostr.0, infostr.1);
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("h", 1920, 960)
	 .position(480, 0).build().unwrap();
	let winwi = 1920i32;
	let winhi = 960i32;
	let mut canvas = window.into_canvas().build().unwrap();

	let _image_context = sdl2::image::init(InitFlag::PNG);
	let texture_creator = canvas.texture_creator();
	let texture = texture_creator.load_texture(std::path::Path::new("sprites.png")).unwrap();

	let mut i: i32 = 0;
	let mut event_pump = sdl_context.event_pump().unwrap();
	let infvec = get_login_name();
	let mut login_states = vec![0; infvec.len() / 2];
	let mut pending_players: Arc<Mutex<Vec<Apioform>>> = Arc::new(Mutex::new(Vec::new()));
	let mut players: Vec<Player> = Vec::new();
	let mut world_tiles = WorldTiles::new();
	for i in 0..(infvec.len() / 2) {
	}
	println!("hhh");
	let mut pindex: i32 = 0;
	let mut cam = (0i32, 0i32);
	let mut cam_tracks: Option<usize> = None;
	'running: loop {
		let now = std::time::Instant::now();
		let now2 = std::time::Instant::now();
		{
			let mut pp = pending_players.lock().unwrap();
			while let Some(apio) = pp.pop() {
				let mut player = Player::new(apio);
				set_login_state(&infvec, &mut login_states, &player.user, 2);
				player.pindex = players.len() as i32;
				qc::initial_commands(&mut player.comque);
				println!("Set up player {}", player.user);
				players.push(player);
			}
		}
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				Event::KeyDown { keycode: Some(Keycode::I), .. } => {
					match cam_tracks {
					None => {
						if players.len() > 0 {
							cam_tracks = Some(0);
						}
					},
					Some(p) => {
						if p + 1 >= players.len() {
							cam_tracks = None;
						} else {
							cam_tracks = Some(p + 1);
						}
					}
					}
				},
				Event::KeyDown { keycode: Some(Keycode::H), .. } => { cam.0 -= 8; cam_tracks = None; },
				Event::KeyDown { keycode: Some(Keycode::J), .. } => { cam.1 += 8; cam_tracks = None; },
				Event::KeyDown { keycode: Some(Keycode::K), .. } => { cam.1 -= 8; cam_tracks = None; },
				Event::KeyDown { keycode: Some(Keycode::L), .. } => { cam.0 += 8; cam_tracks = None; },
				Event::KeyDown { keycode: Some(Keycode::R), .. } => { cam.0 = 0; cam.1 = 0; cam_tracks = None; },
				Event::KeyDown { keycode: Some(Keycode::W), .. } => { qc::place_green_tile(&mut players[cam_tracks.unwrap()].comque, 0); },
				Event::KeyDown { keycode: Some(Keycode::D), .. } => { qc::place_green_tile(&mut players[cam_tracks.unwrap()].comque, 1); },
				Event::KeyDown { keycode: Some(Keycode::S), .. } => { qc::place_green_tile(&mut players[cam_tracks.unwrap()].comque, 2); },
				Event::KeyDown { keycode: Some(Keycode::A), .. } => { qc::place_green_tile(&mut players[cam_tracks.unwrap()].comque, 3); },
				Event::MouseMotion { x, y, .. } => {
				},
				Event::MouseButtonDown { x, y, mouse_btn, .. } => {
					match mouse_btn {
					sdl2::mouse::MouseButton::Left => {
						if x > winwi - 120 && y > 20 && y - 20 < login_states.len() as i32 * 20 {
							let i = ((y - 20) / 20) as usize;
							if login_states[i] == 0 {
								let user = infvec[i * 2].to_string();
								let pass = infvec[i * 2 + 1].to_string();
								set_login_state(&infvec, &mut login_states, &user, 1);
								apio::add_connection_eventually(user, pass, &mut pending_players);
							} else if login_states[i] == 2 {
								let user = &infvec[i * 2];
								set_login_state(&infvec, &mut login_states, &user, 0);
								for i in 0..players.len() {
									if &players[i].user == user {
										players.remove(i);
										cam_tracks = None;
										break;
									}
								}
							}
						} else if players[cam_tracks.unwrap()].play_mode != 0 {
							players[cam_tracks.unwrap()].play_mode = 0;
						} else {
							let wc = statbox::mouse_to_world(&(x, y), &cam, winwi, winhi);
							let relx = wc.0 - players[cam_tracks.unwrap()].x;
							let rely = wc.1 - players[cam_tracks.unwrap()].y;
							players[cam_tracks.unwrap()].try_walk(relx, rely, &mut world_tiles);
						}
					},
					sdl2::mouse::MouseButton::Right => {
						let rx = x - winwi / 3;
						let ry = y - winhi / 2;
						let m: u8 = (((ry > rx) as u8) * 3) ^ ((ry > -rx) as u8);
						players[cam_tracks.unwrap()].play_mode = m + 1;
					},
					sdl2::mouse::MouseButton::Middle => {
						println!(
							"{} ( {} {} ) {}/5",
							players[cam_tracks.unwrap()].user,
							players[cam_tracks.unwrap()].x,
							players[cam_tracks.unwrap()].y,
							players[cam_tracks.unwrap()].health
						);
					},
					_ => ()
					}
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
				qc::assert_pos(&mut player.comque);
				qc::get_stats(&mut player.comque);
//				qc::add_chat_message(&mut player.comque, "test".to_string());
			}
			if i % 32 == 0 {
				qc::get_tiles(&mut player.comque);
				qc::get_entities(&mut player.comque);
			}
			player.game_step(&mut world_tiles).await;
		}
		if let Some(p) = cam_tracks {
			cam.0 = players[p].x;
			cam.1 = players[p].y;
		}
		if now2.elapsed() > Duration::new(0, 1000000000u32 / 2) {
			println!("long time to draw {:?}", now2);
		}
		statbox::draw_world(&players, &mut canvas, &mut world_tiles, &cam, &texture, winwi, winhi);
		statbox::draw_login_states(&mut canvas, &texture, &infvec, &login_states, winwi - 120, 120, 20, 20);
		canvas.present();
		i = i + 1;
		//println!("elapsed: {:?}", now.elapsed());
		let steptime = Duration::new(0, 1000000000u32 / 32);
		let functime = now.elapsed();
		if functime < steptime {
			std::thread::sleep(steptime - functime);
		} else {
			println!("long step time of {:?}", functime);
		}
	}
	world_tiles.save_all();
}

