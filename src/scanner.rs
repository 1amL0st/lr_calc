pub struct Scanner<'a> {
    expr: &'a str,
    iterator: std::iter::Peekable<std::str::CharIndices<'a>>,
    tokens: Vec<Token<'a>>,
    iter_index: usize,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TokenType<'a> {
    Number(f64),
    Str(&'a str),
    Plus,
    Minus,
    Multiplication,
    Division,
    Modulo,
    Power,
    Factorial,
    Comma,
    Lparen,
    Rparen,
    Equals,
    Bar,

    End,
    None,
}
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Token<'a> {
    pub t: TokenType<'a>,
    pub pos: usize,
}

impl<'a> Token<'a> {
    pub fn new(t: TokenType<'a>, pos: usize) -> Self {
        Self { t, pos }
    }
}

impl<'a> Scanner<'a> {
    pub fn new(expr: &'a String) -> Self {
        Self {
            expr: expr,
            iterator: expr.char_indices().peekable(),
            tokens: Vec::new(),
            iter_index: 0,
        }
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }

    pub fn next(&mut self) -> Token<'a> {
        if self.iter_index >= self.tokens.len() {
            Token::new(TokenType::End, 0)
        } else {
            self.iter_index += 1;
            self.tokens[self.iter_index - 1]
        }
    }

    pub fn peek(&self) -> Token<'a> {
        if self.iter_index >= self.tokens.len() {
            Token::new(TokenType::End, 0)
        } else {
            self.tokens[self.iter_index]
        }
    }

    pub fn scan(&mut self) {
        loop {
            let token = self.get_next_token();

            if token.t == TokenType::End {
                self.tokens.push(Token::new(TokenType::End, 0));
                break;
            } else if token.t != TokenType::None {
                self.tokens.push(token);
            }
        }
    }

    fn take_number(&mut self, index: usize) -> TokenType<'a> {
        let start = index;
        let mut end = index;
        loop {
            match self.iterator.peek() {
                Option::None => break,
                Option::Some(d) => {
                    if d.1.is_numeric() || d.1 == '.' || d.1 == 'E' {
                        end = d.0;
                    } else {
                        break;
                    }
                }
            };
            self.iterator.next();
        }

        let s = &self.expr[start..(end + 1)];
        match s.parse::<f64>() {
            Result::Ok(n) => return TokenType::Number(n),
            _ => panic!("Wrong format number!"),
        }
    }

    fn take_str(&mut self, index: usize) -> TokenType<'a> {
        let start = index;
        let mut end = index;
        loop {
            match self.iterator.peek() {
                Option::None => break,
                Option::Some(d) => {
                    if d.1.is_alphabetic() {
                        end = d.0;
                    } else {
                        break;
                    }
                }
            };
            self.iterator.next();
        }

        let s = &self.expr[start..(end + 1)];
        return TokenType::Str(s);
    }

    fn get_next_token(&mut self) -> Token<'a> {
        let oc = match self.iterator.next() {
            Option::None => return Token::new(TokenType::End, 0),
            Option::Some(c) => c,
        };

        let tokenType = match oc.1 {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '/' => TokenType::Division,
            '%' => TokenType::Modulo,
            '^' => TokenType::Power,
            '!' => TokenType::Factorial,
            ',' => TokenType::Comma,
            '(' => TokenType::Lparen,
            ')' => TokenType::Rparen,
            '[' => TokenType::Lparen,
            ']' => TokenType::Rparen,
            '=' => TokenType::Equals,
            '|' => TokenType::Bar,
            '*' => match self.iterator.peek() {
                Option::None => return Token::new(TokenType::Multiplication, oc.0),
                Option::Some(d) => match d.1 {
                    '*' => {
                        self.iterator.next();
                        TokenType::Power
                    }
                    _ => TokenType::Multiplication,
                },
            },
            _ => {
                if oc.1.is_numeric() || oc.1 == '.' {
                    self.take_number(oc.0)
                } else if oc.1.is_alphabetic() {
                    self.take_str(oc.0)
                } else {
                    TokenType::None
                }
            }
        };
        Token::new(tokenType, oc.0)
    }
}

