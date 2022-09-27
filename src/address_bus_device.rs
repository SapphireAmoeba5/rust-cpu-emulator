pub trait AddressBusDevice {
    fn write(&mut self, src: &[u8], address: u64, offset: u64);
    fn read(&mut self, src: &mut [u8], address: u64, offset: u64);
}
