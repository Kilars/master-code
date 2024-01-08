use std::{error::Error, fs::File};

fn read_csv() -> Result<(), Box<dyn Error>> {
    let file = File::open("./sample.csv");
    match file {
        Ok(reader) => {
            let mut rdr = csv::Reader::from_reader(reader);
            for result in rdr.records() {
                let record = result?;
                println!("{:?}", record);
            }
        }
        Err(err) => println!("Error reading file, {err}"),
    }
    Ok(())
}

fn main() {
    if let Err(err) = read_csv() {
        println!("error urnning read_csv {err}");
    }
}
