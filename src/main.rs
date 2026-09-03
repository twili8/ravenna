mod socket;

fn main() {
    println!("Hello, world!");

    let _s = socket::create::open_tcp("127.0.0.1", 8080);

    let s2: u8 = 0x67;

    socket::io::write(_s, s2 as *const u8, 7);
}
