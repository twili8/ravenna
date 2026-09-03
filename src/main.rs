mod socket;

fn main() {
    println!("Hello, world!");

    let _s: socket::Sock = socket::Sock::new(socket::SockRole::Client {
        host: "127.0.0.1".to_string(),
        port: 8080,
    });

    let mut recv;

    let dat = b"RAVENNA ";

    for i in 0..101 {
        unsafe {
            libc::wait(std::ptr::null_mut());
        }

        recv = _s.write(dat);
        println!("iter {i} wrote: {recv} bytes");
    }
}
