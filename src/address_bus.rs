use iset::IntervalMap;

use crate::AddressBusDevice;

pub struct AddressBus {
    entries: IntervalMap<u64, Box<dyn AddressBusDevice>>,
}

impl AddressBus {
    pub fn new() -> Self {
        Self {
            entries: IntervalMap::new(),
        }
    }

    pub fn add_entry(
        &mut self,
        address: u64,
        length: u64,
        callback: impl AddressBusDevice + 'static,
    ) -> Result<(), ()> {
        if !self.entries.has_overlap(address..address + length) {
            self.entries
                .insert(address..address + length, Box::new(callback));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write(&mut self, src: &[u8], address: u64) {
        for entry in self.entries.values_mut(address..address + src.len() as u64) {
            entry.write(src, address, 0);
        }
    }

    pub fn read(&mut self, dest: &mut [u8], address: u64) {
        for entry in self
            .entries
            .values_mut(address..address + dest.len() as u64)
        {
            entry.read(dest, address, 0);
        }
    }
}
