mod instruction_lookup;
mod instructions;
mod register_id;
mod reserved_idt_entries;
mod size;

use crate::debug_println;

use self::instruction_lookup::{LookupEntry, LOOKUP_TABLE};
use super::address_bus::AddressBus;
use crate::port_bus::PortBus;
use instructions::InstructionResult;
use register_id::RegisterId;
use reserved_idt_entries::*;
use size::Size;

use std::{cell::RefCell, fmt::Display, ops::Add, rc::Rc, time::Duration};

#[derive(Debug, Clone, Copy)]
enum CpuFlag {
    Negative = 0,
    Overflow = 1,
    Zero = 2,
    Carry = 3,
    InterruptEnable = 4,
}

pub struct Cpu {
    address_bus: Rc<RefCell<AddressBus>>,
    port_bus: Rc<RefCell<PortBus>>,

    registers: [u64; 7],
    idt: u64,

    flags: u64,
    halted: bool,
}

impl Cpu {
    pub fn new(address_bus: Rc<RefCell<AddressBus>>, port_bus: Rc<RefCell<PortBus>>) -> Self {
        let mut cpu = Self {
            address_bus,
            port_bus,

            registers: [0; 7],
            idt: 0,

            flags: 0,
            halted: false,
        };

        cpu.reset();

        cpu
    }

    pub fn clock(&mut self) {
        if !self.halted {
            let opcode = self.fetch_byte();
            self.execute_opcode(opcode);
        }
    }

    pub fn reset(&mut self) {
        println!("Resetting CPU!");
        #[cfg(debug_assertions)]
        std::thread::sleep(Duration::from_secs_f32(1.0));

        // The first 8 bytes in memory is the address the CPU will start executing code
        let mut execution_start = [0u8; 8];
        self.read(&mut execution_start, 0);

        let execution_start = u64::from_le_bytes(execution_start);

        self.set_flag(CpuFlag::InterruptEnable, true);

        self.register_assign(RegisterId::Ip, execution_start);
        self.register_assign(RegisterId::Sp, 0xffff);
    }

    pub fn halted(&self) -> bool {
        self.halted
    }
}

impl Cpu {
    fn fetch_byte(&mut self) -> u8 {
        let mut byte: [u8; 1] = [0; 1];
        self.address_bus
            .borrow_mut()
            .read(&mut byte, self.register(RegisterId::Ip));

        self.register_add_assign(RegisterId::Ip, 1);
        u8::from_le_bytes(byte)
    }

    fn fetch_word(&mut self) -> u16 {
        let mut word_bytes = [0u8; 2];
        self.address_bus
            .borrow_mut()
            .read(&mut word_bytes, self.register(RegisterId::Ip));

        self.register_add_assign(RegisterId::Ip, 2);
        u16::from_le_bytes(word_bytes)
    }

    fn fetch_dword(&mut self) -> u32 {
        let mut dword_bytes = [0u8; 4];
        self.address_bus
            .borrow_mut()
            .read(&mut dword_bytes, self.register(RegisterId::Ip));

        self.register_add_assign(RegisterId::Ip, 4);
        u32::from_le_bytes(dword_bytes)
    }

    fn fetch_qword(&mut self) -> u64 {
        let mut qword_bytes = [0u8; 8];
        self.address_bus
            .borrow_mut()
            .read(&mut qword_bytes, self.register(RegisterId::Ip));

        self.register_add_assign(RegisterId::Ip, 8);
        u64::from_le_bytes(qword_bytes)
    }

    fn fetch_sized(&mut self, size: Size) -> u64 {
        match size {
            Size::One => self.fetch_byte().into(),
            Size::Two => self.fetch_word().into(),
            Size::Four => self.fetch_dword().into(),
            Size::Eight => self.fetch_qword(),
        }
    }

    fn push_byte(&mut self, value: u8) {
        self.register_sub_assign(RegisterId::Sp, 1);
        let address = self.register(RegisterId::Sp);
        self.write(&value.to_le_bytes(), address);
    }

    fn push_word(&mut self, value: u16) {
        self.register_sub_assign(RegisterId::Sp, 2);
        let address = self.register(RegisterId::Sp);
        self.write(&value.to_le_bytes(), address);
    }

    fn push_dword(&mut self, value: u32) {
        self.register_sub_assign(RegisterId::Sp, 4);
        let address = self.register(RegisterId::Sp);
        self.write(&value.to_le_bytes(), address);
    }

    fn push_qword(&mut self, value: u64) {
        self.register_sub_assign(RegisterId::Sp, 8);
        let address = self.register(RegisterId::Sp);
        self.write(&value.to_le_bytes(), address);
    }

    fn pop_byte(&mut self) -> u8 {
        let address = self.register(RegisterId::Sp);

        let mut value = [0u8; 1];
        self.read(&mut value, address);

        self.register_add_assign(RegisterId::Sp, 1);

        u8::from_le_bytes(value)
    }

    fn pop_word(&mut self) -> u16 {
        let address = self.register(RegisterId::Sp);

        let mut value = [0u8; 2];
        self.read(&mut value, address);

        self.register_add_assign(RegisterId::Sp, 2);

        u16::from_le_bytes(value)
    }

