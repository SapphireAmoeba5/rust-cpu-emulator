use num_derive::FromPrimitive;



#[derive(Debug, FromPrimitive)]
pub enum RegisterId {
    X0 = 1,
    X1 = 2,
    X2 = 3,
    X3 = 4,
    X4 = 5,
    Sp = 6,
    Ip = 7,
}