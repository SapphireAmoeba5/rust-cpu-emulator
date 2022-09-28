// use super::address_bus_device::

use crate::debug_println;
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
        debug_println!("Writing to address {:#x}", address);
        self.memory.splice(
            offset as usize..offset as usize + src.len(),
            src.iter().cloned(),
        );
    }

    fn read(&mut self, dest: &mut [u8], address: u64, offset: u64) {
        debug_println!("Reading from address {:#x}", address);

        let len = dest.len();

        dest.into_iter()
            .zip(self.memory[offset as usize..offset as usize + len].iter())
            .for_each(|(x, y)| *x = *y);
    }
}
