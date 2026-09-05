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

pub fn decode_212_ecg(bytes: &[u8; 3]) -> (i16, i16) {
    // decompose 3 bytes to 12-bit ints
    let mut int_a: i16 = bytes[0] as i16 + (((bytes[1] & 0x0F) as i16) << 8);
    let mut int_b: i16 = bytes[2] as i16 + (((bytes[1] & 0xF0) as i16) << 4);
    
    // interpret reconstructed 12-bit values as signed two's-complement
    if int_a >= 2048 {
        int_a -= 4096;
    } 

    if int_b >= 2048 {
        int_b -= 4096;
    }
    
    (int_a, int_b)
}

pub fn record_212_decode(data_path: &str)-> (Vec<i16>, Vec<i16>) {
    let dat = fs::read(data_path).unwrap();
    let mut a: Vec<i16> = Vec::new();
    let mut b: Vec<i16> = Vec::new();
    for i in (0..dat.len()).step_by(3) {
        let bytes: [u8; 3] = [dat[i], dat[i + 1], dat[i + 2]];

        let (int_a, int_b) = decode_212_ecg(&bytes);
        
        a.push(int_a);
        b.push(int_b);
    }
    (a, b) 

}
