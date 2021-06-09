// mod ucg
// Basic UCGv2 encode/decode schema
use std::convert::TryInto;

mod maps;

pub trait UCGMessage {
    fn from_byte_vec(b: &mut Vec<u8>) -> Option<Self> where Self: std::marker::Sized;
    fn parse_asm_line(line: &String, print_comments: bool) -> Result<Self, String> where Self: std::marker::Sized;
    fn into_byte_vec(self) -> Vec<u8>;
    fn into_asm(&self, print_decimal_data: bool) -> String;
}

pub struct UCGMessageInternal {
    target: u8,
    subtarget: u8,
    source: u8,
    subsource: u8,
    op: u8,
    len: u16,
    data: Vec<u8>,
}

pub struct UCGScriptedMessageInternal {
    rel: bool,
    ts: u32,
    msg: UCGMessageInternal,
}

impl UCGMessage for UCGScriptedMessageInternal {
    fn parse_asm_line(line: &String, print_comments: bool) -> Result<Self, String> {
        /* The first token in the string should be the timestamp, with the rest of them being
           the message that we should pass to UCGMessageInternal.parse_asm_line().
           To have this be simple to do, split the entire string, take the first token, and
           then put the rest back together and pass it to the actual assembly parser. 
        */
        let rel; 
        let ts: u32;
        let mut my_line = line.clone();
        my_line.make_ascii_uppercase();
        let mut tokens: Vec<&str> = my_line.split_whitespace().collect();
        // Parse the first token.  It needs to either begin with a number or a plus sign and then a number.
        let ts_tok = tokens[0];
        if ts_tok.chars().nth(0).unwrap() == '+' {
            // This is an offset timestamp, which is the type we currently support.
            rel = true;
            ts = match ts_tok.split_at(1).1.parse() {
                Ok(u) => u,
                Err(e) => {
                    return Err(format!("Failed to parse relative time offset \"{}\": {}", ts_tok, e));
                }
            };
        } else if ts_tok.chars().nth(0).unwrap().is_digit(10) {
            // This is an absolute timestamp, which we don't yet support.
            rel = false;
            // TODO: Fill this in.
            return Err(format!("Absolute timestamp not supported in this version of rgas: \"{}\"", ts_tok));
        } else {
            return Err(format!("Not a vaild timestamp: \"{}\".  Did you mean to use immediate mode?", ts_tok));
        };
        // Re-assemble the other strings back into one string
        tokens.remove(0);
        let mut asm_string = String::new();
        for tok in &tokens {
            asm_string.push_str(*tok);
            asm_string.push_str(" ");
        }
        let asm = match UCGMessageInternal::parse_asm_line(&String::from(asm_string.trim_end()), print_comments) {
            Ok(m) => m,
            Err(e) => {
                return Err(e);
            }
        };
        Ok(Self {
            ts,
            rel,
            msg: asm
        })
    }

    fn into_byte_vec(self) -> Vec<u8> {
        // Turn the timestamp into bytes.
        let ts = if self.rel {
            self.ts | 0x80000000
        } else {
            self.ts
        };
        // Put that first in the byte vector
        let mut full_vec: Vec<u8> = ts.to_le_bytes().to_vec();
        // Calculate out the rest of the bytes and put them in too
        full_vec.append(&mut self.msg.into_byte_vec());
        // Return this
        full_vec
    }

    fn from_byte_vec(b: &mut Vec<u8>) -> Option<Self> {
        // Take the first 4 bytes off of the front, since they should be the timestamp.
        let mut msg: Vec<u8> = b.split_off(4);
        // Now b contains the timestamp and msg contains the message
        let mut ts: u32 = u32::from_le_bytes(b.as_slice().try_into().unwrap());
        let rel = ts & 0x80000000 > 0;
        ts &= 0x7FFFFFFF; // Clear the top bit before interpretation. 
        if let Some(m) = UCGMessageInternal::from_byte_vec(&mut msg) {
            Some(Self {
                msg: m,
                rel,
                ts
            })
        } else {
            None
        }
    }

    fn into_asm(&self, print_decimal_data: bool) -> String {
        let mut base_string: String = if self.rel {
            format!{"+{}s ", self.ts}
        } else {
            format!{"ABSOLUTE "}
        };
        // Append the other string onto this one
        let asm_string = self.msg.into_asm(print_decimal_data);
        base_string.push_str(asm_string.as_str());
        base_string
    }
}

