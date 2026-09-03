// https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/NetworkingTopics/Articles/UsingSocketsandSocketStreams.html

/*
the article above:
Use POSIX calls if cross-platform portability is required.
 */

/* Documntation:

*/
use libc::sockaddr;

enum SockStatus {
    Initialised,
    Uninitialied, // same as inactive
}

pub enum SockRole {
    Client {
        host: String,
        port: u16,
    },
    Server {
        interface: String,
        port: u16,
        backlog: u32,
    },
}

pub struct Sock {
    sock: std::os::unix::io::RawFd,
    role: SockRole,
    sock_status: SockStatus,
}

impl Sock {
    /*
     manpage:
     socket()  creates  an endpoint for communication and returns a file de‐
    scriptor that refers to that endpoint.  The file descriptor returned by
    a successful call will be the lowest-numbered file descriptor not  cur‐
    rently open for the process.

    The  domain argument specifies a communication domain; this selects the
    protocol family which will be used for communication.   These  families
    are defined in <sys/socket.h>.
    */

    fn _build_sock_addr(ip: std::net::IpAddr, port: u16) -> libc::sockaddr_in {
        let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        // we must ensure the IP is actually IPv4 because sockaddr_in is IPv4 only
        if let std::net::IpAddr::V4(ipv4) = ip {
            addr.sin_family = libc::AF_INET as u16;
            addr.sin_port = libc::htons(port);
            let octets = ipv4.octets();
            addr.sin_addr.s_addr = u32::from_be_bytes(octets);
        } else {
            // Handle IPv6 or error
        }
        addr
    }

    fn _connect_socket(_socket: std::os::unix::io::RawFd, ip: std::net::IpAddr, port: u16) -> i32 {
        let _port: u16 = if port == 0 { 8080 } else { port };

        let addr = Sock::_build_sock_addr(ip, _port);
        let len = size_of::<libc::sockaddr_in>() as libc::socklen_t;
        let res;
        unsafe {
            res = libc::connect(
                _socket,
                &addr as *const libc::sockaddr_in as *const sockaddr,
                len,
            );
        }
        if res != 0 {
            eprintln!("could not connect socket.");
            Self::_err();
        }
        res
    }

    fn _err() {
        let err = std::io::Error::last_os_error();
        let s_err = err.to_string();

        eprintln!("Error:\t{}\n", s_err);

        std::process::exit(1);
    }

