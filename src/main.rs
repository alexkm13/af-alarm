mod wfdb;

#[derive(Debug)]
struct Header {
    record_name: String,
    num_signals: u32,
    sample_rate: u32,
    num_samples: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("data/100.hea")?;
    let mut lines = contents.lines();

    // parse record line
    let first_line = lines.next().unwrap();
    let mut parts = first_line.split_whitespace();
    let _record_name = parts.next().unwrap();
    let num_signals: u32 = parts.next().unwrap().parse()?;

    // test read_signals
    wfdb::read_signals(&mut lines, num_signals);

    Ok(())
}

