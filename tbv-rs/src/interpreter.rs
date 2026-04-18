/// Tree-walking interpreter for Trygve Bjerkreim.

use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use std::net::TcpListener;
use std::time::Duration;
use std::fmt;

use crate::parser::{BinOpKind, Expr, Program, Stmt};

// ─────────────────────────────────────────────────────────────────
// Value
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Func   { params: Vec<String>, body: Vec<Stmt> },
    Object { id: usize, class: String },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n)   => write!(f, "{}", n),
            Value::Float(v) => {
                if *v == v.floor() && v.abs() < 1e15 {
                    write!(f, "{}", *v as i64)
                } else {
                    write!(f, "{}", v)
                }
            }
            Value::Str(s)   => write!(f, "{}", s),
            Value::Bool(b)  => write!(f, "{}", if *b { "ja" } else { "nei" }),
            Value::Null     => write!(f, "tome hender"),
            Value::List(xs) => {
                write!(f, "[")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", x)?;
                }
                write!(f, "]")
            }
            Value::Func { .. }           => write!(f, "<funksjon>"),
            Value::Object { class, .. }  => write!(f, "<{}>", class),
        }
    }
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Bool(b)  => *b,
            Value::Int(n)   => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s)   => !s.is_empty(),
            Value::List(v)  => !v.is_empty(),
            Value::Null     => false,
            _               => true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Control-flow signals (returned up the call stack)
// ─────────────────────────────────────────────────────────────────

enum Signal {
    Return(Value),
    Break,
    Continue,
}

// ─────────────────────────────────────────────────────────────────
// Environment
//
// Simple scope stack: frames[0] = innermost.
// Globals (functions, builtins) live in Interpreter::globals and are
// checked as a fallback after the local chain.
// ─────────────────────────────────────────────────────────────────

struct Env {
    frames: Vec<HashMap<String, Value>>,
}

impl Env {
    fn new() -> Self {
        Env { frames: vec![HashMap::new()] }
    }

    fn push(&mut self) {
        self.frames.push(HashMap::new());
    }

    fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    fn get(&self, name: &str) -> Option<&Value> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn set(&mut self, name: &str, val: Value) {
        // Set in innermost frame
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), val);
        }
    }

    fn assign(&mut self, name: &str, val: Value) {
        // Update existing binding in closest scope; else create in innermost
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), val);
                return;
            }
        }
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), val);
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Object heap + class registry
// ─────────────────────────────────────────────────────────────────

struct HeapObj {
    class:  String,
    fields: HashMap<String, Value>,
}

struct StoredClass {
    field_defaults: Vec<(String, Expr)>,
    methods:        HashMap<String, (Vec<String>, Vec<Stmt>)>,
}

// ─────────────────────────────────────────────────────────────────
// Interpreter
// ─────────────────────────────────────────────────────────────────

