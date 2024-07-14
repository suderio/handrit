use bigdecimal::BigDecimal;
use std::str::FromStr;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(BigDecimal),
    String(String),
    Variable(String),
    Operator(String, OperatorType),
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
    pub func: fn(&mut Vec<Token>),
}

fn op_add(stack: &mut Vec<Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    if let (Token::Number(left_num), Token::Number(right_num)) = (left, right) {
        stack.push(Token::Number(left_num + right_num));
    } else {
        panic!("Operation requires two numbers");
    }

}

fn op_subtract(stack: &mut Vec<Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    if let (Token::Number(left_num), Token::Number(right_num)) = (left, right) {
        stack.push(Token::Number(left_num - right_num));
    } else {
        panic!("Operation requires two numbers");
    }

}

fn op_multiply(stack: &mut Vec<Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    if let (Token::Number(left_num), Token::Number(right_num)) = (left, right) {
        stack.push(Token::Number(left_num * right_num));
    } else {
        panic!("Operation requires two numbers");
    }
}

fn op_divide(stack: &mut Vec<Token>) {
    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();
    if let (Token::Number(left_num), Token::Number(right_num)) = (left, right) {
        stack.push(Token::Number(left_num / right_num));
    } else {
        panic!("Operation requires two numbers");
    }
}

fn op_null(_stack: &mut Vec<Token>) {
    println!("op_null")
}

fn op_infix_number<F>(stack: &mut Vec<Token>, op: F)
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
            let result = op(left_num, from_usize(right_str.len()));
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::String(left_str), Token::Number(right_num)) => {
            let result = op(from_usize(left_str.len()), right_num);
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::String(left_str), Token::String(right_str)) => {
            let result = op(from_usize(left_str.len()), from_usize(right_str.len()));
            stack.push(Token::Number(BigDecimal::from(result)));
        }
        (Token::Variable(_), _) | (_, Token::Variable(_)) => {
            panic!("Add operation cannot be performed with variables");
        }
        _ => {
            panic!("Add operation requires either numbers or strings");
        }
    }
}

pub fn get_standard_operators() -> Vec<Operator> {
    vec![
        Operator { symbol: ":".to_string(), precedence: 1, assoc: Assoc::Right, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "||".to_string(), precedence: 3, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "&&".to_string(), precedence: 4, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "|".to_string(), precedence: 5, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "^".to_string(), precedence: 6, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "&".to_string(), precedence: 7, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "=".to_string(), precedence: 8, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "<>".to_string(), precedence: 8, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: ">".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "<".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: ">=".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "<=".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "+".to_string(), precedence: 11, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_add },
        Operator { symbol: "-".to_string(), precedence: 11, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_subtract },
        Operator { symbol: "$".to_string(), precedence: 11, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "*".to_string(), precedence: 12, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_multiply },
        Operator { symbol: "/".to_string(), precedence: 12, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_divide },
        Operator { symbol: "%".to_string(), precedence: 12, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "**".to_string(), precedence: 13, assoc: Assoc::Right, op_type: OperatorType::Infix, func: op_null },
        Operator { symbol: "-".to_string(), precedence: 14, assoc: Assoc::Right, op_type: OperatorType::Prefix, func: op_null },
        Operator { symbol: "+".to_string(), precedence: 14, assoc: Assoc::Right, op_type: OperatorType::Prefix, func: op_null },
        Operator { symbol: "~".to_string(), precedence: 14, assoc: Assoc::Right, op_type: OperatorType::Prefix, func: op_null },
        Operator { symbol: "?".to_string(), precedence: 16, assoc: Assoc::Left, op_type: OperatorType::Postfix, func: op_null },
        Operator { symbol: ",".to_string(), precedence: 16, assoc: Assoc::Left, op_type: OperatorType::Infix, func: op_null },

        
        // Add more standard operators here
    ]
}

fn get_operator<'a>(symbol: &str, operators: &'a [Operator]) -> Option<&'a Operator> {
    operators.iter().find(|op| op.symbol == symbol)
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
                    if next_ch.is_alphanumeric() || next_ch == ' ' || next_ch == '(' || next_ch == ')' {
                        break;
                    }
                    op.push(next_ch);
                    chars.next();
                }
                if let Some(operator) = get_operator(&op, operators) {
                    tokens.push(Token::Operator(op, operator.op_type.clone()));
                } else {
                    chars.next(); // skip any unrecognized character
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
            Token::Number(_) | Token::String(_) | Token::Variable(_) => output.push(token),
            Token::Operator(op, op_type) => {
                match op_type {
                    OperatorType::Prefix => op_stack.push(token),
                    OperatorType::Postfix => {
                        output.push(token);
                    }
                    OperatorType::Infix => {
                        while let Some(top_op) = op_stack.last() {
                            if let Token::Operator(top_op_str, _) = top_op {
                                if let Some(top_operator) = get_operator(top_op_str, operators) {
                                    if (top_operator.assoc == Assoc::Left && top_operator.precedence >= precedence(&op, operators))
                                        || (top_operator.assoc == Assoc::Right && top_operator.precedence > precedence(&op, operators)) {
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
                }
            }
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
    let mut prefix_map: HashMap<String, fn(&mut Vec<Token>)> = HashMap::new();
    let mut infix_map: HashMap<String, fn(&mut Vec<Token>)> = HashMap::new();
    let mut postfix_map: HashMap<String, fn(&mut Vec<Token>)> = HashMap::new();
    
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
            Token::Number(_) | Token::String(_) | Token::Variable(_) => stack.push(token),
            Token::Operator(op, op_type) => {
                match op_type {
                    OperatorType::Prefix => {
                        if let Some(&func) = prefix_map.get(&op) {
                            func(&mut stack);
                        } else {
                            return Err(format!("Unknown prefix operator: {}", op));
                        }
                    }
                    OperatorType::Infix => {
                        if let Some(&func) = infix_map.get(&op) {
                            func(&mut stack);
                        } else {
                            return Err(format!("Unknown infix operator: {}", op));
                        }
                    }
                    OperatorType::Postfix => {
                        if let Some(&func) = postfix_map.get(&op) {
                            func(&mut stack);
                        } else {
                            return Err(format!("Unknown postfix operator: {}", op));
                        }
                    }
                }
            }
            _ => return Err("Unexpected token".to_string()),
        }
    }

    stack.pop().ok_or_else(|| "Evaluation error: stack is empty".to_string())
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

    #[test]
    fn test_rpn_evaluation() {
        let operators = get_standard_operators();

        // Test addition
        let tokens = tokenize("2 + 5", &operators);
        let rpn = shunting_yard(tokens, &operators);
        let result = evaluate_rpn(rpn, &operators).unwrap();
        assert_eq!(result, Token::Number(BigDecimal::from(7)));

        // Test more operations here...
    }
}
