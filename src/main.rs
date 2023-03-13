use std::{
    io::{self, stdin, stdout, Read, Write},
    vec,
};

//Stack virtual machine
#[derive(Debug, Clone)]
enum Instruction {
    OutStr(String),
}

impl Instruction {
    fn execute<W: Write, R: Read>(
        &self,
        machine: &mut Machine,
        input: &mut R,
        output: &mut W,
    ) -> io::Result<usize> {
        match self {
            Instruction::OutStr(value) => output.write(value.as_bytes()),
        }
    }
}

struct Machine {
    code: Vec<Instruction>,
    stack: Vec<u64>,
    pc: u64,
}

impl Machine {
    fn run<W: Write, R: Read>(&mut self, input: &mut R, output: &mut W) -> io::Result<usize> {
        loop {
            match self.code.get(self.pc as usize) {
                Some(instruction) => instruction.clone().execute(self, input, output)?,
                None => break,
            };
            self.pc += 1;
        }

        Ok(0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vm = Machine {
        code: vec![Instruction::OutStr("Hello, World!".to_owned())],
        stack: vec![],
        pc: 0,
    };

    vm.run(&mut stdin(), &mut stdout())?;

    Ok(())
}
