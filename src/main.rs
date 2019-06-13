use std::env;
use std::thread;
use std::ffi::{c_void, CString};
use libc;
use rand::{Rng, distributions::Alphanumeric};

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    let path = args.next().expect("no port path");
    let data_size: usize = args.next().expect("no data size").parse().expect("invalid data size");

    println!("writing {} bytes to {}", data_size, path);

    let mut rng = rand::thread_rng();
    let data: String = (0..data_size)
        .map(|_| rng.sample(Alphanumeric))
        .collect();
    
    let tx_data = data.as_bytes().to_owned();
    let rx_data = data.to_ascii_uppercase().as_bytes().to_owned();

    // extra spooky
    let port = unsafe {
        libc::open(
            CString::new(path).unwrap().as_ptr(),
            libc::O_RDWR | libc:O_NOCTTY)
    };
    if port <= 0 {
        panic!("couldn't open port");
    }

    unsafe { libc::tcflush(port, libc::TCIOFLUSH); }

    let writer = thread::spawn(move || {
        let mut total = 0;
        while total < tx_data.len() {
            let count = std::cmp::min(tx_data.len() - total, 1024);
            let count = unsafe { libc::write(port, (&tx_data[total..total+count]).as_ptr() as *const c_void, count) };
            if count < 0 {
                panic!("write failed: {}", count);
            }

            total += count as usize;

            println!("write {}", total);
        }

        println!("write completed");
    });

    let reader = thread::spawn(move || {
        let mut total = 0;
        let mut buf = [0u8; 1024];

        while total < rx_data.len() {
            let count = unsafe { libc::read(port, buf.as_mut_ptr() as *mut c_void, buf.len()) };
            if count < 0 {
                panic!("read failed at {} ({})", total, count);
            }

            println!("read: {} / {}", total, rx_data.len());

            let received = &buf[..count as usize];
            let expected = &rx_data[total..total+count as usize];

            if received != expected {
                println!("mismatch at {} ({:?} != {:?})", total, received, expected);
            }

            total += count as usize;
        }

        println!("read completed");
    });

    writer.join().unwrap();
    reader.join().unwrap();
}
