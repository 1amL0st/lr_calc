use super::scanner::{Scanner, Token, TokenType as STokenType};

pub type NodePtr = Option<Box<Node>>;
type ErrMsg = String;

#[derive(PartialEq, Debug)]
pub enum TokenType {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Bar,
    Factorial,
    Modulo,

    PrefixMinus,
    PrefixPlus,
}
#[derive(PartialEq, Debug)]
pub struct Node {
    pub token: TokenType,
    pub left: NodePtr,
    pub right: NodePtr,
}

impl Node {
    pub fn new(token: TokenType, left: NodePtr, right: NodePtr) -> Self {
        Self { token, left, right }
    }

    pub fn new_ptr(token: TokenType, left: NodePtr, right: NodePtr) -> NodePtr {
        Some(Box::new(Node::new(token, left, right)))
    }
}

pub struct Ast<'a> {
    pub root: NodePtr,
    pub scanner: &'a mut Scanner<'a>,
    pub index: usize,
}

impl<'a> Ast<'a> {
    pub fn new(scanner: &'a mut Scanner<'a>) -> Self {
        Self {
            root: None,
            scanner: scanner,
            index: 0,
        }
    }

    fn print_node(&self, node: &NodePtr, offset: usize) {
        match node {
            Some(ref node) => {
                println!("{:w$}{{", "", w = offset);
                println!("{:w$}Token: {:?}", "", node.token, w = (offset + 1));

                if node.left.is_some() {
                    println!("{:w$}Left: ", "", w = (offset + 1));
                    self.print_node(&node.left, offset + 1);
                }

                if node.right.is_some() {
                    println!("{:w$}Right: ", "", w = (offset + 1));
                    self.print_node(&node.right, offset + 1);
                }

                println!("{:w$}}}", "", w = offset);
            }
            None => {
                println!("{:w$}{{ None }}", "", w = offset);
            }
        };
    }

    pub fn print(&self) {
        match self.root {
            Some(_) => self.print_node(&self.root, 0),
            None => println!("This AST is empty!"),
        }
    }

    fn infix_binding_power(token: STokenType) -> Option<(u32, u32)> {
        let o = match token {
            STokenType::Plus | STokenType::Minus => Some((1, 2)),
            STokenType::Multiplication | STokenType::Division | STokenType::Modulo => Some((3, 4)),
            _ => None,
        };
        o
    }

    fn prefix_binding_power(token: STokenType) -> Option<((), u32)> {
        match token {
            STokenType::Plus | STokenType::Minus => Some(((), 5)),
            _ => None
        }
    }

    fn postfix_binding_power(op: STokenType) -> Option<(u32, ())> {
        let res = match op {
            STokenType::Factorial => (7, ()),
            _ => return None,
        };
        Some(res)
    }

    fn is_operator(token: STokenType) -> bool {
        match token {
            STokenType::Minus
            | STokenType::Modulo
            | STokenType::Plus
            | STokenType::Multiplication
            | STokenType::Division
            | STokenType::Factorial => true,
            _ => false,
        }
    }

    fn scanner_token_to_ast_token(token: Token<'a>) -> TokenType {
        match token.t {
            STokenType::Number(number) => TokenType::Number(number),
            STokenType::Plus => TokenType::Plus,
            STokenType::Minus => TokenType::Minus,
            STokenType::Multiplication => TokenType::Multiply,
            STokenType::Division => TokenType::Divide,
            STokenType::Factorial => TokenType::Factorial,
            STokenType::Bar => TokenType::Bar,
            STokenType::Modulo => TokenType::Modulo,
            _ => panic!("CAN'T CONVERT THIS TOKEN!, {:?}", token),
        }
    }

    fn scanner_token_to_prefix_token(token: Token) -> TokenType {
        match token.t {
            STokenType::Plus => TokenType::PrefixPlus,
            STokenType::Minus => TokenType::PrefixMinus,
            _ => panic!("Unkown prefix token! {:?}", token),
        }
    }

    fn log_error(prev_token: Token<'a>, token: Token<'a>) -> Result<NodePtr, ErrMsg> {
        println!("prev_token = {:?} token = {:?}", prev_token, token);
        if Ast::is_operator(prev_token.t) && token.t == STokenType::End {
            Err(format!("Operator {:?} at pos {} expects an operand, but gets End!", prev_token.t, prev_token.pos))
        } else if prev_token.t == STokenType::None && token.t == STokenType::End {
            Err(String::from("Empty expression!"))
        } else {
            let msg = format!("Unkown error! Prev token {:?} at pos {}, last token {:?} at pos {}", prev_token.t, prev_token.pos, token.t, token.pos);
            Err(msg)
        }
    }

