use crate::PortBusDevice;

const INIT: Option<Box<dyn PortBusDevice>> = None;

pub struct PortBus {
    entries: [Option<Box<dyn PortBusDevice>>; 0xffff],
}

impl PortBus {
    pub fn new() -> Self {
        Self {
            entries: [INIT; 0xffff],
        }
    }

    pub fn add_device(
        &mut self,
        port: u16,
        callback: impl PortBusDevice + 'static,
    ) -> Result<(), ()> {
        if self.entries[port as usize].is_none() {
            self.entries[port as usize] = Some(Box::new(callback));

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove_device(&mut self, port: u16) {
        self.entries[port as usize] = None;
    }

    pub fn write(&mut self, port: u16, value: u64) {
        if let Some(entry) = &mut self.entries[port as usize] {
            entry.write(value);
        }
    }

    pub fn read(&mut self, port: u16) -> u64 {
        if let Some(entry) = &mut self.entries[port as usize] {
            entry.read()
        } else {
            0xffffffffffffffff
        }
    }
}
