use prost::Message;

pub mod request {
    include!(concat!(env!("OUT_DIR"), "/mycochip.request.rs"));
}

pub fn serialize_request(req: &request::Request) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(req.encoded_len());

    req.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_request(buf: &[u8]) -> Result<request::Request, prost::DecodeError> {
    request::Request::decode(buf)
}

pub fn send_request(req: &request::Request) -> Result<zmq::Message, zmq::Error> {
    let context = zmq::Context::new();
    let requester = context.socket(zmq::REQ).unwrap();

    assert!(requester.connect("tcp://localhost:6723").is_ok());

    let msg_bytes = serialize_request(req);
    requester.send(msg_bytes, 0).unwrap();

    let mut msg = zmq::Message::new();
    requester.recv(&mut msg, 0).unwrap();
    Ok(msg)
}
