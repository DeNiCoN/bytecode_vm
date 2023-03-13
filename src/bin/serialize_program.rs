use std::fs::File;

pub use bytecode_vm::{deserialize_code, serialize_code, Instruction, Machine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    let mut file = File::create(filename)?;

    let code: Vec<Instruction> = vec![
        Instruction::InByte(),
        Instruction::OutByte(0),
        Instruction::Jmp(0),
    ];

    serialize_code(&code, &mut file)?;

    Ok(())
}
