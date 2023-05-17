use crate::{
    emitter::{BindVariablesError, EmitResult, Emitter, VariableMap},
    parser::tokens_to_rpn,
    tokenizer::Tokenizer,
    EvalError, ParserError, TokenizerError,
};

#[derive(Debug, Clone)]
pub struct Expression {
    pub str_expr: String,
    emitter: Emitter,
}

#[derive(Debug)]
pub enum ExpressionCreationError {
    TokenizerError(TokenizerError),
    ParserError(ParserError),
}

impl std::fmt::Display for ExpressionCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionCreationError::TokenizerError(err) => err.fmt(f),
            ExpressionCreationError::ParserError(err) => err.fmt(f),
        }
    }
}

impl From<TokenizerError> for ExpressionCreationError {
    fn from(value: TokenizerError) -> Self {
        Self::TokenizerError(value)
    }
}

impl From<ParserError> for ExpressionCreationError {
    fn from(value: ParserError) -> Self {
        Self::ParserError(value)
    }
}

impl Expression {
    /// Creates and expression from standard infix string.
    pub fn new(expr: &str) -> Result<Expression, ExpressionCreationError> {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize(expr)?;
        let rpn = tokens_to_rpn(tokens)?;
        let emitter = Emitter::new(rpn);

        Ok(Self {
            str_expr: expr.to_owned(),
            emitter,
        })
    }

    /// Used to bind variables to numbers.
    /// Takes a hashmap as an argument where the keys are the variable names and the values are f32
    /// Not needed if expression doesn't have any variables.
    pub fn bind_variables(&mut self, var_map: &VariableMap) -> Result<(), BindVariablesError> {
        self.emitter.bind_variables(var_map)
    }

    /// Evaluates the function. Panics if variables were not bound.
    pub fn eval(&self) -> Result<EmitResult, EvalError> {
        self.emitter.eval()
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_1() {
        let expr = Expression::new("1+2").unwrap();
        let res: f32 = expr.eval().unwrap().into();

        assert_eq!(res, 3.0);
    }

    #[test]
    fn test_2() {
        let expr = Expression::new("1 + 2 * 3").unwrap();
        let res: f32 = expr.eval().unwrap().into();

        assert_eq!(res, 7.0);
    }

    #[test]
    fn test_3() {
        let expr = Expression::new("(1 + 2) * 3").unwrap();
        let res: f32 = expr.eval().unwrap().into();

        assert_eq!(res, 9.0);
    }

    #[test]
    fn test_4() {
        let expr = Expression::new("5 * (3 + 4) - 2").unwrap();
        let res: f32 = expr.eval().unwrap().into();

        assert_eq!(res, 33.0);
    }

    #[test]
    fn test_5() {
        let mut expr = Expression::new("{a} > {b} && ({c} <= {d} || {e} >= {f}/{g})").unwrap();
        expr.bind_variables(&HashMap::from_iter(vec![
            ("a".to_string(), 3.0),
            ("b".to_string(), 5.0),
            ("c".to_string(), 9.0),
            ("d".to_string(), 4.0),
            ("e".to_string(), 1.0),
            ("f".to_string(), 9.0),
            ("g".to_string(), 3.0),
        ]))
        .unwrap();
        let false_res: bool = expr.eval().unwrap().into();

        assert_eq!(false_res, false);

        expr.bind_variables(&HashMap::from_iter(vec![
            ("a".to_string(), 6.0),
            ("b".to_string(), 5.0),
            ("c".to_string(), 9.0),
            ("d".to_string(), 4.0),
            ("e".to_string(), 4.0),
            ("f".to_string(), 9.0),
            ("g".to_string(), 3.0),
        ]))
        .unwrap();

        let true_res: bool = expr.eval().unwrap().into();

        assert_eq!(true_res, true);
    }

    #[test]
    fn test_6() {
        let expr = Expression::new("true || (false && true)").unwrap();
        let res: bool = expr.eval().unwrap().into();

        assert_eq!(res, true);
    }

    #[test]
    fn test_7() {
        let expr = Expression::new("true || false && true").unwrap();
        let res: bool = expr.eval().unwrap().into();

        assert_eq!(res, true);
    }
}
