mod register_id;

use super::address_bus::AddressBus;
use register_id::RegisterId;

use std::{fmt::Display, ops::Add};

pub struct Cpu<'a> {
    address_bus: &'a AddressBus,

    registers: [u64; 7],

    flags: u64,
}

impl<'a> Cpu<'_> {
    pub fn new(address_bus: &AddressBus) -> Cpu {
        Cpu {
            address_bus,

            registers: [0; 7],

            flags: 0,
        }
    }

    pub fn clock(&mut self) {
        let opcode = self.fetch_byte();

        self.execute_opcode(opcode);
    }
}

impl<'a> Cpu<'_> {
    fn fetch_byte(&mut self) -> u8 {
        let byte: [u8; 1] = [0; 1];
        self.address_bus
            .read(&mut byte, self.register(RegisterId::Ip));

        *self.register_mut(RegisterId::Ip) += 1;
        u8::from_le_bytes(byte)
    }

    fn fetch_word(&mut self) -> u16 {
        let start = self.registers[RegisterId::Ip as usize] as usize;
        let end = (self.registers[RegisterId::Ip as usize] as usize) + 2;

        let word: u16 = u16::from_le_bytes(self.memory[start..end].try_into().unwrap());
        *self.register_mut(RegisterId::Ip) += 2;
        word
    }

    fn fetch_dword(&mut self) -> u32 {
        let start = self.register(RegisterId::Ip) as usize;
        let end = (self.register(RegisterId::Ip) as usize) + 4;

        let dword = u32::from_le_bytes(self.memory[start..end].try_into().unwrap());
        *self.register_mut(RegisterId::Ip) += 4;
        dword
    }

    fn fetch_qword(&mut self) -> u64 {
        let start = self.register(RegisterId::Ip) as usize;
        let end = (self.register(RegisterId::Ip) as usize) + 8;

        let qword = u64::from_le_bytes(self.memory[start..end].try_into().unwrap());
        *self.register_mut(RegisterId::Ip) += 8;
        qword
    }
}

impl Cpu {
    fn register(&self, id: RegisterId) -> u64 {
        self.registers[id as usize]
    }

    fn register_mut(&mut self, id: RegisterId) -> &mut u64 {
        &mut self.registers[id as usize]
    }
}

impl Cpu {
    fn execute_opcode(&mut self, opcode: u8) {
        println!("Executing opcode: {}", opcode);
    }
}
