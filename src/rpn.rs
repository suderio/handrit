use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use bigdecimal::ToPrimitive;
use std::collections::HashMap;
use std::convert::From;
use std::ops::Neg;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(BigDecimal),
    String(String),
    Variable(String),
    Operator(String, OperatorType),
    List(Vec<Token>),
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorType {
    Prefix,
    Infix,
    Postfix,
}

#[derive(Debug, PartialEq)]
pub enum Assoc {
    Left,
    Right,
}

#[derive(Debug)]
pub struct Operator {
    pub symbol: String,
    pub precedence: usize,
    pub assoc: Assoc,
    pub op_type: OperatorType,
    pub func: fn(&mut Vec<Token>, &mut HashMap<String, Token>),
}

trait NumericValue {
    fn n_value(self) -> BigDecimal;
}

impl NumericValue for BigDecimal {
    fn n_value(self) -> BigDecimal {
        self
    }
}

impl NumericValue for String {
    fn n_value(self) -> BigDecimal {
        BigDecimal::from_usize(self.len()).expect("TODO!")
    }
}

impl NumericValue for Vec<Token> {
    fn n_value(self) -> BigDecimal {
        BigDecimal::from_usize(self.len()).expect("TODO!")
    }
}

fn op_infix_number<F>(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>, op: F)
where
    F: Fn(BigDecimal, BigDecimal) -> BigDecimal,
{
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    match (left, right) {
        (Token::Number(left_num), Token::Number(right_num)) => {
            stack.push(Token::Number(op(left_num, right_num)));
        }
        (Token::Number(left_num), Token::String(right_str)) => {
            let result = op(left_num, right_str.n_value());
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::String(left_str), Token::Number(right_num)) => {
            let result = op(left_str.n_value(), right_num);
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::String(left_str), Token::String(right_str)) => {
            let result = op(left_str.n_value(), right_str.n_value());
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::List(left_list), Token::Number(right_num)) => {
            let result = op(left_list.n_value(), right_num);
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::List(left_list), Token::String(right_str)) => {
            let result = op(left_list.n_value(), right_str.n_value());
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::Number(left_num), Token::List(right_list)) => {
            let result = op(left_num, right_list.n_value());
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::String(left_str), Token::List(right_list)) => {
            let result = op(left_str.n_value(), right_list.n_value());
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::List(left_list), Token::List(right_list)) => {
            let result = op(left_list.n_value(), right_list.n_value());
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::Variable(left_var), right) => {
            let left_value = context.get(&left_var).clone();
            // TODO maybe we can avoid undefined variable error here by pushing the variable again?
            stack.push(left_value.expect("No lvalue").clone());
            stack.push(right);
            op_infix_number(stack, context, op);
        }
        (left, Token::Variable(right_var)) => {
            let right_value = context.get(&right_var).clone();
            // TODO maybe we can avoid undefined variable error here by pushing the variable again?
            stack.push(left);
            stack.push(right_value.expect("No rvalue").clone());
            op_infix_number(stack, context, op);
        }
        _ => {
            panic!("Add operation requires either numbers or strings");
        }
    }
}

fn op_infix_bool<F>(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>, op: F)
where
    F: Fn(bool, bool) -> bool,
{
}

fn op_null(_stack: &mut Vec<Token>, _context: &mut HashMap<String, Token>) {
    println!("op_null")
}

fn op_pow(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>) {
    op_infix_number(stack, context, |left, right| {
        let mut result = BigDecimal::from_i32(1).expect("TODO!");
        for _ in 0..(right.to_u128().expect("TODO!")) {
            result *= &left;
        }
        result
    });
}

fn op_minus(stack: &mut Vec<Token>, _context: &mut HashMap<String, Token>) {
    let right = stack.pop().unwrap();
    match right {
        Token::Number(right_num) => stack.push(Token::Number(-right_num)),
        Token::String(right_str) => stack.push(Token::Number(
            BigDecimal::from_usize(right_str.len()).unwrap().neg(),
        )),
        Token::List(right_list) => stack.push(Token::Number(
            BigDecimal::from_usize(right_list.len()).unwrap().neg(),
        )),
        _ => todo!(),
    }
}

