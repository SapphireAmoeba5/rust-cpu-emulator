extern crate iset;
extern crate lazy_static;

mod address_bus;
mod address_bus_device;
mod config_file_parse;
mod cpu;
mod library_device;
mod logger;
mod memory;
mod port_bus;
mod port_bus_device;

use std::cell::RefCell;
use std::rc::Rc;

use address_bus::AddressBus;
use address_bus_device::AddressBusDevice;
use clap::Parser;
use config_file_parse::Config;
use cpu::Cpu;
use library_device::LibraryAddressDevice;
use memory::Memory;
use port_bus::PortBus;
use port_bus_device::PortBusDevice;

#[derive(Parser, Debug)]
struct Args {
    #[clap()]
    input_file: String,

    #[clap(long = "--config")]
    config_file: Option<String>,
}

fn load_file(file: &str, address_bus: &mut AddressBus) -> Result<(), ()> {
    let data: Vec<u8> = match std::fs::read(file) {
        Ok(d) => d,
        Err(_) => return Err(()),
    };

    // The first 8 bytes of the file contains the entry point,
    // but the cpu also reads the first 8 bytes of memory to get the entry point
    // so we can convienently just write the file as is to memory from address 0
    address_bus.write(&data, 0);

    // let entry_point = u64::from_le_bytes(data[0..8].try_into().unwrap());

    // address_bus.write(&entry_point.to_le_bytes(), 0);
    // address_bus.write(&data[8..], 8);

    Ok(())
}

fn main() -> Result<(), ()> {
    let args = Args::parse();

    let mut address_bus: Rc<RefCell<AddressBus>> = Rc::new(RefCell::new(AddressBus::new()));
    let mut port_bus: Rc<RefCell<PortBus>> = Rc::new(RefCell::new(PortBus::new()));

    if let Some(config_file) = args.config_file {
        let config = Config::new(&config_file)?;
        config.apply_config(&mut address_bus.borrow_mut(), &mut port_bus.borrow_mut())?;
    } else {
        println!("No config file found. Using default configuration");

        let memory_size: u64 = 0xa0000;
        let mut memory = Memory::new(memory_size);

        address_bus
            .borrow_mut()
            .add_entry(0, memory_size, memory)
            .unwrap();
    }

    load_file(&args.input_file, &mut *address_bus.borrow_mut())?;

    let mut cpu = Cpu::new(Rc::clone(&address_bus), Rc::clone(&port_bus));

    loop {
        cpu.clock();

        if !cpu.halted() {
            println!();
        }
    }
}
