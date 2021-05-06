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
    pub fn set_op_from_text(&mut self, t: &str) -> () {
        // This is a very long way to check to see if the key is in the map
        if maps::OPCODE_TO_NUM.keys().find(|&&x| x == t).is_some() {
            self.op = maps::OPCODE_TO_NUM[t];
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
        let lrlen: u8 = (self.len & 0xFF00) as u8;
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
}

fn split_address_byte(b: &u8) -> (u8, u8) {
    let main = (b & 0b11111000) >> 3;
    let sub = b & 0b00000111;
    (main, sub)
}

fn into_address_byte(main: &u8, sub: &u8) -> u8 {
    (main & 0b00011111) << 3 + (sub & 0b00000111)
}