fn op_plus(stack: &mut Vec<Token>, _context: &mut HashMap<String, Token>) {
    let right = stack.pop().unwrap();
    match right {
        Token::Number(right_num) => stack.push(Token::Number(right_num)),
        Token::String(right_str) => stack.push(Token::Number(
            BigDecimal::from_usize(right_str.len()).unwrap(),
        )),
        Token::List(right_list) => stack.push(Token::Number(
            BigDecimal::from_usize(right_list.len()).unwrap(),
        )),
        _ => todo!(),
    }
}
fn op_cons(stack: &mut Vec<Token>, _context: &mut HashMap<String, Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    match left {
        Token::List(mut left_list) => {
            left_list.push(right);
            stack.push(Token::List(left_list));
        }
        _ => stack.push(Token::List(vec![left, right])),
    }
}

fn op_assign(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    match (left, right) {
        (Token::Variable(left_var), right) => {
            context.insert(left_var, right.clone());
            stack.push(right);
        }
        _ => panic!("Tried assignment operation without a variable on the left side."),
    }
    for (key, value) in context {
        println!("{key}: {:?}", value);
    }
}

pub fn get_standard_operators() -> Vec<Operator> {
    vec![
        Operator {
            symbol: ":".to_string(),
            precedence: 1,
            assoc: Assoc::Right,
            op_type: OperatorType::Infix,
            func: op_assign,
        },
        Operator {
            symbol: "||".to_string(),
            precedence: 3,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_bool(stack, context, |left, right| left || right),
        },
        Operator {
            symbol: "&&".to_string(),
            precedence: 4,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "|".to_string(),
            precedence: 5,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "^".to_string(),
            precedence: 6,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "&".to_string(),
            precedence: 7,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "=".to_string(),
            precedence: 8,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "<>".to_string(),
            precedence: 8,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: ">".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "<".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: ">=".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "<=".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "+".to_string(),
            precedence: 11,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_number(stack, context, |left, right| left + right),
        },
        Operator {
            symbol: "-".to_string(),
            precedence: 11,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_number(stack, context, |left, right| left - right),
        },
        Operator {
            symbol: "$".to_string(),
            precedence: 11,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_null,
        },
        Operator {
            symbol: "*".to_string(),
            precedence: 12,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_number(stack, context, |left, right| left * right),
        },
        Operator {
            symbol: "/".to_string(),
            precedence: 12,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_number(stack, context, |left, right| left / right),
        },
        Operator {
            symbol: "%".to_string(),
            precedence: 12,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_number(stack, context, |left, right| left % right),
        },
        Operator {
            symbol: "**".to_string(),
            precedence: 13,
            assoc: Assoc::Right,
            op_type: OperatorType::Infix,
            func: op_pow,
        },
        Operator {
            symbol: "-".to_string(),
            precedence: 14,
            assoc: Assoc::Right,
            op_type: OperatorType::Prefix,
            func: op_minus,
        },
        Operator {
            symbol: "+".to_string(),
            precedence: 14,
            assoc: Assoc::Right,
            op_type: OperatorType::Prefix,
            func: op_plus,
        },
        Operator {
            symbol: "~".to_string(),
            precedence: 14,
            assoc: Assoc::Right,
            op_type: OperatorType::Prefix,
            func: op_null,
        },
        Operator {
            symbol: "?".to_string(),
            precedence: 16,
            assoc: Assoc::Left,
            op_type: OperatorType::Postfix,
            func: op_null,
        },
        Operator {
            symbol: ",".to_string(),
            precedence: 16,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: op_cons,
        },
        // Add more standard operators here
    ]
}

fn get_operator<'a>(symbol: &str, operators: &'a [Operator]) -> Option<&'a Operator> {
    operators.iter().find(|op| op.symbol == symbol)
}

