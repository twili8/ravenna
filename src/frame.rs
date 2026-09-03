/* Documntation:
sends one protobuf Envelope as:
[u32 big-endian len][prost bytes]
so the other side knows where one message ends.
*/

use crate::proto::Envelope;
use crate::socket::Sock;
use prost::Message;

// man 2 read/write:
// read() can return less than you asked for, so we loop.

pub fn send_msg(_sock: &Sock, _msg: &Envelope) {
    let mut _buf = Vec::new();
    _msg.encode(&mut _buf).unwrap();

    // write the len first
    let _len = (_buf.len() as u32).to_be_bytes();
    _sock.write_all(&_len);
    _sock.write_all(&_buf);
}

pub fn recv_msg(_sock: &Sock) -> Option<Envelope> {
    // read the 4 byte len first
    let mut _lenbuf = [0u8; 4];
    if _sock.read_exact(&mut _lenbuf) == 0 {
        return None; // other side closed
    }
    let _len = u32::from_be_bytes(_lenbuf) as usize;

    if _len == 0 || _len > 1024 * 1024 {
        eprintln!("bad frame len: ({_len})");
        return None;
    }

    let mut _buf = vec![0u8; _len];
    if _sock.read_exact(&mut _buf) == 0 {
        return None;
    }

    match Envelope::decode(&_buf[..]) {
        Ok(_m) => Some(_m),
        Err(_e) => {
            eprintln!("could not decode protobuf.");
            None
        }
    }
}
