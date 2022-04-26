pub struct Token {
    pub ttype: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            ttype: TokenType::Undefined,
            lexeme: String::new(),
            line: 0,
        }
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Self {
            ttype: self.ttype,
            lexeme: self.lexeme.clone(),
            line: self.line,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    LeftParen = 0,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Assign, // ('=')
    Equals, // ('==')
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    Eof,
    Undefined,
    NumberOfTokens,
}
