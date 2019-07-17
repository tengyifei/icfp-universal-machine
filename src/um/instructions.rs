use super::errors::UmError;
use super::machine::Word;
use std::marker::PhantomData;

/// Identifies an input register by index.
/// `T` hints the type of the value stored in said register.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct In<T> {
    pub idx: u8,
    phantom: PhantomData<T>,
}

impl<T> In<T> {
    fn new(idx: u8) -> In<T> {
        In {
            idx: idx,
            phantom: PhantomData,
        }
    }
}

/// Identifies an output register by index.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Out(pub u8);

impl Out {
    fn new(idx: u8) -> Out {
        Out(idx)
    }
}

/// Offset into an array.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Offset(pub Word);

impl From<u32> for Offset {
    fn from(x: u32) -> Self {
        Offset(x)
    }
}

/// Identifies an array.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ArrayId(pub Word);

impl From<u32> for ArrayId {
    fn from(x: u32) -> Self {
        ArrayId(x)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    ConditionalMove {
        dest: Out,
        src: In<Word>,
        test: In<Word>,
    },
    ArrayIndex {
        dest: Out,
        offset: In<Offset>,
        array: In<ArrayId>,
    },
    ArrayAmend {
        array: In<ArrayId>,
        offset: In<Offset>,
        val: In<Word>,
    },
    Add {
        dest: Out,
        x: In<Word>,
        y: In<Word>,
    },
    Multiply {
        dest: Out,
        x: In<Word>,
        y: In<Word>,
    },
    Divide {
        dest: Out,
        x: In<Word>,
        y: In<Word>,
    },
    Nand {
        dest: Out,
        x: In<Word>,
        y: In<Word>,
    },
    Halt,
    Allocate {
        size: In<Word>,
        result: Out,
    },
    Abandon {
        which: In<ArrayId>,
    },
    Output {
        val: In<Word>,
    },
    Input {
        dest: Out,
    },
    LoadProgram {
        from: In<ArrayId>,
        finger: In<Word>,
    },
    LoadRegister {
        dest: Out,
        val: Word,
    },
}

struct Abc {
    a: u8,
    b: u8,
    c: u8,
}

impl Instruction {
    fn parse_standard_abc(word: Word) -> Abc {
        Abc {
            a: ((word >> 6) & 7) as u8,
            b: ((word >> 3) & 7) as u8,
            c: (word & 7) as u8,
        }
    }

    pub fn decode_from(word: Word) -> Result<Instruction, UmError> {
        let op_number = word >> 28;
        match op_number {
            0 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::ConditionalMove {
                    dest: Out::new(abc.a),
                    src: In::new(abc.b),
                    test: In::new(abc.c),
                })
            }
            1 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::ArrayIndex {
                    dest: Out::new(abc.a),
                    offset: In::new(abc.c),
                    array: In::new(abc.b),
                })
            }
            2 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::ArrayAmend {
                    array: In::new(abc.a),
                    offset: In::new(abc.b),
                    val: In::new(abc.c),
                })
            }
            3 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Add {
                    dest: Out::new(abc.a),
                    x: In::new(abc.b),
                    y: In::new(abc.c),
                })
            }
            4 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Multiply {
                    dest: Out::new(abc.a),
                    x: In::new(abc.b),
                    y: In::new(abc.c),
                })
            }
            5 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Divide {
                    dest: Out::new(abc.a),
                    x: In::new(abc.b),
                    y: In::new(abc.c),
                })
            }
            6 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Nand {
                    dest: Out::new(abc.a),
                    x: In::new(abc.b),
                    y: In::new(abc.c),
                })
            }
            7 => Ok(Instruction::Halt),
            8 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Allocate {
                    size: In::new(abc.c),
                    result: Out::new(abc.b),
                })
            }
            9 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Abandon {
                    which: In::new(abc.c),
                })
            }
            10 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Output {
                    val: In::new(abc.c),
                })
            }
            11 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::Input {
                    dest: Out::new(abc.c),
                })
            }
            12 => {
                let abc = Instruction::parse_standard_abc(word);
                Ok(Instruction::LoadProgram {
                    from: In::new(abc.b),
                    finger: In::new(abc.c),
                })
            }
            13 => {
                let a = ((word >> 25) & 7) as u8;
                let value = word & ((1 << 25) - 1);
                Ok(Instruction::LoadRegister {
                    dest: Out::new(a),
                    val: value,
                })
            }
            _ => Err(UmError::UnknownInstruction { inst: word }),
        }
    }
}
