mod socket;

fn main() {
    println!("Hello, world!");

    let _s: socket::Sock = socket::Sock::new("127.0.0.1", 8080);

    let mut recv;

    for i in 0..101 {
        unsafe {
            libc::wait(std::ptr::null_mut());
        }
        recv = _s.write(&[0x40]);
        println!("iter {i} wrote: {recv} bytes");
    }
}
