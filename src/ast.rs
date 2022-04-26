
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
    Eqexp(Box<EqExp>, RelExp),
    Neqexp(Box<EqExp>, RelExp),
}

// RelExp      ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
#[derive(Debug)]
pub enum RelExp {
    Addexp(AddExp),
    Ltexp(Box<RelExp>, AddExp),
    Gtexp(Box<RelExp>, AddExp),
    Geexp(Box<RelExp>, AddExp),
    Leexp(Box<RelExp>, AddExp),
}

// AddExp      ::= MulExp | AddExp ("+" | "-") MulExp;
#[derive(Debug)]
pub enum AddExp {
    Mulexp(MulExp),
    Addexp(Box<AddExp>, MulExp),
    Subexp(Box<AddExp>, MulExp),
}

// MulExp      ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
#[derive(Debug)]
pub enum MulExp {
    Unaryexp(UnaryExp),
    Mulexp(Box<MulExp>, UnaryExp),
    Divexp(Box<MulExp>, UnaryExp),
    Modexp(Box<MulExp>, UnaryExp),
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