impl UCGMessage for UCGMessageInternal{
    fn from_byte_vec(b: &mut Vec<u8>) -> Option<Self> {
        // Byte order in the header is the following: 
        // 1. T/ST
        // 2. S/SS
        // 3. OP/LRLEN
        // 4. LEN
        // ... data
        // Check to make sure we at least have a full header
        if b.len() < 4 {
            return None;
        }
        // Split the given vector into two vectors: the header and the data. 
        let data = b.split_off(4);
        // Header is now in b. 
        let (target, subtarget): (u8, u8) = split_address_byte(&b[0]);
        let (source, subsource): (u8, u8) = split_address_byte(&b[1]);
        let (op, lrlen): (u8, u8) = split_address_byte(&b[2]);
        let mut len: u16 = b[3] as u16;
        // Combine length variables into one
        len += (lrlen as u16) << 8;
        // Check to make sure the op isn't too big or something and return
        if op > maps::MAX_OPCODE {
            None
        } else {
            Some(Self {
                target,
                subtarget,
                source,
                subsource,
                op,
                len,
                data
            })
        }
    }

    fn into_byte_vec(mut self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(into_address_byte(&self.target, &self.subtarget));
        result.push(into_address_byte(&self.source, &self.subsource));
        let lrlen: u8 = (self.len & 0xFF00) as u8; // Get the upper 3 bits of len
        result.push(into_address_byte(&self.op, &lrlen));
        result.push(self.len as u8);
        result.append(&mut self.data);
        result
    }
    fn into_asm(&self, print_decimal_data: bool) -> String {
        // Header is always a fixed format, so this one's easy.
        let mut result = format!("{:02X}/{:1X} {:02X}/{:1X} {} {:03}",
                            self.target,
                            self.subtarget,
                            self.source,
                            self.subsource,
                            self.op_to_text(),
                            self.len);
        // We should format the data nicely to make it easier to read
        // If there is data at all, the first one is likely a register or subroutine number
        // so we should split it. If len is even after that, chunk them into 2-byte hex values,
        // otherwise print them out as single bytes. 
        if self.len == 0 {
            ()
        } else {
            // Do first argument
            result = format!("{} {:02X}", result, self.data[0]);
            // Judge if even or odd
            if (self.len - 1) % 2 == 0 {
                // Even, so print in groups of 2 bytes, little-endian
                for i in 0..(self.data.len()-1)/2 {
                    let twobyte: u16 = (self.data[(2*i + 1) as usize] as u16) + ((self.data[(2*i + 2) as usize] as u16) << 8);
                    if print_decimal_data {
                        result = format!("{} D{}", result, twobyte);
                    } else {
                        result = format!("{} {:04X}", result, twobyte);
                    }
                }
            } else {
                // Odd, so print one at a time.  No fancy grouping. 
                for i in 1..self.data.len() {
                    if print_decimal_data {
                        result = format!("{} D{}", result, self.data[i]);
                    } else {
                        result = format!("{} {:02X}", result, self.data[i]);
                    }
                }
            }
        }
        // Append a newline to result
        format!("{}{}", result, '\n');
        result
    } 
    
