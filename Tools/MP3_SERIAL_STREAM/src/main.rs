use std::{
    env,
    fs,
    io::Write,
    thread,
    time::{Duration, Instant},
};

fn main() -> anyhow::Result<()> {
    // args: <PORT> <FILE> [BAUD] [RATE] [CHUNK]
    // example: COM5 output.raw 250000 8000 256
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <PORT> <FILE> [BAUD] [RATE] [CHUNK]", args[0]);
        eprintln!("Example: {} COM5 output.raw 250000 8000 256", args[0]);
        std::process::exit(1);
    }

    let port_name = &args[1];
    let file_path = &args[2];
    let baud: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(250_000);
    let rate: u32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(8_000);
    let chunk: usize = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(256);

    let data = fs::read(file_path)?;
    println!("Loaded {} bytes", data.len());

    let mut port = serialport::new(port_name, baud)
        .timeout(Duration::from_millis(0))
        .open()?;
    
    thread::sleep(Duration::from_millis(2000));

    let time = Instant::now();
    let mut sent: usize = 0;

    while sent < data.len() {
        let end = (sent + chunk).min(data.len());
        port.write_all(&data[sent..end])?;
        sent = end;

        let target = time + Duration::from_secs_f64(sent as f64 / rate as f64);

        loop {
            let now = Instant::now();
            if now >= target {
                break;
            }
            let left = target - now;

            if left > Duration::from_millis(2) {
                thread::sleep(Duration::from_millis(1));
            } else {
                std::hint::spin_loop();
            }
        }
    }

    println!("Done.");
    Ok(())
}

mod anyhow;

