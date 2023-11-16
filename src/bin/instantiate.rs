//! Handy utility to test whether the given module deserializes,
//! validates and instantiates successfully.

extern crate wasmi;

use std::env::args;
use std::fs::File;
use wasmi::memory_units::*;
use wasmi::Error;
use wasmi::FuncInstance;
use wasmi::FuncRef;
use wasmi::GlobalDescriptor;
use wasmi::GlobalInstance;
use wasmi::GlobalRef;
use wasmi::ImportsBuilder;
use wasmi::MemoryDescriptor;
use wasmi::MemoryInstance;
use wasmi::MemoryRef;
use wasmi::Module;
use wasmi::ModuleImportResolver;
use wasmi::ModuleInstance;
use wasmi::NopExternals;
use wasmi::RuntimeValue;
use wasmi::Signature;
use wasmi::TableDescriptor;
use wasmi::TableInstance;
use wasmi::TableRef;

fn load_from_file(filename: &str) -> Module {
    use std::io::prelude::*;
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    Module::from_buffer(buf).unwrap()
}

struct ResolveAll;

impl ModuleImportResolver for ResolveAll {
    fn resolve_func(&self, _field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        Ok(FuncInstance::alloc_host(signature.clone(), 0))
    }

    fn resolve_global(
        &self,
        _field_name: &str,
        global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, Error> {
        Ok(GlobalInstance::alloc(
            RuntimeValue::default(global_type.value_type()),
            global_type.is_mutable(),
        ))
    }

    fn resolve_memory(
        &self,
        _field_name: &str,
        memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        Ok(MemoryInstance::alloc(
            Pages(memory_type.initial() as usize),
            memory_type.maximum().map(|m| Pages(m as usize)),
        )
        .unwrap())
    }

    fn resolve_table(
        &self,
        _field_name: &str,
        table_type: &TableDescriptor,
    ) -> Result<TableRef, Error> {
        Ok(TableInstance::alloc(table_type.initial(), table_type.maximum()).unwrap())
    }
}

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 2 {
        println!("Usage: {} <wasm file>", args[0]);
        return;
    }
    let module = load_from_file(&args[1]);
    let _ = ModuleInstance::new(
        &module,
        &ImportsBuilder::default()
            // Well known imports.
            .with_resolver("env", &ResolveAll)
            .with_resolver("global", &ResolveAll)
            .with_resolver("foo", &ResolveAll)
            .with_resolver("global.Math", &ResolveAll)
            .with_resolver("asm2wasm", &ResolveAll)
            .with_resolver("spectest", &ResolveAll),
        None,
    )
    .expect("Failed to instantiate module")
    .run_start(&mut NopExternals)
    .expect("Failed to run start function in module");
}