fn get_infix_operator<'a>(symbol: &str, operators: &'a [Operator]) -> Option<&'a Operator> {
    operators
        .iter()
        .find(|op| op.symbol == symbol && op.op_type == OperatorType::Infix)
}

fn get_prefix_operator<'a>(symbol: &str, operators: &'a [Operator]) -> Option<&'a Operator> {
    operators
        .iter()
        .find(|op| op.symbol == symbol && op.op_type == OperatorType::Prefix)
}

fn tokenize(expression: &str, operators: &[Operator]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = expression.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&digit) = chars.peek() {
                    if digit.is_numeric() || digit == '.' {
                        num_str.push(digit);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(BigDecimal::from_str(&num_str).unwrap()));
            }
            '"' => {
                chars.next(); // skip the opening quote
                let mut str_val = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next(); // skip the closing quote
                        break;
                    }
                    str_val.push(ch);
                    chars.next();
                }
                tokens.push(Token::String(str_val));
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Variable(ident));
            }
            _ => {
                let mut op = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric()
                        || next_ch == ' '
                        || next_ch == '('
                        || next_ch == ')'
                    {
                        break;
                    }
                    op.push(next_ch);
                    chars.next();
                }
                match tokens.last() {
                    Some(Token::Operator(_, _)) | Some(Token::LeftParen) | None => {
                        if let Some(operator) = get_prefix_operator(&op, operators) {
                            tokens.push(Token::Operator(op, operator.op_type.clone()));
                        } else {
                            chars.next(); // skip any unrecognized character
                        }
                    }

                    _ => {
                        if let Some(operator) = get_infix_operator(&op, operators) {
                            tokens.push(Token::Operator(op, operator.op_type.clone()));
                        } else {
                            chars.next(); // skip any unrecognized character
                        }
                    }
                }
            }
        }
    }
    tokens
}

