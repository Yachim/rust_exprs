use crate::{
    emitter::{EmitResult, Emitter, VariableMap},
    parser::tokens_to_rpn,
    tokenizer::Tokenizer,
};

#[derive(Debug)]
pub struct Expression {
    pub str_expr: String,
    emitter: Emitter,
}

impl Expression {
    /// Creates and expression from standard infix string.
    pub fn new(expr: &str) -> Self {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize(&expr);
        let rpn = tokens_to_rpn(tokens);
        let emitter = Emitter::new(rpn);

        Self {
            str_expr: expr.to_owned(),
            emitter,
        }
    }

    /// Used to bind variables to numbers.
    /// Takes a hashmap as an argument where the keys are the variable names and the values are f32
    /// Not needed if expression doesn't have any variables.
    pub fn bind_variables(&mut self, var_map: &VariableMap) {
        self.emitter.bind_variables(var_map);
    }

    /// Evaluates the function. Panics if variables were not bound.
    pub fn eval(&self) -> EmitResult {
        self.emitter.eval()
    }
}

#[cfg(test)]
mod tests {
    use super::EmitResult;
    use crate::expression::Expression;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_1() {
        let expr = Expression::new("1+2");
        assert_eq!(expr.eval(), EmitResult::Number(3.0));
    }

    #[test]
    fn test_2() {
        let expr = Expression::new("1 + 2 * 3");
        let res: f32 = expr.eval().into();

        assert_eq!(res, 7.0);
    }

    #[test]
    fn test_3() {
        let expr = Expression::new("(1 + 2) * 3");
        let res: f32 = expr.eval().into();

        assert_eq!(res, 9.0);
    }

    #[test]
    fn test_4() {
        let expr = Expression::new("5 * (3 + 4) - 2");
        let res: f32 = expr.eval().into();

        assert_eq!(res, 33.0);
    }

    #[test]
    fn test_5() {
        let mut expr = Expression::new("{a} > {b} && ({c} <= {d} || {e} >= {f}/{g})");
        expr.bind_variables(&HashMap::from_iter(vec![
            ("a".to_string(), 3.0),
            ("b".to_string(), 5.0),
            ("c".to_string(), 9.0),
            ("d".to_string(), 4.0),
            ("e".to_string(), 1.0),
            ("f".to_string(), 9.0),
            ("g".to_string(), 3.0),
        ]));
        let false_res: bool = expr.eval().into();

        assert_eq!(false_res, false);

        expr.bind_variables(&HashMap::from_iter(vec![
            ("a".to_string(), 6.0),
            ("b".to_string(), 5.0),
            ("c".to_string(), 9.0),
            ("d".to_string(), 4.0),
            ("e".to_string(), 4.0),
            ("f".to_string(), 9.0),
            ("g".to_string(), 3.0),
        ]));

        let true_res: bool = expr.eval().into();

        assert_eq!(true_res, true);
    }
}