    fn parse_lhs(&mut self, prev_token: Token<'a>) -> Result<NodePtr, ErrMsg> {
        let token = self.scanner.next();
        match token.t {
            STokenType::Number(number) => Ok(Node::new_ptr(TokenType::Number(number), None, None)),
            STokenType::Lparen => {
                let lhs = self.parse_expr(0, token)?;

                let next = self.scanner.next();
                if next.t != STokenType::Rparen {
                    Err(format!("Expected RParen is not found! LParen pos = {}", token.pos))
                } else {
                    Ok(lhs)
                }
            }
            STokenType::Bar => {
                let lhs = Node::new_ptr(TokenType::Bar, self.parse_expr(0, token)?, None);

                let next = self.scanner.next();
                if next.t != STokenType::Bar {
                    Err(format!("Expected Bar is not found! First Bar  pos = {}", token.pos))
                } else {
                    Ok(lhs)
                }
            }
            STokenType::End => Ast::log_error(prev_token, token),
            _ => {
                if Ast::is_operator(token.t) {
                    if let Some(((), r_bp)) = Ast::prefix_binding_power(token.t) {
                        let rhs = self.parse_expr(r_bp, token)?;
                        Ok(Node::new_ptr(
                            Ast::scanner_token_to_prefix_token(token),
                            rhs,
                            None,
                        ))
                    } else {
                        Err(format!("Unknown prefix operator {:?} at pos {}!", token.t, token.pos))
                    }
                } else {
                    Err(format!("Unknown token! {:?}", token))
                }
            }
        }
    }

