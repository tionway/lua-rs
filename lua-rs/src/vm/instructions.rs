use super::opcodes::{ArgKind, OpCode, OpMode, OPCODES};

#[allow(non_upper_case_globals)]
const MAXARG_Bx: u32 = (1 << 18) - 1; // 2^18 - 1 = 262143
#[allow(non_upper_case_globals)]
const MAXARG_sBx: u32 = MAXARG_Bx >> 1; // 262143 / 2 = 131071

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Instruction(u32);

#[allow(non_snake_case)]
impl Instruction {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn opcode(self) -> OpCode {
        OPCODES[(self.0 & 0x3F) as usize]
    }

    pub fn ABC(self) -> (u32, u32, u32) {
        (
            self.0 >> 6 & 0xFF,
            self.0 >> 14 & 0x1FF,
            self.0 >> 23 & 0x1FF,
        )
    }
    pub fn ABx(self) -> (u32, u32) {
        (self.0 >> 6 & 0xFF, self.0 >> 14)
    }

    pub fn AsBx(self) -> (u32, u32) {
        let (x, y) = self.ABx();
        (x, y - MAXARG_sBx)
    }
    pub fn Ax(self) -> u32 {
        self.0 >> 6
    }
    pub fn opname(self) -> &'static str {
        self.opcode().name
    }
    pub fn opmode(self) -> OpMode {
        self.opcode().op_mode
    }
    pub fn b_mode(self) -> ArgKind {
        self.opcode().arg_b_mode
    }
    pub fn c_mode(self) -> ArgKind {
        self.opcode().arg_c_mode
    }
}
