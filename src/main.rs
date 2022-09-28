extern crate iset;
extern crate lazy_static;

mod address_bus;
mod address_bus_device;
mod cpu;
mod logger;
mod memory;

use std::cell::RefCell;
use std::rc::Rc;

use address_bus::AddressBus;
pub use address_bus_device::AddressBusDevice;
use clap::Parser;
use cpu::Cpu;
use memory::Memory;

#[derive(Parser, Debug)]
struct Args {
    #[clap()]
    input_file: String,
}

fn load_file(file: &str, address_bus: &mut AddressBus) {
    let data: Vec<u8> = std::fs::read(file).expect("");

    let entry_point = u64::from_le_bytes(data[0..8].try_into().unwrap());

    address_bus.write(&entry_point.to_le_bytes(), 0);
    address_bus.write(&data[8..], 8);
}

fn main() {
    let args = Args::parse();

    let mut address_bus: Rc<RefCell<AddressBus>> = Rc::new(RefCell::new(AddressBus::new()));

    let memory_size: u64 = 0xa0000;
    let mut memory = Memory::new(memory_size);

    address_bus
        .borrow_mut()
        .add_entry(0, memory_size, memory)
        .unwrap();

    load_file(&args.input_file, &mut *address_bus.borrow_mut());

    let mut cpu = Cpu::new(Rc::clone(&address_bus));

    loop {
        cpu.clock();

        if !cpu.halted() {
            println!();
        }
    }
}
