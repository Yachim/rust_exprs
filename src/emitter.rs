use crate::parser::{OperatorType, Rpn, Value};
use std::collections::{HashMap, VecDeque};

pub type VariableMap = HashMap<String, f32>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EmitResult {
    Number(f32),
    Boolean(bool),
}

impl From<EmitResult> for bool {
    fn from(value: EmitResult) -> Self {
        match value {
            EmitResult::Boolean(val) => val,
            EmitResult::Number(_) => panic!("value is number, not a boolean"),
        }
    }
}

impl From<EmitResult> for f32 {
    fn from(value: EmitResult) -> Self {
        match value {
            EmitResult::Number(val) => val,
            EmitResult::Boolean(_) => panic!("value is boolean, not a number"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Emitter {
    rpn: Rpn,
    no_var_rpn: Option<Rpn>,
}

#[derive(Debug, PartialEq)]
pub enum EvalError {
    NoVariables,
    NotEnoughValues,
    TooMuchValues,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::NoVariables => {
                write!(f, "variables not provided, use `bind_variables` first")
            }
            EvalError::NotEnoughValues => write!(f, "not enough values entered"),
            EvalError::TooMuchValues => write!(f, "too much values entered"),
        }
    }
}

impl Emitter {
    pub fn new(rpn: Rpn) -> Self {
        let contains_variable = rpn.iter().any(|value| matches!(value, Value::Variable(_)));

        let no_var_rpn = if contains_variable {
            None
        } else {
            Some(rpn.to_owned())
        };

        Self { rpn, no_var_rpn }
    }

    /// replaces variable values with numbers
    pub fn bind_variables(&mut self, var_map: &VariableMap) -> Result<(), String> {
        self.no_var_rpn = Some(
            self.rpn
                .iter()
                .map(|value| match value {
                    Value::Variable(name) => Ok(Value::Number(
                        *var_map
                            .get(name)
                            .ok_or(format!("variable {name} does not exist"))?,
                    )),
                    val => Ok(val.to_owned()),
                })
                .collect::<Result<Rpn, String>>()?,
        );

        Ok(())
    }

    /// no_var_rpn cannot be empty
    pub fn eval(&self) -> Result<EmitResult, EvalError> {
        if self.no_var_rpn.is_none() {
            return Err(EvalError::NoVariables);
        }
        let mut rpn = self.no_var_rpn.clone().ok_or(EvalError::NoVariables)?;

        // stack - 0th index in, 0th index out
        let mut value_stack: VecDeque<EmitResult> = VecDeque::new();

        while !rpn.is_empty() {
            let val = rpn.remove(0);
            match val {
                Value::Number(num) => value_stack.push_front(EmitResult::Number(num)),
                Value::Boolean(boolean) => value_stack.push_front(EmitResult::Boolean(boolean)),
                Value::Operator(op) => {
                    if value_stack.len() < 2 {
                        return Err(EvalError::NotEnoughValues);
                    }
                    let second_val = value_stack.pop_front().unwrap_or_else(|| {
                        panic!("could not pop first value of stack: {:#?}", value_stack)
                    });
                    let first_val = value_stack.pop_front().unwrap_or_else(|| {
                        panic!("could not pop first value of stack: {:#?}", value_stack)
                    });

                    // unreachable because the rpn should not be created manually
                    let val: EmitResult = match op {
                        OperatorType::LeftParenthesis => {
                            unreachable!("Rpn cannot have parentheses.")
                        }
                        OperatorType::Plus
                        | OperatorType::Minus
                        | OperatorType::Times
                        | OperatorType::Divide => {
                            let EmitResult::Number(first) = first_val else {unreachable!("value is not number, value: {first_val:#?}")};
                            let EmitResult::Number(second) = second_val else {unreachable!("value is not number, value: {second_val:#?}")};
                            EmitResult::Number(op.eval_nums(first, second))
                        }
                        OperatorType::LT
                        | OperatorType::LE
                        | OperatorType::GT
                        | OperatorType::GE
                        | OperatorType::Eq => {
                            let EmitResult::Number(first) = first_val else {unreachable!("value is not number, value: {first_val:#?}")};
                            let EmitResult::Number(second) = second_val else {unreachable!("value is not number, value: {second_val:#?}")};
                            EmitResult::Boolean(op.eval_comparison(first, second))
                        }
                        OperatorType::And | OperatorType::Or => {
                            let EmitResult::Boolean(first) = first_val else {unreachable!("value is not bool, value: {first_val:#?}")};
                            let EmitResult::Boolean(second) = second_val else {unreachable!("value is not bool, value: {second_val:#?}")};
                            EmitResult::Boolean(op.eval_conditional(first, second))
                        }
                    };
                    value_stack.push_front(val);
                }
                Value::Variable(_) => {
                    unreachable!("Rpn cannot contain variables, use `bind_variables` first")
                }
            };
        }

        if value_stack.len() > 1 {
            println!("{value_stack:#?}{:#?}", self.no_var_rpn);
            return Err(EvalError::TooMuchValues);
        }

        Ok(value_stack[0])
    }
}

#[cfg(test)]
mod tests {
    use super::{EmitResult, Emitter};
    use crate::{
        emitter::VariableMap,
        parser::{OperatorType, Value},
    };
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_new() {
        // 1 + 2
        let no_vars = Emitter::new(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Operator(OperatorType::Plus),
        ]);

        assert_eq!(
            no_vars.no_var_rpn,
            Some(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Operator(OperatorType::Plus),
            ])
        );

        let with_vars = Emitter::new(vec![
            Value::Number(1.0),
            Value::Variable("x".to_string()),
            Value::Operator(OperatorType::Plus),
        ]);

        assert_eq!(with_vars.no_var_rpn, None);
    }

