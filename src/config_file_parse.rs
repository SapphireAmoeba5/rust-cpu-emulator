mod try_parse;

use crate::{
    debug_println, library_device::LibraryPortDevice, AddressBus, LibraryAddressDevice, PortBus,
    PortBusDevice,
};
use path_absolutize::*;
use std::{
    cell::RefCell,
    env,
    ffi::OsStr,
    fs,
    ops::Add,
    path::{self, Path, PathBuf},
    rc::Rc,
};
use try_parse::try_parse_number;

#[derive(Debug, Clone, Copy)]
enum DeviceType {
    AddressBus { start_address: u64, length: u64 },
    PortBus(u16),
}

impl DeviceType {
    pub fn new_address_device(start_address: u64, length: u64) -> Self {
        Self::AddressBus {
            start_address,
            length,
        }
    }

    pub fn new_port_device(port: u16) -> Self {
        Self::PortBus(port)
    }
}

/// The library type means what kind of medium that the code containing the callbacks for the specfic devices will be
#[derive(Debug, Clone, Copy)]
enum LibraryType {
    SharedLibrary,
    Python,
}

impl TryFrom<&str> for LibraryType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "library" => Ok(Self::SharedLibrary),
            "python" => Ok(Self::Python),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct ConfigEntry {
    pub library_path: String,
    pub module_name: String,
    pub device_type: DeviceType,
    pub library_type: LibraryType,
    pub line_number: usize,
}

