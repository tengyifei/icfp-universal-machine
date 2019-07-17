use super::errors;
use super::instructions;
use std::collections::HashMap;
use std::io::Read;

/// A platter in the universal machine; a unit of storage.
pub type Word = u32;

pub struct Machine {
    finger: Word,
    registers: [Word; 8],
    program: Vec<Word>,
    data_arrays: HashMap<Word, Vec<Word>>,
    next_array_id: Word,
}

enum Continue {
    Yes,
    No,
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Machine {
        Machine {
            finger: 0,
            registers: [0; 8],
            program: Machine::load_program_from_bytes(program),
            data_arrays: HashMap::new(),
            next_array_id: 1,
        }
    }

    fn load_program_from_bytes(program_bytes: Vec<u8>) -> Vec<Word> {
        let num_words = if program_bytes.len() % 4 == 0 {
            program_bytes.len() / 4
        } else {
            program_bytes.len() / 4 + 1
        };
        let mut program = Vec::with_capacity(num_words);
        for i in 0..num_words {
            let a: u8;
            let mut b: u8 = 0;
            let mut c: u8 = 0;
            let mut d: u8 = 0;
            a = program_bytes[i * 4];
            if i * 4 + 1 < program_bytes.len() {
                b = program_bytes[i * 4 + 1];
            }
            if i * 4 + 2 < program_bytes.len() {
                c = program_bytes[i * 4 + 2];
            }
            if i * 4 + 3 < program_bytes.len() {
                d = program_bytes[i * 4 + 3];
            }
            let mut word: Word = u32::from(d);
            word += u32::from(c) << 8;
            word += u32::from(b) << 16;
            word += u32::from(a) << 24;
            program.push(word)
        }
        return program;
    }

    fn fetch_instruction(&mut self) -> Option<Word> {
        if self.finger as usize >= self.program.len() {
            None
        } else {
            let word = self.program[self.finger as usize];
            self.finger += 1;
            Some(word)
        }
    }

    fn read_register<T: From<Word>>(&self, reg: instructions::In<T>) -> Result<T, errors::UmError> {
        if reg.idx >= 8 {
            Err(errors::UmError::InvalidRegisterIndex { idx: reg.idx })
        } else {
            Ok(T::from(self.registers[reg.idx as usize]))
        }
    }

    fn set_register(&mut self, reg: instructions::Out, val: Word) -> Result<(), errors::UmError> {
        if reg.0 >= 8 {
            Err(errors::UmError::InvalidRegisterIndex { idx: reg.0 })
        } else {
            self.registers[reg.0 as usize] = val;
            Ok(())
        }
    }

    fn read_array(
        &self,
        array_id: instructions::ArrayId,
        offset: instructions::Offset,
    ) -> Result<Word, errors::UmError> {
        if array_id.0 == 0 {
            if (offset.0 as usize) < self.program.len() {
                Ok(self.program[offset.0 as usize])
            } else {
                Err(errors::UmError::ProgramOutOfRange)
            }
        } else {
            match self.data_arrays.get(&array_id.0) {
                Some(array) => {
                    if (offset.0 as usize) < array.len() {
                        Ok(array[offset.0 as usize])
                    } else {
                        Err(errors::UmError::ArrayOutOfRange)
                    }
                }
                None => Err(errors::UmError::InvalidArrayId),
            }
        }
    }

    fn write_array(
        &mut self,
        array_id: instructions::ArrayId,
        offset: instructions::Offset,
        val: Word,
    ) -> Result<(), errors::UmError> {
        if array_id.0 == 0 {
            if (offset.0 as usize) < self.program.len() {
                self.program[offset.0 as usize] = val;
                Ok(())
            } else {
                Err(errors::UmError::ProgramOutOfRange)
            }
        } else {
            match self.data_arrays.get_mut(&array_id.0) {
                Some(array) => {
                    if (offset.0 as usize) < array.len() {
                        array[offset.0 as usize] = val;
                        Ok(())
                    } else {
                        Err(errors::UmError::ArrayOutOfRange)
                    }
                }
                None => Err(errors::UmError::InvalidArrayId),
            }
        }
    }

