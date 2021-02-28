use crate::ast::NodePtr;

use super::ast::{Ast, TokenType};
use super::scanner::Scanner;

fn factorial(n: f64) -> f64 {
    let mut f = 1.;
    for i in 2..=(n as u64) {
        f *= i as f64;
    }
    f
}

fn recursion<'a>(node: NodePtr) -> f64 {
    match node {
        Some(ptr) => match ptr.token {
            TokenType::Modulo => {
                let left = recursion(ptr.left);
                let right = recursion(ptr.right);
                (left % right) as f64
            },
            TokenType::PrefixMinus => -recursion(ptr.left),
            TokenType::PrefixPlus => recursion(ptr.left),
            TokenType::Plus => recursion(ptr.left) + recursion(ptr.right),
            TokenType::Minus => recursion(ptr.left) - recursion(ptr.right),
            TokenType::Factorial => factorial(recursion(ptr.left)),
            TokenType::Bar => recursion(ptr.left).abs(),
            TokenType::Multiply => recursion(ptr.left) * recursion(ptr.right),
            TokenType::Divide => recursion(ptr.left) / recursion(ptr.right),
            TokenType::Number(n) => n,
            _ => panic!("Unknown token!"),
        },
        None => 0.,
    }
}

pub fn evaluate(expr: &String) -> Result<f64, String> {
    let mut scanner = Scanner::new(expr);
    scanner.scan();

    let mut ast = Ast::new(&mut scanner);
    if let Err(err_msg) = ast.build() {
        Err(format!("Ast build error! {}", err_msg))
    } else {
        Ok(recursion(ast.root))
    }
}

#[cfg(test)]
mod evaluator_tests {
    use super::*;

    fn do_test(expr: &str, expect: f64) {
        println!("Expression = {}", expr);

        let mut expr = expr.to_string();
        let mut result = evaluate(&expr).unwrap();

        println!("Expected result = {}", expect);
        assert_eq!(result, expect);
    }

    #[test]
    fn very_primitive_tests() {
        do_test("1 + 2", 3.);
        do_test("2 / 2 * 3 + 4 * 5", 23.);
        do_test("2 + 6 / 2 * 3 + 4 * 5", 31.);
    }

    #[test]
    fn parenthesis_tests() {
        do_test("(1 + 2) * 3", 9.);
        do_test("(1 + 2!) / 3", 1.);

        do_test("(1 * 2) * (5 + 1)", 12.);
        do_test("((2 + 3) * 2) * (5 + 1)", 60.);

        do_test("-(1 + 3)", -4.);
        do_test("(1 + 2) * 3", 9.);

        do_test("(2 + 1)!", 6.);
        do_test("(1 + 3)! * 2", 48.);
        do_test("((3 - 2) * 2)! * 1.0", 2.);
    }

    #[test]
    fn postfix_operatos_tests() {
        do_test("3!", 6.);
        do_test("-3!", -6.);
        do_test("-3! / 3", -2.);

        do_test("-3 * 1 * -1", 3.);
        do_test("3! * 3", 18.);
        do_test("2 + 3! * 3", 20.);

        do_test("10 - 2! + 3!", 14.);

        do_test("2 + 3! * 3", 20.);
        do_test("3! * 3!", 36.);
    }

    #[test]
    fn prefixes_tests() {
        do_test("-1", -1.);
        do_test("2 + -1", 1.);
        do_test("2 + -1 / 2.", 1.5);

        do_test("+2 + +3", 5.);

        do_test("-1 * 8", -8.);
        do_test("-3 + 1 * 3 / 3", -2.);
    }

    #[test]
    fn abs_value_operator() {
        do_test("|-3|", 3.);
        do_test("|-3 * 1 * -1|", 3.);

        do_test("|-2| + 2", 4.);
        do_test("2 / (|-2| + 2)", 0.5);

        do_test("|(4 - 6)| * 2", 4.);
    }

    #[test]
    fn modulo_operator_tests() {
        do_test("4 % 2", 0.);

        do_test("4 % 3", 1.);

        do_test("(4 + 2) % 3", 0.);
        do_test("(5 + 2) % 9", 7.);
    }
}