    #[test]
    fn test_bind_variables() {
        let mut emitter = Emitter::new(vec![
            Value::Number(1.0),
            Value::Variable("x".to_string()),
            Value::Operator(OperatorType::Plus),
        ]);

        let var_map: VariableMap = HashMap::from_iter(vec![("x".to_string(), 3.0)]);
        emitter.bind_variables(&var_map).unwrap();
        assert_eq!(
            emitter.no_var_rpn,
            Some(vec![
                Value::Number(1.0),
                Value::Number(3.0),
                Value::Operator(OperatorType::Plus),
            ])
        );
    }

    #[test]
    #[should_panic]
    fn test_bind_variables_panic() {
        let mut emitter = Emitter::new(vec![
            Value::Number(1.0),
            Value::Variable("x".to_string()),
            Value::Operator(OperatorType::Plus),
        ]);

        let var_map: VariableMap = HashMap::from_iter(vec![("y".to_string(), 3.0)]);
        emitter.bind_variables(&var_map).unwrap();
        assert_eq!(
            emitter.no_var_rpn,
            Some(vec![
                Value::Number(1.0),
                Value::Number(3.0),
                Value::Operator(OperatorType::Plus),
            ])
        );
    }

    #[test]
    fn test_1() {
        // 1 + 2
        let rpn = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Operator(OperatorType::Plus),
        ];
        let emitter = Emitter::new(rpn);

        assert_eq!(emitter.eval().unwrap(), EmitResult::Number(3.0));
    }

    #[test]
    fn test_2() {
        // 1 + 2 * 3
        let rpn = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Operator(OperatorType::Times),
            Value::Operator(OperatorType::Plus),
        ];
        let emitter = Emitter::new(rpn);

        assert_eq!(emitter.eval().unwrap(), EmitResult::Number(7.0));
    }

    #[test]
    fn test_3() {
        // (1 + 2) * 3
        let rpn = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Operator(OperatorType::Plus),
            Value::Number(3.0),
            Value::Operator(OperatorType::Times),
        ];
        let emitter = Emitter::new(rpn);

        assert_eq!(emitter.eval().unwrap(), EmitResult::Number(9.0));
    }

    #[test]
    fn test_4() {
        // 5 * (3 + 4) - 2
        let rpn = vec![
            Value::Number(5.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Operator(OperatorType::Plus),
            Value::Operator(OperatorType::Times),
            Value::Number(2.0),
            Value::Operator(OperatorType::Minus),
        ];
        let emitter = Emitter::new(rpn);

        assert_eq!(emitter.eval().unwrap(), EmitResult::Number(33.0));
    }

    #[test]
    fn test_5() {
        // a > b && (c <= d || e >= f/g)
        let rpn = vec![
            Value::Variable("a".to_string()),
            Value::Variable("b".to_string()),
            Value::Operator(OperatorType::GT),
            Value::Variable("c".to_string()),
            Value::Variable("d".to_string()),
            Value::Operator(OperatorType::LE),
            Value::Variable("e".to_string()),
            Value::Variable("f".to_string()),
            Value::Variable("g".to_string()),
            Value::Operator(OperatorType::Divide),
            Value::Operator(OperatorType::GE),
            Value::Operator(OperatorType::Or),
            Value::Operator(OperatorType::And),
        ];
        let mut emitter = Emitter::new(rpn);
        emitter
            .bind_variables(&HashMap::from_iter(vec![
                ("a".to_string(), 3.0),
                ("b".to_string(), 5.0),
                ("c".to_string(), 9.0),
                ("d".to_string(), 4.0),
                ("e".to_string(), 1.0),
                ("f".to_string(), 9.0),
                ("g".to_string(), 3.0),
            ]))
            .unwrap();

        assert_eq!(emitter.eval().unwrap(), EmitResult::Boolean(false));

        emitter
            .bind_variables(&HashMap::from_iter(vec![
                ("a".to_string(), 6.0),
                ("b".to_string(), 5.0),
                ("c".to_string(), 9.0),
                ("d".to_string(), 4.0),
                ("e".to_string(), 4.0),
                ("f".to_string(), 9.0),
                ("g".to_string(), 3.0),
            ]))
            .unwrap();

        assert_eq!(emitter.eval().unwrap(), EmitResult::Boolean(true));
    }

    #[test]
    fn test_6() {
        let rpn = vec![Value::Boolean(true)];
        let emitter = Emitter::new(rpn);
        assert_eq!(emitter.eval().unwrap(), EmitResult::Boolean(true));

        let rpn = vec![Value::Boolean(false)];
        let emitter = Emitter::new(rpn);
        assert_eq!(emitter.eval().unwrap(), EmitResult::Boolean(false));
    }

    #[test]
    fn test_7() {
        let rpn = vec![
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Operator(OperatorType::And),
            Value::Boolean(true),
            Value::Operator(OperatorType::Or),
        ];
        let emitter = Emitter::new(rpn);
        assert_eq!(emitter.eval().unwrap(), EmitResult::Boolean(true));
    }
}