pub struct Interpreter {
    globals:  HashMap<String, Value>,
    response: Option<String>,
    heap:     Vec<HeapObj>,
    classes:  HashMap<String, StoredClass>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals:  HashMap::new(),
            response: None,
            heap:     Vec::new(),
            classes:  HashMap::new(),
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        let mut env = Env::new();
        match self.exec_block(program, &mut env)? {
            None | Some(Signal::Return(_)) | Some(Signal::Break) | Some(Signal::Continue) => Ok(()),
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt], env: &mut Env) -> Result<Option<Signal>, String> {
        for stmt in stmts {
            if let Some(sig) = self.exec_stmt(stmt, env)? {
                return Ok(Some(sig));
            }
        }
        Ok(None)
    }

    fn exec_stmt(&mut self, stmt: &Stmt, env: &mut Env) -> Result<Option<Signal>, String> {
        match stmt {
            Stmt::Assign { name, value, .. } => {
                let v = self.eval(value, env)?;
                env.set(name, v);
                Ok(None)
            }

            Stmt::Reassign { name, value, .. } => {
                let v = self.eval(value, env)?;
                env.assign(name, v);
                Ok(None)
            }

            Stmt::Print { value, .. } => {
                let v = self.eval(value, env)?;
                println!("{}", v);
                Ok(None)
            }

            Stmt::Input { name, .. } => {
                io::stdout().flush().ok();
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line).ok();
                let s = line.trim_end_matches('\n').trim_end_matches('\r').to_string();
                env.set(name, Value::Str(s));
                Ok(None)
            }

            Stmt::If { cond, then_body, else_body, .. } => {
                let v = self.eval(cond, env)?;
                let branch = if v.truthy() {
                    Some(then_body.as_slice())
                } else {
                    else_body.as_deref()
                };
                if let Some(b) = branch {
                    env.push();
                    let sig = self.exec_block(b, env)?;
                    env.pop();
                    return Ok(sig);
                }
                Ok(None)
            }

            Stmt::While { cond, body, .. } => {
                loop {
                    let v = self.eval(cond, env)?;
                    if !v.truthy() { break; }
                    env.push();
                    let sig = self.exec_block(body, env)?;
                    env.pop();
                    match sig {
                        None | Some(Signal::Continue) => {}
                        Some(Signal::Break) => break,
                        other => return Ok(other),
                    }
                }
                Ok(None)
            }

            Stmt::ForEach { var, iterable, body, .. } => {
                let items = match self.eval(iterable, env)? {
                    Value::List(xs) => xs,
                    Value::Str(s)   => s.chars().map(|c| Value::Str(c.to_string())).collect(),
                    other           => return Err(format!("kan ikkje iterera over {}", other)),
                };
                'fe: for item in items {
                    env.push();
                    env.set(var, item);
                    let sig = self.exec_block(body, env)?;
                    env.pop();
                    match sig {
                        None | Some(Signal::Continue) => {}
                        Some(Signal::Break) => break 'fe,
                        other => return Ok(other),
                    }
                }
                Ok(None)
            }

            Stmt::InfLoop { body, .. } => {
                loop {
                    env.push();
                    let sig = self.exec_block(body, env)?;
                    env.pop();
                    match sig {
                        None | Some(Signal::Continue) => {}
                        Some(Signal::Break) => break,
                        other => return Ok(other),
                    }
                }
                Ok(None)
            }

            Stmt::CountLoop { count, var, body, .. } => {
                let n = match self.eval(count, env)? {
                    Value::Int(n)   => n,
                    Value::Float(f) => f as i64,
                    other           => return Err(format!("gongar krev heiltal, fekk {}", other)),
                };
                'cl: for i in 0..n {
                    env.push();
                    env.set(var, Value::Int(i));
                    let sig = self.exec_block(body, env)?;
                    env.pop();
                    match sig {
                        None | Some(Signal::Continue) => {}
                        Some(Signal::Break) => break 'cl,
                        other => return Ok(other),
                    }
                }
                Ok(None)
            }

            Stmt::FuncDef { name, params, body, .. } => {
                // Store in globals so recursive calls can find it
                self.globals.insert(
                    name.clone(),
                    Value::Func { params: params.clone(), body: body.clone() },
                );
                Ok(None)
            }

            Stmt::FuncCall { name, args, line } => {
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.eval(a, env)?);
                }
                self.call_func(name, vals, *line)?;
                Ok(None)
            }

            Stmt::Return { value, .. } => {
                let v = self.eval(value, env)?;
                Ok(Some(Signal::Return(v)))
            }

            Stmt::Break { .. } => Ok(Some(Signal::Break)),

            Stmt::Continue { .. } => Ok(Some(Signal::Continue)),

            Stmt::Raise { value, .. } => {
                let v = self.eval(value, env)?;
                Err(v.to_string())
            }

            Stmt::Assert { cond, .. } => {
                let v = self.eval(cond, env)?;
                if !v.truthy() {
                    Err(format!("Vakt broten: {} er ikkje sant", v))
                } else {
                    Ok(None)
                }
            }

            Stmt::Sleep { secs, .. } => {
                let s = match self.eval(secs, env)? {
                    Value::Int(n)   => n as f64,
                    Value::Float(f) => f,
                    other           => return Err(format!("kvil krev tal, fekk {}", other)),
                };
                std::thread::sleep(Duration::from_secs_f64(s.max(0.0)));
                Ok(None)
            }

            Stmt::Respond { value, .. } => {
                let v = self.eval(value, env)?;
                self.response = Some(v.to_string());
                Ok(None)
            }

            Stmt::Serve { port, body, .. } => {
                let p = match self.eval(port, env)? {
                    Value::Int(n)   => n as u16,
                    Value::Float(f) => f as u16,
                    other           => return Err(format!("port må vera heiltal, fekk {}", other)),
                };
                let listener = TcpListener::bind(("0.0.0.0", p))
                    .map_err(|e| format!("kan ikkje lytta på port {}: {}", p, e))?;
                eprintln!("Lyttar på port {} …", p);
                for incoming in listener.incoming() {
                    let mut stream = match incoming {
                        Ok(s)  => s,
                        Err(_) => continue,
                    };
                    let mut reader = match stream.try_clone() {
                        Ok(c)  => io::BufReader::new(c),
                        Err(_) => continue,
                    };
                    // Parse request line
                    let mut request_line = String::new();
                    reader.read_line(&mut request_line).ok();
                    let parts: Vec<&str> = request_line.split_whitespace().collect();
                    let metode = parts.first().copied().unwrap_or("GET").to_string();
                    let vegen  = parts.get(1).copied().unwrap_or("/").to_string();
                    // Read headers
                    let mut content_length = 0usize;
                    loop {
                        let mut hdr = String::new();
                        reader.read_line(&mut hdr).ok();
                        if hdr.trim().is_empty() { break; }
                        let lower = hdr.to_lowercase();
                        if let Some(rest) = lower.strip_prefix("content-length:") {
                            content_length = rest.trim().parse().unwrap_or(0);
                        }
                    }
                    // Read body
                    let mut body_bytes = vec![0u8; content_length];
                    reader.read_exact(&mut body_bytes).ok();
                    let kropp = String::from_utf8_lossy(&body_bytes).into_owned();
                    // Execute handler block
                    env.push();
                    env.set("metode", Value::Str(metode));
                    env.set("vegen",  Value::Str(vegen));
                    env.set("kropp",  Value::Str(kropp));
                    self.response = None;
                    let _ = self.exec_block(body, env);
                    env.pop();
                    // Send response
                    let resp = self.response.take().unwrap_or_default();
                    let http = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{}",
                        resp.len(), resp
                    );
                    stream.write_all(http.as_bytes()).ok();
                }
                Ok(None)
            }

            Stmt::ClassDef { name, body, .. } => {
                let mut field_defaults = Vec::new();
                let mut methods = HashMap::new();
                for stmt in body {
                    match stmt {
                        Stmt::Assign { name: f, value, .. } =>
                            field_defaults.push((f.clone(), value.clone())),
                        Stmt::FuncDef { name: m, params, body: mb, .. } =>
                            { methods.insert(m.clone(), (params.clone(), mb.clone())); }
                        _ => return Err("Klasse kan berre innehalda felt og metodar".into()),
                    }
                }
                self.classes.insert(name.clone(), StoredClass { field_defaults, methods });
                Ok(None)
            }

            Stmt::FieldAssign { obj, field, value, .. } => {
                let id = match env.get(obj).or_else(|| self.globals.get(obj)) {
                    Some(Value::Object { id, .. }) => *id,
                    _ => return Err(format!("'{}' er ikkje eit objekt", obj)),
                };
                let v = self.eval(value, env)?;
                self.heap[id].fields.insert(field.clone(), v);
                Ok(None)
            }

            Stmt::MethodCall { obj, method, args, line } => {
                let (id, class) = match env.get(obj).or_else(|| self.globals.get(obj)) {
                    Some(Value::Object { id, class }) => (*id, class.clone()),
                    _ => return Err(format!("'{}' er ikkje eit objekt", obj)),
                };
                let mut vals = Vec::new();
                for a in args { vals.push(self.eval(a, env)?); }
                self.call_method(id, &class, method, vals, *line)?;
                Ok(None)
            }

            Stmt::TryCatch { try_body, catch_body, .. } => {
                env.push();
                let res = self.exec_block(try_body, env);
                env.pop();
                match res {
                    Ok(sig) => Ok(sig),
                    Err(e) => {
                        env.push();
                        env.set("feilen", Value::Str(e));
                        let sig = self.exec_block(catch_body, env)?;
                        env.pop();
                        Ok(sig)
                    }
                }
            }
        }
    }

    // ── Expression evaluator ──────────────────────────────────

    fn eval(&mut self, expr: &Expr, env: &Env) -> Result<Value, String> {
        match expr {
            Expr::Int(n)   => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Str(s)   => Ok(Value::Str(s.clone())),
            Expr::Bool(b)  => Ok(Value::Bool(*b)),
            Expr::Null     => Ok(Value::Null),

            Expr::Var(name) => {
                env.get(name)
                    .cloned()
                    .or_else(|| self.globals.get(name).cloned())
                    .ok_or_else(|| format!("'{}' er ikkje definert", name))
            }

            Expr::List(elems) => {
                let mut vals = Vec::new();
                for e in elems {
                    vals.push(self.eval(e, env)?);
                }
                Ok(Value::List(vals))
            }

            Expr::Index { obj, idx } => {
                let obj_val = self.eval(obj, env)?;
                let idx_val = self.eval(idx, env)?;
                let i = match &idx_val {
                    Value::Int(n)   => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err(format!("listeindeks må vera heiltal, fekk {}", idx_val)),
                };
                match obj_val {
                    Value::List(xs) => {
                        let i = normalize_idx(i, xs.len())?;
                        Ok(xs[i].clone())
                    }
                    Value::Str(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        let i = normalize_idx(i, chars.len())?;
                        Ok(Value::Str(chars[i].to_string()))
                    }
                    _ => Err(format!("kan ikkje indeksera {}", obj_val)),
                }
            }

            Expr::New { class } => {
                let defaults: Vec<(String, Expr)> = self.classes.get(class)
                    .ok_or_else(|| format!("klasse '{}' er ikkje definert", class))?
                    .field_defaults.clone();
                let id = self.heap.len();
                self.heap.push(HeapObj { class: class.clone(), fields: HashMap::new() });
                let empty = Env::new();
                for (fname, default_expr) in defaults {
                    let v = self.eval(&default_expr, &empty)?;
                    self.heap[id].fields.insert(fname, v);
                }
                Ok(Value::Object { id, class: class.clone() })
            }

            Expr::Field { obj, field } => {
                let obj_val = self.eval(obj, env)?;
                match obj_val {
                    Value::Object { id, .. } => self.heap[id].fields.get(field)
                        .cloned()
                        .ok_or_else(|| format!("felt '{}' finst ikkje", field)),
                    other => Err(format!("kan ikkje lesa felt frå {}", other)),
                }
            }

            Expr::MethodCall { obj, method, args } => {
                let obj_val = self.eval(obj, env)?;
                let (id, class) = match obj_val {
                    Value::Object { id, class } => (id, class),
                    other => return Err(format!("kan ikkje kalla metode på {}", other)),
                };
                let mut vals = Vec::new();
                for a in args { vals.push(self.eval(a, env)?); }
                self.call_method(id, &class, method, vals, 0)
            }

            Expr::BinOp { op, left, right } => {
                let l = self.eval(left, env)?;
                let r = self.eval(right, env)?;
                apply_binop(op, l, r)
            }

            Expr::Not(inner) => {
                let v = self.eval(inner, env)?;
                Ok(Value::Bool(!v.truthy()))
            }

            Expr::Call { name, args } => {
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.eval(a, env)?);
                }
                self.call_func(name, vals, 0)
            }
        }
    }

    fn call_method(&mut self, id: usize, class: &str, method: &str, args: Vec<Value>, _line: usize) -> Result<Value, String> {
        let (params, body) = self.classes.get(class)
            .ok_or_else(|| format!("klasse '{}' finst ikkje", class))?
            .methods.get(method)
            .ok_or_else(|| format!("'{}' har ingen metode '{}'", class, method))?
            .clone();
        if args.len() != params.len() {
            return Err(format!("{}.{}: ventar {} argument, fekk {}", class, method, params.len(), args.len()));
        }
        let mut menv = Env::new();
        menv.set("sjølv", Value::Object { id, class: class.to_string() });
        for (p, a) in params.iter().zip(args) { menv.set(p, a); }
        match self.exec_block(&body, &mut menv)? {
            Some(Signal::Return(v)) => Ok(v),
            _                       => Ok(Value::Null),
        }
    }

    fn call_func(&mut self, name: &str, args: Vec<Value>, _line: usize) -> Result<Value, String> {
        // Built-ins take priority
        if let Some(result) = call_builtin(name, &args)? {
            return Ok(result);
        }

        // Look up in globals
        let func = self.globals.get(name).cloned()
            .ok_or_else(|| format!("'{}' er ikkje definert", name))?;

        match func {
            Value::Func { params, body } => {
                if args.len() != params.len() {
                    return Err(format!(
                        "{}: ventar {} argument, fekk {}",
                        name, params.len(), args.len()
                    ));
                }
                let mut func_env = Env::new();
                for (p, a) in params.iter().zip(args) {
                    func_env.set(p, a);
                }
                match self.exec_block(&body, &mut func_env)? {
                    None                       => Ok(Value::Null),
                    Some(Signal::Return(v))    => Ok(v),
                    Some(Signal::Break)        => Ok(Value::Null),
                    Some(Signal::Continue)     => Ok(Value::Null),
                }
            }
            _ => Err(format!("'{}' er ikkje ein funksjon", name)),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Built-in functions
// ─────────────────────────────────────────────────────────────────

fn call_builtin(name: &str, args: &[Value]) -> Result<Option<Value>, String> {
    match name {
        "lengd" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::List(xs) => Ok(Some(Value::Int(xs.len() as i64))),
                Value::Str(s)   => Ok(Some(Value::Int(s.chars().count() as i64))),
                other           => Err(format!("lengd: uforventa type {}", other)),
            }
        }
        "heiltal" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::Int(n)   => Ok(Some(Value::Int(*n))),
                Value::Float(f) => Ok(Some(Value::Int(*f as i64))),
                Value::Str(s)   => s.trim().parse::<i64>()
                    .map(|n| Some(Value::Int(n)))
                    .map_err(|_| format!("heiltal: kan ikkje konvertera «{}»", s)),
                Value::Bool(b)  => Ok(Some(Value::Int(if *b { 1 } else { 0 }))),
                other           => Err(format!("heiltal: uforventa type {}", other)),
            }
        }
        "desimaltal" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::Int(n)   => Ok(Some(Value::Float(*n as f64))),
                Value::Float(f) => Ok(Some(Value::Float(*f))),
                Value::Str(s)   => s.trim().parse::<f64>()
                    .map(|f| Some(Value::Float(f)))
                    .map_err(|_| format!("desimaltal: kan ikkje konvertera «{}»", s)),
                other           => Err(format!("desimaltal: uforventa type {}", other)),
            }
        }
        "tekst" => {
            expect_args(name, args, 1)?;
            Ok(Some(Value::Str(args[0].to_string())))
        }
        "legg til" => {
            expect_args(name, args, 2)?;
            match &args[0] {
                Value::List(xs) => {
                    let mut v = xs.clone();
                    v.push(args[1].clone());
                    Ok(Some(Value::List(v)))
                }
                other => Err(format!("legg_til: fyrste argument må vera liste, fekk {}", other)),
            }
        }
        "del frå" => {
            expect_args(name, args, 2)?;
            match (&args[0], &args[1]) {
                (Value::List(xs), Value::Int(i)) => {
                    let i = normalize_idx(*i, xs.len())?;
                    let mut v = xs.clone();
                    v.remove(i);
                    Ok(Some(Value::List(v)))
                }
                _ => Err("del_frå: forventar (liste, heiltal)".to_string()),
            }
        }
        "del opp" => {
            match args.len() {
                1 => match &args[0] {
                    Value::Str(s) => Ok(Some(Value::List(
                        s.chars().map(|c| Value::Str(c.to_string())).collect()
                    ))),
                    other => Err(format!("del_opp: forventar streng, fekk {}", other)),
                },
                2 => match (&args[0], &args[1]) {
                    (Value::Str(s), Value::Str(sep)) => Ok(Some(Value::List(
                        s.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect()
                    ))),
                    _ => Err("del_opp: forventar (streng, streng)".to_string()),
                },
                n => Err(format!("del_opp: forventar 1 eller 2 argument, fekk {}", n)),
            }
        }
        "sett saman" => {
            match args.len() {
                1 => match &args[0] {
                    Value::List(xs) => Ok(Some(Value::Str(
                        xs.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("")
                    ))),
                    other => Err(format!("sett_saman: forventar liste, fekk {}", other)),
                },
                2 => match (&args[0], &args[1]) {
                    (Value::List(xs), Value::Str(sep)) => Ok(Some(Value::Str(
                        xs.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(sep)
                    ))),
                    _ => Err("sett_saman: forventar (liste, streng)".to_string()),
                },
                n => Err(format!("sett_saman: forventar 1 eller 2 argument, fekk {}", n)),
            }
        }
        "sorter" => {
            expect_args(name, args, 1)?;
            match &args[0] {
                Value::List(xs) => {
                    let mut v = xs.clone();
                    v.sort_by(compare_values);
                    Ok(Some(Value::List(v)))
                }
                other => Err(format!("sorter: forventar liste, fekk {}", other)),
            }
        }
        "kvart tal" => {
            expect_args(name, args, 1)?;
            let n = match &args[0] {
                Value::Int(n)   => *n,
                Value::Float(f) => *f as i64,
                other           => return Err(format!("kvart_tal: forventar heiltal, fekk {}", other)),
            };
            Ok(Some(Value::List((0..n).map(Value::Int).collect())))
        }
        _ => Ok(None),
    }
}

