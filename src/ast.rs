
// ------------------------------ Function ------------------------------------------

// CompUnit  ::= [CompUnit] FuncDef;
#[derive(Debug)]
pub enum CompUnit {
    Single(FuncDef),
    Multiple(FuncDef, Box<CompUnit>),
}


// [...]代表里面出现0或1次.
// FuncDef   ::= FuncType IDENT "(" [FuncFParams] ")" Block;
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
    pub params: Option<FuncFParams>,
}

// FuncFParams ::= FuncFParam {"," FuncFParam};
#[derive(Debug)]
pub struct FuncFParams {
    pub params: Vec<FuncFParam>,
}

// FuncFParam  ::= BType IDENT;
#[derive(Debug)]
pub struct FuncFParam {
    pub btype: BType,
    pub ident: String,
}


// FuncType  ::= "void" | "int";
#[derive(Debug)]
pub enum FuncType {
    Int,
    Void,
}




// ------------------------------ body ------------------------------------------

// Block         ::= "{" {BlockItem} "}";
#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}


// BlockItem     ::= Decl | Stmt;
#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Statement(Statement),
}


// statement: open_statement
//          | closed_statement
#[derive(Debug)]
pub enum Statement {
    Open(OpenStatement),
    Closed(ClosedStatement),
}


// open_statement: IF '(' expression ')' statement
//               | IF '(' expression ')' closed_statement ELSE open_statement
#[derive(Debug)]
pub enum OpenStatement {
    If(Exp, Box<Statement>),
    Ifelse(Exp, ClosedStatement, Box<OpenStatement>),
    While(Exp, Box<Statement>),
}


// closed_statement: non_if_statement
//                 | IF '(' expression ')' closed_statement ELSE closed_statement
#[derive(Debug)]
pub enum ClosedStatement {
    Stmt(Stmt),
    Ifelse(Exp, Box<ClosedStatement>, Box<ClosedStatement>),
}


// Stmt ::= LVal "=" Exp ";"
//        | [Exp] ";"
//        | Block
//        | "return" [Exp] ";";
#[derive(Debug)]
pub enum Stmt {
    RetExp(Exp),
    LvalExp(LVal, Exp),
    SingleExp(Exp),
    Block(Block),
    ZeroExp(),
    BreakKeyWord(),
    ContinueKeyWord(),
}







// ------------------------------ Variable ------------------------------------------

// LVal          ::= IDENT;
#[derive(Debug)]
pub struct LVal {
    pub ident: String,
}

// Decl          ::= ConstDecl | VarDecl;
#[derive(Debug)]
pub enum Decl {
    Constdecl(ConstDecl),
    Vardecl(VarDecl),
}

// ConstDecl     ::= "const" BType ConstDef {"," ConstDef} ";";
#[derive(Debug)]
pub struct ConstDecl {
    pub btype: BType,
    pub constdefs: Vec<ConstDef>,
}
// VarDecl       ::= BType VarDef {"," VarDef} ";";
#[derive(Debug)]
pub struct VarDecl {
    pub btype: BType,
    pub vardefs: Vec<VarDef>,
}

// BType         ::= "int";
#[derive(Debug)]
pub enum BType {
    Int,
    // Void,
    // Double,
    // Float,
    // String,
}

// ConstDef      ::= IDENT "=" ConstInitVal;
#[derive(Debug)]
pub struct ConstDef {
    pub ident: String,
    pub constinitval: ConstInitVal,
}
// VarDef        ::= IDENT | IDENT "=" InitVal;
#[derive(Debug)]
pub enum VarDef {
    Ident(String),
    Identinitval(String, InitVal),
}

// ConstInitVal  ::= ConstExp;
#[derive(Debug)]
pub struct ConstInitVal {
    pub constexp: ConstExp,
}

// InitVal       ::= Exp;
#[derive(Debug)]
pub struct InitVal {
    pub exp: Exp,
}







// ------------------------------ Expression ------------------------------------------

// ConstExp      ::= Exp;
#[derive(Debug)]
pub struct ConstExp {
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

// UnaryExp    ::= PrimaryExp
//               | IDENT "(" [FuncRParams] ")"
//               | UnaryOp UnaryExp
#[derive(Debug)]
pub enum UnaryExp {
    Primaryexp(PrimaryExp),
    Unaryexp(UnaryOp, Box<UnaryExp>),
    Funcall(String, Option<FuncRParams>),
}

// FuncRParams ::= Exp {"," Exp};
#[derive(Debug)]
pub struct FuncRParams {
    pub params: Vec<Exp>,
}

// PrimaryExp    ::= "(" Exp ")" | LVal | Number;
#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Lval(LVal),
    Num(i32),
}

// UnaryOp     ::= "+" | "-" | "!";
#[derive(Debug)]
pub enum UnaryOp {
    Add,
    Sub,
    Not,
}