    fn _build_socket_fd() -> std::os::unix::io::RawFd {
        let mut _sockfd: std::ffi::c_int = 0;

        // hardcode tcp
        _sockfd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0 as std::ffi::c_int) };

        if _sockfd < 0 {
            eprintln!("Could not create socket file descriptor.");
            Sock::_err();
        }

        _sockfd
    }

    fn _open(_socket: std::os::unix::io::RawFd, ip: &str, port: u16) -> std::os::unix::io::RawFd {
        let _ip: std::net::IpAddr = ip.parse().expect("Invalid IP address.");
        Sock::_connect_socket(_socket, _ip, port);
        _socket
    } // internal

    fn _bind(&self, interface: String, port: u16) {
        // man setsockopt:
        // SO_REUSEADDR lets us re-bind fast after a restart
        let _one: std::ffi::c_int = 1;
        unsafe {
            libc::setsockopt(
                self.sock,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &_one as *const std::ffi::c_int as *const std::ffi::c_void,
                size_of::<std::ffi::c_int>() as libc::socklen_t,
            );
        }
        let sockaddr = Sock::_build_sock_addr(interface.parse().unwrap(), port);
        let len = size_of::<libc::sockaddr_in>() as libc::socklen_t; // hardcodes ipv4
        let res;
        unsafe {
            res = libc::bind(
                self.sock,
                &sockaddr as *const libc::sockaddr_in as *const libc::sockaddr,
                len,
            );
        }
        if res < 0 {
            // error path
            eprintln!("Could not bind, error: ({res})");
            Sock::_err();
        }
    }

    pub fn accept(&self) -> Sock {
        // man 2 accept:
        // accept returns a fd to the new sock, the old one keeps listening.
        // so we return a brand new Sock and leave self alone.
        // maybe it will cause a problem with Drop ...
        let mut _addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut _len = size_of::<libc::sockaddr_in>() as libc::socklen_t;

        let _fd;
        unsafe {
            _fd = libc::accept(
                self.sock as libc::c_int,
                &mut _addr as *mut libc::sockaddr_in as *mut libc::sockaddr,
                &mut _len,
            );
        }
        if _fd < 0 {
            eprintln!("Could not accept.");
            Sock::_err();
        }

        // the accepted sock has no role yet, mark it client-ish
        Sock {
            sock: _fd,
            role: SockRole::Client {
                host: "accepted".to_string(),
                port: 0,
            },
            sock_status: SockStatus::Initialised,
        }
    }

    fn _listen(&self) {
        let backlog = match &self.role {
            SockRole::Server { backlog, .. } => *backlog as std::ffi::c_int,
            SockRole::Client { .. } => {
                eprintln!("Cannot listen on a client socket.");
                Sock::_err();
                return;
            }
        };

        let res;

        unsafe {
            res = libc::listen(self.sock, backlog);
        }
        if res < 0 {
            eprintln!("Could not listen on socket.");
            Self::_err();
        }
    }

    fn _change_sock_status(&mut self, _new_sock_status: SockStatus) {
        self.sock_status = _new_sock_status;
    }

    pub fn new(sock_role: SockRole) -> Self {
        let mut _s: Sock;
        match &sock_role {
            SockRole::Client { host, port } => {
                _s = Sock {
                    sock: 0,
                    role: SockRole::Client {
                        host: host.to_string(),
                        port: *port,
                    },
                    sock_status: SockStatus::Uninitialied,
                };
                _s.sock = Self::_build_socket_fd();
                _s.sock = Sock::_open(_s.sock, host, *port);
            }
            SockRole::Server {
                interface,
                port,
                backlog,
            } => {
                _s = Sock {
                    sock: 0,
                    role: SockRole::Server {
                        interface: interface.to_string(),
                        port: *port,
                        backlog: *backlog,
                    },
                    sock_status: SockStatus::Uninitialied,
                };
                _s.sock = Self::_build_socket_fd();
                Self::_bind(&_s, interface.clone(), *port);
                Self::_listen(&_s);
            }
        }

        Sock::_change_sock_status(&mut _s, SockStatus::Initialised);

        _s
    }

    pub fn fd(&self) -> std::os::unix::io::RawFd {
        self.sock
    }

    pub fn write(&self, data: &[u8]) -> libc::ssize_t {
        // send data to network sockets created with socket::create!
        let res: libc::ssize_t;

        unsafe {
            res = libc::write(
                self.sock,
                data.as_ptr() as *const std::ffi::c_void,
                data.len() as libc::size_t,
            )
        }
        if res < 0 {
            // error path
            eprintln!("Could not write, error: ({res})");
            Sock::_err();
        }
        res
    }

    pub fn read(&self, buffer: &mut [u8]) -> libc::ssize_t {
        let res;
        unsafe {
            res = libc::read(
                self.sock,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                buffer.len() as libc::size_t,
            );
        }
        if res < 0 {
            // error path
            eprintln!("Could not read, error: ({res})");
            Sock::_err();
        }

        res
    }

    pub fn read_exact(&self, mut buffer: &mut [u8]) -> libc::ssize_t {
        // man 2 read:
        // read can give back less than asked, so loop till full or closed.
        let mut _total: libc::ssize_t = 0;
        while !buffer.is_empty() {
            let mut _one = [0u8; 4096];
            let _want = std::cmp::min(buffer.len(), _one.len());
            let res;
            unsafe {
                res = libc::read(
                    self.sock,
                    _one.as_mut_ptr() as *mut std::ffi::c_void,
                    _want as libc::size_t,
                );
            }
            if res < 0 {
                eprintln!("Could not read, error: ({res})");
                Sock::_err();
            }
            if res == 0 {
                break; // other side closed
            }
            let _n = res as usize;
            buffer[.._n].copy_from_slice(&_one[.._n]);
            buffer = &mut buffer[_n..];
            _total += res;
            if _total > 0 && buffer.is_empty() {
                break;
            }
        }
        _total
    }

    pub fn write_all(&self, mut data: &[u8]) -> libc::ssize_t {
        // man 2 write:
        // write can write less than asked, so loop till all sent.
        let mut _total: libc::ssize_t = 0;
        while !data.is_empty() {
            let res;
            unsafe {
                res = libc::write(
                    self.sock,
                    data.as_ptr() as *const std::ffi::c_void,
                    data.len() as libc::size_t,
                );
            }
            if res <= 0 {
                eprintln!("Could not write, error: ({res})");
                Sock::_err();
            }
            let _n = res as usize;
            data = &data[_n..];
            _total += res;
        }
        _total
    }
}

impl Drop for Sock {
    fn drop(&mut self) {
        let res;
        unsafe {
            res = libc::close(self.sock);
        }
        // man 2 close
        if res < 0 {
            eprintln!("Could not close socket.");
            Sock::_err();
        } else {
            eprintln!("Closed socket!");
        }
    }
}
