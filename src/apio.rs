use std::thread;
use std::convert::TryInto;
use std::sync::Arc;
use std::borrow::Cow;
use tokio::runtime::Builder;
use std::borrow::Borrow;
use std::pin::Pin;
use websocket::futures::stream::SplitStream;
use websocket::futures::stream::SplitSink;
use websocket::futures::{Future, Sink, Stream};
use websocket::header::Cookie;
use websocket::header::Headers;
use websocket::r#async::client::{Client,ClientNew};
use websocket::r#async::TcpStream;
use websocket::r#async::MessageCodec;
use websocket::client::r#async::TlsStream;
use websocket::client::r#async::Framed;
use websocket::ClientBuilder;
use websocket::OwnedMessage;
use websocket::WebSocketError;
use futures::Async;
/*
use futures::stream::{SplitStream, SplitSink, Stream};
use futures::future::Future;
use futures::sink::Sink;
use std::future::Future;
use native_tls::TlsStream;
use std::io;
use std::convert::TryFrom;
use futures::Async;

*/

mod signin;

pub struct Apioform {
	ready: bool,
	sink: Option<SplitSink<Framed<TlsStream<TcpStream>, MessageCodec<OwnedMessage>>>>,
	stream: Option<SplitStream<Framed<TlsStream<TcpStream>, MessageCodec<OwnedMessage>>>>,
	str_recv_vec: Vec<String>,
	user: String,
	pass: String
}
// sends strings to and from os:2626
// every player account has their own apioform
impl Apioform {
	pub(crate) fn new(user: String, pass: String) -> Self {
//		let (dtx, drx) = std::sync::mpsc::channel();
		Self {
			ready: false,
			sink: None,
			stream: None,
			str_recv_vec: Vec::new(),
			user: user,
			pass: pass
		}
	}

	pub fn build(&mut self) {
		let mut core = tokio_core::reactor::Core::new().unwrap();
		let consid = signin::login(self.user.to_string(), self.pass.to_string());
		let mut headers_owo = Headers::new();
		headers_owo.set(Cookie(vec![consid]));
		let client_future = ClientBuilder::new("wss://ostracodapps.com:2626/gameUpdate")
		.unwrap().custom_headers(&headers_owo)
		.async_connect_secure(None)
		.map(|(client, headers)| {
			let (sink, stream) = client.split();
			println!("Got sink and stream");
//		stream.map(|message| { // download
//			match message {
//				OwnedMessage::Text(e) => {
//					println!("Apioform: receiving data {}", e);
//					self.str_recv_vec.push(e);
//				},
//				_ => {
//					println!("Apioform: receiving non-text data");
//				}
//			}
//		});
			(sink, stream)
//			client
		})
		.and_then(|(sink, stream)| -> Result<_, _> {
			self.sink = Some(sink);
			self.stream = Some(stream);
			self.ready = true;
			Ok(())
		});
		core.run(client_future).unwrap();
//		self.sink = Some(sink);
//		self.stream = Some(stream);
//		self.ready = true;
//		async {
//		while let raoo = stream.poll() { // Result Async Option OwnedMessage
//			println!("hh");
//			match raoo.unwrap() { // Async Option OwnedMessage
//			Async::Ready(oo) => {
//				match oo { // Option OwnedMessage
//				Some(om) => {
//					match om { // OwnedMessage
//					OwnedMessage::Text(e) => {
//						println!("Apioform: getting data {}", e);
//					},
//					_ => {
//						println!("Apioform: getting non-text data");
//					}
//					};
//				},
//				None => ()
//				};
//			},
//			_ => ()
//			};
//		}
//		}.await;
		//stream.poll();
	}

	pub fn is_ready(&self) -> bool {
		self.ready
	}

	pub fn send(&mut self, data: String) {
		match &mut self.sink {
			Some(x) => {
				println!("Apioform: sending data {}", data);
				x.send(OwnedMessage::Text(data)).poll().unwrap();
			},
			None => {
				println!("Apioform: not sending data {}", data);
			}
		}
	}

	pub fn poll_next(&mut self) -> Option<String> {
//		self.str_recv_vec.pop()
//	}
		match &mut self.stream {
		Some(stream) => {
			match stream.poll().unwrap() {
			Async::Ready(t) => {
				println!("Apioform: owned message");
				match t { // Option OwnedMessage
				Some(j) => {
					match j {
					OwnedMessage::Text(text) => {
						Some(text)
					},
					_ => {
						println!("Apioform: non-text data");
						None
					}
					}
				},
				None => None
				}
			},
			Async::NotReady => {
				println!("Apioform: not ready");
				None
			}
			}
		},
		None => None,
		}
	}
}



/*

		thread::spawn(move || {
			let mut core = tokio_core::reactor::Core::new().unwrap();
//			let mut runtime = Builder::new_current_thread().build().unwrap();
			let consid = signin::login(user, pass);
			let mut headers_owo = Headers::new();
			headers_owo.set(Cookie(vec![consid]));
			let client_future = ClientBuilder::new("wss://ostracodapps.com:2626/gameUpdate")
				.unwrap().custom_headers(&headers_owo)
				.async_connect_secure(None)
				.map(|(client, headers)| {
					let (sink, stream) = client.split();
					stream.map(|message| { // download
						match message {
							OwnedMessage::Text(e) => {
								dtx.send(e);
							}
							_ => ()
						}
					});
					println!("Got sink and stream");
					let reff = vec![sink];
					let x = 5;
//					unsafe {
//					urx.into_iter()
//					.map(|updog: String| {
//						(*reff).send(OwnedMessage::Text(updog));
//						x + 5;
//					});
//					}
					loop {
						let updog: String = urx.recv().unwrap();
						reff[0].send(OwnedMessage::Text(updog));
					}
//					client.split()
				});
			core.run(client_future).unwrap();
//			tx.send("tedd".to_string());
//			'message_loop: for message in client_recv.incoming_messages() {
//				let message = match message {
//					Ok(m) => m,
//					Err(e) => {println!("Exiting message loop due to {:?}", e); break 'message_loop;}
//				};
//				let vecstr = match message {
//					websocket::OwnedMessage::Text(stri) => stri,
//					_ => {break 'message_loop;}
//				};
//				tx.send(vecstr);
//			}
		});
		*/
