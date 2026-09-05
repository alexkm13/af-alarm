use std::fs;
use std::io;

pub fn parse_record_line() -> io::Result<()> {
    let contents = fs::read_to_string("data/100.hea")?;
    let mut lines = contents.lines();
    let first_line = lines.next().unwrap();
    let mut split_first_line = first_line.split_whitespace();
    
    let record_name = split_first_line.next().unwrap();
    let num_signals: u32 = split_first_line.next().unwrap().parse().unwrap();
    let sample_rate: u32 = split_first_line.next().unwrap().parse().unwrap();
    let num_samples: u32 = split_first_line.next().unwrap().parse().unwrap();

    println!(
        "{} {} {} {}",
        record_name, num_signals, sample_rate, num_samples
    );

    Ok(())
}

pub fn read_signals(lines: &mut std::str::Lines<'_>, num_signals: u32) -> () {
    // iterate through # of lines that are signal
    for _ in 0..num_signals {
        let mut curr_file = lines.next().unwrap().split_whitespace();
        // read first two elements of signal line
        let file_name = curr_file.next().unwrap();
        let format: u32 = curr_file.next().unwrap().parse().unwrap();
        
        println!("{} {}", file_name, format)
    }
}

