use std::cell::RefCell;

use crate::chunk::*;
use crate::scanner::*;
use crate::token::*;
use crate::value::*;
use crate::*;

pub struct Compiler<'a> {
    parser: Parser,
    scanner: Scanner,
    chunk: &'a mut Chunk,
    rules: Vec<ParseRule<'a>>,
    locals: RefCell<Vec<Local>>,
    scope_depth: usize,
}

#[derive(Default)]
pub struct Parser {
    current: Token,
    previous: Token,
    had_error: RefCell<bool>,
    panic_mode: RefCell<bool>,
}

#[derive(Copy, Clone)]
struct ParseRule<'a> {
    prefix: Option<fn(&mut Compiler<'a>, bool)>,
    infix: Option<fn(&mut Compiler<'a>, bool)>,
    precedence: Precedence,
}

#[derive(PartialEq, PartialOrd, Copy, Clone)]
enum Precedence {
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

struct Local {
    name: Token,
    depth: Option<usize>,
}

impl From<usize> for Precedence {
    fn from(v: usize) -> Self {
        match v {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            v => panic!("cannot convert {v} into Precedence"),
        }
    }
}

impl Precedence {
    fn next(self) -> Self {
        if self == Precedence::Primary {
            panic!("no next after Primary");
        }
        let p = self as usize;
        (p + 1).into()
    }

    /*
    fn previous(self) -> Self {
        if self == Precedence::None {
            panic!("no previous before None");
        }
        let p = self as usize;
        (p - 1).into()
    }
    */
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        // lazy_static instead?
        let mut rules = vec![
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            };
            TokenType::NumberOfTokens as usize
        ];

        rules[TokenType::LeftParen as usize].prefix = Some(Compiler::grouping);

        rules[TokenType::Minus as usize] = ParseRule {
            prefix: Some(Compiler::unary),
            infix: Some(Compiler::binary),
            precedence: Precedence::Term,
        };
        rules[TokenType::Plus as usize] = ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Term,
        };
        rules[TokenType::Slash as usize] = ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Factor,
        };
        rules[TokenType::Star as usize] = ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Factor,
        };
        rules[TokenType::Number as usize].prefix = Some(Compiler::number);
        rules[TokenType::False as usize].prefix = Some(Compiler::literal);
        rules[TokenType::True as usize].prefix = Some(Compiler::literal);
        rules[TokenType::Nil as usize].prefix = Some(Compiler::literal);
        rules[TokenType::Bang as usize].prefix = Some(Compiler::unary);

        rules[TokenType::BangEqual as usize] = ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Equality,
        };
        rules[TokenType::Equals as usize] = rules[TokenType::BangEqual as usize];

        rules[TokenType::Greater as usize] = ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Comparison,
        };
        rules[TokenType::GreaterEqual as usize] = rules[TokenType::Greater as usize];
        rules[TokenType::Less as usize] = rules[TokenType::Greater as usize];
        rules[TokenType::LessEqual as usize] = rules[TokenType::Greater as usize];

        rules[TokenType::String as usize].prefix = Some(Compiler::string);
        rules[TokenType::Identifier as usize].prefix = Some(Compiler::variable);

        Self {
            parser: Parser::default(),
            scanner: Scanner::new(&"".to_string()),
            chunk,
            rules,
            locals: RefCell::new(Vec::new()),
            scope_depth: 0,
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<(), InterpretResult> {
        self.scanner = Scanner::new(source);
        self.advance();

        while !self.is_match(TokenType::Eof) {
            self.declaration();
        }

        self.end_compiler();

        if *self.parser.had_error.borrow() {
            Err(InterpretResult::CompileError)
        } else {
            Ok(())
        }
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.ttype != TokenType::Error {
                break;
            }

            let message = self.parser.current.lexeme.as_str();
            self.error_at_current(message);
        }
    }

    fn consume(&mut self, ttype: TokenType, message: &str) {
        if self.parser.current.ttype == ttype {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn check(&self, ttype: TokenType) -> bool {
        self.parser.current.ttype == ttype
    }

    fn is_match(&mut self, ttype: TokenType) -> bool {
        if self.check(ttype) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8) {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.into());
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        if let Some(constant) = self.chunk.add_constant(value) {
            constant
        } else {
            self.error("Too many constants in one chunk.");
            0
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant, constant);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature = "debug_print_code")]
        if !*self.parser.had_error.borrow() {
            self.chunk.disassemble("code");
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while self.locals.borrow().len() > 0
            && self.locals.borrow().last().unwrap().depth.unwrap() > self.scope_depth
        {
            self.emit_byte(OpCode::Pop.into());
            self.locals.borrow_mut().pop();
        }
    }

    fn binary(&mut self, _: bool) {
        let operator_type = self.parser.previous.ttype;
        let rule = self.rules[operator_type as usize].precedence.next();

        self.parse_precedence(rule);

        match operator_type {
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not.into()),
            TokenType::Equals => self.emit_byte(OpCode::Equal.into()),
            TokenType::Greater => self.emit_byte(OpCode::Greater.into()),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not.into()),
            TokenType::Less => self.emit_byte(OpCode::Less.into()),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not.into()),
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            _ => todo!(),
        }
    }

    fn literal(&mut self, _: bool) {
        match self.parser.previous.ttype {
            TokenType::False => self.emit_byte(OpCode::False.into()),
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            TokenType::True => self.emit_byte(OpCode::True.into()),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self, _: bool) {
        let value = self.parser.previous.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn string(&mut self, _: bool) {
        let len = self.parser.previous.lexeme.len() - 1;
        let string = self.parser.previous.lexeme[1..len].to_string();
        self.emit_constant(Value::Str(string));
    }

    fn resolve_local(&self, name: &Token) -> Option<u8> {
        for (e, v) in self.locals.borrow().iter().rev().enumerate() {
            if v.name.lexeme == name.lexeme {
                if v.depth.is_none() {
                    self.error("Can't read local variable in its own initializer.");
                }
                return Some((self.locals.borrow().len() - e - 1) as u8);
            }
        }
        None
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let (arg, get_op, set_op) = if let Some(local_arg) = self.resolve_local(name) {
            (local_arg, OpCode::GetLocal, OpCode::SetLocal)
        } else {
            (
                self.identifier_constant(name),
                OpCode::GetGlobal,
                OpCode::SetGlobal,
            )
        };

        if can_assign && self.is_match(TokenType::Assign) {
            self.expression();
            self.emit_bytes(set_op, arg);
        } else {
            self.emit_bytes(get_op, arg);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        let name = self.parser.previous.clone();
        self.named_variable(&name, can_assign);
    }

    fn unary(&mut self, _: bool) {
        let operator_type = self.parser.previous.ttype;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            TokenType::Bang => self.emit_byte(OpCode::Not.into()),
            _ => unimplemented!("nope"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(prefix_rule) = self.rules[self.parser.previous.ttype as usize].prefix {
            let can_assign = precedence <= Precedence::Assignment;
            prefix_rule(self, can_assign);

            while precedence <= self.rules[self.parser.current.ttype as usize].precedence {
                self.advance();
                if let Some(infix_rule) = self.rules[self.parser.previous.ttype as usize].infix {
                    infix_rule(self, can_assign);
                }

                if can_assign && self.is_match(TokenType::Assign) {
                    self.error("Invalid assignment target.");
                }
            }
        } else {
            self.error("Expect expression.");
        }
    }

    fn identifier_constant(&mut self, name: &Token) -> u8 {
        self.make_constant(Value::Str(name.lexeme.clone()))
    }

    fn add_local(&self, name: &Token) {
        if self.locals.borrow().len() >= 256 {
            self.error("Too many local variables in function.");
            return;
        }

        let loc = Local {
            name: name.clone(),
            depth: None,
        };
        self.locals.borrow_mut().push(loc);
    }

    fn declare_variable(&mut self) {
        if self.scope_depth != 0 {
            let name = self.parser.previous.lexeme.clone();
            if self
                .locals
                .borrow()
                .iter()
                .filter(|x| x.name.lexeme == name)
                .count()
                != 0
            {
                self.error("Already a variable with this name in this scope.");
            } else {
                self.add_local(&self.parser.previous);
            }
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();

        if self.scope_depth == 0 {
            let name = self.parser.previous.clone();
            self.identifier_constant(&name)
        } else {
            0
        }
    }

    fn mark_initialized(&mut self) {
        let last = self.locals.borrow().len() - 1;
        let mut locals = self.locals.borrow_mut();
        locals[last].depth = Some(self.scope_depth);
    }

    fn define_variable(&mut self, global: u8) {
        if self.scope_depth == 0 {
            self.emit_bytes(OpCode::DefineGlobal, global);
        } else {
            self.mark_initialized();
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.is_match(TokenType::Assign) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }

        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode.replace(false);

        while self.parser.current.ttype != TokenType::Eof {
            if self.parser.previous.ttype == TokenType::SemiColon {
                return;
            }
            if matches!(
                self.parser.current.ttype,
                TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return
            ) {
                return;
            }
            self.advance();
        }
    }

    fn declaration(&mut self) {
        if self.is_match(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement();
        }

        if *self.parser.panic_mode.borrow() {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.is_match(TokenType::Print) {
            self.print_statement();
        } else if self.is_match(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn error_at_current(&self, message: &str) {
        self.error_at(&self.parser.current, message);
    }

    fn error(&self, message: &str) {
        self.error_at(&self.parser.previous, message);
    }

    fn error_at(&self, token: &Token, message: &str) {
        if *self.parser.panic_mode.borrow() {
            return;
        }

        self.parser.panic_mode.replace(true);

        eprint!("[line {}] Error", token.line);

        if token.ttype == TokenType::Eof {
            eprint!(" at end");
        } else if token.ttype == TokenType::Error {
            // ignore
        } else {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {message}");
        self.parser.had_error.replace(true);
    }
}