fn shunting_yard(tokens: Vec<Token>, operators: &[Operator]) -> Vec<Token> {
    let mut output = Vec::new();
    let mut op_stack = Vec::new();

    for token in tokens {
        match token.clone() {
            Token::Number(_) | Token::String(_) | Token::Variable(_) | Token::List(_) => {
                output.push(token)
            }
            Token::Operator(op, op_type) => match op_type {
                OperatorType::Prefix => op_stack.push(token),
                OperatorType::Postfix => {
                    output.push(token);
                }
                OperatorType::Infix => {
                    while let Some(top_op) = op_stack.last() {
                        if let Token::Operator(top_op_str, _) = top_op {
                            if let Some(top_operator) = get_operator(top_op_str, operators) {
                                if (top_operator.assoc == Assoc::Left
                                    && top_operator.precedence >= precedence(&op, operators))
                                    || (top_operator.assoc == Assoc::Right
                                        && top_operator.precedence > precedence(&op, operators))
                                {
                                    output.push(op_stack.pop().unwrap());
                                } else {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    op_stack.push(token);
                }
            },
            Token::LeftParen => op_stack.push(token),
            Token::RightParen => {
                while let Some(top_op) = op_stack.pop() {
                    if top_op == Token::LeftParen {
                        break;
                    }
                    output.push(top_op);
                }
            }
        }
    }

    while let Some(op) = op_stack.pop() {
        output.push(op);
    }

    output
}

fn precedence(op: &str, operators: &[Operator]) -> usize {
    if let Some(operator) = get_operator(op, operators) {
        operator.precedence
    } else {
        0
    }
}

pub fn convert_to_rpn(expression: &str, operators: &[Operator]) -> String {
    let tokens = tokenize(expression, operators);
    let rpn_tokens = shunting_yard(tokens, operators);

    let mut result = String::new();
    for token in rpn_tokens {
        match token {
            Token::Number(num) => result.push_str(&format!("{} ", num)),
            Token::String(s) => result.push_str(&format!("\"{}\" ", s)),
            Token::Variable(var) => result.push_str(&format!("{} ", var)),
            Token::Operator(op, _) => result.push_str(&format!("{} ", op)),
            _ => {}
        }
    }

    result.trim().to_string()
}

pub fn evaluate_rpn(tokens: Vec<Token>, operators: &[Operator]) -> Result<Token, String> {
    let mut stack = Vec::new();
    let mut prefix_map: HashMap<String, fn(&mut Vec<Token>, &mut HashMap<String, Token>)> =
        HashMap::new();
    let mut infix_map: HashMap<String, fn(&mut Vec<Token>, &mut HashMap<String, Token>)> =
        HashMap::new();
    let mut postfix_map: HashMap<String, fn(&mut Vec<Token>, &mut HashMap<String, Token>)> =
        HashMap::new();
    let mut context: HashMap<String, Token> = HashMap::new();

    for op in operators {
        match op.op_type {
            OperatorType::Prefix => {
                prefix_map.insert(op.symbol.clone(), op.func);
            }
            OperatorType::Infix => {
                infix_map.insert(op.symbol.clone(), op.func);
            }
            OperatorType::Postfix => {
                postfix_map.insert(op.symbol.clone(), op.func);
            }
        }
    }

    for token in tokens {
        match token {
            Token::Number(_) | Token::String(_) | Token::Variable(_) | Token::List(_) => {
                stack.push(token)
            }
            Token::Operator(op, op_type) => match op_type {
                OperatorType::Prefix => {
                    if let Some(&func) = prefix_map.get(&op) {
                        func(&mut stack, &mut context);
                    } else {
                        return Err(format!("Unknown prefix operator: {}", op));
                    }
                }
                OperatorType::Infix => {
                    if let Some(&func) = infix_map.get(&op) {
                        func(&mut stack, &mut context);
                    } else {
                        return Err(format!("Unknown infix operator: {}", op));
                    }
                }
                OperatorType::Postfix => {
                    if let Some(&func) = postfix_map.get(&op) {
                        func(&mut stack, &mut context);
                    } else {
                        return Err(format!("Unknown postfix operator: {}", op));
                    }
                }
            },
            _ => return Err("Unexpected token".to_string()),
        }
    }

    stack
        .pop()
        .ok_or_else(|| "Evaluation error: stack is empty".to_string())
}

pub fn evaluate(expression: &str, operators: &[Operator]) -> Result<Token, String> {
    let tokens = tokenize(expression, operators);
    let rpn_tokens = shunting_yard(tokens, operators);
    let result = evaluate_rpn(rpn_tokens, operators);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_expression(expression: &str, expected: &str) {
        let operators = get_standard_operators();

        // Test addition
        let tokens = tokenize(expression, &operators);
        let rpn = shunting_yard(tokens, &operators);
        let result = evaluate_rpn(rpn, &operators).unwrap();
        assert_eq!(
            result,
            Token::Number(BigDecimal::from_str(expected).unwrap())
        );
    }
    #[test]
    fn test_arith_evaluation() {
        test_expression("2 + 5", "7");
        test_expression("5 - 2", "3");
        test_expression("2 * 5", "10");
        test_expression("5 / 2", "2.5");
        test_expression("0, 0 + 5", "7");
        test_expression("5 % 2", "1");
        test_expression("(1 + 2) * 3", "9");
        test_expression("-1", "-1");
        test_expression("0 - 1", "-1");
        test_expression("(1 - 1) - 1", "-1");
        test_expression("(1 - 1) + -1", "-1");
        test_expression("+1", "1");
        test_expression("+1+1-1", "1");
        test_expression("(1 - 1) + +1", "1");
        test_expression("\"test\" - +1", "3");
        test_expression("2 ** 3", "8");
    }
    #[test]
    fn test_var_evaluation() {
        test_expression("x: 1 - 1", "0");
        test_expression("(x: 1) - x", "0");
    }
}
