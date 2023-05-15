use crate::parser::{OperatorType, Value, RPN};
use std::collections::{HashMap, VecDeque};

pub type VariableMap = HashMap<String, f32>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EmitResult {
    Number(f32),
    Bool(bool),
}

impl Into<bool> for EmitResult {
    fn into(self) -> bool {
        match self {
            Self::Bool(val) => val,
            Self::Number(_) => panic!("Value is number. Not a boolean."),
        }
    }
}

impl Into<f32> for EmitResult {
    fn into(self) -> f32 {
        match self {
            Self::Number(val) => val,
            Self::Bool(_) => panic!("Value is boolean. Not a number."),
        }
    }
}

#[derive(Debug)]
pub struct Emitter {
    rpn: RPN,
    no_var_rpn: Option<RPN>,
}

impl Emitter {
    pub fn new(rpn: RPN) -> Self {
        let contains_variable = rpn.iter().any(|value| match value {
            Value::Variable(_) => true,
            _ => false,
        });

        let no_var_rpn = if contains_variable {
            None
        } else {
            Some(rpn.to_owned())
        };

        Self { rpn, no_var_rpn }
    }

    /// replaces variable values with numbers
    pub fn bind_variables(&mut self, var_map: &VariableMap) {
        self.no_var_rpn = Some(
            self.rpn
                .iter()
                .map(|value| match value {
                    Value::Variable(name) => Value::Number(
                        *var_map
                            .get(name)
                            .expect(&format!("Variable {name} does not exist.")),
                    ),
                    val => val.to_owned(),
                })
                .collect(),
        );
    }

    /// no_var_rpn cannot be empty
    pub fn eval(&self) -> EmitResult {
        assert!(self.no_var_rpn != None);
        let mut rpn = self
            .no_var_rpn
            .clone()
            .expect("The provided rpn contains variables.");

        // stack - 0th index in, 0th index out
        let mut value_stack: VecDeque<EmitResult> = VecDeque::new();

        while rpn.len() > 0 {
            let val = rpn.remove(0);
            match val {
                Value::Number(num) => value_stack.push_front(EmitResult::Number(num)),
                Value::Operator(op) => {
                    assert!(value_stack.len() >= 2, "Not enough values entered.");
                    let second_val = value_stack.pop_front().unwrap();
                    let first_val = value_stack.pop_front().unwrap();

                    let val: EmitResult = match op {
                        OperatorType::LeftParenthesis => {
                            unreachable!("RPN cannot have parentheses.")
                        }
                        OperatorType::Plus
                        | OperatorType::Minus
                        | OperatorType::Times
                        | OperatorType::Divide => {
                            let EmitResult::Number(first) = first_val else {panic!("Value is not number. Value: {first_val:#?}.")};
                            let EmitResult::Number(second) = second_val else {panic!("Value is not number. Value: {second_val:#?}.")};
                            EmitResult::Number(op.eval_nums(first, second))
                        }
                        OperatorType::LT
                        | OperatorType::LE
                        | OperatorType::GT
                        | OperatorType::GE
                        | OperatorType::Eq => {
                            let EmitResult::Number(first) = first_val else {panic!("Value is not number. Value: {first_val:#?}.")};
                            let EmitResult::Number(second) = second_val else {panic!("Value is not number. Value: {second_val:#?}.")};
                            EmitResult::Bool(op.eval_comparison(first, second))
                        }
                        OperatorType::And | OperatorType::Or => {
                            let EmitResult::Bool(first) = first_val else {panic!("Value is not bool. Value: {first_val:#?}.")};
                            let EmitResult::Bool(second) = second_val else {panic!("Value is not bool. Value: {second_val:#?}.")};
                            EmitResult::Bool(op.eval_conditional(first, second))
                        }
                    };
                    value_stack.push_front(val);
                }
                Value::Variable(_) => {
                    unreachable!("RPN cannot contain variables. Use `bind_variables` first.")
                }
            };
        }

        assert_eq!(value_stack.len(), 1, "Too much values entered.");

        value_stack[0]
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
        emitter.bind_variables(&var_map);
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
        emitter.bind_variables(&var_map);
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

        assert_eq!(emitter.eval(), EmitResult::Number(3.0));
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

        assert_eq!(emitter.eval(), EmitResult::Number(7.0));
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

        assert_eq!(emitter.eval(), EmitResult::Number(9.0));
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

        assert_eq!(emitter.eval(), EmitResult::Number(33.0));
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
        emitter.bind_variables(&HashMap::from_iter(vec![
            ("a".to_string(), 3.0),
            ("b".to_string(), 5.0),
            ("c".to_string(), 9.0),
            ("d".to_string(), 4.0),
            ("e".to_string(), 1.0),
            ("f".to_string(), 9.0),
            ("g".to_string(), 3.0),
        ]));

        assert_eq!(emitter.eval(), EmitResult::Bool(false));

        emitter.bind_variables(&HashMap::from_iter(vec![
            ("a".to_string(), 6.0),
            ("b".to_string(), 5.0),
            ("c".to_string(), 9.0),
            ("d".to_string(), 4.0),
            ("e".to_string(), 4.0),
            ("f".to_string(), 9.0),
            ("g".to_string(), 3.0),
        ]));

        assert_eq!(emitter.eval(), EmitResult::Bool(true));
    }
}
