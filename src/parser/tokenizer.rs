use regex::Regex;

const OPERATOR_REGEX: &str = r"^(\+|-|\*|/)";
const LOGICAL_OPERATOR_REGEX: &str = r"^(&&|\|\||!)";
const EQUALITY_OPERATOR_REGEX: &str = r"^(<=|>=|<|>|=)";
const NUMBER_REGEX: &str = r"^(\d+(?:.\d+)?)";
const VARIABLE_REGEX: &str = r"^\{(.+?)}";
const PARENTHESIS_REGEX: &str = r"^(\(|\))";
const WHITESPACE_REGEX: &str = r"^\s+";

/// each member contains a regex match
enum TokenType {
    Operator,
    LogicalOperator,
    EqualityOperator,
    Number,
    Variable,
    Parenthesis,
    Whitespace,
}

struct Token {
    token_type: TokenType,
    value: String,
}

pub type Tokens = Vec<Token>;
