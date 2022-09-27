extern crate iset;

mod address_bus;
mod address_bus_device;
mod memory;
// mod cpu;

use address_bus::AddressBus;
pub use address_bus_device::AddressBusDevice;
use memory::Memory;
// use cpu::Cpu;

fn main() {
    let mut address_bus = AddressBus::new();

    let mut memory0 = Memory::new(10);
    let mut memory1 = Memory::new(10);

    address_bus.add_entry(0, 10, memory0).unwrap();
    address_bus.add_entry(10, 10, memory1).unwrap();

    let data = [0u8; 5];
    address_bus.write(&data, 6);
}
