use super::machine::Word;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum UmError {
    UnknownInstruction { inst: Word },
    InvalidRegisterIndex { idx: u8 },
    ProgramOutOfRange,
    ArrayOutOfRange,
    InvalidArrayId,
    DivideByZero,
    CannotAbandonProgram,
    InvalidOutput { val: Word },
}

impl fmt::Display for UmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for UmError {}
