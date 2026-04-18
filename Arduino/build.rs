use std::process::Command;

fn main() {
    let output = Command::new("python")
        .args([
            "-c",
            "from datetime import datetime; now=datetime.now(); print(now.year, now.month, now.day, now.hour, now.minute, now.second, now.isoweekday())",
        ])
        .output()
        .unwrap();

    let s = String::from_utf8(output.stdout).unwrap();
    let parts: Vec<&str> = s.split_whitespace().collect();

    println!("cargo:rustc-env=BUILD_YEAR={}", parts[0]);
    println!("cargo:rustc-env=BUILD_MONTH={}", parts[1]);
    println!("cargo:rustc-env=BUILD_DAY={}", parts[2]);
    println!("cargo:rustc-env=BUILD_HOUR={}", parts[3]);
    println!("cargo:rustc-env=BUILD_MIN={}", parts[4]);
    println!("cargo:rustc-env=BUILD_SEC={}", parts[5]);
    println!("cargo:rustc-env=BUILD_WEEKDAY={}", parts[6]);
}