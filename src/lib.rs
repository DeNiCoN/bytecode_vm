use std::io::{self, BufRead, BufReader, Read, Write};

//Stack virtual machine
#[derive(Debug, Clone, PartialEq)]
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
    InByte(),
    OutByte(u64),
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
            Self::InByte() => {
                output.write(&[11])?;
            }
            Self::OutByte(a) => {
                output.write(&[12])?;
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
            11 => Ok(Self::InByte()),
            12 => deserialize_variant!(OutByte, input, a),
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
                writeln!(output, "{}", value)?;
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
                    machine.pc = *pc;
                    return Ok(0);
                }
            }
            Instruction::Eq(l, r, pc) => {
                let l_value = machine.stack[machine.stack.len() - 1 - *l as usize];
                let r_value = machine.stack[machine.stack.len() - 1 - *r as usize];
                if l_value == r_value {
                    machine.pc = *pc;
                    return Ok(0);
                }
            }
            Instruction::Jmp(value) => {
                machine.pc = *value;
                return Ok(0);
            }
            Instruction::Dec(pointer) => {
                let index = machine.stack.len() - 1 - *pointer as usize;
                machine.stack[index] -= 1;
            }
            Instruction::Inc(pointer) => {
                let index = machine.stack.len() - 1 - *pointer as usize;
                machine.stack[index] += 1;
            }
            Instruction::InByte() => {
                let mut buf = [0];
                input.read_exact(&mut buf)?;
                let value = u8::from_le_bytes(buf);
                machine.stack.push(value as u64);
            }
            Instruction::OutByte(pointer) => {
                let value: u8 =
                    u8::try_from(machine.stack[machine.stack.len() - 1 - *pointer as usize])
                        .unwrap();
                output.write(&[value])?;
            }
        };

        machine.pc += 1;

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
                    match instruction.clone().execute(self, &mut input, output) {
                        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(e),
                        Ok(_) => (),
                    }
                }
                None => break,
            };
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn test_instruction_execution(
        instruction: Instruction,
        machine: &mut Machine,
        expected_machine: Machine,
        input_data: &[u8],
        expected_output: &[u8],
    ) {
        let mut input = Cursor::new(input_data);
        let mut output = Vec::new();
        instruction
            .execute(machine, &mut input, &mut output)
            .unwrap();

        assert_eq!(machine.stack, expected_machine.stack);
        assert_eq!(machine.pc, expected_machine.pc);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_push() {
        let instruction = Instruction::Push(42);
        let mut machine = Machine {
            code: Vec::new(),
            stack: Vec::new(),
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![42],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_out() {
        let instruction = Instruction::Out(0);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![5],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![5],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], b"5\n");
    }

    #[test]
    fn test_in() {
        let instruction = Instruction::In();
        let mut machine = Machine {
            code: Vec::new(),
            stack: Vec::new(),
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![42],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, b"42\n", &[]);
    }

    #[test]
    fn test_add() {
        let instruction = Instruction::Add(0, 1);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![2, 3],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![5],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_copy() {
        let instruction = Instruction::Copy(0);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![5],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![5, 5],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_gt_true() {
        let instruction = Instruction::Gt(0, 1, 5);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![2, 4],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![2, 4],
            pc: 5,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_gt_false() {
        let instruction = Instruction::Gt(0, 1, 5);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![4, 2],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![4, 2],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_eq_true() {
        let instruction = Instruction::Eq(0, 1, 5);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![4, 4],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![4, 4],
            pc: 5,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_eq_false() {
        let instruction = Instruction::Eq(0, 1, 5);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![2, 4],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![2, 4],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_jmp() {
        let instruction = Instruction::Jmp(5);
        let mut machine = Machine {
            code: Vec::new(),
            stack: Vec::new(),
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: Vec::new(),
            pc: 5,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_dec() {
        let instruction = Instruction::Dec(0);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![5],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![4],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_inc() {
        let instruction = Instruction::Inc(0);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![5],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![6],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[]);
    }

    #[test]
    fn test_in_byte() {
        let instruction = Instruction::InByte();
        let mut machine = Machine {
            code: Vec::new(),
            stack: Vec::new(),
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![65],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, b"A", &[]);
    }

    #[test]
    fn test_out_byte() {
        let instruction = Instruction::OutByte(0);
        let mut machine = Machine {
            code: Vec::new(),
            stack: vec![65],
            pc: 0,
        };
        let expected_machine = Machine {
            code: Vec::new(),
            stack: vec![65],
            pc: 1,
        };
        test_instruction_execution(instruction, &mut machine, expected_machine, &[], &[65]);
    }
}

#[cfg(test)]
mod test_serialization {
    use super::*;

    fn test_serialize_deserialize(instruction: Instruction) {
        let mut serialized = Vec::new();
        instruction.serialize(&mut serialized).unwrap();

        let mut deserialized = &serialized[..];
        let instruction_back = Instruction::deserialize(&mut deserialized).unwrap();

        assert_eq!(instruction, instruction_back);
    }

    #[test]
    fn test_serialize_push() {
        test_serialize_deserialize(Instruction::Push(42));
    }

    #[test]
    fn test_serialize_out() {
        test_serialize_deserialize(Instruction::Out(1));
    }

    #[test]
    fn test_serialization_dec() {
        test_serialize_deserialize(Instruction::Dec(10));
    }

    #[test]
    fn test_serialization_inc() {
        test_serialize_deserialize(Instruction::Inc(15));
    }

    #[test]
    fn test_serialization_in_byte() {
        test_serialize_deserialize(Instruction::InByte());
    }

    #[test]
    fn test_serialization_out_byte() {
        test_serialize_deserialize(Instruction::OutByte(10));
    }

    #[test]
    fn test_serialization_in() {
        test_serialize_deserialize(Instruction::In());
    }

    #[test]
    fn test_serialization_out_str() {
        test_serialize_deserialize(Instruction::OutStr("Hello, world!".to_string()));
    }

    #[test]
    fn test_serialization_copy() {
        test_serialize_deserialize(Instruction::Copy(5));
    }

    #[test]
    fn test_serialization_add() {
        test_serialize_deserialize(Instruction::Add(5, 7));
    }

    #[test]
    fn test_serialization_gt() {
        test_serialize_deserialize(Instruction::Gt(3, 4, 5));
    }

    #[test]
    fn test_serialization_eq() {
        test_serialize_deserialize(Instruction::Eq(3, 4, 5));
    }

    #[test]
    fn test_serialization_jmp() {
        test_serialize_deserialize(Instruction::Jmp(6));
    }
}
