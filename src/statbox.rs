use crate::player::Player;
use crate::WorldTiles;
use sdl2::pixels::Color;

pub fn load_fontboi(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> u8 {
	5
}

pub fn draw_text(
	canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
	texture: &sdl2::render::Texture,
	strboi: &str,
	x: i32, y: i32) {
	let strboi = strboi.as_bytes();
	{
		canvas.set_draw_color(Color::RGB(255,255,255));
		let rct = sdl2::rect::Rect::new(x, y, (strboi.len() * 8) as u32, 8);
		canvas.fill_rect(Some(rct)).ok();
	}
	for i in 0..strboi.len() {
		let rct = sdl2::rect::Rect::new(i as i32 * 8 + x, y, 8, 8);
		let tile = strboi[i];
		let srcrct = sdl2::rect::Rect::new(
			((tile & 15) << 3) as i32,
			((tile >> 4) << 3) as i32, 8, 8);
		canvas.copy(texture, Some(srcrct), Some(rct)).unwrap();
		
	}
}

pub fn draw_statbox_at(player: &Player, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, x: u32, y: u32) {
	//           mag  red  yel  grn  blu
	let r: u8 = [255, 255, 255,   0,   0][player.health as usize - 1];
	let g: u8 = [  0,   0, 255, 255, 170][player.health as usize - 1];
	let b: u8 = [  0,   0,   0,   0, 255][player.health as usize - 1];
	canvas.set_draw_color(Color::RGB(r,g,b));
	let rct = sdl2::rect::Rect::new(x as i32, y as i32, x + 100, y + 30);
	canvas.fill_rect(Some(rct)).ok();
	
}

pub fn mouse_to_world(
	mouse: &(i32, i32),
	cam: &(i32, i32),
	ww: i32, wh: i32) -> (i32, i32) {
	let mxr = mouse.0 - ww / 3;
	let myr = mouse.1 - wh / 2;
	((mxr >> 3) + cam.0, (myr >> 3) + cam.1)
}

pub fn draw_tile(
	tile: u8,
	canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
	texture: &sdl2::render::Texture,
	dx: i32, dy: i32
) {
	use sdl2::rect::Rect;
	let su: u32 = 8;
	if tile >= 0x80 && tile <= 0x88 {
		let r = [255,255,255,238,  0,  0,  0,204,170][tile as usize - 0x80];
		let g = [255,  0,170,238,204,204,  0,  0,170][tile as usize - 0x80];
		let b = [255,  0,  0,  0,  0,204,255,204,170][tile as usize - 0x80];
		canvas.set_draw_color(Color::RGB(r,g,b));
		let rct = Rect::new(dx, dy, su, su);
		canvas.fill_rect(Some(rct)).ok();
	} else if tile >= 0x89 && tile <= 0x90 {
		canvas.set_draw_color(Color::RGB(255,255,255));
		let rct = Rect::new(dx, dy, su, su);
		canvas.fill_rect(Some(rct)).ok();
		let r = [255,255,255,170,170,170,255,238][tile as usize - 0x89];
		let g = [170,238,255,255,255,170,170,238][tile as usize - 0x89];
		let b = [170,170,170,170,255,255,255,238][tile as usize - 0x89];
		canvas.set_draw_color(Color::RGB(r,g,b));
		let rct = Rect::new(dx + 2, dy + 2, su - 4, su - 4);
		canvas.fill_rect(Some(rct)).ok();
	} else if (tile >= 0x21 && tile <= 0x7e) || (tile >= 0x91 && tile <= 0x94) {
		canvas.set_draw_color(Color::RGB(255,255,255));
		let rct = Rect::new(dx, dy, su, su);
		canvas.fill_rect(Some(rct)).ok();
		let srcrct = sdl2::rect::Rect::new(
			((tile & 15) << 3) as i32,
			((tile >> 4) << 3) as i32, 8, 8);
		canvas.copy(texture, Some(srcrct), Some(rct)).unwrap();
	} else if tile == 0x95 || tile == 0x96 {
		let rct = Rect::new(dx, dy, su, su);
		let srcrct = sdl2::rect::Rect::new(
			((tile & 15) << 3) as i32,
			((tile >> 4) << 3) as i32, 8, 8);
		canvas.copy(texture, Some(srcrct), Some(rct)).unwrap();
	} else {
		let r = 51;
		let g = 51;
		let b = 51;
		canvas.set_draw_color(Color::RGB(r,g,b));
		let rct = Rect::new(dx, dy, su, su);
		canvas.fill_rect(Some(rct)).ok();
	}
}

pub fn draw_chunk_portion(
	canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
	texture: &sdl2::render::Texture,
	world_tiles: &mut WorldTiles,
	ci: usize,
	x0: i32, y0: i32, x2: i32, y2: i32,
	ox: i32, oy: i32
) {
	for y in y0..y2 {
	for x in x0..x2 {
		let tile = world_tiles.chunks[ci].get_tile_at_off(x, y);
		draw_tile(tile, canvas, texture, ox + x * 8, oy + y * 8);
	}}
}

pub fn draw_chunks(
	canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
	world_tiles: &mut WorldTiles,
	texture: &sdl2::render::Texture,
	x0: i32, y0: i32, x2: i32, y2: i32
) {
	let cx0 = x0 & -128i32;
	let cy0 = y0 & -128i32;
	let cx2 = x2 & -128i32;
	let cy2 = y2 & -128i32;
	for cy in (cy0..=cy2).step_by(128) {
	for cx in (cx0..=cx2).step_by(128) {
		let ci = world_tiles.get_chunk_id_at(cx, cy);
		let x4 = if cx < x0 { x0 - cx } else { 0 };
		let y4 = if cy < y0 { y0 - cy } else { 0 };
		let x6 = if cx + 128 > x2 { x2 - cx } else { 128 };
		let y6 = if cy + 128 > y2 { y2 - cy } else { 128 };
		draw_chunk_portion(
			canvas, texture, world_tiles, ci,
			x4, y4, x6, y6,
			(cx - x0) * 8, (cy - y0) * 8
		);
	}}
}

pub fn draw_login_states(
	canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
	texture: &sdl2::render::Texture,
	namevec: &Vec<String>,
	login_states: &Vec<u8>,
	x0: i32, boxwidth: u32, y0: i32, ydiff: i32
) {
	canvas.set_draw_color(Color::RGB(255,255,255));
	let rct = sdl2::rect::Rect::new(
		x0, y0, boxwidth,
		ydiff as u32 * login_states.len() as u32);
	canvas.fill_rect(Some(rct)).ok();
	
	for i in 0..login_states.len() {
		draw_text(
			canvas, texture,
			&format!("{} {}", login_states[i], &namevec[2 * i]),
			x0, y0 + ydiff * (i as i32) + ydiff/2 - 4);
	}
}

pub fn draw_world(
	players: &Vec<Player>,
	canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
	world_tiles: &mut WorldTiles,
	cam: &(i32, i32),
	texture: &sdl2::render::Texture,
	ww: i32, wh: i32) {
	let s = 8;
	let x0 = cam.0 - ww / 3 / s;
	let x2 = cam.0 + ww / 3 / s;
	let y0 = cam.1 - wh / 2 / s;
	let y2 = cam.1 + wh / 2 / s;
	draw_chunks(canvas, world_tiles, texture, x0, y0, x2, y2);

	{
		canvas.set_draw_color(Color::RGB(255,255,255));
		let rct = sdl2::rect::Rect::new(0,0,
			128, players.len() as u32 * 20 + 16);
		canvas.fill_rect(Some(rct)).ok();
	}
	draw_text(canvas, texture,
		&format!("{} {}", cam.0, cam.1), 4, 4);
	let mut pi = 0;
	for player in players {
		let rct = sdl2::rect::Rect::new(
			(player.x - x0) * s,
			(player.y - y0) * s, s as u32, s as u32);
		let srcrct = sdl2::rect::Rect::new(
			((0xa7 & 15) << 3) as i32,
			((0xa7 >> 4) << 3) as i32, 8, 8);
		canvas.copy(texture, Some(srcrct), Some(rct)).unwrap();
		draw_text(canvas, texture,
			&format!("{} {} {}", player.health, player.x, player.y),
			4, pi as i32 * 20 + 16);
		draw_text(canvas, texture,
			&format!("{}", player.user),
			4, pi as i32 * 20 + 24);
		pi += 1;
	}
	for player in players {
		for enemy in &player.enemies {
			let rct = sdl2::rect::Rect::new(
				(enemy.x - x0) * s,
				(enemy.y - y0) * s, s as u32, s as u32);
			let srcrct = sdl2::rect::Rect::new(
				((0xa0 & 15) << 3) as i32,
				((0xa0 >> 4) << 3) as i32, 8, 8);
			canvas.copy(texture, Some(srcrct), Some(rct)).unwrap();
		}
	}
}
