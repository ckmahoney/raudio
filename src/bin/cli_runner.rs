use std::process::Command;
use std::process;

fn main() {
    let sample_rates = vec![44100];
    let selectors = vec![ "all", "odd", ];
    let max_harmonics = 1..=101; // Range of max harmonics for simplicity

    for &sample_rate in &sample_rates {
        for &selector in &selectors {
            for max_harmonic in max_harmonics.clone() {
                let output = Command::new("./target/release/make-wavelets")
                    .arg(max_harmonic.to_string())
                    .arg("--sample-rate")
                    .arg(sample_rate.to_string())
                    .arg("--selector")
                    .arg(selector)
                    .output()
                    .expect("Failed to execute command");

                if !output.stderr.is_empty() {
                    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                } else {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
            }
        }
    }
    process::exit(0);
}