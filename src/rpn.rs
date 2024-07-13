use bigdecimal::BigDecimal;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(BigDecimal),
    String(String),
    Variable(String),
    Operator(String, OperatorType),
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, PartialEq)]
enum OperatorType {
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
}

pub fn get_standard_operators() -> Vec<Operator> {
    vec![
        Operator { symbol: ":".to_string(), precedence: 1, assoc: Assoc::Right, op_type: OperatorType::Infix },
        Operator { symbol: "||".to_string(), precedence: 3, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "&&".to_string(), precedence: 4, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "|".to_string(), precedence: 5, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "^".to_string(), precedence: 6, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "&".to_string(), precedence: 7, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "=".to_string(), precedence: 8, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "<>".to_string(), precedence: 8, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: ">".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "<".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: ">=".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "<=".to_string(), precedence: 9, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "+".to_string(), precedence: 11, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "-".to_string(), precedence: 11, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "$".to_string(), precedence: 11, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "*".to_string(), precedence: 12, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "/".to_string(), precedence: 12, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "%".to_string(), precedence: 12, assoc: Assoc::Left, op_type: OperatorType::Infix },
        Operator { symbol: "**".to_string(), precedence: 13, assoc: Assoc::Right, op_type: OperatorType::Infix },
        Operator { symbol: "-".to_string(), precedence: 14, assoc: Assoc::Right, op_type: OperatorType::Prefix },
        Operator { symbol: "+".to_string(), precedence: 14, assoc: Assoc::Right, op_type: OperatorType::Prefix },
        Operator { symbol: "~".to_string(), precedence: 14, assoc: Assoc::Right, op_type: OperatorType::Prefix },
        Operator { symbol: "?".to_string(), precedence: 16, assoc: Assoc::Left, op_type: OperatorType::Postfix },
        Operator { symbol: ",".to_string(), precedence: 16, assoc: Assoc::Left, op_type: OperatorType::Infix },

        
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