    fn parse_asm_line(line: &String, print_comments: bool) -> Result<Self, String> {
        let mut result: Self = Self {
            target: 0,
            subtarget: 0,
            source: 0,
            subsource: 0,
            op: 0,
            len: 0,
            data: Vec::new(),
        };
        let mut my_line = line.clone();
        // Uppercase the whole line to make parsing more uniform
        my_line.make_ascii_uppercase();
        // Get all of the tokens from the line
        let tokens: Vec<&str> = my_line.split_whitespace().collect();
        // Begin parsing the tokens: first token should be either a comment (begins with #) or the target address
        if tokens[0].chars().nth(0) == Some('#') {
            // This is a comment
            if print_comments {
                return Err(my_line);
            } else {
                return Err(String::from(""));
            }
        }
        if tokens[0].len() < 3  || tokens[0].len() > 4 || !tokens[0].contains('/') {
            // This first token isn't valid. 
            return Err(format!("Invalid target address syntax: \"{}\".", tokens[0]));
        } else {
            // This first token is valid.  Parse it. 
            if let Some((target, subtarget)) = address_byte_from_string(tokens[0].to_string()) {
                result.target = target;
                result.subtarget = subtarget;
            } else {
                return Err(format!("Invalid target address syntax: \"{}\".", tokens[0]));
            };
        }
        // Do the same thing for the source
        if tokens[1].len() < 3  || tokens[1].len() > 4 || !tokens[1].contains('/') {
            // This second token isn't valid. 
            return Err(format!("Invalid source address syntax: \"{}\".", tokens[1]));
        } else {
            // This second token is valid.  Parse it. 
            if let Some((source, subsource)) = address_byte_from_string(tokens[1].to_string()) {
                result.source = source;
                result.subsource = subsource;
            } else {
                return Err(format!("Invalid source address syntax: \"{}\".", tokens[1]));
            };
        }
        // Third token should be the opcode mnemonic.  Let the matching thing sort it out. 
        if !result.set_op_from_text(tokens[2]) {
            // If this was false, the opcode wasn't in the list
            return Err(format!("Invalid opcode: \"{}\".", tokens[2]));
        }
        // Fourth should be the length.  This one's not too bad, we just have to make sure it's valid. 
        // Length field should always be written in decimal. 
        if let Ok(len) = u16::from_str_radix(tokens[3], 10) {
            if len < 0x07FF {
                result.len = len;
            } else {
                return Err(format!("Payload length {} too large.", len));
            }
        } else {
            return Err(format!("Invalid length specifier: \"{}\"", tokens[3]));
        }
        // Now we get into the tough stuff: the data.
        // Data tokens can start with any one of these characters: 
        // D: decimal
        // F: float
        // L: double
        // C: character string (until the next space)
        // other: hexadecimal argument
        for i in 4..tokens.len() {
            if let Some(first) = tokens[i].chars().nth(0) {
                match first {
                    'D' => {
                        // read this into an i128, then downsize depending on size
                        let just_num = tokens[i].trim_start_matches('D');
                        let num_big = match i128::from_str_radix(just_num, 10) {
                            Ok(num) => num,
                            Err(_) => {
                                return Err(format!("Malformed decimal data argument: \"{}\"", tokens[i]));
                            }
                        };
                        let num_bytes = determine_integer_size(num_big);
                        let num_big_bytes = num_big.to_le_bytes();
                        result.data.extend_from_slice(&num_big_bytes[0..num_bytes]);
                    },
                    'F' => {
                        // Fortunately we know how big a float is.
                        let just_num = tokens[i].trim_start_matches('F');
                        let num_float = match just_num.parse::<f32>() {
                            Ok(num) => num,
                            Err(_) => {
                                return Err(format!("Malformed floating-point data argument: \"{}\"", tokens[i]));
                            }
                        };
                        result.data.extend_from_slice(&num_float.to_le_bytes());
                    },
                    'L' => {
                        // Fortunately we know how big a double is.
                        let just_num = tokens[i].trim_start_matches('L');
                        let num_float = match just_num.parse::<f64>() {
                            Ok(num) => num,
                            Err(_) => {
                                return Err(format!("Malformed double-precision data argument: \"{}\"", tokens[i]));
                            }
                        };
                        result.data.extend_from_slice(&num_float.to_le_bytes());
                    },
                    'C' => {
                        // We also know how big the character string is (probably)
                        // TODO: Fix this so strings can start with C. 
                        let just_string = tokens[i].trim_start_matches('C');
                        result.data.extend_from_slice(just_string.as_bytes());
                    },
                    _ => {
                        // Interpret this as a hex integer
                        // If it's too long to be a u128, error.  This is 32 hex characters
                        if tokens[i].len() > 32 {
                            return Err(format!("Integer argument too large for rgas: \"{}\"", tokens[i]));
                        }
                        let num_big = match u128::from_str_radix(tokens[i], 16) {
                            Ok(num) => num,
                            Err(_) => {
                                return Err(format!("Malformed hexadecimal data argument: \"{}\"", tokens[i]));
                            }
                        };
                        let num_bytes = determine_integer_size(num_big as i128);
                        let num_big_bytes = num_big.to_le_bytes();
                        result.data.extend_from_slice(&num_big_bytes[0..num_bytes]);
                    }
                };
            } else {
                return Err(format!("Failed to parse data argument {}.", i-3));
            }
        }
        // Perform final checks to see if the statement was more than the length. 
        if result.data.len() > result.len as usize {
            return Err(format!("Data arguments of size {} exceed payload length {}.", result.data.len(), result.len));
        }
        Ok(result)
    }
}

impl UCGMessageInternal {
    pub fn op_to_text(&self) -> String {
        String::from(maps::NUM_TO_OPCODE[self.op as usize])
    }
    pub fn set_op_from_text(&mut self, t: &str) -> bool {
        // This is a very long way to check to see if the key is in the map
        if maps::OPCODE_TO_NUM.keys().find(|&&x| x == t).is_some() {
            self.op = maps::OPCODE_TO_NUM[t];
            true
        } else {
            false
        }
    }
}

fn split_address_byte(b: &u8) -> (u8, u8) {
    let main = (b & 0b11111000) >> 3;
    let sub = b & 0b00000111;
    (main, sub)
}

