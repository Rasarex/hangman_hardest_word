use std::fmt;
#[allow(dead_code)]
#[derive(Clone)]
pub enum CmdError {
    IterDefault,
    InputDefault,
    SingleWordDefault,
    IterWrongArgumentType,
    NoSuchWord,
    NoCmd
}

impl std::fmt::Debug for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CmdError::*;
        match self {
            IterDefault => write!(f, "Usage -I <num>"),
            InputDefault => write!(f, "Usage -i <input>"),
            SingleWordDefault => write!(f, "Usage -w <word>"),
            IterWrongArgumentType => write!(f, "Numbers of iteration must be a unsigned integer"),
            NoCmd => write!(f, "No such command"),
            NoSuchWord => write!(f, "No such word"),
        }
    }
}
impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CmdError::*;
        match self {
            IterDefault => write!(f, "Usage -I <num>"),
            InputDefault => write!(f, "Usage -i <input>"),
            SingleWordDefault => write!(f, "Usage -w <word>"),
            IterWrongArgumentType => write!(f, "Numbers of iteration must be a unsigned integer"),
            NoCmd => write!(f, "No such command"),
            NoSuchWord => write!(f, "No such word"),
        }
    }
}
impl std::error::Error for CmdError {}
