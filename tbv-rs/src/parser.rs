/// Recursive-descent parser for Trygve Bjerkreim.
///
/// Grammar summary (top-level constructs):
///   program    := stmt*
///   stmt       := assign | reassign | print | input | if | while | foreach
///               | infinite_loop | count_loop | funcdef | funccall | return
///               | trycatch
///   expr       := comparison
///   comparison := additive (cmp_op additive)?
///   additive   := multiplicative ((og | utan) multiplicative)*
///   multi      := primary ((gongar | delt på) primary)*
///   primary    := literal | var | func_call | list | (expr)
///               | resten av … delt på …  | ikkje …

use crate::lexer::{Token, TokenKind};

// ─────────────────────────────────────────────────────────────────
// AST
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign    { name: String, value: Expr, line: usize },
    Reassign  { name: String, value: Expr, line: usize },
    Print     { value: Expr, line: usize },
    Input     { name: String, line: usize },
    If        { cond: Expr, then_body: Vec<Stmt>, else_body: Option<Vec<Stmt>>, line: usize },
    While     { cond: Expr, body: Vec<Stmt>, line: usize },
    ForEach   { var: String, iterable: Expr, body: Vec<Stmt>, line: usize },
    InfLoop   { body: Vec<Stmt>, line: usize },
    CountLoop { count: Expr, var: String, body: Vec<Stmt>, line: usize },
    FuncDef   { name: String, params: Vec<String>, body: Vec<Stmt>, line: usize },
    FuncCall  { name: String, args: Vec<Expr>, line: usize },
    Return    { value: Expr, line: usize },
    TryCatch  { try_body: Vec<Stmt>, catch_body: Vec<Stmt>, line: usize },
    Break     { line: usize },
    Continue  { line: usize },
    Serve     { port: Expr, body: Vec<Stmt>, line: usize },
    Respond   { value: Expr, line: usize },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    Var(String),
    List(Vec<Expr>),
    Index { obj: Box<Expr>, idx: Box<Expr> },
    BinOp { op: BinOpKind, left: Box<Expr>, right: Box<Expr> },
    Not(Box<Expr>),
    Call { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt,
}

pub type Program = Vec<Stmt>;

