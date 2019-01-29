use ast::AST;
use ast::Stat;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Var(String),
    Abs(String),
    Lpar,
    Rpar,
    Let(String),
}

fn consume_while<F>(it: &mut Peekable<Chars>, x: F) -> Vec<char>
where
    F: Fn(char) -> bool,
{
    let mut v: Vec<char> = vec![];

    while let Some(&ch) = it.peek() {
        if x(ch) {
            it.next().unwrap();
            v.push(ch);
        } else {
            break;
        }
    }
    v
}

pub type TokenErr = (String, usize);
pub fn error_message(input: &String, err: TokenErr) -> String {
    let underbar: String = String::from_utf8(vec![b'_'; input.len() - err.1]).unwrap();
    format!(
        "error:{}\n{}\n{}^ {}",
        input.len() - err.1,
        input,
        underbar,
        err.0
    )
}

pub fn tokenize(input: &String) -> Result<Vec<Token>, String> {
    let mut it = input.chars().peekable();
    let mut tokens: Vec<Token> = vec![];
    loop {
        match it.peek() {
            Some(&ch) => match ch {
                '\\' => {
                    it.next().unwrap();
                    loop {
                        let var: String = consume_while(&mut it, |a| a.is_alphanumeric())
                            .into_iter()
                            .collect();
                        if var.len() == 0 {
                            return Err(error_message(
                                &input,
                                (format!("variable must be at least 1 character"), it.count()),
                            ));
                        }
                        consume_while(&mut it, |a| a == ' ');
                        tokens.push(Token::Abs(var.clone()));
                        if let Some(&ch) = it.peek() {
                            if ch == '.' {
                                it.next().unwrap();
                                break;
                            } else if !ch.is_alphabetic() {
                                return Err(error_message(
                                    &input,
                                    (format!("expected '.',found '{}'", ch), it.count()),
                                ));
                            }
                        } else {
                            return Err(error_message(
                                &input,
                                (format!("expected '.',found EOF"), it.count()),
                            ));
                        }
                    }
                }
                ch if ch.is_alphabetic() => {
                    let var: String = consume_while(&mut it, |a| a.is_alphanumeric())
                        .into_iter()
                        .collect();
                    if var == "let" {
                        consume_while(&mut it, |a| a == ' ');
                        let var: String = consume_while(&mut it, |a| a.is_alphanumeric())
                            .into_iter()
                            .collect();
                        tokens.push(Token::Let(var));
                    } else {
                        tokens.push(Token::Var(var));
                    }
                }
                '(' => {
                    tokens.push(Token::Lpar);
                    it.next().unwrap();
                }
                ')' => {
                    tokens.push(Token::Rpar);
                    it.next().unwrap();
                }
                ' ' => {
                    it.next().unwrap();
                }
                _ => {
                    return Err(error_message(
                        &input,
                        (format!("invalid char:'{}'", ch), it.count()),
                    ));
                }
            },
            None => {
                break;
            }
        }
    }
    Ok(tokens)
}

pub struct Parser {
    cur: usize,
    len: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            cur: 0,
            len: 0,
            tokens: Vec::new(),
        }
    }
    pub fn parse(&mut self, input: &String) -> Option<Stat> {
        self.tokens = match tokenize(input) {
            Ok(tokens) => tokens,
            Err(e) => {
                println!("{}", e);
                return None;
            }
        };
        // println!("{:?}", self.tokens);
        self.cur = 0;
        self.len = self.tokens.len();
        self.read_statement()
    }

    fn read_statement(&mut self) -> Option<Stat> {
        let token = self.tokens[self.cur].clone();
        if let Token::Let(ref v) = token {
            self.cur = self.cur + 1;
            match self.read_expression() {
                Some(exp) => Some(Stat::Let(v.clone(), Box::new(exp))),
                None => None,
            }
        } else {
            match self.read_expression() {
                Some(exp) => Some(Stat::Exp(Box::new(exp))),
                None => None,
            }
        }
    }

    fn read_expression(&mut self) -> Option<AST> {
        let mut expleft: Option<AST> = None;
        while self.cur < self.len && self.tokens[self.cur] != Token::Rpar {
            let token = self.tokens[self.cur].clone();
            let expright = match token {
                Token::Abs(ref v) => {
                    self.cur = self.cur + 1;
                    let exp = match &self.read_expression() {
                        &Some(ref exp) => exp.clone(),
                        &None => {
                            return None;
                        }
                    };
                    AST::Abs(v.clone(), Box::new(exp))
                }
                _ => match self.read_term() {
                    Some(term) => term,
                    None => {
                        return None;
                    }
                },
            };
            if expleft != None {
                let el = expleft.unwrap().clone();
                expleft = Some(AST::App(Box::new(el), Box::new(expright)));
            } else {
                expleft = Some(expright);
            }
        }
        expleft
    }

    fn read_term(&mut self) -> Option<AST> {
        let token = self.tokens[self.cur].clone();
        match token {
            Token::Lpar => {
                self.cur = self.cur + 1;
                let exp = match &self.read_expression() {
                    &Some(ref exp) => exp.clone(),
                    &None => {
                        return None;
                    }
                };
                if self.cur >= self.len || self.tokens[self.cur] != Token::Rpar {
                    println!("unterminated (");
                    return None;
                }
                self.cur = self.cur + 1;
                Some(exp)
            }
            Token::Rpar => None,
            Token::Var(ref v) => {
                self.cur = self.cur + 1;
                Some(AST::Var(v.clone()))
            }
            _ => None,
        }
    }
}
