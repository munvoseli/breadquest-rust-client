use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use futures_util::{StreamExt, future, pin_mut, SinkExt, Sink, Stream};
use http::request::Request;
use std::sync::{Arc, Mutex};

pub fn add_connection_eventually(user: String, pass: String, conns: &mut Arc<Mutex<Vec<Apioform>>>) {
	let mut conns = conns.clone();
	tokio::spawn(async move {
		let consid = creds_to_consid(&user, &pass);
		let mut request = Request::builder()
		  .uri("wss://ostracodapps.com:2626/gameUpdate")
		  .header("Cookie", consid)
		  .body(()).unwrap();
		let (mut ws, resp) = tokio_tungstenite::connect_async(request).await.expect("Can't connnect");
		let (mut sink, mut stream) = ws.split();
		sink.send(tungstenite::Message::Text("[]".to_string())).await.unwrap();
		let mut strs = Arc::new(Mutex::new(Vec::new()));
		let mut strsc = strs.clone();
		{
			let mut conns = conns.lock().unwrap();
			conns.push(Apioform {
				user: user, pass: pass,
				sink: sink, strs: strs
			});
		}
		while let Some(Ok(tungstenite::Message::Text(message))) = stream.next().await {
			let mut strs = strsc.lock().unwrap();
			strs.push(message);
		}
	});
}

pub struct Apioform {
	pub user: String,
	pass: String,
	strs: Arc<Mutex<Vec<String>>>,
	sink: futures::stream::SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tungstenite::Message>,
//	stream: futures::stream::SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>
}
impl Apioform {
	pub async fn send(&mut self, data: String) {
		self.sink.send(tungstenite::Message::Text(data)).await.unwrap();
	}
	pub fn poll_next(&mut self) -> Option<String> {
		let mut strs = self.strs.lock().unwrap();
		if strs.len() > 0 {
			let m = strs.remove(0);
			return Some(m);
		}
		None
	}
}

pub fn creds_to_consid(user: &String, pass: &String) -> String {
	use std::net::TcpStream;
	use native_tls::TlsConnector;
	use std::io::{Write, Read};
	let connector = TlsConnector::new().unwrap();
	let stream = TcpStream::connect("ostracodapps.com:2626").unwrap();
	let mut stream = connector.connect("ostracodapps.com", stream).unwrap();
	let clen = user.len() + pass.len() + 19;
	let fstr = format!("POST /loginAction HTTP/1.1\r\nHost: ostracodapps.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\nusername={}&password={}\r\n", clen, user, pass);
	println!("Sending post request");
	stream.write_all(fstr.as_bytes()).unwrap();
	println!("Waiting for post response");
	let mut pt = Vec::new();
	stream.read_to_end(&mut pt).unwrap();
	let strig = String::from_utf8(pt).unwrap();
	let considn = strig.find("set-cookie").unwrap() + 12;
	let considm = &strig[considn..].find("\n").unwrap();
	let concookie = (&strig[considn..considn+considm]).to_string();
	let considm = &strig[considn..].find(";").unwrap();
	let concookie = (&strig[considn..considn+considm]).to_string();
	println!("Got post response, cookie {}", concookie);
	concookie
}