fn into_address_byte(main: &u8, sub: &u8) -> u8 {
    (main & 0b00011111) << 3 | (sub & 0b00000111)
}

fn address_byte_from_string(s: String) -> Option<(u8, u8)> {
    // use a bit more memory
    let my_s = s.clone();
    let numbers: Vec<&str> = my_s.split('/').collect();
    if numbers.len() != 2 {
        None
    } else {
        //let t: u8 = match numbers[0].parse::<u8>() {
        let t: u8 = match u8::from_str_radix(numbers[0], 16) {
            Ok(tt) => tt,
            Err(_) => {
                return None;
            }
        };
        //let s: u8 = match numbers[1].parse::<u8>() {
        let s: u8 = match u8::from_str_radix(numbers[1], 16) {
            Ok(tt) => tt,
            Err(_) => {
                return None;
            }
        };
        Some((t, s))
    }
}

fn determine_integer_size(a: i128) -> usize {
    if a < 0 {
        // Do signed comparisons
        if a < i8::MAX as i128 && a > i8::MIN as i128 {
            return 1;
        } else if a < i16::MAX as i128 && a > i16::MIN as i128 {
            return 2;
        } else if a < i32::MAX as i128 && a > i32::MIN as i128 {
            return 4;
        } else {
            return 8;
        }
    } else {
        // Do unsigned comparisons
        let b: u128 = a as u128;
        if b < u8::MAX as u128 {
            return 1;
        } else if b < u16::MAX as u128 {
            return 2;
        } else if b < u32::MAX as u128 {
            return 4;
        } else {
            return 8;
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn assembly_from_struct() {
        let mut a = UCGMessageInternal {
            target: 3,
            subtarget: 4,
            source: 0x1f,
            subsource: 7,
            op: 1,
            len: 2,
            data: vec![1, 2],
        };
        let result = a.into_asm(false);
        assert_eq!(result, "03/4 1F/7 RQRY 002 01 02");
        a.data = vec![1, 0x00, 0xFF];
        a.len = 3;
        let result = a.into_asm(false);
        assert_eq!(result, "03/4 1F/7 RQRY 003 01 FF00");
        a.data = vec![1, 0x39, 0x30];
        a.len = 3;
        let result = a.into_asm(true);
        assert_eq!(result, "03/4 1F/7 RQRY 003 01 D12345");
    }

    #[test]
    fn struct_from_assembly() {
        let test_str = "03/4 1F/7 RQRY 001 01";
        match UCGMessageInternal::parse_asm_line(&String::from(test_str), false) {
            Ok(m) => {
                assert_eq!(m.target, 3);
                assert_eq!(m.subtarget, 4);
                assert_eq!(m.source, 0x1f);
                assert_eq!(m.subsource, 7);
                assert_eq!(m.op, 1);
                assert_eq!(m.len, 1);
                assert_eq!(m.data, vec![1]);
            }
            Err(s) => {
                panic!("{}", s);
            }
        }
    }

    #[test]
    fn struct_from_assembly_decimal() {
        let test_str = "03/4 1F/7 RVAL 003 01 D10000";
        match UCGMessageInternal::parse_asm_line(&String::from(test_str), false) {
            Ok(m) => {
                assert_eq!(m.target, 3);
                assert_eq!(m.subtarget, 4);
                assert_eq!(m.source, 0x1f);
                assert_eq!(m.subsource, 7);
                assert_eq!(m.op, 5);
                assert_eq!(m.len, 3);
                assert_eq!(m.data, vec![1, 0x10, 0x27]);
            }
            Err(s) => {
                panic!("{}", s);
            }
        }
    }

    #[test]
    fn struct_from_assembly_float() {
        let test_str = "03/4 1F/7 RVAL 005 01 F202.5";
        match UCGMessageInternal::parse_asm_line(&String::from(test_str), false) {
            Ok(m) => {
                assert_eq!(m.data, vec![1, 0x00, 0x80, 0x4a, 0x43]);
            }
            Err(s) => {
                panic!("{}", s);
            }
        }
    }

    #[test]
    fn struct_from_binary_vector_basic() {
        let mut test_vec = vec![0x1C, 0xFF, 0x08, 0x01, 0x01];
        if let Some(m) = UCGMessageInternal::from_byte_vec(&mut test_vec) {
            assert_eq!(m.target, 3);
            assert_eq!(m.subtarget, 4);
            assert_eq!(m.source, 0x1f);
            assert_eq!(m.subsource, 7);
            assert_eq!(m.op, 1);
            assert_eq!(m.len, 1);
            assert_eq!(m.data, vec![1]);
        } else {
            panic!();
        }
    }
}