// ─────────────────────────────────────────────────────────────────
// Parser state
// ─────────────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn cur(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(self.tokens.last().unwrap())
    }

    fn peek(&self, offset: usize) -> &Token {
        let i = self.pos + offset;
        self.tokens.get(i).unwrap_or(self.tokens.last().unwrap())
    }

    fn is_eof(&self) -> bool {
        matches!(self.cur().kind, TokenKind::Eof)
    }

    fn is_word(&self, w: &str) -> bool {
        self.cur().is_word(w)
    }

    fn is_word_at(&self, offset: usize, w: &str) -> bool {
        self.peek(offset).is_word(w)
    }

    fn is_phrase(&self, words: &[&str]) -> bool {
        words.iter().enumerate().all(|(i, w)| self.peek(i).is_word(w))
    }

    fn is_kind(&self, k: &TokenKind) -> bool {
        std::mem::discriminant(&self.cur().kind) == std::mem::discriminant(k)
    }

    fn eat_word(&mut self, w: &str) -> Result<(), String> {
        if self.cur().is_word(w) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!(
                "Line {}: expected '{}', got {:?}",
                self.cur().line, w, self.cur().kind
            ))
        }
    }

    fn eat_phrase(&mut self, words: &[&str]) -> Result<(), String> {
        for w in words {
            self.eat_word(w)?;
        }
        Ok(())
    }

    fn eat_ident(&mut self) -> Result<String, String> {
        match self.cur().kind.clone() {
            TokenKind::Word(s) => { self.pos += 1; Ok(s) }
            _ => Err(format!("Line {}: expected identifier, got {:?}", self.cur().line, self.cur().kind))
        }
    }

    fn eat_kind(&mut self, k: &TokenKind) -> Result<(), String> {
        if std::mem::discriminant(&self.cur().kind) == std::mem::discriminant(k) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("Line {}: expected {:?}, got {:?}", self.cur().line, k, self.cur().kind))
        }
    }

    fn line(&self) -> usize {
        self.cur().line
    }

    // ── Block / terminator ──────────────────────────────────────

    fn at_block_end(&self) -> bool {
        self.is_eof()
            || self.is_phrase(&["Det", "er", "nok"])
            || self.is_phrase(&["Men", "om", "ikkje"])
            || self.is_phrase(&["Ver", "ikkje", "redd"])
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while !self.at_block_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // ── Statements ──────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let ln = self.line();

        // lat <name> vera <expr>
        if self.is_word("lat") {
            self.eat_word("lat")?;
            let name = self.eat_ident()?;
            self.eat_word("vera")?;
            let value = self.parse_expr()?;
            return Ok(Stmt::Assign { name, value, line: ln });
        }

        // Syng ut: <expr>
        if self.is_phrase(&["Syng", "ut"]) {
            self.eat_phrase(&["Syng", "ut"])?;
            self.eat_kind(&TokenKind::Colon)?;
            let value = self.parse_expr()?;
            return Ok(Stmt::Print { value, line: ln });
        }

        // Takk at du tok mine byrder: <expr>
        if self.is_phrase(&["Takk", "at", "du", "tok", "mine", "byrder"]) {
            self.eat_phrase(&["Takk", "at", "du", "tok", "mine", "byrder"])?;
            self.eat_kind(&TokenKind::Colon)?;
            let value = self.parse_expr()?;
            return Ok(Stmt::Return { value, line: ln });
        }

        // Kom med din <name>
        if self.is_phrase(&["Kom", "med", "din"]) {
            self.eat_phrase(&["Kom", "med", "din"])?;
            let name = self.eat_ident()?;
            return Ok(Stmt::Input { name, line: ln });
        }

        // Du kjem ikkje utanom <cond>: <then> [Men om ikkje: <else>] Det er nok.
        if self.is_phrase(&["Du", "kjem", "ikkje", "utanom"]) {
            self.eat_phrase(&["Du", "kjem", "ikkje", "utanom"])?;
            let cond = self.parse_expr()?;
            self.eat_kind(&TokenKind::Colon)?;
            let then_body = self.parse_block()?;
            let else_body = if self.is_phrase(&["Men", "om", "ikkje"]) {
                self.eat_phrase(&["Men", "om", "ikkje"])?;
                self.eat_kind(&TokenKind::Colon)?;
                Some(self.parse_block()?)
            } else {
                None
            };
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::If { cond, then_body, else_body, line: ln });
        }

        // Eit øyeblikk om gangen, medan <cond>: <body> Det er nok.
        if self.is_phrase(&["Eit", "øyeblikk", "om", "gangen"]) {
            self.eat_phrase(&["Eit", "øyeblikk", "om", "gangen"])?;
            self.eat_kind(&TokenKind::Comma)?;
            self.eat_word("medan")?;
            let cond = self.parse_expr()?;
            self.eat_kind(&TokenKind::Colon)?;
            let body = self.parse_block()?;
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::While { cond, body, line: ln });
        }

        // kvar <var> i <iterable>: <body> Det er nok.
        if self.is_word("kvar") {
            self.eat_word("kvar")?;
            let var = self.eat_ident()?;
            self.eat_word("i")?;
            let iterable = self.parse_expr()?;
            self.eat_kind(&TokenKind::Colon)?;
            let body = self.parse_block()?;
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::ForEach { var, iterable, body, line: ln });
        }

        // Evig i lysets rike: <body> Det er nok.
        if self.is_phrase(&["Evig", "i", "lysets", "rike"]) {
            self.eat_phrase(&["Evig", "i", "lysets", "rike"])?;
            self.eat_kind(&TokenKind::Colon)?;
            let body = self.parse_block()?;
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::InfLoop { body, line: ln });
        }

        // Topp attom toppar <n> [som <var>] gongar: <body> Det er nok.
        if self.is_phrase(&["Topp", "attom", "toppar"]) {
            self.eat_phrase(&["Topp", "attom", "toppar"])?;
            let count = self.parse_primary()?;
            let var = if self.is_word("som") {
                self.eat_word("som")?;
                self.eat_ident()?
            } else {
                "_".to_string()
            };
            self.eat_word("gongar")?;
            self.eat_kind(&TokenKind::Colon)?;
            let body = self.parse_block()?;
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::CountLoop { count, var, body, line: ln });
        }

        // Gud har ein plan med <name>(<params>): <body> Det er nok.
        if self.is_phrase(&["Gud", "har", "ein", "plan", "med"]) {
            self.eat_phrase(&["Gud", "har", "ein", "plan", "med"])?;
            let name = self.eat_ident()?;
            self.eat_kind(&TokenKind::LParen)?;
            let mut params = Vec::new();
            if !matches!(self.cur().kind, TokenKind::RParen) {
                params.push(self.eat_ident()?);
                while matches!(self.cur().kind, TokenKind::Comma) {
                    self.eat_kind(&TokenKind::Comma)?;
                    params.push(self.eat_ident()?);
                }
            }
            self.eat_kind(&TokenKind::RParen)?;
            self.eat_kind(&TokenKind::Colon)?;
            let body = self.parse_block()?;
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::FuncDef { name, params, body, line: ln });
        }

        // Bli med til <name> [med <args>]
        if self.is_phrase(&["Bli", "med", "til"]) {
            self.eat_phrase(&["Bli", "med", "til"])?;
            let name = self.eat_ident()?;
            let args = if self.is_word("med") {
                self.eat_word("med")?;
                self.parse_arg_list()?
            } else {
                Vec::new()
            };
            return Ok(Stmt::FuncCall { name, args, line: ln });
        }

        // Prøv å få gjort det du kan: <try> Ver ikkje redd: <catch> Det er nok.
        if self.is_phrase(&["Prøv", "å", "få", "gjort", "det", "du", "kan"]) {
            self.eat_phrase(&["Prøv", "å", "få", "gjort", "det", "du", "kan"])?;
            self.eat_kind(&TokenKind::Colon)?;
            let try_body = self.parse_block()?;
            self.eat_phrase(&["Ver", "ikkje", "redd"])?;
            self.eat_kind(&TokenKind::Colon)?;
            let catch_body = self.parse_block()?;
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::TryCatch { try_body, catch_body, line: ln });
        }

        // <name> tek imot <expr>
        if matches!(self.cur().kind, TokenKind::Word(_))
            && self.is_word_at(1, "tek")
            && self.is_word_at(2, "imot")
        {
            let name = self.eat_ident()?;
            self.eat_phrase(&["tek", "imot"])?;
            let value = self.parse_expr()?;
            return Ok(Stmt::Reassign { name, value, line: ln });
        }

        // stansar stilt.  — break out of loop (from «og skyttelen stansar stilt»)
        if self.is_phrase(&["stansar", "stilt"]) {
            self.eat_phrase(&["stansar", "stilt"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::Break { line: ln });
        }

        // atter ein gong.  — continue to next iteration (from «Atter ein gong ser eg»)
        if self.is_phrase(&["atter", "ein", "gong"]) {
            self.eat_phrase(&["atter", "ein", "gong"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::Continue { line: ln });
        }

        // Lytt ved port <n>: <body> Det er nok.
        if self.is_phrase(&["Lytt", "ved", "port"]) {
            self.eat_phrase(&["Lytt", "ved", "port"])?;
            let port = self.parse_primary()?;
            self.eat_kind(&TokenKind::Colon)?;
            let body = self.parse_block()?;
            self.eat_phrase(&["Det", "er", "nok"])?;
            self.eat_kind(&TokenKind::Dot)?;
            return Ok(Stmt::Serve { port, body, line: ln });
        }

        // Svar med: <expr>
        if self.is_phrase(&["Svar", "med"]) {
            self.eat_phrase(&["Svar", "med"])?;
            self.eat_kind(&TokenKind::Colon)?;
            let value = self.parse_expr()?;
            return Ok(Stmt::Respond { value, line: ln });
        }

        Err(format!("Line {}: unexpected token {:?}", ln, self.cur().kind))
    }

    // ── Expressions ─────────────────────────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left = self.parse_additive()?;

        if self.is_phrase(&["er", "mindre", "enn"]) {
            self.eat_phrase(&["er", "mindre", "enn"])?;
            return Ok(Expr::BinOp { op: BinOpKind::Lt, left: Box::new(left), right: Box::new(self.parse_additive()?) });
        }
        if self.is_phrase(&["er", "større", "enn"]) {
            self.eat_phrase(&["er", "større", "enn"])?;
            return Ok(Expr::BinOp { op: BinOpKind::Gt, left: Box::new(left), right: Box::new(self.parse_additive()?) });
        }
        if self.is_phrase(&["er", "ikkje"]) {
            self.eat_phrase(&["er", "ikkje"])?;
            return Ok(Expr::BinOp { op: BinOpKind::Ne, left: Box::new(left), right: Box::new(self.parse_additive()?) });
        }
        // Plain 'er' — but not 'er nok' or 'er dyr'
        if self.is_word("er")
            && !self.is_phrase(&["er", "nok"])
            && !self.is_phrase(&["er", "dyr"])
        {
            self.eat_word("er")?;
            return Ok(Expr::BinOp { op: BinOpKind::Eq, left: Box::new(left), right: Box::new(self.parse_additive()?) });
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            if self.is_word("og") && !self.is_phrase(&["og", "gongen"]) {
                self.eat_word("og")?;
                let right = self.parse_multiplicative()?;
                left = Expr::BinOp { op: BinOpKind::Add, left: Box::new(left), right: Box::new(right) };
            } else if self.is_word("utan") {
                self.eat_word("utan")?;
                let right = self.parse_multiplicative()?;
                left = Expr::BinOp { op: BinOpKind::Sub, left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        loop {
            if self.is_word("gongar") {
                self.eat_word("gongar")?;
                let right = self.parse_primary()?;
                left = Expr::BinOp { op: BinOpKind::Mul, left: Box::new(left), right: Box::new(right) };
            } else if self.is_phrase(&["delt", "på"]) {
                self.eat_phrase(&["delt", "på"])?;
                let right = self.parse_primary()?;
                left = Expr::BinOp { op: BinOpKind::Div, left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        // resten av <a> delt på <b>
        if self.is_phrase(&["resten", "av"]) {
            self.eat_phrase(&["resten", "av"])?;
            let a = self.parse_primary()?;
            self.eat_phrase(&["delt", "på"])?;
            let b = self.parse_primary()?;
            return Ok(Expr::BinOp { op: BinOpKind::Mod, left: Box::new(a), right: Box::new(b) });
        }

        // ikkje <expr>
        if self.is_word("ikkje")
            && !self.is_phrase(&["ikkje", "utanom"])
            && !self.is_phrase(&["ikkje", "redd"])
        {
            self.eat_word("ikkje")?;
            return Ok(Expr::Not(Box::new(self.parse_primary()?)));
        }

        // Bli med til <name> [med <args>]  — function call expression
        if self.is_phrase(&["Bli", "med", "til"]) {
            self.eat_phrase(&["Bli", "med", "til"])?;
            let name = self.eat_ident()?;
            let args = if self.is_word("med") {
                self.eat_word("med")?;
                self.parse_arg_list()?
            } else {
                Vec::new()
            };
            return self.maybe_index(Expr::Call { name, args });
        }

        // Literals
        match self.cur().kind.clone() {
            TokenKind::Int(n) => { self.pos += 1; return self.maybe_index(Expr::Int(n)); }
            TokenKind::Float(f) => { self.pos += 1; return self.maybe_index(Expr::Float(f)); }
            TokenKind::Str(s) => { self.pos += 1; return self.maybe_index(Expr::Str(s)); }
            _ => {}
        }

        if self.is_word("ja")  { self.pos += 1; return self.maybe_index(Expr::Bool(true)); }
        if self.is_word("nei") { self.pos += 1; return self.maybe_index(Expr::Bool(false)); }

        if self.is_phrase(&["tome", "hender"]) {
            self.eat_phrase(&["tome", "hender"])?;
            return self.maybe_index(Expr::Null);
        }

        // List [a, b, c]
        if matches!(self.cur().kind, TokenKind::LBracket) {
            self.eat_kind(&TokenKind::LBracket)?;
            let mut elements = Vec::new();
            if !matches!(self.cur().kind, TokenKind::RBracket) {
                elements.push(self.parse_expr()?);
                while matches!(self.cur().kind, TokenKind::Comma) {
                    self.eat_kind(&TokenKind::Comma)?;
                    elements.push(self.parse_expr()?);
                }
            }
            self.eat_kind(&TokenKind::RBracket)?;
            return self.maybe_index(Expr::List(elements));
        }

        // Parenthesised expression
        if matches!(self.cur().kind, TokenKind::LParen) {
            self.eat_kind(&TokenKind::LParen)?;
            let expr = self.parse_expr()?;
            self.eat_kind(&TokenKind::RParen)?;
            return self.maybe_index(expr);
        }

        // Two-word builtin call: «legg til(args)», «del frå(args)», etc.
        if let (TokenKind::Word(w1), TokenKind::Word(w2)) =
            (self.cur().kind.clone(), self.peek(1).kind.clone())
        {
            if matches!(self.peek(2).kind, TokenKind::LParen) {
                let combined = format!("{} {}", w1, w2);
                if is_two_word_builtin(&combined) {
                    self.pos += 2;
                    self.eat_kind(&TokenKind::LParen)?;
                    let mut args = Vec::new();
                    if !matches!(self.cur().kind, TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        while matches!(self.cur().kind, TokenKind::Comma) {
                            self.eat_kind(&TokenKind::Comma)?;
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.eat_kind(&TokenKind::RParen)?;
                    return self.maybe_index(Expr::Call { name: combined, args });
                }
            }
        }

        // Variable reference (or name(args) call)
        if let TokenKind::Word(name) = self.cur().kind.clone() {
            self.pos += 1;
            // name(args) call syntax
            if matches!(self.cur().kind, TokenKind::LParen) {
                self.eat_kind(&TokenKind::LParen)?;
                let mut args = Vec::new();
                if !matches!(self.cur().kind, TokenKind::RParen) {
                    args.push(self.parse_expr()?);
                    while matches!(self.cur().kind, TokenKind::Comma) {
                        self.eat_kind(&TokenKind::Comma)?;
                        args.push(self.parse_expr()?);
                    }
                }
                self.eat_kind(&TokenKind::RParen)?;
                return self.maybe_index(Expr::Call { name, args });
            }
            return self.maybe_index(Expr::Var(name));
        }

        Err(format!("Line {}: unexpected token in expression: {:?}", self.line(), self.cur().kind))
    }

    fn maybe_index(&mut self, mut node: Expr) -> Result<Expr, String> {
        while matches!(self.cur().kind, TokenKind::LBracket) {
            self.eat_kind(&TokenKind::LBracket)?;
            let idx = self.parse_expr()?;
            self.eat_kind(&TokenKind::RBracket)?;
            node = Expr::Index { obj: Box::new(node), idx: Box::new(idx) };
        }
        Ok(node)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = vec![self.parse_expr()?];
        while matches!(self.cur().kind, TokenKind::Comma) {
            self.eat_kind(&TokenKind::Comma)?;
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }
}

fn is_two_word_builtin(name: &str) -> bool {
    matches!(name, "legg til" | "del frå" | "del opp" | "sett saman" | "kvart tal")
}

// ─────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────

pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut p = Parser::new(tokens);
    let prog = p.parse_block()?;
    if !p.is_eof() {
        return Err(format!("Line {}: unexpected token {:?}", p.line(), p.cur().kind));
    }
    Ok(prog)
}
