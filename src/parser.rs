use std::collections::VecDeque;

use crate::tokenizer::Token;

use super::tokenizer::{TokenList, TokenType};

pub type RPN = Vec<Value>;

#[derive(PartialEq, Eq, Debug)]
enum OperatorType {
    /// +
    Plus,

    /// -
    Minus,

    /// *
    Times,

    /// /
    Divide,

    /// &&
    And,

    /// ||
    Or,

    /// <
    LT,

    /// <=
    LE,

    /// >
    GT,

    /// >=
    GE,

    /// =
    Eq,

    /// (
    LeftParenthesis,

    /// )
    RightParenthesis,
}

impl OperatorType {
    fn from_str(str: &str) -> Self {
        match str {
            "+" => OperatorType::Plus,
            "-" => OperatorType::Minus,
            "*" => Self::Times,
            "/" => Self::Divide,
            "&&" => Self::And,
            "||" => Self::Or,
            "<" => Self::LT,
            "<=" => Self::LE,
            ">" => Self::GT,
            ">=" => Self::GE,
            "=" => Self::Eq,
            _ => unreachable!("Invalid value: {str}"),
        }
    }

    /// https://en.wikipedia.org/wiki/Order_of_operations#Programming_languages
    /// the higher the number, the higher the priority
    fn get_priority(&self) -> u32 {
        match self {
            Self::LeftParenthesis | Self::RightParenthesis => {
                unreachable!("Trying to get priority of parantheses")
            }

            Self::Or => 1,
            Self::And => 2,
            Self::Eq => 3,
            Self::LT | Self::LE | Self::GT | Self::GE => 4,
            Self::Plus | Self::Minus => 5,
            Self::Times | Self::Divide => 6,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Value {
    Operator(OperatorType),
    Number(f32),
    Variable(String),
}

struct Parser {}

impl Parser {
    pub fn tokens_to_rpn(tokens: TokenList) -> RPN {
        // queue - last index in, 0th index out
        let mut token_queue: RPN = vec![];
        // stack - 0th index in, 0th index out
        let mut operator_stack: VecDeque<OperatorType> = VecDeque::new();

        tokens.iter().for_each(|token| match token.token_type {
            TokenType::Number => token_queue.push(Value::Number(
                token.value.parse().expect("Failed to parse float"),
            )),
            TokenType::Variable => token_queue.push(Value::Variable(token.value.clone())),
            TokenType::Parenthesis => match token.value.as_str() {
                "(" => operator_stack.push_front(OperatorType::LeftParenthesis),
                ")" => {
                    while operator_stack.len() > 0
                        && operator_stack[0] != OperatorType::LeftParenthesis
                    {
                        token_queue.push(Value::Operator(operator_stack.pop_front().unwrap()))
                    }
                    assert_eq!(
                        operator_stack[0],
                        OperatorType::LeftParenthesis,
                        "No matching left parenthesis."
                    );

                    operator_stack.pop_front();
                }
                _ => unreachable!(
                    "Token of type parenthesis has invalid value. Value: {}.",
                    token.value
                ),
            },
            TokenType::Operator => {
                let op = OperatorType::from_str(token.value.as_str());

                while operator_stack.len() > 0
                    && operator_stack[0] != OperatorType::LeftParenthesis
                    && operator_stack[0].get_priority() >= op.get_priority()
                {
                    token_queue.push(Value::Operator(operator_stack.pop_front().unwrap()));
                }

                operator_stack.push_front(op);
            }
            TokenType::Whitespace => unimplemented!("Whitespace in token list"),
        });

        while operator_stack.len() > 0 {
            let op = operator_stack.pop_front().unwrap();

            assert_ne!(
                op,
                OperatorType::LeftParenthesis,
                "Left paranthesis on top of operator stack."
            );
            token_queue.push(Value::Operator(op));
        }

        token_queue
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::{
        parser::{OperatorType, Value},
        tokenizer::{Token, TokenList, TokenType},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_1() {
        // "1 + 2" -> "1 2 +"
        let tokens: TokenList = vec![
            Token {
                value: "1".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: "+".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "2".to_string(),
                token_type: TokenType::Number,
            },
        ];

        let rpn = Parser::tokens_to_rpn(tokens);
        assert_eq!(
            rpn,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Operator(OperatorType::Plus)
            ]
        );
    }

    #[test]
    fn test_2() {
        // "1 + 2 * 3" -> "1 2 3 * +"
        let tokens: TokenList = vec![
            Token {
                value: "1".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: "+".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "2".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: "*".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "3".to_string(),
                token_type: TokenType::Number,
            },
        ];

        let rpn = Parser::tokens_to_rpn(tokens);
        assert_eq!(
            rpn,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Operator(OperatorType::Times),
                Value::Operator(OperatorType::Plus),
            ]
        );
    }

    #[test]
    fn test_3() {
        // "(1 + 2) * 3" -> "1 2 + 3 *"
        let tokens: TokenList = vec![
            Token {
                value: "(".to_string(),
                token_type: TokenType::Parenthesis,
            },
            Token {
                value: "1".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: "+".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "2".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: ")".to_string(),
                token_type: TokenType::Parenthesis,
            },
            Token {
                value: "*".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "3".to_string(),
                token_type: TokenType::Number,
            },
        ];

        let rpn = Parser::tokens_to_rpn(tokens);
        assert_eq!(
            rpn,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Operator(OperatorType::Plus),
                Value::Number(3.0),
                Value::Operator(OperatorType::Times),
            ]
        );
    }

    #[test]
    fn test_4() {
        // "5 * (3 + 4) - 2" -> "5 3 4 + * 2 -"
        let tokens: TokenList = vec![
            Token {
                value: "5".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: "*".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "(".to_string(),
                token_type: TokenType::Parenthesis,
            },
            Token {
                value: "3".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: "+".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "4".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: ")".to_string(),
                token_type: TokenType::Parenthesis,
            },
            Token {
                value: "-".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "2".to_string(),
                token_type: TokenType::Number,
            },
        ];

        let rpn = Parser::tokens_to_rpn(tokens);
        assert_eq!(
            rpn,
            vec![
                Value::Number(5.0),
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Operator(OperatorType::Plus),
                Value::Operator(OperatorType::Times),
                Value::Number(2.0),
                Value::Operator(OperatorType::Minus),
            ]
        );
    }

    #[test]
    fn test_5() {
        // "a || b && c" -> "a b c && ||"
        let tokens: TokenList = vec![
            Token {
                value: "a".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: "||".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "b".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: "&&".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "c".to_string(),
                token_type: TokenType::Variable,
            },
        ];

        let rpn = Parser::tokens_to_rpn(tokens);
        assert_eq!(
            rpn,
            vec![
                Value::Variable("a".to_string()),
                Value::Variable("b".to_string()),
                Value::Variable("c".to_string()),
                Value::Operator(OperatorType::And),
                Value::Operator(OperatorType::Or),
            ]
        );
    }

    #[test]
    fn test_6() {
        // "a > b && (c <= d || e >= f)" -> "a b > c d <= e f >= || &&"
        let tokens: TokenList = vec![
            Token {
                value: "a".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: ">".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "b".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: "&&".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "(".to_string(),
                token_type: TokenType::Parenthesis,
            },
            Token {
                value: "c".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: "<=".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "d".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: "||".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "e".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: ">=".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "f".to_string(),
                token_type: TokenType::Variable,
            },
            Token {
                value: ")".to_string(),
                token_type: TokenType::Parenthesis,
            },
        ];

        let rpn = Parser::tokens_to_rpn(tokens);
        assert_eq!(
            rpn,
            vec![
                Value::Variable("a".to_string()),
                Value::Variable("b".to_string()),
                Value::Operator(OperatorType::GT),
                Value::Variable("c".to_string()),
                Value::Variable("d".to_string()),
                Value::Operator(OperatorType::LE),
                Value::Variable("e".to_string()),
                Value::Variable("f".to_string()),
                Value::Operator(OperatorType::GE),
                Value::Operator(OperatorType::Or),
                Value::Operator(OperatorType::And),
            ]
        );
    }
}