    fn execute_instruction(
        &mut self,
        inst: instructions::Instruction,
    ) -> Result<Continue, errors::UmError> {
        use instructions::Instruction;

        match inst {
            Instruction::ConditionalMove { dest, src, test } => {
                let test_val = self.read_register(test)?;
                if test_val != 0 {
                    self.set_register(dest, self.read_register(src)?)?;
                }
                Ok(Continue::Yes)
            }
            Instruction::ArrayIndex {
                dest,
                offset,
                array,
            } => {
                let offset_val = self.read_register(offset)?;
                let array_id = self.read_register(array)?;
                let val = self.read_array(array_id, offset_val)?;
                self.set_register(dest, val)?;
                Ok(Continue::Yes)
            }
            Instruction::ArrayAmend { array, offset, val } => {
                let offset_val = self.read_register(offset)?;
                let array_id = self.read_register(array)?;
                let val_val = self.read_register(val)?;
                self.write_array(array_id, offset_val, val_val)?;
                Ok(Continue::Yes)
            }
            Instruction::Add { dest, x, y } => {
                let x_val = self.read_register(x)?;
                let y_val = self.read_register(y)?;
                let result = x_val.wrapping_add(y_val);
                self.set_register(dest, result)?;
                Ok(Continue::Yes)
            }
            Instruction::Multiply { dest, x, y } => {
                let x_val = self.read_register(x)?;
                let y_val = self.read_register(y)?;
                let result = x_val.wrapping_mul(y_val);
                self.set_register(dest, result)?;
                Ok(Continue::Yes)
            }
            Instruction::Divide { dest, x, y } => {
                let x_val = self.read_register(x)?;
                let y_val = self.read_register(y)?;
                if y_val == 0 {
                    Err(errors::UmError::DivideByZero)
                } else {
                    let result = x_val / y_val;
                    self.set_register(dest, result)?;
                    Ok(Continue::Yes)
                }
            }
            Instruction::Nand { dest, x, y } => {
                let x_val = self.read_register(x)?;
                let y_val = self.read_register(y)?;
                let result = !(x_val & y_val);
                self.set_register(dest, result)?;
                Ok(Continue::Yes)
            }
            Instruction::Halt => Ok(Continue::No),
            Instruction::Allocate { size, result } => {
                let size_val = self.read_register(size)?;
                let new_array = vec![0; size_val as usize];
                self.data_arrays.insert(self.next_array_id, new_array);
                self.set_register(result, self.next_array_id)?;
                self.next_array_id = self.next_array_id.wrapping_add(1);
                if self.next_array_id == 0 {
                    self.next_array_id += 1;
                }
                Ok(Continue::Yes)
            }
            Instruction::Abandon { which } => {
                let which_val = self.read_register(which)?;
                if which_val.0 == 0 {
                    Err(errors::UmError::CannotAbandonProgram)
                } else {
                    match self.data_arrays.remove(&which_val.0) {
                        Some(_) => Ok(Continue::Yes),
                        None => Err(errors::UmError::InvalidArrayId),
                    }
                }
            }
            Instruction::Output { val } => {
                let val_val = self.read_register(val)?;
                if val_val <= 255 {
                    print!("{}", val_val as u8 as char);
                    Ok(Continue::Yes)
                } else {
                    Err(errors::UmError::InvalidOutput { val: val_val })
                }
            }
            Instruction::Input { dest } => {
                let input: Option<i32> = std::io::stdin()
                    .bytes()
                    .next()
                    .and_then(|result| result.ok())
                    .map(|byte| byte as i32);
                match input {
                    Some(c) => {
                        self.set_register(dest, c as Word)?;
                        Ok(Continue::Yes)
                    }
                    None => {
                        self.set_register(dest, u32::max_value())?;
                        Ok(Continue::Yes)
                    }
                }
            }
            Instruction::LoadProgram { from, finger } => {
                let array_id = self.read_register(from)?;
                let finger_val = self.read_register(finger)?;
                if array_id.0 == 0 {
                    self.finger = finger_val;
                    Ok(Continue::Yes)
                } else {
                    match self.data_arrays.get_mut(&array_id.0) {
                        Some(array) => {
                            self.program = array.clone();
                            self.finger = finger_val;
                            Ok(Continue::Yes)
                        }
                        None => Err(errors::UmError::InvalidArrayId),
                    }
                }
            }
            Instruction::LoadRegister { dest, val } => {
                self.set_register(dest, val)?;
                Ok(Continue::Yes)
            }
        }
    }

    /// Starts the universal machine.
    /// Runs indefinitely until an error or the end of a program.
    pub fn execute(mut self) -> Result<(), errors::UmError> {
        loop {
            match self.fetch_instruction() {
                Some(word) => {
                    let inst = instructions::Instruction::decode_from(word)?;
                    let cont = self.execute_instruction(inst)?;
                    match cont {
                        Continue::Yes => {}
                        Continue::No => return Ok(()),
                    }
                }
                None => {
                    return Ok(());
                }
            }
        }
    }
}
