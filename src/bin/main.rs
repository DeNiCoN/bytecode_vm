use std::{
    fs::File,
    io::{stdin, stdout},
};

use bytecode_vm::{deserialize_code, Machine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    let mut file = File::open(filename)?;

    let mut vm = Machine {
        code: deserialize_code(&mut file)?,
        stack: vec![],
        pc: 0,
    };

    vm.run(&mut stdin(), &mut stdout())?;

    Ok(())
}