#[cfg(test)]
mod scanner_tests {
    use super::*;

    fn do_test(expr: &str, expected: Vec<TokenType>) {
        let s = expr.to_string();
        let mut scanner = Scanner::new(&s);

        scanner.scan();

        let result = scanner.get_tokens();

        println!("result = {:?}", result);
        println!("expected = {:?}", expected);

        for i in 0..expected.len() {
            assert_eq!(result[i].t, expected[i]);
        }
    }

    #[test]
    fn very_primitive_tests() {
        do_test("", vec![TokenType::End]);

        do_test("+", vec![TokenType::Plus, TokenType::End]);
        do_test("-", vec![TokenType::Minus, TokenType::End]);
        do_test("/", vec![TokenType::Division, TokenType::End]);
        do_test("*", vec![TokenType::Multiplication, TokenType::End]);

        do_test("1", vec![TokenType::Number(1.), TokenType::End]);
        do_test("103", vec![TokenType::Number(103.), TokenType::End]);
        do_test(
            "103 + 1",
            vec![
                TokenType::Number(103.),
                TokenType::Plus,
                TokenType::Number(1.),
                TokenType::End,
            ],
        );
        do_test(
            "+103+1",
            vec![
                TokenType::Plus,
                TokenType::Number(103.),
                TokenType::Plus,
                TokenType::Number(1.),
                TokenType::End,
            ],
        );
    }

    #[test]
    fn easy_tests() {
        do_test(
            "321.23 * 10.23",
            vec![
                TokenType::Number(321.23),
                TokenType::Multiplication,
                TokenType::Number(10.23),
                TokenType::End,
            ],
        );
        do_test(
            "+2 - 3 / 6 * 7",
            vec![
                TokenType::Plus,
                TokenType::Number(2.),
                TokenType::Minus,
                TokenType::Number(3.),
                TokenType::Division,
                TokenType::Number(6.),
                TokenType::Multiplication,
                TokenType::Number(7.),
                TokenType::End,
            ],
        );
        do_test(
            "(123.20 + 1.21) * 40",
            vec![
                TokenType::Lparen,
                TokenType::Number(123.20),
                TokenType::Plus,
                TokenType::Number(1.21),
                TokenType::Rparen,
                TokenType::Multiplication,
                TokenType::Number(40.),
                TokenType::End,
            ],
        );
    }

    #[test]
    fn string_fetch_tests() {
        do_test(
            "max()",
            vec![
                TokenType::Str("max"),
                TokenType::Lparen,
                TokenType::Rparen,
                TokenType::End,
            ],
        );
        do_test(
            "max(1, 2)",
            vec![
                TokenType::Str("max"),
                TokenType::Lparen,
                TokenType::Number(1.),
                TokenType::Comma,
                TokenType::Number(2.),
                TokenType::Rparen,
                TokenType::End,
            ],
        );
        do_test(
            "max()",
            vec![
                TokenType::Str("max"),
                TokenType::Lparen,
                TokenType::Rparen,
                TokenType::End,
            ],
        );
        do_test(
            "max(min(3, 2), someFunc(4))",
            vec![
                TokenType::Str("max"),
                TokenType::Lparen,
                TokenType::Str("min"),
                TokenType::Lparen,
                TokenType::Number(3.),
                TokenType::Comma,
                TokenType::Number(2.),
                TokenType::Rparen,
                TokenType::Comma,
                TokenType::Str("someFunc"),
                TokenType::Lparen,
                TokenType::Number(4.),
                TokenType::Rparen,
                TokenType::Rparen,
                TokenType::End,
            ],
        );
    }

    #[test]
    #[should_panic]
    fn wrong_number_format_parsing() {
        do_test("123.45.3", vec![]);
    }
}