    fn parse_expr(&mut self, min_bp: u32, prev_token: Token<'a>) -> Result<NodePtr, ErrMsg> {
        let mut lhs = match self.parse_lhs(prev_token) {
            Ok(ptr) => ptr,
            Err(token) => return Err(token),
        };

        loop {
            let token = self.scanner.peek();
            let op = if Ast::is_operator(token.t)
                || token.t == STokenType::Rparen
                || token.t == STokenType::Bar
            {
                token
            } else if token.t == STokenType::End {
                break;
            } else {
                return Err(format!("Unkown token {:?} at pos {}!", token.t, token.pos))
            };

            if let Some((l_bp, ())) = Ast::postfix_binding_power(op.t) {
                if l_bp < min_bp {
                    break;
                }
                self.scanner.next();
                let token_type = Ast::scanner_token_to_ast_token(op);
                lhs = Node::new_ptr(token_type, lhs, None);
                continue;
            }

            if let Some((l_bp, r_bp)) = Ast::infix_binding_power(op.t) {
                if l_bp < min_bp {
                    break;
                }

                self.scanner.next();
                let token_type = Ast::scanner_token_to_ast_token(token);
                lhs = Node::new_ptr(token_type, lhs, self.parse_expr(r_bp, token)?);
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub fn build(&mut self) -> Result<(), ErrMsg> {
        match self.parse_expr(0, Token::new(STokenType::None, 0)) {
            Ok(ptr) => self.root = ptr,
            Err(err_msg) => return Err(err_msg),
        }
        Ok(())
    }
}

#[cfg(test)]
mod ast_tests {
    use super::*;

    fn build_tree_with_compare(expr: &str, expect: NodePtr) {
        println!("Expression = {}", expr);

        let s = expr.to_string();
        let mut scanner = Scanner::new(&s);
        scanner.scan();

        let mut ast = Ast::new(&mut scanner);
        ast.build();
        ast.print();

        assert_eq!(ast.root, expect);
    }

    #[test]
    fn tree_build_test_0() {
        let tree = Some(Box::new(Node {
            token: TokenType::Plus,
            left: Some(Box::new(Node {
                token: TokenType::Number(1.),
                left: None,
                right: None,
            })),
            right: Some(Box::new(Node {
                token: TokenType::Number(2.),
                left: None,
                right: None,
            })),
        }));

        build_tree_with_compare("1 + 2", tree);
    }

    #[test]
    fn tree_build_test_1() {
        let tree = Some(Box::new(Node {
            token: TokenType::Number(1.),
            left: None,
            right: None,
        }));

        build_tree_with_compare("1", tree);
    }

    #[test]
    fn tree_build_prefixes_test() {
        let tree = Some(Box::new(Node {
            token: TokenType::PrefixMinus,
            left: Some(Box::new(Node {
                token: TokenType::Number(1.),
                left: None,
                right: None,
            })),
            right: None,
        }));

        build_tree_with_compare("-1", tree);
    }

    #[test]
    fn tree_build_parenthesis_test() {
        let tree = Some(Box::new(Node {
            token: TokenType::Plus,
            left: Some(Box::new(Node {
                token: TokenType::Number(1.),
                left: None,
                right: None,
            })),
            right: Some(Box::new(Node {
                token: TokenType::Plus,
                left: Some(Box::new(Node {
                    token: TokenType::Number(2.),
                    left: None,
                    right: None,
                })),
                right: Some(Box::new(Node {
                    token: TokenType::Number(3.),
                    left: None,
                    right: None,
                })),
            })),
        }));

        build_tree_with_compare("1 + (2 + 3)", tree);
    }

    #[test]
    fn tree_build_postfixes_test() {
        let tree = Some(Box::new(Node {
            token: TokenType::Factorial,
            left: Some(Box::new(Node {
                token: TokenType::Number(3.),
                left: None,
                right: None,
            })),
            right: None,
        }));

        build_tree_with_compare("3!", tree);
    }

    #[test]
    fn tree_build_prefixes_test_1() {
        let tree = Some(Box::new(Node {
            token: TokenType::Plus,
            left: Some(Box::new(Node {
                token: TokenType::PrefixMinus,
                left: Some(Box::new(Node {
                    token: TokenType::Number(1.),
                    left: None,
                    right: None,
                })),
                right: None,
            })),
            right: Some(Box::new(Node {
                token: TokenType::Number(2.),
                left: None,
                right: None,
            })),
        }));

        build_tree_with_compare("-1 + 2", tree);
    }

    #[test]
    fn tree_build_test_2() {
        let tree = Some(Box::new(Node {
            token: TokenType::Plus,
            left: Some(Box::new(Node {
                token: TokenType::Number(1.0),
                left: None,
                right: None,
            })),
            right: Some(Box::new(Node {
                token: TokenType::Number(2.),
                left: None,
                right: None,
            })),
        }));

        build_tree_with_compare("1 + 2", tree);
    }

    #[test]
    fn tree_build_test_3() {
        let tree = Some(Box::new(Node {
            token: TokenType::Minus,
            left: Some(Box::new(Node {
                token: TokenType::Plus,
                left: Some(Box::new(Node {
                    token: TokenType::Number(1.),
                    left: None,
                    right: None,
                })),
                right: Some(Box::new(Node {
                    token: TokenType::Number(2.),
                    left: None,
                    right: None,
                })),
            })),
            right: Some(Box::new(Node {
                token: TokenType::Number(4.0),
                left: None,
                right: None,
            })),
        }));

        build_tree_with_compare("1 + 2 - 4", tree);
    }

    #[test]
    fn tree_build_test_4() {
        let tree = Some(Box::new(Node {
            token: TokenType::Plus,
            left: Some(Box::new(Node {
                token: TokenType::Multiply,
                left: Some(Box::new(Node {
                    token: TokenType::Number(2.0),
                    left: None,
                    right: None,
                })),
                right: Some(Box::new(Node {
                    token: TokenType::Number(3.0),
                    left: None,
                    right: None,
                })),
            })),
            right: Some(Box::new(Node {
                token: TokenType::Multiply,
                left: Some(Box::new(Node {
                    token: TokenType::Number(4.),
                    left: None,
                    right: None,
                })),
                right: Some(Box::new(Node {
                    token: TokenType::Number(5.),
                    left: None,
                    right: None,
                })),
            })),
        }));

        build_tree_with_compare("2 * 3 + 4 * 5", tree);
    }

    fn build_illegal_tree(expr: &str, expected_msg: &str) {
        println!("Expression = {}", expr);

        let s = expr.to_string();
        let mut scanner = Scanner::new(&s);
        scanner.scan();

        let mut ast = Ast::new(&mut scanner);
        if let Err(msg) = ast.build() {
            assert_eq!(expected_msg, msg);
        }
    }

    #[test]
    fn tree_build_error_msg_test() {
        build_illegal_tree("1 + ", "Operator Plus at pos 2 expects an operand, but gets End!");
        build_illegal_tree("1 + 2 - ", "Operator Minus at pos 6 expects an operand, but gets End!");

        build_illegal_tree("+", "Operator Plus at pos 0 expects an operand, but gets End!");
        build_illegal_tree("", "Empty expression!")
    }
}
