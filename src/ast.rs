
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
    landexp(LAndExp),
    orexp(Box<LOrExp>, LAndExp),
}

// LAndExp     ::= EqExp | LAndExp "&&" EqExp;
#[derive(Debug)]
pub enum LAndExp {
    eqexp(EqExp),
    andexp(Box<LAndExp>, EqExp),
}

// EqExp       ::= RelExp | EqExp ("==" | "!=") RelExp;
#[derive(Debug)]
pub enum EqExp {
    relexp(RelExp),
    eqexp(Box<EqExp>, RelExp),
    neqexp(Box<EqExp>, RelExp),
}

// RelExp      ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
#[derive(Debug)]
pub enum RelExp {
    addexp(AddExp),
    ltexp(Box<RelExp>, AddExp),
    gtexp(Box<RelExp>, AddExp),
    geexp(Box<RelExp>, AddExp),
    leexp(Box<RelExp>, AddExp),
}

// AddExp      ::= MulExp | AddExp ("+" | "-") MulExp;
#[derive(Debug)]
pub enum AddExp {
    mulexp(MulExp),
    addexp(Box<AddExp>, MulExp),
    subexp(Box<AddExp>, MulExp),
}

// MulExp      ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
#[derive(Debug)]
pub enum MulExp {
    unaryexp(UnaryExp),
    mulexp(Box<MulExp>, UnaryExp),
    divexp(Box<MulExp>, UnaryExp),
    modexp(Box<MulExp>, UnaryExp),
}

// UnaryExp    ::= PrimaryExp | UnaryOp UnaryExp;
#[derive(Debug)]
pub enum UnaryExp {
    primaryexp(PrimaryExp),
    unaryexp(UnaryOp, Box<UnaryExp>),
}

// PrimaryExp  ::= "(" Exp ")" | Number;
#[derive(Debug)]
pub enum PrimaryExp {
    exp(Box<Exp>),
    num(i32),
}



// UnaryOp     ::= "+" | "-" | "!";
#[derive(Debug)]
pub enum UnaryOp {
    Add,
    Sub,
    Exclamation,
}