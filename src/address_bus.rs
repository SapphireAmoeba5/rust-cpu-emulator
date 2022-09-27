use iset::IntervalMap;

use crate::AddressBusDevice;

use std::cmp::{max, min};

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
        for (entry_location, entry) in self.entries.iter_mut(address..address + src.len() as u64) {
            let start_address = max(entry_location.start.into(), address);
            let end_address = min(entry_location.end, address + src.len() as u64);

            let offset = start_address - entry_location.start;

            entry.write(
                &src[(start_address - address) as usize
                    ..(start_address - address) as usize + (end_address - start_address) as usize],
                address,
                offset,
            );
        }
    }

    pub fn read(&mut self, dest: &mut [u8], address: u64) {
        for (entry_location, entry) in self.entries.iter_mut(address..address + dest.len() as u64) {
            let start_address = max(entry_location.start.into(), address);
            let end_address = min(entry_location.end, address + dest.len() as u64);

            let offset = start_address - entry_location.start;

            entry.read(
                &mut dest[(start_address - address) as usize
                    ..(start_address - address) as usize + (end_address - start_address) as usize],
                address,
                offset,
            );
        }
    }
}
