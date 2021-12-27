use std::fmt;
use crate::apio::Apioform as Apioform;

pub fn send_commands(
	apio: &mut Apioform,
	command_vec: &Vec<String>
	) {
	if command_vec.len() == 0 {return;}
	//println!("Sending commands...");
	let mut comstr = "[".to_string();
	comstr.push_str(command_vec[0].as_str());
	for i in 1..command_vec.len() {
		comstr.push(',');
		comstr.push_str(command_vec[i].as_str());
	}
	comstr.push(']');
	apio.send(comstr);
}

pub fn get_entities(command_vec: &mut Vec<String>) {
	command_vec.push("{\"commandName\": \"getEntities\"}".to_string());
}

pub fn start_playing(command_vec: &mut Vec<String>) {
	command_vec.push("{\"commandName\": \"startPlaying\"}".to_string());
}

pub fn assert_pos(command_vec: &mut Vec<String>) {
	command_vec.push("{\"commandName\":\"assertPos\", \"pos\":{\"x\":0,\"y\":0}}".to_string());
}

pub fn get_stats(command_vec: &mut Vec<String>) {
	command_vec.push("{\"commandName\":\"getStats\"}".to_string());
}

pub fn get_tiles(command_vec: &mut Vec<String>) {
	command_vec.push("{\"commandName\":\"getTiles\",\"size\":31}".to_string());
}

pub fn add_chat_message(command_vec: &mut Vec<String>, text: String) {
	command_vec.push(format!("{{\"commandName\":\"addChatMessage\",\"text\":\"{}\"}}", text));
}

pub fn walk(command_vec: &mut Vec<String>, dir: u8) {
	command_vec.push(format!("{{\"commandName\":\"walk\",\"direction\":{}}}", dir));
}

pub fn remove_tile(command_vec: &mut Vec<String>, dir: u8) {
	command_vec.push(format!("{{\"commandName\":\"removeTile\",\"direction\":{}}}", dir));
}

pub fn place_green_tile(command_vec: &mut Vec<String>, dir: u8) {
	command_vec.push(format!("{{\"commandName\":\"removeTile\",\"direction\":{},\"tile\":132}}", dir));
}

pub fn initial_commands(command_vec: &mut Vec<String>) {
	start_playing(command_vec);
	assert_pos(command_vec);
	get_tiles(command_vec);
}