fn expect_args(name: &str, args: &[Value], n: usize) -> Result<(), String> {
    if args.len() != n {
        Err(format!("{}: forventar {} argument, fekk {}", name, n, args.len()))
    } else {
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────
// Binary operations
// ─────────────────────────────────────────────────────────────────

fn apply_binop(op: &BinOpKind, l: Value, r: Value) -> Result<Value, String> {
    match op {
        BinOpKind::Add => match (&l, &r) {
            (Value::Str(_), _) | (_, Value::Str(_)) =>
                Ok(Value::Str(format!("{}{}", l, r))),
            (Value::Int(a), Value::Int(b))     => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b))   => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a + *b as f64)),
            (Value::List(a), Value::List(b))   => {
                let mut v = a.clone(); v.extend(b.iter().cloned()); Ok(Value::List(v))
            }
            _ => Err(format!("kan ikkje legga saman {} og {}", l, r)),
        },
        BinOpKind::Sub => num_op(l, r, |a, b| a - b, |a, b| a - b),
        BinOpKind::Mul => num_op(l, r, |a, b| a * b, |a, b| a * b),
        BinOpKind::Div => {
            match (&r,) {
                (Value::Int(0),)                     => return Err("deling på null".to_string()),
                (Value::Float(f),) if *f == 0.0      => return Err("deling på null".to_string()),
                _                                    => {}
            }
            match (&l, &r) {
                (Value::Int(a), Value::Int(b))     => Ok(Value::Int(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b))   => Ok(Value::Float(*a as f64 / b)),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a / *b as f64)),
                _ => Err(format!("kan ikkje dela {} på {}", l, r)),
            }
        }
        BinOpKind::Mod => {
            match (&r,) {
                (Value::Int(0),) => return Err("deling på null".to_string()),
                _                => {}
            }
            match (&l, &r) {
                (Value::Int(a), Value::Int(b))     => Ok(Value::Int(a % b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                (Value::Int(a), Value::Float(b))   => Ok(Value::Float(*a as f64 % b)),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a % *b as f64)),
                _ => Err(format!("kan ikkje ta rest av {} delt på {}", l, r)),
            }
        }
        BinOpKind::Eq => Ok(Value::Bool(values_equal(&l, &r))),
        BinOpKind::Ne => Ok(Value::Bool(!values_equal(&l, &r))),
        BinOpKind::Lt => {
            match compare_values(&l, &r) {
                std::cmp::Ordering::Less => Ok(Value::Bool(true)),
                _                        => Ok(Value::Bool(false)),
            }
        }
        BinOpKind::Gt => {
            match compare_values(&l, &r) {
                std::cmp::Ordering::Greater => Ok(Value::Bool(true)),
                _                           => Ok(Value::Bool(false)),
            }
        }
    }
}

