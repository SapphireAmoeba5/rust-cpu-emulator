use num_derive::FromPrimitive;

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum RegisterId {
    X0 = 1,
    X1 = 2,
    X2 = 3,
    X3 = 4,
    X4 = 5,
    Sp = 6,
    Ip = 7,
}

impl RegisterId {
    pub fn to_index(self) -> usize {
        // We need to subtract 1 because the Id of a register is always one higher than it's index
        self as usize - 1
    }
}
