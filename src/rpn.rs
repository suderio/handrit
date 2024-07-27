use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use bigdecimal::One;
use bigdecimal::ToPrimitive;
use bigdecimal::Zero;
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
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    LeftRef,
    RightRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorType {
    Prefix,
    Infix,
    Postfix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Assoc {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Operator {
    pub symbol: String,
    pub precedence: usize,
    pub assoc: Assoc,
    pub op_type: OperatorType,
    pub func: fn(&mut Vec<Token>, &mut HashMap<String, Token>),
}

pub struct Machine {
    operators: Vec<Operator>,
}

impl Machine {
    pub fn new() -> Machine {
        Machine { operators: get_standard_operators().clone() }
    }

    fn add_operator(&mut self, op_str: String) -> Token {
        // TODO this is capturing any leftxxx or rightxxx variable
        let mut op_type = OperatorType::Postfix;
        let mut op_func: fn(&mut Vec<Token>, &mut HashMap<String, Token>) = |stack, context| op_posfix_op(stack, context);
        if op_str.contains("left") && op_str.contains("right") {
            op_type = OperatorType::Infix;
            op_func = |stack, context| op_infix_op(stack, context);
        } else if op_str.contains(" right ") {
            op_type = OperatorType::Prefix;
            op_func = |stack, context| op_prefix_op(stack, context);
        };
        let op_symbol = format!("{{{}}}", op_str.clone());

        self.operators.push(Operator {
            symbol: op_symbol.clone(),
            precedence: 20,
            assoc: Assoc::Left,
            op_type: op_type.clone(),
            func: op_func,
        });
        Token::Operator(op_symbol, op_type)
    }

    fn tokenize(&mut self, expression: &str) -> Vec<Token> {
        //        tokenize(expression, &self.operators)
        let mut tokens = Vec::new();
        let mut chars = expression.chars().peekable();
        //        let mut func_operator = Vec::new();

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
                '[' => {
                    tokens.push(Token::LeftBracket);
                    chars.next();
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    chars.next();
                }
                '{' => {
                    chars.next(); // skip the opening brace
                    let mut func_str = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' {
                            chars.next(); // skip the closing brace
                            break;
                        }
                        func_str.push(ch);
                        chars.next();
                    }
                    let op = &self.add_operator(func_str);
                    tokens.push(op.clone());
                }
                '}' => {
                    tokens.push(Token::RightBrace);
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
                    if ident == "left" {
                        tokens.push(Token::LeftRef);
                    } else if ident == "right" {
                        tokens.push(Token::RightRef);
                    } else {
                        tokens.push(Token::Variable(ident));
                    }
                }
                _ => {
                    let mut op = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric()
                            || next_ch == ' '
                            || next_ch == '('
                            || next_ch == ')'
                            || next_ch == '['
                            || next_ch == ']'
                            || next_ch == '{'
                            || next_ch == '}'
                    {
                        break;
                    }
                        op.push(next_ch);
                        chars.next();
                    }
                    match tokens.last() {
                        Some(Token::Operator(_, _)) | Some(Token::LeftParen) | None => {
                            if let Some(operator) = get_prefix_operator(&op, &self.operators) {
                                tokens.push(Token::Operator(op, operator.op_type.clone()));
                            } else {
                                chars.next(); // skip any unrecognized character
                            }
                        }

                        _ => {
                            if let Some(operator) = get_infix_operator(&op, &self.operators) {
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

    fn shunting_yard(&self, tokens: Vec<Token>) -> Vec<Token> {
        //        shunting_yard(tokens, &self.operators)
        let mut output = Vec::new();
        let mut op_stack = Vec::new();

        for token in tokens {
            match token.clone() {
                Token::Number(_)
            | Token::String(_)
            | Token::Variable(_)
            | Token::List(_)
            | Token::LeftRef
            | Token::RightRef => output.push(token),
                Token::Operator(op, op_type) => match op_type {
                    OperatorType::Prefix => op_stack.push(token),
                    OperatorType::Postfix => {
                        output.push(token);
                    }
                    OperatorType::Infix => {
                        while let Some(top_op) = op_stack.last() {
                            if let Token::Operator(top_op_str, _) = top_op {
                                if let Some(top_operator) = get_operator(top_op_str, &self.operators) {
                                    if (top_operator.assoc == Assoc::Left
                                        && top_operator.precedence >= precedence(&op, &self.operators))
                                        || (top_operator.assoc == Assoc::Right
                                            && top_operator.precedence > precedence(&op, &self.operators))
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
                /*
                Token::LeftBracket => {
                    output.push(Token::LeftBracket);
                    op_stack.push(token);
                }
                Token::RightBracket => {
                    while let Some(top_op) = op_stack.pop() {
                        if top_op == Token::LeftBracket {
                            break;
                        }
                        output.push(top_op);
                    }
                    output.push(Token::RightBracket);
                }
                Token::LeftBrace => {
                    output.push(Token::LeftBrace);
                    op_stack.push(token);
                }
                Token::RightBrace => {
                    while let Some(top_op) = op_stack.pop() {
                        if top_op == Token::LeftBrace {
                            break;
                        }
                        output.push(top_op);
                    }
                    output.push(Token::RightBrace);
                }
                 */
                _ => todo!()
            }
        }

        while let Some(op) = op_stack.pop() {
            output.push(op);
        }

        output
    }

    fn evaluate(&self, tokens: Vec<Token>) -> Result<Token, String> {
        //        evaluate_rpn(tokens, &self.operators)
        let mut stack = Vec::new();
        let mut prefix_map: HashMap<String, fn(&mut Vec<Token>, &mut HashMap<String, Token>)> =
            HashMap::new();
        let mut infix_map: HashMap<String, fn(&mut Vec<Token>, &mut HashMap<String, Token>)> =
            HashMap::new();
        let mut postfix_map: HashMap<String, fn(&mut Vec<Token>, &mut HashMap<String, Token>)> =
            HashMap::new();
        let mut context: HashMap<String, Token> = HashMap::new();

        for op in &self.operators {
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

        stack.pop().ok_or_else(|| "[]".to_string())
    }

    pub fn to_rpn(&mut self, expression: &str) -> String {
        let tokens = self.tokenize(expression);
        let rpn_tokens = self.shunting_yard(tokens);

        let mut result = String::new();
        for token in rpn_tokens {
            match token {
                Token::Number(num) => result.push_str(&format!("{} ", num)),
                Token::String(s) => result.push_str(&format!("\"{}\" ", s)),
                Token::Variable(var) => result.push_str(&format!("{} ", var)),
                Token::Operator(op, _) => result.push_str(&format!("{} ", op)),
                Token::LeftParen => result.push_str(&format!("{} ", "(")),
                Token::RightParen => result.push_str(&format!("{} ", ")")),
                Token::LeftBracket => result.push_str(&format!("{} ", "[")),
                Token::RightBracket => result.push_str(&format!("{} ", "]")),
                Token::LeftBrace => result.push_str(&format!("{} ", "{")),
                Token::RightBrace => result.push_str(&format!("{} ", "}")),
                Token::LeftRef => result.push_str(&format!("{} ", "left")),
                Token::RightRef => result.push_str(&format!("{} ", "right")),
                _ => {}
            }
        }

        result.trim().to_string()
    }
    pub fn run(&mut self, expression: &str) -> Result<Token, String> {
        let tokens = self.tokenize(expression);
        let rpn_tokens = self.shunting_yard(tokens);
        let result = self.evaluate(rpn_tokens);
        result
    }
}

trait NumericValue {
    fn n_value(self) -> BigDecimal;
    fn b_value(self) -> bool;
}

impl NumericValue for BigDecimal {
    fn n_value(self) -> BigDecimal {
        self
    }
    fn b_value(self) -> bool {
        self != BigDecimal::zero()
    }
}

impl NumericValue for String {
    fn n_value(self) -> BigDecimal {
        BigDecimal::from_usize(self.len()).expect("TODO!")
    }
    fn b_value(self) -> bool {
        BigDecimal::from_usize(self.len()).expect("TODO!") != BigDecimal::zero()
    }
}

impl NumericValue for Vec<Token> {
    fn n_value(self) -> BigDecimal {
        BigDecimal::from_usize(self.len()).expect("TODO!")
    }
    fn b_value(self) -> bool {
        BigDecimal::from_usize(self.len()).expect("TODO!") != BigDecimal::zero()
    }
}

fn op_infix_numeric_value(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    match (left, right) {
        (Token::Number(left_num), Token::Number(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::Number(left_num), Token::String(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::String(left_num), Token::Number(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::String(left_num), Token::String(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::List(left_num), Token::Number(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::List(left_num), Token::String(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::Number(left_num), Token::List(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::String(left_num), Token::List(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::List(left_num), Token::List(right_num)) => {
            stack.push(Token::Number(BigDecimal::from(left_num.n_value())));
            stack.push(Token::Number(BigDecimal::from(right_num.n_value())));
        }
        (Token::Variable(left_var), right) => {
            let left_value = context.get(&left_var).clone();
            // TODO maybe we can avoid undefined variable error here by pushing the variable again?
            stack.push(left_value.expect("No lvalue").clone());
            stack.push(right);
            op_infix_numeric_value(stack, context);
        }
        (left, Token::Variable(right_var)) => {
            let right_value = context.get(&right_var).clone();
            // TODO maybe we can avoid undefined variable error here by pushing the variable again?
            stack.push(left);
            stack.push(right_value.expect("No rvalue").clone());
            op_infix_numeric_value(stack, context);
        }
        _ => {
            panic!("Add operation requires either numbers or strings");
        }
    }
}
fn op_infix_number<F>(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>, op: F)
where
    F: Fn(BigDecimal, BigDecimal) -> BigDecimal,
{
    op_infix_numeric_value(stack, context);
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    match (left, right) {
        (Token::Number(left_num), Token::Number(right_num)) => {
            stack.push(Token::Number(op(left_num, right_num)));
        }
        _ => panic!("Add operation requires numbers"),
    }
}

fn op_infix_bool<F>(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>, op: F)
where
    F: Fn(bool, bool) -> bool,
{
    op_infix_numeric_value(stack, context);
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    match (left, right) {
        (Token::Number(left_num), Token::Number(right_num)) => {
            stack.push(Token::Number(
                if op(left_num.b_value(), right_num.b_value()) {
                    BigDecimal::one()
                } else {
                    BigDecimal::zero()
                },
            ));
        }
        _ => panic!("Add operation requires booleans"),
    }
}

fn op_infix_op(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    context.insert(String::from("right"), right);
    context.insert(String::from("left"), left);
    stack.push(Token::String(String::from("undefined")));
    context.remove("right");
    context.remove("left");
}
fn op_prefix_op(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>) {
    let right = stack.pop().unwrap();
    context.insert(String::from("right"), right);
    stack.push(Token::String(String::from("undefined")));
    context.remove("right");
}
fn op_posfix_op(stack: &mut Vec<Token>, context: &mut HashMap<String, Token>) {
    let left = stack.pop().unwrap();
    context.insert(String::from("left"), left);
    stack.push(Token::String(String::from("undefined")));
    context.remove("left");
}
pub fn get_standard_operators() -> Vec<Operator> {
    vec![
        Operator {
            symbol: ":".to_string(),
            precedence: 1,
            assoc: Assoc::Right,
            op_type: OperatorType::Infix,
            func: |stack, context| {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                match (left, right) {
                    (Token::Variable(left_var), right) => {
                        context.insert(left_var, right.clone());
                        stack.push(right);
                    }
                    _ => panic!("Tried assignment operation without a variable on the left side."),
                }
            },
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
            func: |stack, context| op_infix_bool(stack, context, |left, right| left && right),
        },
        Operator {
            symbol: "|".to_string(),
            precedence: 5,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_bool(stack, context, |left, right| left | right),
        },
        Operator {
            symbol: "^".to_string(),
            precedence: 6,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_bool(stack, context, |left, right| left ^ right),
        },
        Operator {
            symbol: "&".to_string(),
            precedence: 7,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| op_infix_bool(stack, context, |left, right| left & right),
        },
        Operator {
            symbol: "=".to_string(),
            precedence: 8,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| {
                op_infix_number(stack, context, |left, right| {
                    if left == right {
                        BigDecimal::one()
                    } else {
                        BigDecimal::zero()
                    }
                })
            },
        },
        Operator {
            symbol: "<>".to_string(),
            precedence: 8,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| {
                op_infix_number(stack, context, |left, right| {
                    if left != right {
                        BigDecimal::one()
                    } else {
                        BigDecimal::zero()
                    }
                })
            },
        },
        Operator {
            symbol: ">".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| {
                op_infix_number(stack, context, |left, right| {
                    if left > right {
                        BigDecimal::one()
                    } else {
                        BigDecimal::zero()
                    }
                })
            },
        },
        Operator {
            symbol: "<".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| {
                op_infix_number(stack, context, |left, right| {
                    if left < right {
                        BigDecimal::one()
                    } else {
                        BigDecimal::zero()
                    }
                })
            },
        },
        Operator {
            symbol: ">=".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| {
                op_infix_number(stack, context, |left, right| {
                    if left >= right {
                        BigDecimal::one()
                    } else {
                        BigDecimal::zero()
                    }
                })
            },
        },
        Operator {
            symbol: "<=".to_string(),
            precedence: 9,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, context| {
                op_infix_number(stack, context, |left, right| {
                    if left <= right {
                        BigDecimal::one()
                    } else {
                        BigDecimal::zero()
                    }
                })
            },
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
            func: |_, _| println!("$"),
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
            func: |stack, context| {
                op_infix_number(stack, context, |left, right| {
                    let mut result = BigDecimal::from_i32(1).expect("TODO!");
                    for _ in 0..(right.to_u128().expect("TODO!")) {
                        result *= &left;
                    }
                    result
                });
            },
        },
        Operator {
            symbol: "-".to_string(),
            precedence: 14,
            assoc: Assoc::Right,
            op_type: OperatorType::Prefix,
            func: |stack, _context| {
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
            },
        },
        Operator {
            symbol: "+".to_string(),
            precedence: 14,
            assoc: Assoc::Right,
            op_type: OperatorType::Prefix,
            func: |stack, _context| {
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
            },
        },
        Operator {
            symbol: "~".to_string(),
            precedence: 14,
            assoc: Assoc::Right,
            op_type: OperatorType::Prefix,
            func: |_, _| println!("~"),
        },
        Operator {
            symbol: "?".to_string(),
            precedence: 16,
            assoc: Assoc::Left,
            op_type: OperatorType::Postfix,
            func: |_, _| println!("?"),
        },
        Operator {
            symbol: ",".to_string(),
            precedence: 16,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |stack, _context| {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                match left {
                    Token::List(mut left_list) => {
                        left_list.push(right);
                        stack.push(Token::List(left_list));
                    }
                    _ => stack.push(Token::List(vec![left, right])),
                }
            },
        },
        Operator {
            symbol: ";".to_string(),
            precedence: 16,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |_, _| println!(";"),
        },
        Operator {
            symbol: ".".to_string(),
            precedence: 16,
            assoc: Assoc::Left,
            op_type: OperatorType::Infix,
            func: |_, _| println!("."),
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


fn precedence(op: &str, operators: &[Operator]) -> usize {
    if let Some(operator) = get_operator(op, operators) {
        operator.precedence
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_expression(expression: &str, expected: &str) {
        let mut machine = Machine::new();
        let result = machine.run(expression).unwrap();
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

    #[test]
    fn test_bool_evaluation() {
        test_expression("3 || 0", "1");
        test_expression("(1 - 1) || 0", "0");
        test_expression("0 && 0", "0");
    }
}
