use libc::wait;

mod socket;

fn main() {
    println!("Hello, world!");

    let _s = socket::create::open_tcp("127.0.0.1", 8080);

    let s2: u8 = 0x40;

    let mut recv;
    loop {
        unsafe {
            wait(std::ptr::null_mut());
        }

        recv = socket::io::write(_s, s2 as *const u8, 1);
        println!("Wrote: {recv} bytes");
    }
}
