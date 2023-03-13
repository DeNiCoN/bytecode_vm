use std::{
    io::{self, stdin, stdout, BufRead, BufReader, Read, Write},
    vec,
};

//Stack virtual machine
#[derive(Debug, Clone)]
enum Instruction {
    Push(u64),
    Out(u64),
    In(),
    OutStr(String),
}

impl Instruction {
    fn execute<W: Write, R: BufRead>(
        &self,
        machine: &mut Machine,
        input: &mut R,
        output: &mut W,
    ) -> io::Result<usize> {
        match self {
            Instruction::Push(value) => {
                machine.stack.push(*value);
            }
            Instruction::In() => {
                let input_str = input.lines().next().unwrap()?;
                let value: u64 = input_str.parse().unwrap();

                machine.stack.push(value);
            }
            Instruction::Out(pointer) => {
                writeln!(
                    output,
                    "{}",
                    machine.stack[machine.stack.len() - 1 - *pointer as usize]
                )?;
            }
            Instruction::OutStr(value) => {
                output.write(value.as_bytes())?;
            }
        };

        Ok(0)
    }
}

struct Machine {
    code: Vec<Instruction>,
    stack: Vec<u64>,
    pc: u64,
}

impl Machine {
    fn run<W: Write, R: Read>(&mut self, input: &mut R, output: &mut W) -> io::Result<usize> {
        let mut input = BufReader::new(input);
        loop {
            match self.code.get(self.pc as usize) {
                Some(instruction) => instruction.clone().execute(self, &mut input, output)?,
                None => break,
            };
            self.pc += 1;
        }

        Ok(0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vm = Machine {
        code: vec![Instruction::In(), Instruction::Out(0)],
        stack: vec![],
        pc: 0,
    };

    vm.run(&mut stdin(), &mut stdout())?;

    Ok(())
}
