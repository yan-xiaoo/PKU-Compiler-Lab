#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SysyParseError {
    // 部分函数要求特定的返回类型（比如 main 函数），如果此函数实际不满足特定的返回类型，那么出现下方的错误
    // 参数：错误的函数名称-错误的返回类型-正确的返回类型。
    InvalidReturnType(String, String, String)
}


impl std::fmt::Display for SysyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidReturnType(func_name, wrong, correct) => write!(f, "Function '{}' must return '{}', found '{}'", func_name, correct, wrong)
        }
    }
}


#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncType {
    Int,
    Void
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt
}

#[derive(Debug)]
pub struct Stmt {
    pub expr: Exp
}

#[derive(Debug)]
pub struct Exp {
    pub unary_exp: UnaryExp
}

#[derive(Debug)]
pub enum UnaryExp {
    PrimaryExp(PrimaryExp),
    CompoundUnaryExp(UnaryOp, Box<UnaryExp>)
}

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Number(i32)
}

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Not => write!(f, "!")
        }
    }
}

impl std::fmt::Display for FuncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Void => write!(f, "void")
        }
    }
}