use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use futures_util::{StreamExt, future, pin_mut, SinkExt, Sink};
use mio::net::SocketAddr;
use http::request::Request;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

mod signin;

pub struct Apioform {
	ready: bool,
	user: String,
	pass: String,
	rxdn: Option<futures_channel::mpsc::Receiver<String>>,
	strs: Arc<Mutex<Vec<String>>>,
	txup: Option<std::sync::mpsc::Sender<String>>,
	sink: Arc<Mutex<Option<futures::stream::SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tungstenite::Message>>>>,
}
impl Apioform {
	pub(crate) fn new(user: String, pass: String) -> Self {
		Self {
			ready: false, user: user, pass: pass, rxdn: None,
			strs: Arc::new(Mutex::new(Vec::new())),
			sink: Arc::new(Mutex::new(None)),
			txup: None
		}
	}
	pub fn build(&mut self) {
		let user = self.user.to_string();
		let pass = self.pass.to_string();
		let strsvec = Arc::clone(&self.strs);
		let susink = Arc::clone(&self.sink);
		let (txup, rxup) = std::sync::mpsc::channel();
		self.txup = Some(txup);
		tokio::spawn(async move {
			let url = url::Url::parse("wss://ostracodapps.com:2626/gameUpdate").unwrap();
			let consid = signin::creds_to_consid(user, pass);
			let mut request = Request::builder()
			  .uri("wss://ostracodapps.com:2626/gameUpdate")
			  .header("Cookie", consid)
			  .body(()).unwrap();
			let (mut ws, resp) = tokio_tungstenite::connect_async(request).await.expect("Can't connnect");
			let (mut sink, mut stream) = ws.split();
//			println!("Got ws sink and stream, sending brackets");
			sink.send(tungstenite::Message::Text("[]".to_string())).await.unwrap();
//			println!("Sent brackets");
//			if let Ok(mut x) = susink.lock() {
//				*x = Some(sink);
//			}
			std::thread::spawn(move || {
				let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
				for y in rxup.iter() {
					//println!("HHHH");
					let fut = sink.send(tungstenite::Message::Text(y));
					//println!("AAAA");
					rt.block_on(fut).unwrap();
				}
			});
			while let Some(message) = stream.next().await {
				let data = message.unwrap().into_data();
				match std::str::from_utf8(&data) {
					Ok(v) => {
						//println!("Received data");
						if let Ok(mut x) = strsvec.lock() {
							x.push(v.to_string());
						}
					},
					Err(e) => {
						panic!("Just some invalid, {}", e);
					}
				}
				();
			}
		});
	}
	pub fn send(&self, data: String) {
		match &self.txup {
		Some(h) => {
//			println!("Passing data along txup");
			h.send(data).unwrap();
		  },
		None => (),
		};
//		let susink = Arc::clone(&self.sink);
//		tokio::spawn(async move {
//			let x = &*susink.lock().unwrap();
//			match x {
//			Some(mut y) => {
//					y.send(tungstenite::Message::Text("[]".to_string())).await.unwrap();
//				()
//			},
//			None => ()
//			};
//		});
	}
	pub fn poll_next(&mut self) -> Option<String> {
		if let Ok(mut x) = self.strs.lock() {
			return x.pop();
		}
		None
	}
}



//pub struct Apioform {
//	ready: bool,
//	user: String,
//	pass: String
//}
//impl Apioform {
//	pub(crate) fn new(user: String, pass: String) -> Self {
//	}
//	pub fn build(&mut self) {
//	}
//	pub fn send(&self, data: String) {
//	}
//	pub fn poll_next(&mut self) -> Option<String> {
//	}
//}
