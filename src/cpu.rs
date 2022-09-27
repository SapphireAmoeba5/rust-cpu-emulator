mod register_id;

use super::address_bus::AddressBus;
use register_id::RegisterId;

use std::{cell::RefCell, fmt::Display, ops::Add, rc::Rc};

pub struct Cpu {
    address_bus: Rc<RefCell<AddressBus>>,

    registers: [u64; 7],

    flags: u64,
}

impl Cpu {
    pub fn new(address_bus: Rc<RefCell<AddressBus>>) -> Cpu {
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

impl Cpu {
    fn fetch_byte(&mut self) -> u8 {
        let mut byte: [u8; 1] = [0; 1];
        self.address_bus
            .borrow_mut()
            .read(&mut byte, self.register(RegisterId::Ip));

        *self.register_mut(RegisterId::Ip) += 1;
        u8::from_le_bytes(byte)
    }

    fn fetch_word(&mut self) -> u16 {
        let mut word_bytes = [0u8; 2];
        self.address_bus
            .borrow_mut()
            .read(&mut word_bytes, self.register(RegisterId::Ip));

        *self.register_mut(RegisterId::Ip) += 2;
        u16::from_le_bytes(word_bytes)
    }

    fn fetch_dword(&mut self) -> u32 {
        let mut dword_bytes = [0u8; 4];
        self.address_bus
            .borrow_mut()
            .read(&mut dword_bytes, self.register(RegisterId::Ip));

        *self.register_mut(RegisterId::Ip) += 4;
        u32::from_le_bytes(dword_bytes)
    }

    fn fetch_qword(&mut self) -> u64 {
        let mut qword_bytes = [0u8; 8];
        self.address_bus
            .borrow_mut()
            .read(&mut qword_bytes, self.register(RegisterId::Ip));

        *self.register_mut(RegisterId::Ip) += 8;
        u64::from_le_bytes(qword_bytes)
    }

    // Wrapper functions to make reading and writing from the address more ergonomic
    fn write(&mut self, src: &[u8], address: u64) {
        self.address_bus.borrow_mut().write(src, address);
    }

    fn read(&mut self, dest: &mut [u8], address: u64) {
        self.address_bus.borrow_mut().read(dest, address);
    }
}

impl Cpu {
    fn register(&self, id: RegisterId) -> u64 {
        self.registers[id as usize - 1]
    }

    fn register_mut(&mut self, id: RegisterId) -> &mut u64 {
        &mut self.registers[id as usize - 1]
    }
}

impl Cpu {
    fn execute_opcode(&mut self, opcode: u8) {
        println!("Executing opcode: {}", opcode);
    }
}
