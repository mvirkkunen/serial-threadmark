use std::env;
use std::time::{Duration, Instant};
use std::thread;
use rand::{Rng, distributions::Alphanumeric};
use serialport;

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

    let mut tx_port = serialport::open(&path).unwrap();
    tx_port.set_timeout(Duration::from_secs(1));

    // Discard any buffered leftovers
    loop {
        match tx_port.read(&mut [0u8; 1024]) {
            Ok(0) => break,
            Err(ref err) if err.kind() == std::io::ErrorKind::TimedOut => break,
            Err(err) => panic!(err),
            _ => continue,
        }
    }

    let mut rx_port = tx_port.try_clone().unwrap();

    let start = Instant::now();

    let writer = thread::spawn(move || {
        let mut total = 0;
        while total < tx_data.len() {
            let count = std::cmp::min(tx_data.len() - total, 1024);
            let count = tx_port.write(&tx_data[total..total+count]).unwrap();

            total += count as usize;

            println!("write {}", total);
        }

        println!("write completed");
    });

    let reader = thread::spawn(move || {
        let mut total = 0;
        let mut buf = [0u8; 1024];

        while total < rx_data.len() {
            let count = rx_port.read(&mut buf).unwrap();

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

    let duration = start.elapsed();

    let elapsed = duration.as_secs() as f64 + (duration.subsec_micros() as f64) * 0.000_001;
    let throughput = (data_size * 8) as f64 / 1_000_000.0 / elapsed;

    println!("Time elapsed: {:?}, throughput is {:.3} Mbit/s", duration, throughput);
}
