// https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/NetworkingTopics/Articles/UsingSocketsandSocketStreams.html

// I do not have the ios dev headers on this computer so i will use posix API

/*
the article above:
Use POSIX calls if cross-platform portability is required.

If you are writing networking code that runs exclusively in OS X and iOS, you should generally avoid POSIX networking calls, because they are harder to work with than higher-level APIs. However, if you are writing networking code that must be shared with other platforms, you can use the POSIX networking APIs so that you can use the same code everywhere.

Never use synchronous POSIX networking APIs on the main thread of a GUI application. If you use synchronous networking calls in a GUI application, you must do so on a separate thread.

Note: POSIX networking does not activate the cellular radio on iOS. For this reason, the POSIX networking API is generally discouraged in iOS.
 */
mod sockets {
    /* Documntation:
      open_tcp returns an opaque handle that must be used with sockets::io::write.
    */
    mod create {
        use std::ffi::c_int;

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
            unsafe {
                let mut addr: libc::sockaddr_in;
                addr = std::mem::zeroed();
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
        }

        unsafe fn _connect_socket(
            _socket: std::os::unix::io::RawFd,
            ip: std::net::IpAddr,
            port: u16,
        ) -> i32 {
            unsafe {
                let _port: u16 = if port == 0 { 8080 } else { port };

                let addr = _build_sock_addr(ip, _port);
                let len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;

                let res = libc::connect(
                    _socket,
                    &addr as *const libc::sockaddr_in as *const libc::sockaddr,
                    len,
                );

                res
            }
        }
        unsafe fn _open(ip: &str, port: u16) -> std::os::unix::io::RawFd {
            let _ip: std::net::IpAddr = ip.parse().expect("Invalid IP address.");
            let mut _sockfd: c_int = 0;
            unsafe {
                // file descriptor of to-be socket
                _sockfd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0 as c_int);
            }

            if (_sockfd == libc::EACCES) {
                return -1; // privileges problem
            }
            unsafe {
                _connect_socket(_sockfd, _ip, port);
            }

            _sockfd
        } // internal

        pub fn open_tcp(ip: &str, port: u16) -> std::os::unix::io::RawFd {
            let res;
            unsafe {
                res = _open(ip, port);
            }

            res
        }
    }
    mod io {
        use libc::ssize_t;
        use std::ffi::c_void;

        pub fn write(
            _sock: std::os::fd::RawFd,
            data: *const u8,
            data_len: libc::size_t,
        ) -> ssize_t {
            // this function sends data to network sockets created with _sock!
            unsafe { libc::write(_sock, data as *const c_void, data_len.try_into().unwrap()) }
        }
        pub fn read(
            _sock: std::os::fd::RawFd,
            buffer: *mut u8,
            buffer_len: libc::size_t,
        ) -> ssize_t {
            unsafe { libc::read(_sock, buffer as *mut c_void, buffer_len) }
        }
    }
}
