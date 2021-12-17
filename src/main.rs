// you know what a good name for this is?
// crust
// breadquest and rust
use websocket::header::{Headers, Cookie};
use std::fs::File;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::convert::{TryInto, TryFrom};
use std::sync::Arc;
use std::io::{Write, Read, stdout};
use websocket::ws::Message;
//use std::io;

struct Player {
	x: i32,
	y: i32,
}

/* i am not entirely convinced
   that the song sunny came home
   is not about death note
   think about it
   especially that list of names stanza
   */
fn main() {
	println!("Hello, world!");
	let infostr = get_login_name();
	println!("Login info: {} {}", infostr.0, infostr.1);
	let tcpclient = login_socket(infostr.0, infostr.1);
	lop(tcpclient);
}

fn get_login_name() -> (String, String) {
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
	(String::from(infvec[0]), String::from(infvec[1]))
}

fn login_socket(user: String, pass: String) -> websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>> {
	let consid = login(user, pass);
	let tcpclient = sus_socket(consid);
	tcpclient.set_nonblocking(true).unwrap();
	tcpclient
}


fn sus_socket(consid: String) -> websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>> {
	let connector = native_tls::TlsConnector::new().unwrap();
	let mut headers_owo = Headers::new();
	headers_owo.set(Cookie(vec![consid]));
	let mut client = websocket::ClientBuilder::new("wss://ostracodapps.com:2626/gameUpdate")
		.unwrap().custom_headers(&headers_owo)
		.connect_secure(Some(connector)).unwrap();
	let message = websocket::Message::text("[{\"commandName\": \"startPlaying\"},{\"commandName\": \"getTiles\", \"size\": 31},{\"commandName\":\"assertPos\", \"pos\":{\"x\":0,\"y\":0}}]");
	client.send_message(&message).unwrap();
	//println!("{}", client.buffered_read_size().unwrap());
//	let resp = client.recv_message().unwrap();
//	println!("{:?}", resp);
	client

}

fn login(user: String, pass: String) -> String {
	let mut root_store = rustls::RootCertStore::empty();
	root_store.add_server_trust_anchors(
		webpki_roots::TLS_SERVER_ROOTS
		.0
		.iter()
		.map(|ta| {
			rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
				ta.subject, ta.spki, ta.name_constraints
			)
		})
	);
	let config = rustls::ClientConfig::builder()
		.with_safe_defaults()
		.with_root_certificates(root_store)
		.with_no_client_auth();
	let example_com = "ostracodapps.com".try_into().unwrap();
	let mut clientconn = rustls::ClientConnection::new
		(Arc::new(config), example_com).unwrap();
	//clientconn.writer().write(b"GET / HTTP/1.1\r\n\r\n").unwrap();
	let mut socket = std::net::TcpStream::connect("ostracodapps.com:2626").unwrap();
	let mut tls = rustls::Stream::new(&mut clientconn, &mut socket);

	let clen = user.len() + pass.len() + 19;
	let fstr = format!("POST /loginAction HTTP/1.1\r\nHost: ostracodapps.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\nusername={}&password={}\r\n", clen, user, pass);
	println!("{}", clen);
	tls.write(fstr.as_bytes()).unwrap();
	let ciphersuite = tls.conn.negotiated_cipher_suite().unwrap();
	println!("Current ciphersuite: {:?}", ciphersuite.suite());
	let mut pt = Vec::new();
	tls.read_to_end(&mut pt).unwrap();
	stdout().write_all(&pt).unwrap();
	println!();

	let strig = String::from_utf8(pt).unwrap();
	let considn = strig.find("set-cookie").unwrap() + 12;
	let considm = &strig[considn..].find(";").unwrap();
	(&strig[considn..considn+considm]).to_string()
}

fn lop(mut tcpclient: websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>>) {
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("h", 600, 600)
	.position_centered().build().unwrap();
	let mut canvas = window.into_canvas().build().unwrap();
	let mut i:i32 = 0;
	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut tile_arr: [u8; 128*128] = [60; 128*128];
	let mut player: Player = Player {x: 0, y: 0};
	'running: loop {
		i = i + 1;
		let j:u8 = (i % 256) as u8;
		canvas.set_draw_color(Color::RGB(0,0,0));
		canvas.clear();
		canvas.set_draw_color(Color::RGB(j, 64, 255 - j));
		let r = sdl2::rect::Rect::new(i%256,(i/2)%500, 100,100);
		canvas.fill_rect(Some(r)).ok();
		draw(&mut canvas, tile_arr, &player);
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				_ => {}
			}
		}
		canvas.present();
//		::std::thread::sleep(Duration::new(0, 1000000000u32 / 60));
//		::std::thread::sleep(Duration::new(0, 16666000u32));
		::std::thread::sleep(Duration::new(0, 1000000000u32 / 32));
		for message in tcpclient.incoming_messages() {
			if message.is_err() {
				break;
			}
//			println!("{:?}", message);
			let mut strr: Vec<u8> = Vec::new();
			message.unwrap().serialize(&mut strr, false).unwrap();
//			println!("{:?}", strr);
			let vecstr = String::from_utf8((&strr[4..]).to_vec()).unwrap();
			let respdata = json::parse(&vecstr).unwrap();
/*			println!("{:?}", respdata);
			println!("{:?}", respdata.dump());
			println!("{:?}", respdata["commandList"]);
*/			for command in respdata["commandList"].members() {
				let typ:&str = command["commandName"].as_str().unwrap();
				println!("{:?}", command.dump());
				let ty = String::from(typ);
				if ty.eq("setTiles") {
					cq_set_tiles(command, &mut tile_arr);
				} else if ty.eq("setLocalPlayerPos") {
					player.x = command["pos"]["x"].as_i32().unwrap();
					player.y = command["pos"]["y"].as_i32().unwrap();
				}
			}
		}
//		println!("{:?}", tcpclient.into_stream());
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

fn get_tile_at_relpos(player: &Player, xrel: i32, yrel: i32, tile_arr: [u8; 128*128]) -> u8 {
	let x = (player.x + xrel) & 127;
	let y = (player.y + yrel) & 127;
	let i = (x + y * 128) as usize;
	return tile_arr[i];
}

fn draw(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, tile_arr: [u8; 128*128], player: &Player) {
//	let mut i = 0;
//	for x in 0..128 {
//		for y in 0..128 {
//			canvas.set_draw_color(Color::RGB(tile_arr[i], tile_arr[i], tile_arr[i]));
//			let r = sdl2::rect::Rect::new(x * 4, y * 4, 4, 4);
//			canvas.fill_rect(Some(r)).ok();
//			i = i + 1;
//		}
//	}
	for y in -30..30 {
		for x in -30..30 {
			let tile = get_tile_at_pos(player.x + x, player.y + y, tile_arr);
			if tile >= 0x80 && tile <= 0x89 {
				canvas.set_draw_color(Color::RGB(
							[255,255,255,255,  0,  0,  0,255,170][tile as usize - 0x80],
							[255,  0,170,255,255,255,  0,  0,170][tile as usize - 0x80],
							[255,  0,  0,  0,  0,255,255,255,170][tile as usize - 0x80]));
			} else {
				canvas.set_draw_color(Color::RGB(tile, tile, tile));
			}
			let r = sdl2::rect::Rect::new((x + 30) * 8, (y + 30) * 8, 8, 8);
			canvas.fill_rect(Some(r)).ok();
		}
	}
}
