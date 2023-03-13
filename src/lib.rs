use std::io::{self, BufRead, BufReader, Read, Write};

//Stack virtual machine
#[derive(Debug, Clone)]
pub enum Instruction {
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

macro_rules! deserialize_variant {
    ($variant:ident, $input:ident, $($field:ident),*) => {{
        let mut buf = [0; 8];
        $(
            $input.read_exact(&mut buf)?;
            let $field = u64::from_le_bytes(buf);
        )*
            Ok(Instruction::$variant($($field),*))
    }}
}

impl Instruction {
    fn serialize<W: Write>(&self, output: &mut W) -> io::Result<()> {
        match &self {
            Self::Push(a) => {
                output.write(&[0])?;
                output.write(&a.to_le_bytes())?;
            }
            Self::Out(a) => {
                output.write(&[1])?;
                output.write(&a.to_le_bytes())?;
            }
            Self::In() => {
                output.write(&[2])?;
            }
            Self::OutStr(a) => {
                output.write(&[3])?;
                output.write(&a.as_bytes())?;
            }
            Self::Copy(a) => {
                output.write(&[4])?;
                output.write(&a.to_le_bytes())?;
            }
            Self::Add(a, b) => {
                output.write(&[5])?;
                output.write(&a.to_le_bytes())?;
                output.write(&b.to_le_bytes())?;
            }
            Self::Gt(a, b, c) => {
                output.write(&[6])?;
                output.write(&a.to_le_bytes())?;
                output.write(&b.to_le_bytes())?;
                output.write(&c.to_le_bytes())?;
            }
            Self::Eq(a, b, c) => {
                output.write(&[7])?;
                output.write(&a.to_le_bytes())?;
                output.write(&b.to_le_bytes())?;
                output.write(&c.to_le_bytes())?;
            }
            Self::Jmp(a) => {
                output.write(&[8])?;
                output.write(&a.to_le_bytes())?;
            }
            Self::Dec(a) => {
                output.write(&[9])?;
                output.write(&a.to_le_bytes())?;
            }
            Self::Inc(a) => {
                output.write(&[10])?;
                output.write(&a.to_le_bytes())?;
            }
        }
        Ok(())
    }

    fn deserialize<R: Read>(input: &mut R) -> io::Result<Self> {
        let mut tag = [0];
        input.read_exact(&mut tag)?;
        match tag[0] {
            0 => deserialize_variant!(Push, input, a),
            1 => deserialize_variant!(Out, input, a),
            2 => Ok(Self::In()),
            3 => {
                let mut buf = Vec::new();
                input.read_to_end(&mut buf)?;
                let a = String::from_utf8(buf).unwrap();
                Ok(Self::OutStr(a))
            }
            4 => deserialize_variant!(Copy, input, a),
            5 => deserialize_variant!(Add, input, a, b),
            6 => deserialize_variant!(Gt, input, a, b, c),
            7 => deserialize_variant!(Eq, input, a, b, c),
            8 => deserialize_variant!(Jmp, input, a),
            9 => deserialize_variant!(Dec, input, a),
            10 => deserialize_variant!(Inc, input, a),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid tag")),
        }
    }

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

pub struct Machine {
    pub code: Vec<Instruction>,
    pub stack: Vec<u64>,
    pub pc: u64,
}

impl Machine {
    pub fn run<W: Write, R: Read>(&mut self, input: &mut R, output: &mut W) -> io::Result<usize> {
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

pub fn serialize_code<W: Write>(instructions: &[Instruction], writer: &mut W) -> io::Result<()> {
    for instr in instructions {
        instr.serialize(writer)?;
    }
    Ok(())
}

pub fn deserialize_code<R: Read>(reader: &mut R) -> io::Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    loop {
        match Instruction::deserialize(reader) {
            Ok(instr) => instructions.push(instr),
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }
    Ok(instructions)
}
