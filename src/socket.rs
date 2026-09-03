// https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/NetworkingTopics/Articles/UsingSocketsandSocketStreams.html

// I do not have the ios dev headers on this computer so i will use posix API

/*
the article above:
Use POSIX calls if cross-platform portability is required.

If you are writing networking code that runs exclusively in OS X and iOS, you should generally avoid POSIX networking calls, because they are harder to work with than higher-level APIs. However, if you are writing networking code that must be shared with other platforms, you can use the POSIX networking APIs so that you can use the same code everywhere.

Never use synchronous POSIX networking APIs on the main thread of a GUI application. If you use synchronous networking calls in a GUI application, you must do so on a separate thread.

Note: POSIX networking does not activate the cellular radio on iOS. For this reason, the POSIX networking API is generally discouraged in iOS.
 */

/* Documntation:
  open_tcp returns an opaque handle that must be used with sockets::io::write.
*/

pub struct Sock {
    sock: std::os::unix::io::RawFd,
    initialised: bool,
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

    unsafe fn _build_sock_addr(ip: std::net::IpAddr, port: u16) -> libc::sockaddr_in {
        let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        // we must ensure the IP is actually IPv4 because sockaddr_in is IPv4 only
        if let std::net::IpAddr::V4(ipv4) = ip {
            addr.sin_family = libc::AF_INET as u16;
            addr.sin_port = libc::htons(port);
            let octets = ipv4.octets();
            addr.sin_addr.s_addr = u32::from_be_bytes(octets).to_be();
        } else {
            // Handle IPv6 or error
        }
        addr
    }

    unsafe fn _connect_socket(
        _socket: std::os::unix::io::RawFd,
        ip: std::net::IpAddr,
        port: u16,
    ) -> i32 {
        unsafe {
            let _port: u16 = if port == 0 { 8080 } else { port };

            let addr = self::Sock::_build_sock_addr(ip, _port);
            let len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;

            let res = libc::connect(
                _socket,
                &addr as *const libc::sockaddr_in as *const libc::sockaddr,
                len,
            );
            if (res != 0) {
                eprintln!("could not connect socket.");
                Self::_err();
            }
            res
        }
    }

    fn _err() {
        let err = std::io::Error::last_os_error();
        let s_err = err.to_string();

        eprintln!("Error:\t{}\n", s_err);

        std::process::exit(1);
    }

    unsafe fn _open(ip: &str, port: u16) -> std::os::unix::io::RawFd {
        let _ip: std::net::IpAddr = ip.parse().expect("Invalid IP address.");
        let mut _sockfd: std::ffi::c_int = 0;
        unsafe {
            // file descriptor of to-be socket
            _sockfd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0 as std::ffi::c_int);
        }

        if (_sockfd < 0) {
            eprintln!("File descriptor is negative ({_sockfd})");
            self::Sock::_err();
        }

        unsafe {
            self::Sock::_connect_socket(_sockfd, _ip, port);
        }

        _sockfd
    } // internal

    pub fn new(ip: &str, port: u16) -> Self {
        let mut _s = Sock {
            sock: 0,
            initialised: false,
        };
        _s.sock = unsafe { self::Sock::_open(ip, port) };
        _s.initialised = true;

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
            self::Sock::_err();
        }
        res
    }

    pub fn read(&self, buffer: &[u8]) -> libc::ssize_t {
        let res;
        unsafe {
            res = libc::read(
                self.sock,
                buffer.as_ptr() as *mut std::ffi::c_void,
                buffer.len() as libc::size_t,
            );
        }
        if res < 0 {
            // error path
            eprintln!("Could not read, error: ({res})");
            self::Sock::_err();
        }

        res
    }

    pub fn close(&self) -> libc::ssize_t {
        // man 2 close
        let res;

        unsafe {
            res = libc::close(self.sock);
        }

        if (res < 0) {
            eprintln!("Could not close socket.");
            self::Sock::_err();
        }

        res as libc::ssize_t
    }
}