fn num_op(l: Value, r: Value,
          int_op: fn(i64, i64) -> i64,
          flt_op: fn(f64, f64) -> f64)
    -> Result<Value, String>
{
    match (&l, &r) {
        (Value::Int(a), Value::Int(b))     => Ok(Value::Int(int_op(*a, *b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(flt_op(*a, *b))),
        (Value::Int(a), Value::Float(b))   => Ok(Value::Float(flt_op(*a as f64, *b))),
        (Value::Float(a), Value::Int(b))   => Ok(Value::Float(flt_op(*a, *b as f64))),
        _ => Err(format!("aritmetisk operasjon på {} og {}", l, r)),
    }
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(x), Value::Int(y))     => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(x), Value::Float(y))   => (*x as f64).partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(x), Value::Int(y))   => x.partial_cmp(&(*y as f64)).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Str(x), Value::Str(y))     => x.cmp(y),
        _                                  => std::cmp::Ordering::Equal,
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y))     => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y))   => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y))   => *x == (*y as f64),
        (Value::Str(x), Value::Str(y))     => x == y,
        (Value::Bool(x), Value::Bool(y))   => x == y,
        (Value::Null, Value::Null)         => true,
        _                                  => false,
    }
}

fn normalize_idx(i: i64, len: usize) -> Result<usize, String> {
    let n = len as i64;
    let idx = if i < 0 { n + i } else { i };
    if idx < 0 || idx >= n {
        Err(format!("indeks {} utanfor [0, {})", i, len))
    } else {
        Ok(idx as usize)
    }
}

impl Default for Interpreter {
    fn default() -> Self { Self::new() }
}
