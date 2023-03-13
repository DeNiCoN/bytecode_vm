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
    Copy(u64),
    Add(u64, u64),
    Gt(u64, u64, u64),
    Eq(u64, u64, u64),
    Jmp(u64),
    Dec(u64),
    Inc(u64),
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
            Instruction::Add(l, r) => {
                let l = machine.stack.len() - 1 - *l as usize;
                let r = machine.stack.len() - 1 - *r as usize;
                let l_value = machine.stack[l];
                let r_value = machine.stack[r];
                let correct = (r > l) as usize;
                machine.stack.remove(l);
                machine.stack.remove(r - correct);

                machine.stack.push(l_value + r_value);
            }
            Instruction::Copy(pointer) => {
                let value = machine.stack[machine.stack.len() - 1 - *pointer as usize];
                machine.stack.push(value);
            }
            Instruction::Gt(l, r, pc) => {
                let l_value = machine.stack[machine.stack.len() - 1 - *l as usize];
                let r_value = machine.stack[machine.stack.len() - 1 - *r as usize];
                if l_value > r_value {
                    machine.pc = *pc - 1;
                }
            }
            Instruction::Eq(l, r, pc) => {
                let l_value = machine.stack[machine.stack.len() - 1 - *l as usize];
                let r_value = machine.stack[machine.stack.len() - 1 - *r as usize];
                if l_value == r_value {
                    machine.pc = *pc - 1;
                }
            }
            Instruction::Jmp(value) => {
                machine.pc = *value - 1;
            }
            Instruction::Dec(pointer) => {
                let index = machine.stack.len() - 1 - *pointer as usize;
                machine.stack[index] -= 1;
            }
            Instruction::Inc(pointer) => {
                let index = machine.stack.len() - 1 - *pointer as usize;
                machine.stack[index] += 1;
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
                Some(instruction) => {
                    // println!("{:?}", instruction);
                    instruction.clone().execute(self, &mut input, output)?
                }
                None => break,
            };
            self.pc += 1;
            // println!("{:?}", self.stack);
            // println!("{}", self.pc);
        }

        Ok(0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vm = Machine {
        code: vec![
            Instruction::Push(0), //0
            Instruction::In(),
            Instruction::Push(0),
            Instruction::Push(1),
            Instruction::Eq(2, 3, 9), //4
            Instruction::Copy(0),     //5
            Instruction::Add(1, 2),
            Instruction::Dec(2),
            Instruction::Jmp(4),
            Instruction::Out(0), //9
        ],
        stack: vec![],
        pc: 0,
    };

    vm.run(&mut stdin(), &mut stdout())?;

    Ok(())
}
