extern crate rgas;
extern crate argparse;
use argparse::{ArgumentParser, StoreTrue, Store};
use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut verbose = false;
    let mut immediate = false;
    let mut hex = false;
    let mut force_interactive = false;
    let mut outfile = String::new();
    let mut infile = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Command grammar assembler for UCGv2.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Be more verbose.");
        ap.refer(&mut immediate)
            .add_option(&["-i", "--immediate"], StoreTrue, "Use UCGv2 immediate mode.");
        ap.refer(&mut hex)
            .add_option(&["-x", "--hex"], StoreTrue, "Output hexadecimal strings instead of binary.");
        ap.refer(&mut outfile)
            .add_option(&["-o", "--outfile"], Store, "Output file to write to.  Defaults to STDOUT.");
        ap.refer(&mut infile)
            .add_option(&["-i", "--infile"], Store, "Input assembly file to read from.  Forces interactive mode if not provided.");
        ap.refer(&mut force_interactive)
            .add_option(&["-I", "--interactive"], StoreTrue, "Force interactive mode.");
        ap.parse_args_or_exit();
    }
    // Enter interactive mode if forced or if no input file was given.
    let interactive_mode = force_interactive || infile.len() == 0;
    if interactive_mode {
        println!("rgas: UCGv2 Command Grammar Assembler.");
        println!("Copyright (c) 2021 Logan Power.  All Rights Reserved.");
    }
    if verbose {
        println!("[:] Increased verbosity.");
        if immediate {
            println!("[:] Using immediate mode");
        }
        if hex {
            println!("[:] Using hexadecimal output.");
        }
        if force_interactive {
            println!("[:] Forcing interactive mode.");
        }
    }
    // We need to have somewhere to write to.
    // If no input file was given, this should be to stdout.
    // Otherwise, it should be to a real file.
    let output_writer: Box<dyn Write>;
    if outfile.len == 0 {
        // No output file was provided, so make the writer write to stdout.
    } else {
        // Output file was provided.  Write to it, truncating it.
        
    }
    // If we are in interactive mode, use rustyline and read lines in from the user.
    // If we aren't, read lines in from the file.

}

