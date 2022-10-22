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

const MAXTSB: u8 = 64; // should only have to go up to like 16

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
	pub dwalks_left: u8,
	pub time_since_break: u8,
	pub play_mode: u8, // manual / bore
	pub apio: Apioform
}

impl Player {
	pub fn new(apio: Apioform) -> Self {
		Self {
			pindex: 0, x: 0, y: 0, rx: 0, ry: 0, health: 0,
			user: apio.user.clone(), comque: Vec::new(), enemies: Vec::new(),
			apio: apio,
			walks_to: [255; 67*67], dwalks_left: 64, play_mode: 0,
			time_since_break: MAXTSB,
		}
	}
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
	// [0] represents the last direction in the walk
	// [len-1] represents the first direction in the walk
	pub fn get_walk_to(&mut self, crx: i32, cry: i32, world_tiles: &mut WorldTiles) -> Vec<u8> {
		let mut relx = crx;
		let mut rely = cry;
		self.generate_pathing(world_tiles);
		let mut steps = self.get_walk_relpos(relx, rely);
		let mut walks: Vec<u8> = Vec::new();
		for i in 0..steps {
			let mut localmin = true;
			let otsep = self.get_walk_relpos(relx, rely);
			for (x, y, d) in [(0,-1,2), (1,0,3), (0,1,0), (-1,0,1)] {
				let tsep = self.get_walk_relpos(relx+x, rely+y);
				if tsep < otsep {
					relx += x;
					rely += y;
					walks.push(d);
					localmin = false;
					break;
				}
			}
			if localmin { // should only happen if it's out of bounds somehow
				break;
			}
		}
		walks
	}
	pub fn try_walk(&mut self, crx: i32, cry: i32, world_tiles: &mut WorldTiles) {
		let mut walks = self.get_walk_to(crx, cry, world_tiles);
		if walks.len() * 2 > self.dwalks_left.into() || walks.len() == 0 {
			return;
		}
		let tile = world_tiles.get_tile_at(self.x + crx, self.y + cry);
		if tile >= 0x81 && tile <= 0x88 {
			self.dwalks_left -= 2 * (walks.len() as u8);
			while walks.len() > 1 {
				let d = walks.pop().unwrap();
				qc::walk(&mut self.comque, d);
			}
			if let Some(d) = walks.pop() {
				qc::remove_tile(&mut self.comque, d);
				self.x += crx - [0, 1, 0, -1][d as usize];
				self.y += cry - [-1, 0, 1, 0][d as usize];
			}
		} else {
			self.dwalks_left -= 2 * walks.len() as u8;
			while walks.len() > 0 {
				let d = walks.pop().unwrap();
				qc::walk(&mut self.comque, d);
			}
			self.x += crx;
			self.y += cry;
		}
		qc::assert_pos(&mut self.comque);
		qc::get_tiles(&mut self.comque);
	}
	pub fn generate_pathing(&mut self, world_tiles: &mut WorldTiles) {
		let mut i = 0;
		const WALKABLE: u8 = 255;
		const UNWALK: u8 = 254;
		for y in -33..=33 { for x in -33..=33 {
			let tile = world_tiles.get_tile_at(self.x + x, self.y + y);
			self.walks_to[i] = if
				(tile > 0 && tile <= 0x80) ||
				(tile >= 0x89 && tile <= 0x94)
				{ WALKABLE } else { UNWALK };
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
					if ntile == WALKABLE {
						self.set_walk_relpos(relx+tup.0, rely+tup.1, tile + 1);
						newcoords.push((coord.0 + tup.0, coord.1 + tup.1));
					}
				}
			}
			if newcoords.len() == 0 {
				break;
			}
			coords = newcoords;
		}
	}

	pub fn is_near_enemy(&self, dist: i32) -> bool {
		for enemy in &self.enemies {
			if (enemy.x - self.x).abs() < dist && (enemy.y - self.y).abs() < dist {
				return true;
			}
		}
		false
	}

	pub fn try_step(&mut self, world_tiles: &mut WorldTiles, dir: u8, ox: i32, oy: i32) -> bool {
		let tile = world_tiles.get_tile_at(self.x+ox, self.y+oy);
		if (tile >= 0x81 && tile <= 0x88) || tile == 0x95 || tile == 0x96 {
			return false;
		}
		qc::walk(&mut self.comque, dir);
		self.dwalks_left = self.dwalks_left - 2;
		self.x += ox;
		self.y += oy;
		true
	}

	pub async fn game_step_univ(&mut self, world_tiles: &mut WorldTiles) {
		let mut recvcom: String = self.user.to_string();
		let mut has_recv = false;
		'message_loop: loop {
			let vecstr = match self.apio.poll_next() {
				Some(str) => str,
				None => { break 'message_loop; }
			};
			has_recv = false;
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
					self.generate_pathing(world_tiles);
				} else if ty.eq("setLocalPlayerPos") {
					self.x = command["pos"]["x"].as_i32().unwrap();
					self.y = command["pos"]["y"].as_i32().unwrap();
					self.generate_pathing(world_tiles);
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
	}

	pub fn game_step_bore(&mut self, world_tiles: &mut WorldTiles) {
		if self.time_since_break < 18 {
			return;
		}
		let ox: i32 = [5,  0,1,0,-1][self.play_mode as usize];
		let oy: i32 = [5,  -1,0,1,0][self.play_mode as usize];
		if self.time_since_break == 18 {
			qc::walk(&mut self.comque, self.play_mode - 1);
			self.dwalks_left -= 2;
			self.x += ox;
			self.y += oy;
		}
		let tile = world_tiles.get_tile_at(self.x+ox, self.y+oy);
		if tile >= 0x81 && tile <= 0x88 {
			qc::remove_tile(&mut self.comque, self.play_mode - 1);
			world_tiles.set_tile_at(self.x+ox, self.y+oy, 2);
			self.time_since_break = 0;
			return;
		}
		while self.dwalks_left > 48 {
			let success = self.try_step(world_tiles, self.play_mode - 1, ox, oy);
			if !success {
				return;
			}
		}
		while self.is_near_enemy(8) && self.dwalks_left > 2 {
			qc::walk(&mut self.comque, self.play_mode - 1);
			self.dwalks_left = self.dwalks_left - 2;
			self.x += ox;
			self.y += oy;
		}
	}

	pub async fn game_step(&mut self, world_tiles: &mut WorldTiles) {
		self.game_step_univ(world_tiles).await;
		if self.play_mode == 0 { // manual single control
		} else if self.play_mode >= 1 && self.play_mode <= 5 { // bore
			self.game_step_bore(world_tiles);
		} else if self.play_mode == 6 { // multi mode
			
		}
		// closing stuffs
		if self.dwalks_left < 64 {
			self.dwalks_left = self.dwalks_left + 1;
		}
		if self.time_since_break < MAXTSB {
			self.time_since_break += 1;
		}
		qc::send_commands(&mut self.apio, &self.comque).await;
		self.comque = Vec::new();
	}
}
