// you know what a good name for this is?
// crust
// breadquest and rust
use std::thread;
use std::fs::File;
use std::future::Future;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::convert::TryInto;
//use std::convert::TryFrom;
use std::io::{Write, Read, stdout};
//use std::io;

mod qc;
mod signin;

struct Enemy { x: i32, y: i32, }

struct Player {
	pindex: i32,
	x: i32,
	y: i32,
	user: String,
	comque: Vec<String>,
	enemies: Vec<Enemy>,
	tile_arr: [u8; 128*128],
	tcp: websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>>
}


/* i am not entirely convinced
   that the song sunny came home
   is not about death note
   think about it
   especially that list of names stanza
   */
fn main() {
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
	let infvec = get_login_name();
	let mut player_sockets: Vec<std::sync::mpsc::Receiver<websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>>>> = Vec::new();
	for i in 0..(infvec.len() / 2) {
		let (tx, rx) = std::sync::mpsc::channel();
		let user = infvec[i * 2].to_string();
		let pass = infvec[i * 2 + 1].to_string();
		thread::spawn(move || {
			let tcpclient = signin::login_socket(user, pass);
			tx.send(tcpclient).unwrap();
		});
		player_sockets.push(rx);
	}
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

		for rx in &player_sockets { match rx.try_recv() { // websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>>
			Ok(tcpclient) => {
				let mut player = Player {pindex: pindex, x: 0, y: 0, user: "fluttershy".to_string(), tile_arr: [60; 128*128], enemies: Vec::new(), comque: Vec::new(), tcp: tcpclient};
				qc::initial_commands(&mut player.comque);
				println!("Set up player {}", player.user);
				players.push(player);
				pindex = pindex + 1;
			},
			Err(_) => {}
		}}
		// now do things for each player
		for mut player in &mut players {
		draw(&mut canvas, &player);
		if i % 64 == 0 {
			qc::get_entities(&mut player.comque);
			qc::get_tiles(&mut player.comque);
		}
		'message_loop: for message in player.tcp.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(_) => {break 'message_loop;}
			};
			let vecstr = match message {
				websocket::OwnedMessage::Text(stri) => stri,
				_ => {break 'message_loop;}
			};
//			println!("{:?}", vecstr);
			let respdata = json::parse(&vecstr).unwrap();
			for command in respdata["commandList"].members() {
				let typ:&str = command["commandName"].as_str().unwrap();
				println!("{} {}", player.user, typ);
				/*
					println!("{}", command.dump());
				*/
				let ty = String::from(typ);
				if ty.eq("setTiles") {
					cq_set_tiles(command, &mut player.tile_arr);
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
		qc::send_commands(&mut player.tcp, &player.comque);
		player.comque = Vec::new();
		}
//		println!("{:?}", tcpclient.into_stream());
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

fn draw(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, player: &Player) {
//	let mut i = 0;
//	for x in 0..128 {
//		for y in 0..128 {
//			canvas.set_draw_color(Color::RGB(tile_arr[i], tile_arr[i], tile_arr[i]));
//			let r = sdl2::rect::Rect::new(x * 4, y * 4, 4, 4);
//			canvas.fill_rect(Some(r)).ok();
//			i = i + 1;
//		}
//	}
	let ssize: i32 = 60;
	let shsize: i32 = ssize >> 1;
	for y in 0..ssize {
		for x in 0..ssize {
			let tile = get_tile_at_pos(player.x + x - shsize, player.y + y - shsize, player.tile_arr);
			if tile >= 0x80 && tile <= 0x89 {
				canvas.set_draw_color(Color::RGB(
							[255,255,255,255,  0,  0,  0,255,170][tile as usize - 0x80],
							[255,  0,170,255,255,255,  0,  0,170][tile as usize - 0x80],
							[255,  0,  0,  0,  0,255,255,255,170][tile as usize - 0x80]));
			} else {
				canvas.set_draw_color(Color::RGB(tile, tile, tile));
			}
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
