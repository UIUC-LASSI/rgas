// mod ucg
// Basic UCGv2 encode/decode schema

mod maps;

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
    pub fn from_byte_vec(b: &mut Vec<u8>) -> Option<Self> {
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
    pub fn into_byte_vec(mut self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(into_address_byte(&self.target, &self.subtarget));
        result.push(into_address_byte(&self.source, &self.subsource));
        let lrlen: u8 = (self.len & 0xFF00) as u8; // Get the upper 3 bits of len
        result.push(into_address_byte(&self.op, &lrlen));
        result.push(self.len as u8);
        result.append(&mut self.data);
        result
    }
    pub fn into_asm(&self, print_decimal_data: bool) -> String {
        // Header is always a fixed format, so this one's easy.
        let mut result = format!("{:2X}/{:1X} {:2X}/{:1X} {} {:03} ",
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
                    let twobyte: u16 = (self.data[(2*i + 1) as usize] as u16) + (self.data[(2*i + 2) as usize] as u16) << 8;
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
    pub fn parse_asm_line(line: &String, print_comments: bool) -> Result<Self, String> {
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

fn split_address_byte(b: &u8) -> (u8, u8) {
    let main = (b & 0b11111000) >> 3;
    let sub = b & 0b00000111;
    (main, sub)
}

fn into_address_byte(main: &u8, sub: &u8) -> u8 {
    (main & 0b00011111) << 3 + (sub & 0b00000111)
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
        Some((s, t))
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