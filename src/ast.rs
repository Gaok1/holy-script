/// Holy language types.
#[derive(Debug, Clone)]
pub enum HolyType {
    Atom,           // i64
    Fractional,     // f64
    Word,           // String
    Dogma,          // bool: blessed (true) | forsaken (false)
    Void,
    Custom(String), // user-defined scripture or covenant
}

/// Literal values.
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// Binary operators (arithmetic and comparison).
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Gt, Lt, Ge, Le,
}

/// Expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Literal),
    Var(String),
    /// `negate <expr>`  →  unary minus
    Negate(Box<Expr>),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    /// `hail salm (praying args)?`
    FnCall { name: String, args: Vec<Expr> },
    /// `hail method upon target (praying args)?`
    MethodCall { method: String, target: String, args: Vec<Expr> },
    /// `manifest Scripture (praying args)?`
    Manifest { scripture: String, args: Vec<Expr> },
    /// `field from <expr>`  — supports chaining: `b from fieldComposite from its`
    FieldAccess { field: String, object: Box<Expr> },
    /// `field from its`  — inside a method_salm (leaf of a from-chain)
    SelfFieldAccess { field: String },
}

/// A single variant inside a `covenant` declaration.
#[derive(Debug, Clone)]
pub struct CovenantVariantDecl {
    pub name:   String,
    pub fields: Vec<(String, HolyType)>, // empty = unit variant
}

/// A single branch inside a `discern` block.
#[derive(Debug, Clone)]
pub struct DiscernBranch {
    pub variant:  String,
    pub bindings: Vec<String>, // positional; empty if unit variant
    pub body:     Vec<Stmt>,
}

/// A single `answer for` clause inside a `confess` block.
#[derive(Debug, Clone)]
pub struct SinHandler {
    pub sin_type: String,
    pub binding:  Option<String>,
    pub body:     Vec<Stmt>,
}

/// Statements.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let there be x of type`  — zero-initialised
    DeclNoVal { name: String, ty: HolyType },
    /// `let there x of type be expr`
    DeclVal   { name: String, ty: HolyType, val: Expr },
    /// `x become expr`
    Assign    { name: String, val: Expr },
    /// `hail salm (praying args)?`  as a statement
    FnCallStmt     { name: String, args: Vec<Expr> },
    /// `hail method upon target (praying args)?`  as a statement
    MethodCallStmt { method: String, target: String, args: Vec<Expr> },
    /// `reveal expr`  — return value from a salm
    Reveal(Expr),
    /// `whether / otherwise so / otherwise`
    Conditional {
        branches:  Vec<(Expr, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
    },
    /// `litany for <cond>`  — while loop
    Litany { cond: Expr, body: Vec<Stmt> },
    /// `forsake`  — break out of a litany
    Forsake,
    /// `ascend`  — continue to next litany iteration
    Ascend,
    /// `confess / answer for / absolve`  — try/catch/finally
    Confess {
        try_block: Vec<Stmt>,
        handlers:  Vec<SinHandler>,
        absolve:   Option<Vec<Stmt>>,
    },
    /// `discern x  as Variant (bearing b1, b2)? ...`  — pattern match on a covenant variant
    Discern {
        target:    String,
        branches:  Vec<DiscernBranch>,
        otherwise: Option<Vec<Stmt>>,
    },
    /// `transgress SinType (praying args)?`  — throw a sin
    Transgress { sin_type: String, args: Vec<Expr> },
}

/// Top-level declarations (appear before statements in the program).
#[derive(Debug, Clone)]
pub enum TopDecl {
    /// Regular function.
    Salm {
        name:     String,
        params:   Vec<(String, HolyType)>,
        ret_type: HolyType,
        body:     Vec<Stmt>,
    },
    /// Method bound to a scripture type (`upon`).
    /// `its` is available inside the body as a reference to the instance.
    MethodSalm {
        name:        String,
        target_type: String,
        params:      Vec<(String, HolyType)>,
        ret_type:    HolyType,
        body:        Vec<Stmt>,
    },
    /// Pure data structure (no behaviour).
    Scripture {
        name:   String,
        fields: Vec<(String, HolyType)>,
    },
    /// Throwable/catchable error type.
    SinDecl {
        name:   String,
        fields: Vec<(String, HolyType)>,
    },
    /// Sum type with named variants; each variant may carry named fields.
    Covenant {
        name:     String,
        variants: Vec<CovenantVariantDecl>,
    },
}

/// Import declaration at the top of a program.
#[derive(Debug, Clone)]
pub struct Testament {
    pub name:      String,
    /// `None` — import everything; `Some(list)` — import only the listed symbols.
    pub revealing: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub testaments: Vec<Testament>,
    pub top_decls:  Vec<TopDecl>,
    pub stmts:      Vec<Stmt>,
}
