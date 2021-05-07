// mod maps
// Contains static PHF maps for opcode equivalency and such

use phf::{phf_map, Map};

pub static OPCODE_TO_NUM: Map<&'static str, u8> = phf_map! {
    "NOP" => 0,
    "RQRY" => 1,
    "SQST" => 2,
    "SVAL" => 3,
    "RTYP" => 4,
    "RVAL" => 5,
    "RWRT" => 6,
    "RRTC" => 7,
    "SRUN" => 8,
    "STAT" => 9,
    "STOP" => 10,
    "SRET" => 11,
    "MACK" => 12,
    "OPOK" => 13,
    "FAIL" => 14,
    "NSUP" => 15,
    "DERR" => 16,
    "DDIE" => 17,
    "REDY" => 18,
};

pub static MAX_OPCODE: u8 = 18;

pub static NUM_TO_OPCODE: [&'static str; 19] = [
    "NOP",
    "RQRY",
    "SQST",
    "SVAL",
    "RTYP",
    "RVAL",
    "RWRT",
    "RRTC",
    "SRUN",
    "STAT",
    "STOP",
    "SRET",
    "MACK",
    "OPOK",
    "FAIL",
    "NSUP",
    "DERR",
    "DDIE",
    "REDY",
];