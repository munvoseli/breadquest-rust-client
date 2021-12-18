use std::io::{Write, Read};
use websocket::header::{Headers, Cookie};
use std::sync::Arc;
use std::convert::TryInto;
//use std::io::stdout;

pub fn login_socket(user: String, pass: String) -> websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>> {
	let consid = login(user, pass);
	let tcpclient = sus_socket(consid);
	tcpclient.set_nonblocking(true).unwrap();
	tcpclient
}


fn sus_socket(consid: String) -> websocket::client::sync::Client<native_tls::TlsStream<std::net::TcpStream>> {
	let connector = native_tls::TlsConnector::new().unwrap();
	let mut headers_owo = Headers::new();
	headers_owo.set(Cookie(vec![consid]));
	let mut client = websocket::ClientBuilder::new("wss://ostracodapps.com:2626/gameUpdate")
		.unwrap().custom_headers(&headers_owo)
		.connect_secure(Some(connector)).unwrap();
	//println!("{}", client.buffered_read_size().unwrap());
//	let resp = client.recv_message().unwrap();
//	println!("{:?}", resp);
	client

}

fn login(user: String, pass: String) -> String {
	let mut root_store = rustls::RootCertStore::empty();
	root_store.add_server_trust_anchors(
		webpki_roots::TLS_SERVER_ROOTS
		.0
		.iter()
		.map(|ta| {
			rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
				ta.subject, ta.spki, ta.name_constraints
			)
		})
	);
	let config = rustls::ClientConfig::builder()
		.with_safe_defaults()
		.with_root_certificates(root_store)
		.with_no_client_auth();
	let example_com = "ostracodapps.com".try_into().unwrap();
	println!("User {}", user);
	println!("Setting up TCP socket");
	let mut clientconn = rustls::ClientConnection::new
		(Arc::new(config), example_com).unwrap();
	//clientconn.writer().write(b"GET / HTTP/1.1\r\n\r\n").unwrap();
	let mut socket = std::net::TcpStream::connect("ostracodapps.com:2626").unwrap();
	let mut tls = rustls::Stream::new(&mut clientconn, &mut socket);

	let clen = user.len() + pass.len() + 19;
	let fstr = format!("POST /loginAction HTTP/1.1\r\nHost: ostracodapps.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\nusername={}&password={}\r\n", clen, user, pass);
//	println!("{}", clen);
	println!("Sending post request");
	tls.write(fstr.as_bytes()).unwrap();
	let ciphersuite = tls.conn.negotiated_cipher_suite().unwrap();
	println!("Current ciphersuite: {:?}", ciphersuite.suite());
	let mut pt = Vec::new();
	println!("Waiting for post request");
	tls.read_to_end(&mut pt).unwrap();
//	stdout().write_all(&pt).unwrap();
//	println!();

	let strig = String::from_utf8(pt).unwrap();
	let considn = strig.find("set-cookie").unwrap() + 12;
	let considm = &strig[considn..].find("\n").unwrap();
	let concookie = (&strig[considn..considn+considm]).to_string();
	println!("Got cookie line {}", concookie);
	let considm = &strig[considn..].find(";").unwrap();
	let concookie = (&strig[considn..considn+considm]).to_string();
	println!("Got cookie {}", concookie);
	concookie
}

