extern crate iset;

mod address_bus;
mod address_bus_device;
mod cpu;
mod memory;
// mod cpu;

use std::cell::RefCell;
use std::rc::Rc;

use address_bus::AddressBus;
pub use address_bus_device::AddressBusDevice;
use cpu::Cpu;
use memory::Memory;

fn main() {
    let mut address_bus: Rc<RefCell<AddressBus>> = Rc::new(RefCell::new(AddressBus::new()));

    let mut memory = Memory::new(1000);

    address_bus.borrow_mut().add_entry(0, 1000, memory).unwrap();

    let arr = [69u8; 1000];
    address_bus.borrow_mut().write(&arr, 0);

    let mut cpu = Cpu::new(Rc::clone(&address_bus));

    loop {
        cpu.clock();
    }
}
