use crate::chunk::WorldTiles;
use sdl2::pixels::Color;
use crate::qc;
use crate::Enemy;
use crate::apio::Apioform;

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

pub struct Player {
	pub pindex: i32,
	pub x: i32,
	pub y: i32,
	pub rx: i32,
	pub ry: i32,
	pub health: u8,
	pub user: String,
	pub comque: Vec<String>,
	pub enemies: Vec<Enemy>,
	pub walks_to: [u8; 67*67],
	pub walks_left: u8,
	pub play_mode: u8, // manual / bore
}

impl Player {
	pub fn get_walk_relpos(&self, x: i32, y: i32) -> u8 {
		if x.abs() > 33 || y.abs() > 33 {
			return 255;
		}
		let i = (x + 33) + (y + 33) * 67;
		self.walks_to[i as usize]
	}
	pub fn set_walk_relpos(&mut self, x: i32, y: i32, tile: u8) {
		let i = (x + 33) + (y + 33) * 67;
		self.walks_to[i as usize] = tile;
	}
	pub fn try_walk(&mut self, xpix: i32, ypix: i32, world_tiles: &WorldTiles) {
		let sctx: i32 = xpix / 8;
		let scty: i32 = ypix / 8;
		let mut relx = sctx - 30;
		let mut rely = scty - 30;
		let crx = relx;
		let cry = rely;
		let steps = self.get_walk_relpos(relx, rely);//get_tile_at_relpos(player, relx, rely, player.walks_to);
		//println!("{} {} {}", relx, rely, steps);
		let mut walks: Vec<u8> = Vec::new();
		for i in 0..steps {
			for (x, y, d) in [(0,-1,2), (1,0,3), (0,1,0), (-1,0,1)] {
				let tsep = self.get_walk_relpos(relx+x, rely+y);
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
				qc::walk(&mut self.comque, d);
			}
		}
		self.x += crx;
		self.y += cry;
		self.generate_pathing(world_tiles);
		qc::assert_pos(&mut self.comque);
		qc::get_tiles(&mut self.comque);
	}

	pub fn generate_pathing(&mut self, world_tiles: &WorldTiles) {
		let mut i = 0;
		for y in -33..=33 { for x in -33..=33 {
			let tile = world_tiles.get_tile_at(self.x + x, self.y + y);
			self.walks_to[i] = if
				(tile > 0 && tile <= 0x80) ||
				(tile >= 0x89 && tile <= 0x94)
				{ 255 } else { 254 };
			i = i + 1;
		}}
		self.set_walk_relpos(0, 0, 0);
		let mut coords: Vec<(i32, i32)> = Vec::new();
		coords.push((self.x, self.y));
		for j in 0..30 { // increase to 32 or 33 if possible
			let mut newcoords: Vec<(i32, i32)> = Vec::new();
			for coord in coords {
				let relx = coord.0 - self.x;
				let rely = coord.1 - self.y;
				let tile = self.get_walk_relpos(relx, rely);
				for tup in [(0i32,-1i32),(1,0),(0,1),(-1,0)] {
					let ntile = self.get_walk_relpos(relx+tup.0, rely+tup.1);
					if ntile != 255 { continue; }
					self.set_walk_relpos(relx+tup.0, rely+tup.1, tile + 1);
					newcoords.push((coord.0 + tup.0, coord.1 + tup.1));
				}
			}
			if newcoords.len() == 0 {
				break;
			}
			coords = newcoords;
		}
	}

	pub fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, world_tiles: &WorldTiles) {
		let ssize: i32 = 60;
		let shsize: i32 = ssize >> 1;
		let cou = 60 * 8 * (self.pindex % 3);
		let cov = 60 * 8 * (self.pindex / 3);
		for y in 0..ssize {
			for x in 0..ssize {
				let tile = world_tiles.get_tile_at(self.x + x - shsize + self.rx, self.y + y - shsize + self.ry);
				let mut r:u8;
				let mut g:u8;
				let mut b:u8;
				let mut mul: f32;
				if self.rx == 0 && self.ry == 0 {
					mul = 5.0 / (self.get_walk_relpos(x - shsize, y - shsize) as f32 + 6.0) + 1.0 / 6.0;
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
				let r = sdl2::rect::Rect::new((x + 60 * (self.pindex % 3)) * 8, (y + 60 * (self.pindex / 3)) * 8, 8, 8);
				canvas.fill_rect(Some(r)).ok();
			}
		}
		canvas.set_draw_color(Color::RGB(255,0,85));
		for enemy in &self.enemies {
			let relx = enemy.x - self.x;
			let rely = enemy.y - self.y;
			if relx.abs() < shsize && rely.abs() < shsize {
				let r = sdl2::rect::Rect::new((relx + shsize + 60 * (self.pindex % 3)) * 8 + 1, ((rely + shsize + 60 * (self.pindex / 3))) * 8 + 1, 6, 6);
				canvas.fill_rect(Some(r)).ok();
			}
		}
		canvas.set_draw_color(Color::RGB(85,0,255));
		let r = sdl2::rect::Rect::new(shsize * 8 + 1 + cou, shsize * 8 + 1 + cov, 6, 6);
		canvas.fill_rect(Some(r)).ok();
	}

	pub fn game_step(&mut self, apio: &mut Apioform, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, world_tiles: &mut WorldTiles) {
		let mut recvcom: String = self.user.to_string();
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
					cq_set_tiles(command, world_tiles);
					self.generate_pathing(&world_tiles);
				} else if ty.eq("setLocalPlayerPos") {
					self.x = command["pos"]["x"].as_i32().unwrap();
					self.y = command["pos"]["y"].as_i32().unwrap();
				} else if ty.eq("addEntity") {
					cq_add_entity(command, &mut self.enemies);
				} else if ty.eq("removeAllEntities") {
					self.enemies = Vec::new();
				} else if ty.eq("setLocalPlayerInfo") {
					self.user = command["username"].as_str().unwrap().to_string();
				} else if ty.eq("setStats") {
					//println!("{}", command.dump());
					self.health = command["health"].as_u8().unwrap();
				} else if ty.eq("addChatMessage") {
					println!("{}", command.dump());
				}
			}
		}
		if has_recv {
			println!("{}", recvcom);
		}
		if self.walks_left < 32 {
			self.walks_left = self.walks_left + 1;
		}
		qc::send_commands(apio, &self.comque);
		self.comque = Vec::new();
		self.draw(canvas, &world_tiles);
	}
}
