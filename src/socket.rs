// https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/NetworkingTopics/Articles/UsingSocketsandSocketStreams.html

/*
the article above:
Use POSIX calls if cross-platform portability is required.

If you are writing networking code that runs exclusively in OS X and iOS, you should generally avoid POSIX networking calls, because they are harder to work with than higher-level APIs. However, if you are writing networking code that must be shared with other platforms, you can use the POSIX networking APIs so that you can use the same code everywhere.

Never use synchronous POSIX networking APIs on the main thread of a GUI application. If you use synchronous networking calls in a GUI application, you must do so on a separate thread.

Note: POSIX networking does not activate the cellular radio on iOS. For this reason, the POSIX networking API is generally discouraged in iOS.
 */

/* Documntation:

*/
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
                &addr as *const libc::sockaddr_in as *const libc::sockaddr,
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
