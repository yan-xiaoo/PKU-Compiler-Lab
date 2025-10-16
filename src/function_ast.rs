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
    pub block: Block,
    pub span: Span
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncType {
    Int,
    Void
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize
}

#[derive(Debug)]
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl)
}

#[derive(Debug)]
pub struct ConstDecl {
    pub b_type: BType,
    pub const_def: Vec<ConstDef>
}

#[derive(Debug)]
pub enum BType {
    Int
}

#[derive(Debug)]
pub struct ConstDef {
    pub ident: String,
    pub const_init_val: ConstInitVal,
    pub span: Span
}

#[derive(Debug)]
pub struct ConstInitVal {
    pub const_exp: ConstExp
}

#[derive(Debug)]
pub struct VarDecl {
    pub b_type: BType,
    pub var_def: Vec<VarDef>
}

#[derive(Debug)]
pub struct VarDef {
    pub ident: String,
    pub init_val: Option<InitVal>,
    pub span: Span
}

#[derive(Debug)]
pub struct InitVal {
    pub exp: Exp
}

#[derive(Debug)]
pub struct Block {
    pub block_items: Vec<BlockItem>
}

#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt)
}

#[derive(Debug)]
pub enum Stmt {
    Exp(Exp),
    LValExp(LVal, Exp)
}

#[derive(Debug)]
pub struct ConstExp {
    pub exp: Exp
}

#[derive(Debug)]
pub struct Exp {
    pub l_or_exp: LOrExp
}

#[derive(Debug)]
pub enum UnaryExp {
    PrimaryExp(PrimaryExp),
    CompoundUnaryExp(UnaryOp, Box<UnaryExp>)
}

#[derive(Debug)]
pub enum LVal {
    Ident(String, Span),
}

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Number(i32),
    LVal(LVal)
}

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not
}

#[derive(Debug)]
pub enum MulOp {
    Mul,
    Div,
    Mod
}

#[derive(Debug)]
pub enum MulExp {
    UnaryExp(UnaryExp),
    CompoundMulExp(Box<MulExp>, UnaryExp, MulOp)
}

#[derive(Debug)]
pub enum AddOp {
    Plus,
    Minus
}

#[derive(Debug)]
pub enum AddExp {
    MulExp(MulExp),
    CompoundAddExp(Box<AddExp>, MulExp, AddOp)
}

#[derive(Debug)]
pub enum RelOp {
    // 小于
    Lt,
    // 大于
    Gt,
    // 小于等于
    Le,
    // 大于等于
    Ge
}

#[derive(Debug)]
pub enum RelExp {
    AddExp(AddExp),
    CompoundRelExp(Box<RelExp>, AddExp, RelOp)
}

#[derive(Debug)]
pub enum EqOp {
    Eq,
    Ne
}

#[derive(Debug)]
pub enum EqExp {
    RelExp(RelExp),
    CompoundEqExp(Box<EqExp>, RelExp, EqOp)
}

#[derive(Debug)]
pub enum LAndExp {
    EqExp(EqExp),
    CompoundLAndExp(Box<LAndExp>, EqExp)
}

#[derive(Debug)]
pub enum LOrExp {
    LAndExp(LAndExp),
    CompoundLOrExp(Box<LOrExp>, LAndExp)
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

impl std::fmt::Display for AddOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-")
        }
    }
}

impl std::fmt::Display for MulOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%")
        }
    }
}

impl std::fmt::Display for EqOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "="),
            Self::Ne => write!(f, "!=")
        }
    }
}

impl std::fmt::Display for RelOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::Le => write!(f, "<="),
            Self::Ge => write!(f, ">=")
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

impl std::fmt::Display for BType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "int")
        }
    }
}