type IdKey = u64;

pub enum Op2 {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Lt,
    Gt,
    Eq,
    Ne,
    Xor,
    BAnd,
    BOr,
    BXor,
}

pub enum Op1 {
    Not,
    BNot,
}

pub enum Bytecode {
    Num(i64),
    Ref(IdKey),
    Op2(Op2),
    Op1(Op1),
    Call(IdKey),
    Store(IdKey),
    Jump(u64),
    Branch(u64),
    //Alloc(),
    //Slice(),
}
