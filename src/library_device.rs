use std::ffi::OsStr;

use crate::{AddressBusDevice, PortBusDevice};
use libc::c_void;
use libloading::{Error, Library, Symbol};

pub struct LibraryAddressDevice {
    library: Library,

    write_function: unsafe extern "C" fn(
        data: *const u8,
        length: u64,
        offset_from_base: u64,
        address: u64,
        private_data: *mut c_void,
    ),

    read_function: unsafe extern "C" fn(
        data: *mut u8,
        length: u64,
        offset_from_base: u64,
        address: u64,
        private_data: *mut c_void,
    ),

    shutdown_function: unsafe extern "C" fn(private_data: *mut c_void),

    private_data: *mut c_void,
}

impl LibraryAddressDevice {
    pub fn new(library_path: &str, identifier_prefix: &str, length: u64) -> Result<Self, ()> {
        let library = match unsafe { Library::new(library_path) } {
            Ok(lib) => lib,
            Err(_) => {
                println!(
                    "Error: Failed to load the address bus library at \"{}\"",
                    library_path
                );
                return Err(());
            }
        };

        let initialize_function: unsafe extern "C" fn(u64) -> *mut c_void = match unsafe {
            library.get::<unsafe extern "C" fn(u64) -> *mut c_void>(
                format!("{}_address_bus_init", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Address bus initialize function not found for library module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let write_function = match unsafe {
            library.get::<unsafe extern "C" fn(*const u8, u64, u64, u64, *mut c_void)>(
                format!("{}_address_bus_write", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Address bus write function not found for library module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let read_function = match unsafe {
            library.get::<unsafe extern "C" fn(*mut u8, u64, u64, u64, *mut c_void)>(
                format!("{}_address_bus_read", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Address bus read function not found for library module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let shutdown_function = match unsafe {
            library.get::<unsafe extern "C" fn(*mut c_void)>(
                format!("{}_address_bus_shutdown", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Address bus shutdown function not found for library module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let private_data = unsafe { initialize_function(length) };

        if private_data as u64 == 0 {
            println!(
                "Error: Address bus initialize function ecountered an error on module \"{}\"",
                identifier_prefix
            );
            return Err(());
        }

        // if unsafe { initialize_function(length, id) } != 0 {
        //     return Err(());
        // }

        Ok(Self {
            library,
            write_function,
            read_function,
            shutdown_function,

            private_data,
        })
    }
}

impl AddressBusDevice for LibraryAddressDevice {
    fn write(&mut self, src: &[u8], address: u64, offset: u64) {
        unsafe {
            (self.write_function)(
                src.as_ptr(),
                src.len() as u64,
                offset,
                address,
                self.private_data,
            )
        };
    }

    fn read(&mut self, dest: &mut [u8], address: u64, offset: u64) {
        unsafe {
            (self.read_function)(
                dest.as_mut_ptr(),
                dest.len() as u64,
                offset,
                address,
                self.private_data,
            )
        };
    }
}

impl Drop for LibraryAddressDevice {
    fn drop(&mut self) {
        unsafe { (self.shutdown_function)(self.private_data) };
    }
}

pub struct LibraryPortDevice {
    library: Library,
    port: u16,
    private_data: *mut c_void,

    write_function: unsafe extern "C" fn(value: u64, port: u16, private_data: *mut c_void),
    read_function: unsafe extern "C" fn(port: u16, private_data: *mut c_void) -> u64,

    shutdown_function: unsafe extern "C" fn(port: u16, private_data: *mut c_void),
}

impl LibraryPortDevice {
    pub fn new<A: AsRef<OsStr> + std::fmt::Display, B: AsRef<str> + std::fmt::Display>(
        library_path: A,
        identifier_prefix: B,
        port: u16,
    ) -> Result<Self, ()> {
        let library = match unsafe { Library::new(&library_path) } {
            Ok(lib) => lib,
            Err(_) => {
                println!(
                    "Error: Failed to load the port bus library at \"{}\"",
                    library_path
                );
                return Err(());
            }
        };

        let initialize_function = match unsafe {
            library.get::<unsafe extern "C" fn(u16) -> *mut c_void>(
                format!("{}_port_bus_init", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Port bus intitialize function not found for module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let write_function = match unsafe {
            library.get::<unsafe extern "C" fn(u64, u16, *mut c_void)>(
                format!("{}_port_bus_write", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Port bus write function not found for module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let read_function = match unsafe {
            library.get::<unsafe extern "C" fn(u16, *mut c_void) -> u64>(
                format!("{}_port_bus_read", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Port bus read function not found for module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let shutdown_function = match unsafe {
            library.get::<unsafe extern "C" fn(u16, *mut c_void)>(
                format!("{}_port_bus_shutdown", identifier_prefix).as_bytes(),
            )
        } {
            Ok(f) => *f,
            Err(_) => {
                println!(
                    "Error: Port bus shutdown function not found for module \"{}\"",
                    identifier_prefix
                );
                return Err(());
            }
        };

        let private_data = unsafe { initialize_function(port) };

        if private_data as u64 == 0 {
            println!(
                "Error: Port bus initialize function ecountered an error on module \"{}\"",
                identifier_prefix
            );

            return Err(());
        }

        Ok(Self {
            library,
            private_data,
            port,

            write_function,
            read_function,
            shutdown_function,
        })
    }
}

impl PortBusDevice for LibraryPortDevice {
    fn write(&mut self, value: u64) {
        unsafe { (self.write_function)(value, self.port, self.private_data) };
    }

    fn read(&mut self) -> u64 {
        unsafe { (self.read_function)(self.port, self.private_data) }
    }
}

impl Drop for LibraryPortDevice {
    fn drop(&mut self) {
        unsafe { (self.shutdown_function)(self.port, self.private_data) };
    }
}
