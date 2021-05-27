extern crate rgas;
use argparse::{ArgumentParser, StoreTrue, Store};
use std::fs::{File,OpenOptions};
use std::io;
use std::io::BufRead;

macro_rules! check {
    ($result:expr, $message:literal) => {
        match($result) {Ok(val) => {val} Err(err) => panic!($message, err)};
    };
}

fn main() {
    println!("Hello world!");
    let mut infile = String::new();
    let mut outfile = String::new();
    
    {
        let mut ap=ArgumentParser::new();
        ap.refer(&mut infile).add_argument("infile", Store, "Input file.  The decompiler, obviously, does not have an interactive mode.");
        ap.refer(&mut outfile).add_argument("outfile", Store, "Output file.  The default is stdout.");
        ap.parse_args_or_exit();
    }

    let fout: Box<dyn io::Write> = if outfile.len() == 0 {
        let stdout = io::stdout();
        Box::new(stdout.lock())
    } else {
        match OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(outfile) {
            Ok(file) => {Box::new(file)}
            Err(msg) => {panic!("Unable to open output file: {}", msg)}
        }
    };

    let mut fin = io::BufReader::new(check!(File::open(infile), "Unable to open input file: {}"));
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match fin.read_until(b'\n', &mut buf) {
            Err(msg) => {panic!("Error reading line: {}", msg)}
            Ok(0) => {break}
            Ok(n) => {
                // this seems incredibly unsafe, not passing it a length or anything.
                // but the data has a length byte in it, so i guess i don't really care
                let opcode = check!(rgas::UCGMessageInternal::from_byte_vec(&mut buf), "parse error: {}");
                fout.write(opcode.into_asm());
            }
        }
    }

}