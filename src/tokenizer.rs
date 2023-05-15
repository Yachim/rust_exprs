use regex::Regex;

const WHITESPACE_REGEX: &str = r"^\s+";
// "unregognized escape sequence" - on `\/` (division)
const OPERATOR_REGEX: &str = r"^(\+|-|\*|\/|&&|\|\||<=|>=|<|>|=)";
const NUMBER_REGEX: &str = r"^(\d+(?:\.\d+)?)";
const VARIABLE_REGEX: &str = r"^\{(.+?)\}";
const PARENTHESIS_REGEX: &str = r"^(\(|\))";

/// each member contains a regex match
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Whitespace,
    Operator,
    Number,
    Variable,
    Parenthesis,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

pub type TokenList = Vec<Token>;

struct Matcher {
    regex: Regex,
    token_type: TokenType,
}

pub struct Tokenizer {
    matchers: Vec<Matcher>,
}

#[derive(Debug, PartialEq)]
pub enum TokenizerError {
    // contains the substring
    NoTokenMatched(String),
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizerError::NoTokenMatched(substr) => write!(
                f,
                "token not matched at the beginning of substring: '{}'",
                substr
            ),
        }
    }
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            matchers: vec![
                Matcher {
                    regex: Regex::new(WHITESPACE_REGEX)
                        .expect("error while creating whitespace matcher regex"),
                    token_type: TokenType::Whitespace,
                },
                Matcher {
                    regex: Regex::new(OPERATOR_REGEX)
                        .expect("error while creating operator matcher regex"),
                    token_type: TokenType::Operator,
                },
                Matcher {
                    regex: Regex::new(NUMBER_REGEX)
                        .expect("error while creating number matcher regex"),
                    token_type: TokenType::Number,
                },
                Matcher {
                    regex: Regex::new(VARIABLE_REGEX)
                        .expect("error while creating variable matcher regex"),
                    token_type: TokenType::Variable,
                },
                Matcher {
                    regex: Regex::new(PARENTHESIS_REGEX)
                        .expect("error while creating parenthesis matcher regex"),
                    token_type: TokenType::Parenthesis,
                },
            ],
        }
    }

    pub fn tokenize(&self, str: &str) -> Result<TokenList, TokenizerError> {
        let mut tokens: TokenList = vec![];

        let mut index = 0;
        while index < str.len() {
            let (match_type, captures) = self
                .matchers
                .iter()
                .find_map(|matcher| {
                    let m = matcher.regex.captures(&str[index..]);

                    if let Some(m_val) = m {
                        return Some((matcher.token_type, m_val));
                    }

                    None
                })
                .ok_or(
                    /*&format!(
                        "No token matched\nindex: `{index}`\nstring: `{str}`\nsubstring: `{}`",
                        &str[index..]
                    )*/
                    TokenizerError::NoTokenMatched(str[index..].to_owned()),
                )?;

            index += captures
                .get(0)
                .expect("couldn't get 0th capture group")
                .len();
            if match_type != TokenType::Whitespace {
                tokens.push(Token {
                    value: captures
                        .get(1)
                        .expect("couldn't get 1th group (should be present in every regex)")
                        .as_str()
                        .to_string(),
                    token_type: match_type,
                });
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, TokenType, Tokenizer};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_tokenizer() {
        let tokenizer = Tokenizer::new();
        let tokens1 = tokenizer.tokenize("4/2 + (2.5 * {var} - 2)");
        assert_eq!(
            tokens1.unwrap(),
            vec![
                Token {
                    token_type: TokenType::Number,
                    value: "4".to_string()
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "/".to_string()
                },
                Token {
                    token_type: TokenType::Number,
                    value: "2".to_string()
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "+".to_string()
                },
                Token {
                    token_type: TokenType::Parenthesis,
                    value: "(".to_string()
                },
                Token {
                    token_type: TokenType::Number,
                    value: "2.5".to_string()
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "*".to_string()
                },
                Token {
                    token_type: TokenType::Variable,
                    value: "var".to_string()
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "-".to_string()
                },
                Token {
                    token_type: TokenType::Number,
                    value: "2".to_string()
                },
                Token {
                    token_type: TokenType::Parenthesis,
                    value: ")".to_string()
                },
            ]
        );

        let tokens2 = tokenizer.tokenize("1 < 2 && (3.5 > {var} || {x} = {y})");
        assert_eq!(
            tokens2.unwrap(),
            vec![
                Token {
                    token_type: TokenType::Number,
                    value: "1".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "<".to_string(),
                },
                Token {
                    token_type: TokenType::Number,
                    value: "2".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "&&".to_string(),
                },
                Token {
                    token_type: TokenType::Parenthesis,
                    value: "(".to_string(),
                },
                Token {
                    token_type: TokenType::Number,
                    value: "3.5".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: ">".to_string(),
                },
                Token {
                    token_type: TokenType::Variable,
                    value: "var".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "||".to_string(),
                },
                Token {
                    token_type: TokenType::Variable,
                    value: "x".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "=".to_string(),
                },
                Token {
                    token_type: TokenType::Variable,
                    value: "y".to_string(),
                },
                Token {
                    token_type: TokenType::Parenthesis,
                    value: ")".to_string(),
                },
            ]
        );

        let tokens3 = tokenizer.tokenize("1 <= {xx} && 3.5 >= {var}");
        assert_eq!(
            tokens3.unwrap(),
            vec![
                Token {
                    token_type: TokenType::Number,
                    value: "1".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "<=".to_string(),
                },
                Token {
                    token_type: TokenType::Variable,
                    value: "xx".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: "&&".to_string(),
                },
                Token {
                    token_type: TokenType::Number,
                    value: "3.5".to_string(),
                },
                Token {
                    token_type: TokenType::Operator,
                    value: ">=".to_string(),
                },
                Token {
                    token_type: TokenType::Variable,
                    value: "var".to_string(),
                },
            ]
        );
    }
}
