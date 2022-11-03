pub trait PortBusDevice {
    fn write(&mut self, value: u64);
    fn read(&mut self) -> u64;
}
