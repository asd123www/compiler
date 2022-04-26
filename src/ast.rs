
// CompUnit  ::= FuncDef;
#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}


// FuncDef   ::= FuncType IDENT "(" ")" Block;
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

// FuncType  ::= "int";
#[derive(Debug)]
pub enum FuncType {
    Int,
    // Void,
    // Double,
    // Float,
    // String,
}

// Block     ::= "{" Stmt "}";
#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

// Stmt      ::= "return" Number ";";
#[derive(Debug)]
pub struct Stmt {
    pub exp: Exp,
}




// Exp         ::= LOrExp;
#[derive(Debug)]
pub struct Exp {
    pub lorexp: LOrExp,
}


// LOrExp      ::= LAndExp | LOrExp "||" LAndExp;
#[derive(Debug)]
pub enum LOrExp {
    Landexp(LAndExp),
    Orexp(Box<LOrExp>, LAndExp),
}

// LAndExp     ::= EqExp | LAndExp "&&" EqExp;
#[derive(Debug)]
pub enum LAndExp {
    Eqexp(EqExp),
    Andexp(Box<LAndExp>, EqExp),
}

// EqExp       ::= RelExp | EqExp ("==" | "!=") RelExp;
#[derive(Debug)]
pub enum EqExp {
    Relexp(RelExp),
    Eqexp(Box<EqExp>, RelExp, String),
    Neqexp(Box<EqExp>, RelExp, String),
}

// RelExp      ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
#[derive(Debug)]
pub enum RelExp {
    Addexp(AddExp),
    Ltexp(Box<RelExp>, AddExp, String),
    Gtexp(Box<RelExp>, AddExp, String),
    Geexp(Box<RelExp>, AddExp, String),
    Leexp(Box<RelExp>, AddExp, String),
}

// AddExp      ::= MulExp | AddExp ("+" | "-") MulExp;
#[derive(Debug)]
pub enum AddExp {
    Mulexp(MulExp),
    Addexp(Box<AddExp>, MulExp, String),
    Subexp(Box<AddExp>, MulExp, String),
}

// MulExp      ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
#[derive(Debug)]
pub enum MulExp {
    Unaryexp(UnaryExp),
    Mulexp(Box<MulExp>, UnaryExp, String),
    Divexp(Box<MulExp>, UnaryExp, String),
    Modexp(Box<MulExp>, UnaryExp, String),
}

// UnaryExp    ::= PrimaryExp | UnaryOp UnaryExp;
#[derive(Debug)]
pub enum UnaryExp {
    Primaryexp(PrimaryExp),
    Unaryexp(UnaryOp, Box<UnaryExp>),
}

// PrimaryExp  ::= "(" Exp ")" | Number;
#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Num(i32),
}



// UnaryOp     ::= "+" | "-" | "!";
#[derive(Debug)]
pub enum UnaryOp {
    Add,
    Sub,
    Exclamation,
}