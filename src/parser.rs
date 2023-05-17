use crate::tokenizer::{TokenList, TokenType};
use std::collections::VecDeque;

pub type Rpn = Vec<Value>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum OperatorType {
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
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    NoMatchingLeftParenthesis,
    ExtraLeftParenthesis,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::NoMatchingLeftParenthesis => {
                write!(f, "right parenthesis has no matching left parenthesis")
            }
            Self::ExtraLeftParenthesis => {
                write!(f, "extra left parenthesis")
            }
        }
    }
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
            _ => unreachable!("invalid value: {str}"),
        }
    }

    /// https://en.wikipedia.org/wiki/Order_of_operations#Programming_languages
    /// the higher the number, the higher the priority
    fn get_priority(&self) -> u32 {
        match self {
            Self::LeftParenthesis => {
                unreachable!("trying to get priority of paranthesis")
            }

            Self::Or => 1,
            Self::And => 2,
            Self::Eq => 3,
            Self::LT | Self::LE | Self::GT | Self::GE => 4,
            Self::Plus | Self::Minus => 5,
            Self::Times | Self::Divide => 6,
        }
    }

    pub fn eval_nums(&self, first: f32, second: f32) -> f32 {
        match self {
            Self::LeftParenthesis => {
                unreachable!("OperatorType parenthesis")
            }

            Self::Plus => first + second,
            Self::Minus => first - second,
            Self::Times => first * second,
            Self::Divide => first / second,
            Self::And | Self::Or => {
                panic!("method `eval_conditional` should be used instead")
            }
            Self::LT | Self::LE | Self::GT | Self::GE | Self::Eq => {
                panic!("method `eval_comparison` should be used instead")
            }
        }
    }

    pub fn eval_conditional(&self, first: bool, second: bool) -> bool {
        match self {
            Self::LeftParenthesis => {
                unreachable!("OperatorType parenthesis")
            }

            Self::Plus | Self::Minus | Self::Times | Self::Divide => {
                panic!("method `eval_nums` should be used instead")
            }
            Self::And => first && second,
            Self::Or => first || second,
            Self::LT | Self::LE | Self::GT | Self::GE | Self::Eq => {
                panic!("method `eval_comparison` should be used instead")
            }
        }
    }

    pub fn eval_comparison(&self, first: f32, second: f32) -> bool {
        match self {
            Self::LeftParenthesis => {
                unreachable!("operatorType parenthesis")
            }

            Self::Plus | Self::Minus | Self::Times | Self::Divide => {
                panic!("method `eval_nums` should be used instead.")
            }
            Self::And | Self::Or => {
                panic!("method `eval_conditional` should be used instead.")
            }
            Self::LT => first < second,
            Self::LE => first <= second,
            Self::GT => first > second,
            Self::GE => first >= second,
            Self::Eq => first == second,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Operator(OperatorType),
    Number(f32),
    Boolean(bool),
    Variable(String),
}

pub fn tokens_to_rpn(tokens: TokenList) -> Result<Rpn, ParserError> {
    // queue - last index in, 0th index out
    let mut token_queue: Rpn = vec![];
    // stack - 0th index in, 0th index out
    let mut operator_stack: VecDeque<OperatorType> = VecDeque::new();

    for token in tokens {
        match token.token_type {
            TokenType::Number => token_queue.push(Value::Number(
                token.value.parse().expect("failed to parse float"),
            )),
            TokenType::Boolean => token_queue.push(Value::Boolean(match token.value.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!(
                    "token of type parenthesis has invalid value, value: {}",
                    token.value
                ),
            })),
            TokenType::Variable => token_queue.push(Value::Variable(token.value.clone())),
            TokenType::Parenthesis => match token.value.as_str() {
                "(" => operator_stack.push_front(OperatorType::LeftParenthesis),
                ")" => {
                    while !operator_stack.is_empty()
                        && operator_stack[0] != OperatorType::LeftParenthesis
                    {
                        token_queue.push(Value::Operator(
                            operator_stack.pop_front().unwrap_or_else(|| {
                                panic!("could not pop first value of stack: {:#?}", operator_stack)
                            }),
                        ))
                    }
                    if operator_stack[0] != OperatorType::LeftParenthesis {
                        return Err(ParserError::NoMatchingLeftParenthesis);
                    }

                    operator_stack.pop_front().unwrap_or_else(|| {
                        panic!("could not pop first value of stack: {:#?}", operator_stack)
                    });
                }
                _ => unreachable!(
                    "token of type parenthesis has invalid value, value: {}",
                    token.value
                ),
            },
            TokenType::Operator => {
                let op = OperatorType::from_str(token.value.as_str());

                while !operator_stack.is_empty()
                    && operator_stack[0] != OperatorType::LeftParenthesis
                    && operator_stack[0].get_priority() >= op.get_priority()
                {
                    token_queue.push(Value::Operator(operator_stack.pop_front().unwrap_or_else(
                        || panic!("could not pop first value of stack: {:#?}", operator_stack),
                    )));
                }

                operator_stack.push_front(op);
            }
            TokenType::Whitespace => unimplemented!("whitespace in token list"),
        }
    }

    while !operator_stack.is_empty() {
        let op = operator_stack.pop_front().unwrap_or_else(|| {
            panic!(
                "could not pop first stack from stack: {:#?}",
                operator_stack
            )
        });

        if op == OperatorType::LeftParenthesis {
            return Err(ParserError::ExtraLeftParenthesis);
        }
        token_queue.push(Value::Operator(op));
    }

    Ok(token_queue)
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::{tokens_to_rpn, OperatorType, Value},
        tokenizer::{Token, TokenList, TokenType},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_1() {
        // "1 + 4/2" -> "1 4 2 / +"
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
                value: "4".to_string(),
                token_type: TokenType::Number,
            },
            Token {
                value: "/".to_string(),
                token_type: TokenType::Operator,
            },
            Token {
                value: "2".to_string(),
                token_type: TokenType::Number,
            },
        ];

        let rpn = tokens_to_rpn(tokens);
        assert_eq!(
            rpn.unwrap(),
            vec![
                Value::Number(1.0),
                Value::Number(4.0),
                Value::Number(2.0),
                Value::Operator(OperatorType::Divide),
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

        let rpn = tokens_to_rpn(tokens);
        assert_eq!(
            rpn.unwrap(),
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

        let rpn = tokens_to_rpn(tokens);
        assert_eq!(
            rpn.unwrap(),
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

        let rpn = tokens_to_rpn(tokens);
        assert_eq!(
            rpn.unwrap(),
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

        let rpn = tokens_to_rpn(tokens);
        assert_eq!(
            rpn.unwrap(),
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

        let rpn = tokens_to_rpn(tokens);
        assert_eq!(
            rpn.unwrap(),
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

    #[test]
    fn test_7() {
        let tokens: TokenList = vec![
            Token {
                token_type: TokenType::Boolean,
                value: "true".to_string(),
            },
            Token {
                token_type: TokenType::Operator,
                value: "||".to_string(),
            },
            Token {
                token_type: TokenType::Boolean,
                value: "false".to_string(),
            },
        ];

        let rpn = tokens_to_rpn(tokens);
        assert_eq!(
            rpn.unwrap(),
            vec![
                Value::Boolean(true),
                Value::Boolean(false),
                Value::Operator(OperatorType::Or)
            ]
        )
    }

    #[test]
    fn test_eval_nums() {
        assert_eq!(OperatorType::Plus.eval_nums(2.0, 3.0), 5.0);
        assert_eq!(OperatorType::Minus.eval_nums(5.0, 3.0), 2.0);
        assert_eq!(OperatorType::Times.eval_nums(2.0, 3.0), 6.0);
        assert_eq!(OperatorType::Divide.eval_nums(6.0, 3.0), 2.0);
    }

    #[test]
    fn test_eval_comparison() {
        assert_eq!(OperatorType::LT.eval_comparison(2.0, 3.0), true);
        assert_eq!(OperatorType::LT.eval_comparison(5.0, 3.0), false);

        assert_eq!(OperatorType::LE.eval_comparison(3.0, 3.0), true);
        assert_eq!(OperatorType::LE.eval_comparison(5.0, 3.0), false);

        assert_eq!(OperatorType::GT.eval_comparison(6.0, 3.0), true);
        assert_eq!(OperatorType::GT.eval_comparison(1.0, 3.0), false);

        assert_eq!(OperatorType::GE.eval_comparison(3.0, 3.0), true);
        assert_eq!(OperatorType::GE.eval_comparison(2.0, 3.0), false);

        assert_eq!(OperatorType::Eq.eval_comparison(3.0, 3.0), true);
        assert_eq!(OperatorType::Eq.eval_comparison(4.0, 3.0), false);
        assert_eq!(OperatorType::Eq.eval_comparison(2.0, 3.0), false);
    }

    #[test]
    fn test_eval_conditional() {
        assert_eq!(OperatorType::And.eval_conditional(true, true), true);
        assert_eq!(OperatorType::And.eval_conditional(true, false), false);
        assert_eq!(OperatorType::And.eval_conditional(false, true), false);
        assert_eq!(OperatorType::And.eval_conditional(false, false), false);

        assert_eq!(OperatorType::Or.eval_conditional(true, true), true);
        assert_eq!(OperatorType::Or.eval_conditional(true, false), true);
        assert_eq!(OperatorType::Or.eval_conditional(false, true), true);
        assert_eq!(OperatorType::Or.eval_conditional(false, false), false);
    }
}
