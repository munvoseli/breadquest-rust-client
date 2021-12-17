pub fn send_commands(
	tcpclient: &mut websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>>,
	command_vec: &Vec<String>
	) {
	if command_vec.len() == 0 {return;}
	let mut comstr = "[".to_string();
	comstr.push_str(command_vec[0].as_str());
	for i in 1..command_vec.len() {
		comstr.push(',');
		comstr.push_str(command_vec[i].as_str());
	}
	comstr.push(']');
	let message = websocket::Message::text(comstr.as_str());
	tcpclient.send_message(&message).unwrap();
}

pub fn get_entities(command_vec: &mut Vec<String>) {
	command_vec.push("{\"commandName\": \"getEntities\"}".to_string());
}