impl ConfigEntry {
    pub fn new(
        library_path: String,
        module_name: String,
        line_number: usize,
        device_type: DeviceType,
        library_type: LibraryType,
    ) -> Self {
        Self {
            library_path,
            module_name,
            line_number,
            device_type,
            library_type,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    entries: Vec<ConfigEntry>,
}

impl Config {
    pub fn new<P>(config_path: P) -> Result<Self, ()>
    where
        P: AsRef<Path> + std::fmt::Display,
    {
        let cwd = env::current_dir().expect("Error getting working directory");

        // let path = Path::new(&config_path);

        let config = match std::fs::read_to_string(config_path.as_ref()) {
            Ok(s) => s,
            Err(e) => {
                println!(
                    "Error reading config file at \"{}\" Error: {}",
                    config_path, e
                );
                return Err(());
            }
        };

        env::set_current_dir(config_path.as_ref().parent().unwrap())
            .expect("Error changing working directory");

        let this = Self::parse_config(config)?;

        env::set_current_dir(cwd).unwrap();

        Ok(this)
    }

    pub fn apply_config(
        &self,
        address_bus: &mut AddressBus,
        port_bus: &mut PortBus,
    ) -> Result<(), ()> {
        for entry in &self.entries {
            match entry.device_type {
                DeviceType::AddressBus {
                    start_address,
                    length,
                } => Self::apply_address_device(
                    &entry.library_path,
                    &entry.module_name,
                    entry.line_number,
                    entry.library_type,
                    start_address,
                    length,
                    address_bus,
                )?,

                DeviceType::PortBus(port) => Self::apply_port_device(
                    &entry.library_path,
                    &entry.module_name,
                    entry.line_number,
                    entry.library_type,
                    port,
                    port_bus,
                )?,
            }
        }

        Ok(())
    }

    fn apply_address_device(
        library_path: &str,
        module_name: &str,
        line_number: usize,
        library_type: LibraryType,
        start_address: u64,
        length: u64,
        address_bus: &mut AddressBus,
    ) -> Result<(), ()> {
        match library_type {
            LibraryType::SharedLibrary => Self::apply_address_device_shared_library(
                library_path,
                module_name,
                line_number,
                start_address,
                length,
                address_bus,
            ),

            LibraryType::Python => todo!(),
        }
    }

    fn apply_address_device_shared_library(
        library_path: &str,
        module_name: &str,
        line_number: usize,
        start_address: u64,
        length: u64,
        address_bus: &mut AddressBus,
    ) -> Result<(), ()> {
        let library = LibraryAddressDevice::new(library_path, module_name, length)?;

        match address_bus.add_entry(start_address, length, library) {
            Ok(_) => Ok(()),
            Err(_) => {
                println!(
                    "Error adding line {} to the address bus. Check for address overlaps",
                    line_number
                );
                Err(())
            }
        }
    }

    fn apply_port_device(
        library_path: &str,
        module_name: &str,
        line_number: usize,
        library_type: LibraryType,
        port: u16,
        port_bus: &mut PortBus,
    ) -> Result<(), ()> {
        match library_type {
            LibraryType::SharedLibrary => Self::apply_port_device_shared_library(
                library_path,
                module_name,
                line_number,
                port,
                port_bus,
            ),

            LibraryType::Python => todo!(),
        }
    }

    fn apply_port_device_shared_library(
        library_path: &str,
        module_name: &str,
        line_number: usize,
        port: u16,
        port_bus: &mut PortBus,
    ) -> Result<(), ()> {
        let library = LibraryPortDevice::new(library_path, module_name, port)?;

        match port_bus.add_device(port, library) {
            Ok(_) => Ok(()),
            Err(_) => {
                println!(
                    "Error adding line {} to the port bus. Check for duplicate port numbers",
                    line_number
                );
                Err(())
            }
        }
    }
}

impl Config {
    fn parse_config<P>(config: P) -> Result<Self, ()>
    where
        P: AsRef<str> + std::fmt::Display,
    {
        let mut entries: Vec<ConfigEntry> = Vec::new();

        for (line_idx, line) in config.as_ref().lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let line_number = line_idx + 1;

            let entry = Self::parse_config_line(line, line_number)?;
            entries.push(entry);
        }

        Ok(Self { entries })
    }

    fn parse_config_line<P>(line: P, line_number: usize) -> Result<ConfigEntry, ()>
    where
        P: AsRef<str> + std::fmt::Display,
    {
        let split = line.as_ref().split_ascii_whitespace().collect::<Vec<_>>();

        match split[0] {
            "address-device" => {
                if split.len() == 6 {
                    Self::parse_address_device_line(
                        split[1],
                        split[2],
                        line_number,
                        split[3],
                        split[4],
                        split[5],
                    )
                } else {
                    println!("Invalid address device entry on line {}", line_number);
                    Err(())
                }
            }

            "port-device" => {
                if split.len() == 5 {
                    Self::parse_port_bus_line(split[1], split[2], line_number, split[3], split[4])
                } else {
                    println!("Invalid port device entry on line {}", line_number);
                    Err(())
                }
            }

            _ => {
                println!("Invalid device type on line {}", line_number);
                return Err(());
            }
        }
    }

    fn parse_address_device_line(
        library_type: &str,
        library_path: &str,
        line_number: usize,
        start_address: &str,
        length: &str,
        module_name: &str,
    ) -> Result<ConfigEntry, ()> {
        let library_type = match LibraryType::try_from(library_type) {
            Ok(lib) => lib,
            Err(_) => {
                println!("Invalid library type on line {}", line_number);
                return Err(());
            }
        };

        let start_address = match try_parse_number(start_address) {
            Ok(addr) => addr,
            Err(e) => {
                println!(
                    "Error: {e} on line \"{}\" when parsing start address",
                    line_number
                );
                return Err(());
            }
        };

        let length = match try_parse_number(length) {
            Ok(len) => len,
            Err(e) => {
                println!("Error: {e} on line \"{}\" when parsing length", line_number);
                return Err(());
            }
        };

        let path = Path::new(library_path);

        Ok(ConfigEntry::new(
            path.absolutize().unwrap().to_string_lossy().to_string(),
            module_name.to_string(),
            line_number,
            DeviceType::new_address_device(start_address, length),
            library_type,
        ))
    }

    fn parse_port_bus_line(
        library_type: &str,
        library_path: &str,
        line_number: usize,
        port: &str,
        module_name: &str,
    ) -> Result<ConfigEntry, ()> {
        let library_type = match LibraryType::try_from(library_type) {
            Ok(lib) => lib,
            Err(_) => {
                println!("Invalid library type on line {}", line_number);
                return Err(());
            }
        };

        let port: u16 = match try_parse_number(port) {
            Ok(addr) => match addr.try_into() {
                Ok(addr) => addr,
                Err(_) => {
                    println!(
                        "Port too large. Port should be within the range 0-{}",
                        u16::MAX
                    );
                    return Err(());
                }
            },
            Err(e) => {
                println!("Error: {e} on line \"{}\" when parsing port", line_number);
                return Err(());
            }
        };

        let path = Path::new(library_path);

        Ok(ConfigEntry::new(
            path.absolutize().unwrap().to_string_lossy().to_string(),
            module_name.to_string(),
            line_number,
            DeviceType::new_port_device(port),
            library_type,
        ))
    }
}
