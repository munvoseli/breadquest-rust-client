use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, future, pin_mut, SinkExt};
use mio::net::SocketAddr;

mod signin;

pub struct Apioform {
	ready: bool,
	user: String,
	pass: String,
	rxdn: Option<futures_channel::mpsc::Receiver<String>>,
}
impl Apioform {
	pub(crate) fn new(user: String, pass: String) -> Self {
		Self {ready: false, user: user, pass: pass, rxdn: None}
	}
	pub fn build(&mut self) {
		let user = self.user.to_string();
		let pass = self.pass.to_string();
//		let (mut txdn, rxdn) = futures_channel::mpsc::channel(1024);
		tokio::spawn(async move {
			let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
			let url = url::Url::parse("wss://ostracodapps.com:2626/gameUpdate").unwrap();
			let consid = signin::creds_to_consid(user, pass);//mio::net::TcpListener::bind().unwrap().local_addr().unwrap()

//			let mut config = rustls::ClientConfig::new();
//			config.root_store.add_server_trust_anchors(
//				&webpki_roots::TLS_SERVER_ROOTS
//			);
//			let dnsname = webpki::DnsNameRef::try_from_ascii_str(
//				"ostracodapps.com").unwrap();
//			let strem = TcpStream::connect(&addrconfig.connect(dnsname, strem).await?;
			let mio_listen = mio::net::TcpListener::bind("67.205.178.12:2626".parse().unwrap());
			let strem = mio::net::TcpStream::connect("67.205.178.12:2626".parse().unwrap()).unwrap();
			let (mut ws, resp) = tungstenite::client(url, strem).expect("Can't connnect");
//			let fut = async {connect_async(url).await.expect("Apioform: failed to connect")};
//			let (mut ws, _) = rt.block_on(fut);
//			let (write, mut read) = ws.split();
//			while let Some(message) = read.next().await {
//				let data = message.unwrap().into_data();
//				txdn.send("".to_string()).await.unwrap();
//				();
//			}
		});
	}
	pub fn send(&self, data: String) {
	}
	pub fn poll_next(&mut self) -> Option<String> {
		match &mut self.rxdn {
			Some(rx) => {
				match rx.try_next() {
					Ok(Some(strr)) => {
						println!("Apioform: receiving data {}", strr);
						Some(strr)
					},
					Ok(None) => None,
					Err(_) => None
				}
			},
			None => None
		}
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