    fn pop_dword(&mut self) -> u32 {
        let address = self.register(RegisterId::Sp);

        let mut value = [0u8; 4];
        self.read(&mut value, address);

        self.register_add_assign(RegisterId::Sp, 4);

        u32::from_le_bytes(value)
    }

    fn pop_qword(&mut self) -> u64 {
        let address = self.register(RegisterId::Sp);

        let mut value = [0u8; 8];
        self.read(&mut value, address);

        self.register_add_assign(RegisterId::Sp, 8);

        u64::from_le_bytes(value)
    }

    fn push_flags(&mut self) {
        self.push_qword(self.flags);
    }

    fn pop_flags(&mut self) {
        self.flags = self.pop_qword();
    }

    // Wrapper functions to make reading and writing from the address more ergonomic
    fn write(&mut self, src: &[u8], address: u64) {
        self.address_bus.borrow_mut().write(src, address);
    }

    fn read(&mut self, dest: &mut [u8], address: u64) {
        self.address_bus.borrow_mut().read(dest, address);
    }

    fn port_bus_write(&mut self, port: u16, value: u64) {
        self.port_bus.borrow_mut().write(port, value)
    }

    fn port_bus_read(&mut self, port: u16) -> u64 {
        self.port_bus.borrow_mut().read(port)
    }
}

impl Cpu {
    fn is_register(&self, idx: usize) -> bool {
        idx < self.registers.len()
    }

    fn register(&self, id: RegisterId) -> u64 {
        self.registers[id as usize - 1]
    }

    fn register_mut(&mut self, id: RegisterId) -> &mut u64 {
        &mut self.registers[id as usize - 1]
    }

    fn register_assign(&mut self, id: RegisterId, value: u64) {
        *self.register_mut(id) = value;
    }

    fn register_assign_sized(&mut self, id: RegisterId, value: u64, size: Size) {
        let idx: usize = id.to_index();

        match size {
            Size::One => {
                self.registers[idx] &= 0xffffffffffffff00;
                self.registers[idx] |= value & 0x00000000000000ff;
            }

            Size::Two => {
                self.registers[idx] &= 0xffffffffffff0000;
                self.registers[idx] |= value & 0x000000000000ffff;
            }

            Size::Four => {
                self.registers[idx] &= 0xffffffff00000000;
                self.registers[idx] |= value & 0x00000000ffffffff;
            }

            Size::Eight => {
                self.registers[idx] = value;
            }
        }
    }

    fn register_add_assign(&mut self, id: RegisterId, value: u64) {
        let register_value = self.register(id);
        *self.register_mut(id) = register_value.wrapping_add(value);
    }

    fn register_sub_assign(&mut self, id: RegisterId, value: u64) {
        let register_value = self.register(id);
        *self.register_mut(id) = register_value.wrapping_sub(value);
    }

    fn get_flag(&self, flag: CpuFlag) -> bool {
        (self.flags >> flag as u64 & 1) == 1
    }

    fn set_flag(&mut self, flag: CpuFlag, value: bool) {
        let flag = flag as u64;

        self.flags &= !(1 << flag);
        self.flags |= (value as u64) << flag;
    }
}

impl Cpu {
    fn execute_opcode(&mut self, opcode: u8) {
        if let Some(callback) = LOOKUP_TABLE[opcode as usize].callback {
            debug_println!(
                "Executing instruction '{}' {:#x}",
                LOOKUP_TABLE[opcode as usize].instruction,
                opcode
            );
            if let Err(idt_entry) = callback(self) {
                self.non_maskable_interrupt_request(idt_entry);
            }
        } else {
            debug_println!(
                "Invalid instruction {:#x} at {:#x}",
                opcode,
                self.register(RegisterId::Ip) - 1
            );
            self.non_maskable_interrupt_request(INVALID_INSTRUCTION);
        }
    }
}

impl Cpu {
    fn interrupt_request(&mut self, idt_entry: u8) {
        debug_println!("Interrupt request recieved for entry {}", idt_entry);

        if self.get_flag(CpuFlag::InterruptEnable) == true {
            self.interrupt_handler(idt_entry);
        } else {
            debug_println!("Interrupts disabled");
        }
    }

    fn non_maskable_interrupt_request(&mut self, idt_entry: u8) {
        debug_println!(
            "Non maskable interrupt request recieved for entry {}",
            idt_entry
        );
        self.interrupt_handler(idt_entry);
    }

    fn interrupt_handler(&mut self, idt_entry: u8) {
        let sizeof_idt_entry: u64 = 8;

        if self.idt != 0 {
            let idt_entry_address = self.idt + (idt_entry as u64 * sizeof_idt_entry);

            let mut handler_address = [0u8; 8];
            self.read(&mut handler_address, idt_entry_address);

            let handler_address = u64::from_le_bytes(handler_address);

            if handler_address != 0 {
                self.push_flags();
                self.push_qword(self.register(RegisterId::Ip));

                self.register_assign(RegisterId::Ip, handler_address);
            } else {
                debug_println!("Invalid entry at IDT entry {}", idt_entry);
                self.reset();
            }
        } else {
            debug_println!("IDT not defined");
            self.reset();
        }
    }
}
