// use super::address_bus_device::

use crate::AddressBusDevice;

pub struct Memory {
    memory: Vec<u8>,
}

impl Memory {
    pub fn new(length: u64) -> Self {
        Self {
            memory: vec![0; length as usize],
        }
    }
}

impl AddressBusDevice for Memory {
    fn write(&mut self, src: &[u8], address: u64, offset: u64) {
        println!(
            "Writing {} bytes to memory at address {:#x}",
            src.len(),
            address
        );
    }

    fn read(&mut self, dest: &mut [u8], address: u64, offset: u64) {
        println!(
            "Reading {} bytes from memory at address {:#x}",
            dest.len(),
            address
        );
    }
}
