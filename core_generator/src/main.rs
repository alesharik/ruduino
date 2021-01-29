extern crate avr_mcu;

mod gen;

use avr_mcu::*;
use std::fs::{self, File};
use std::{env, io};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

/// The MCU that will be assumed when running 'cargo doc' targeting
/// archicectures that are not AVR.
const DEFAULT_MCU_FOR_NON_AVR_DOCS: &'static str = "atmega328";

const DEFAULT_FREQUENCY_HZ_FOR_NON_AVR_DOCS: u64 = 16_000_000;

fn base_output_path() -> PathBuf {
    match std::env::args().skip(1).next() {
        Some(path) => Path::new(&path).to_owned(),
        None => panic!("please pass a destination path for the generated cores on the command line"),
    }
}

fn cores_path() -> PathBuf {
    base_output_path().join("cores")
}

fn core_module_name(mcu: &Mcu) -> String {
    normalize_device_name(&mcu.device.name)
}

fn normalize_device_name(device_name: &str) -> String {
    device_name.to_lowercase().to_owned()
}

fn main() {
    if !cores_path().exists() {
        fs::create_dir_all(&cores_path()).expect("could not create cores directory");
    }

    let current_mcu = if avr_mcu::current::is_compiling_for_avr() {
        avr_mcu::current::mcu()
            .expect("no target cpu specified")
    } else {
        avr_mcu::microcontroller(DEFAULT_MCU_FOR_NON_AVR_DOCS)
    };
    let current_mcu_name = current_mcu.device.name.clone();

    generate_cores(&[current_mcu]).unwrap();

    println!("cargo:rustc-cfg=avr_mcu_{}", normalize_device_name(&current_mcu_name));
}

fn generate_cores(mcus: &[Mcu]) -> Result<(), io::Error> {
    for mcu in mcus {
        generate_core_module(mcu).expect("failed to generate mcu core");
    }
    generate_cores_mod_rs(mcus)
}

fn generate_core_module(mcu: &Mcu) -> Result<(), io::Error> {
    let path = cores_path().join(format!("{}.rs", core_module_name(mcu)));
    let mut file = File::create(&path)?;
    write_core_module(mcu, &mut file)
}

fn generate_cores_mod_rs(mcus: &[Mcu]) -> Result<(), io::Error> {
    let path = cores_path().join("mod.rs");
    let mut w = File::create(&path)?;

    writeln!(w)?;
    for mcu in mcus {
        let module_name = core_module_name(mcu);
        writeln!(w, "/// The {}.", mcu.device.name)?;
        writeln!(w, "pub mod {};", module_name)?;

        writeln!(w, "#[cfg(avr_mcu_{})]", module_name)?;
        writeln!(w, "pub use self::{} as current;", module_name)?;
    }
    writeln!(w)
}

fn write_core_module(mcu: &Mcu, w: &mut dyn Write) -> Result<(), io::Error> {
    writeln!(w, "//! Core for {}.", mcu.device.name)?;
    writeln!(w)?;
    writeln!(w, "use crate::{{modules, RegisterBits, Register}};")?;
    writeln!(w)?;

    gen::write_registers(mcu, w)?;
    gen::write_pins(mcu, w)?;
    gen::write_spi_modules(mcu, w)?;
    gen::write_usarts(mcu, w)?;
    gen::write_timers(mcu, w)?;

    writeln!(w)
}

