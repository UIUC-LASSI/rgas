extern crate rgas;
extern crate argparse;
use argparse::{ArgumentParser, StoreTrue, Store};
use std::io::prelude::*;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::process::exit;
use rgas::UCGMessage;

macro_rules! check {
    ($result:expr, $message:literal) => {
        match($result) {Ok(val) => {val} Err(err) => panic!($message, err)};
    };
}

fn hexlify(vec:&[u8]) -> Vec<u8> {
    let hex = b"0123456789abcdef";
    let mut ret: Vec<u8> = Vec::with_capacity(vec.len()*2);
    for ch in vec {
        ret.push(hex[((ch & 0xf0) >> 4) as usize]);
        ret.push(hex[(ch & 0x0f) as usize]);
    }
    return ret;
}

// for unit testing.
fn test_parse(line: &String, _verbose:bool) -> Result<Vec<u8>, String> {
    return Ok(Vec::from(line.as_bytes()));
}

macro_rules! process_file {
    ($fin:expr, $fout:expr, $verbose:expr, $hex:expr, $interactive:expr, $immediate:expr) => {
    let mut lineno = 1;
    for line in $fin.lines() {
        match(line) {
            Err(err) => {panic!("readline() failed: {}", err)}
            Ok(line) => {
                let res: Result<Box<dyn UCGMessage>, String> = if $immediate {
                    rgas::UCGMessageInternal::parse_asm_line(&line, false)
                } else {
                    rgas::UCGScriptedMessageInternal::parse_asm_line(&line, false)
                };
                match(res) {
                //match(test_parse(&line, $verbose)) {
                    Ok(bytecode) => {
                        let mut bytes = bytecode.into_byte_vec();
                        //let mut bytes = bytecode;
                        
                        // i know i don't have to put parenthesees around my if statements, but old habits die hard
                        if $hex {
                           bytes = hexlify(&bytes);
                        }
                        
                        check!($fout.write(&bytes), "write() call failed: {}");
                        check!($fout.write(b"\r\n"), "write() call failed: {}");
                    }
                    Err(msg) => {
                        // the compiler returns an error with an empty string on comment lines.
                        // i mean, idk how else you would do it, but that was a quirk i was not prepared for
                        if msg.len() != 0 {
                            if $interactive {
                                println!("parse error: {}", msg);
                                // TODO make rustyline put the previous line right back into the linebuffer.
                            } else {
                                panic!("parse error on line {}: {}", lineno, msg);
                            }
                        }
                    }
                }
            }
        };
        lineno+=1;
    }
    };
}

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
            .add_option(&["-m", "--immediate"], StoreTrue, "Use UCGv2 immediate mode.");
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

    
    if outfile.len() == 0 && !hex {
        println!("No output file specified and -x not specified.  Refusing to output binary data to the terminal.");
        exit(1);
    }

    // Enter interactive mode if forced or if no input file was given.
    let interactive_mode = force_interactive || infile.len() == 0;
    if interactive_mode {
        println!("rgas: UCGv2 Command Grammar Assembler.");
        println!("Copyright (c) 2021 Logan Power and Sean Worley.  All Rights Reserved.");
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
    {
        let stdout; // for some reason the Stdout object is required by, but not referenced by, the return value of stdout.lock() so we must keep it alive on our own
                    // why isn't there an implicit reference by keeping the lock object alive?  good question
        let mut fout:Box<dyn io::Write> = if outfile.len() == 0 {
            stdout = io::stdout();
            Box::new(stdout.lock())
        } else {
            // Output file was provided.  Write to it, truncating it.
            match fs::OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .create(true)
                                .open(outfile) {
                Ok(file) => {Box::new(file)}
                Err(msg) => {panic!("Unable to open output file: {}", msg)}
            }
        };

        // For some reason that is utterly beyond me, you can't invoke the lines() method on a trait object, because it has to be sized.
        // So I used a macro to process input.
        // Ah well, I needed a special case to setup rustyline anyway.

        if interactive_mode {
            // If we are in interactive mode, use rustyline and read lines in from the user.
            // TODO actually use rustyline.
            let stdin = io::stdin();
            process_file!(stdin.lock(), fout, verbose, hex, true, immediate);
        } else {
            // If we aren't, read lines in from the file.

            
            // i can only assume that there is a less syntactically lame way to handle errors like this
            // the me of 5 years ago would have taken one look at this and dismissed Rust out of hand
            match fs::File::open(infile) {
                Ok(file) => {
                    process_file!(io::BufReader::new(file), fout, verbose, hex, false, immediate);
                    println!("Processing the file completed successfully.");
                }
                Err(msg) => {
                    panic!("Unable to open input file: {}", msg);
                }
            }
            return;
        }
    }

}

