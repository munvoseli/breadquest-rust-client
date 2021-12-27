use std::fs::File;
use std::io::{Read, Write};

struct Chunk {
	x: i32,
	y: i32,
	tiles: [u8; 128 * 128],
}
impl Chunk {
	fn new_empty(x: i32, y: i32) -> Self {
		Self {
			x: x, y: y,
			tiles: [1; 128 * 128]
		}
	}
	// load chunk from file, generate if necessary
	fn new_maybe_file(x: i32, y: i32) -> Self {
		let f = File::open(format!("{}_{}", x, y));
		let mut chunk = Self::new_empty(x, y);
		if let Ok(mut file) = f {
			let mut buf: [u8; 9 + 128 * 128] = [1; 128*128+9];
			file.read(&mut buf).unwrap();
			for i in 0..128*128 {
				chunk.tiles[i] = buf[i + 9];
			}
		}
		chunk
	}
	fn save(&self) {
		let mut buf: [u8; 9 + 128 * 128] = [0; 128*128+9];
		for i in 0..128*128 {
			buf[i + 9] = self.tiles[i];
		}
		let mut f = File::create(format!("{}_{}", self.x, self.y)).unwrap();
		f.write(&buf).unwrap();
	}
}

pub struct WorldTiles {
	chunks: Vec<Chunk>
}

impl WorldTiles {
	fn loaded_chunk_at(&self, x: i32, y: i32) -> bool {
		for chunk in &self.chunks {
			if chunk.x == x && chunk.y == y {
				return true;
			}
		}
		return false;
	}
	fn get_chunk_id_at(&mut self, x: i32, y: i32) -> usize {
		for i in 0..self.chunks.len() {
			if self.chunks[i].x == x && self.chunks[i].y == y {
				return i;
			}
		}
		self.chunks.push(Chunk::new_empty(x, y));
		self.chunks.len() - 1
	}
	pub fn get_tile_at(&self, x: i32, y: i32) -> u8 {
		let cx = x & -128i32;
		let cy = y & -128i32;
		for chunk in &self.chunks {
			if chunk.x == cx && chunk.y == cy {
				let i = (x & 0x7F) | ((y & 0x7F) << 7);
				return chunk.tiles[i as usize];
			}
		}
		1
	}
	pub fn set_tile_at(&mut self, x: i32, y: i32, tile: u8) {
		let i = self.get_chunk_id_at(x & -128i32, y & -128i32);
		let mut chunk = &mut self.chunks[i];
		let i = (x & 127) | ((y & 127) << 7);
		chunk.tiles[i as usize] = tile;
	}
	pub fn new() -> Self {
		Self { chunks: Vec::new() }
	}
}
