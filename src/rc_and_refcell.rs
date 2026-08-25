#![allow(unused)]

// Ch3: Misusing Rc and RefCell

// This file demonstrates the anti-pattern of overusing Rc<RefCell<>>
// in a calculator implementation, followed by a better approach.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// COMMON STRUCTURES

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Variable(String),
    Operator(char),
}

// BAD APPROACH: Overusing Rc<RefCell<>>
// ANTI-PATTERN: Wrapping everything in Rc<RefCell<>> hides ownership
// and makes borrow rules implicit, leading to potential runtime panics.
//
// Rc<RefCell<T>> is used instead of just RefCell<T> because RefCell alone
// only allows a single owner; Rc provides shared ownership so multiple
// parts of the code can reference the same mutable data.

// Rc<RefCell<T>> lets multiple parts of the code hold references to the same mutable data, which is
// exactly the anti-pattern being demonstrated.

struct BadExpression {
    tokens: Rc<RefCell<Vec<Token>>>,
    result: Rc<RefCell<Option<f64>>>,
}

struct BadCalculator {
    current_expression: Rc<RefCell<Option<BadExpression>>>,
    variables: Rc<RefCell<HashMap<String, f64>>>,
}

impl BadCalculator {
    fn new() -> Self {
        Self {
            current_expression: Rc::new(RefCell::new(None)),
            variables: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn set_expression(&self, expr: &str) {
        let tokens = self.tokenize(expr);
        *self.current_expression.borrow_mut() = Some(BadExpression {
            tokens: Rc::new(RefCell::new(tokens)),
            result: Rc::new(RefCell::new(None)),
        });
    }

    fn tokenize(&self, expr: &str) -> Vec<Token> {
        // Simple tokenizer for demonstration
        let mut tokens = Vec::new();

        for part in expr.split_whitespace() {
            if let Ok(num) = part.parse::<f64>() {
                tokens.push(Token::Number(num));
            } else if "+-*/".contains(part) && part.len() == 1 {
                tokens.push(Token::Operator(part.chars().next().unwrap()));
            } else {
                tokens.push(Token::Variable(part.to_string()));
            }
        }

        tokens
    }

    fn evaluate(&self) -> Result<f64, String> {
        // ANTI-PATTERN: Multiple nested borrow_mut() calls can panic at runtime
        // if borrowed incorrectly, and ownership is unclear.
        let expr = self.current_expression.borrow();
        let expr = expr.as_ref().ok_or("No expression set")?;

        let mut tokens = expr.tokens.borrow_mut();
        let vars = self.variables.borrow();

        // Process tokens...
        let result = self.process_tokens(&mut tokens, &vars)?;

        *expr.result.borrow_mut() = Some(result);
        Ok(result)
    }

    fn set_variable(&self, name: &str, value: f64) {
        self.variables.borrow_mut().insert(name.to_string(), value);
    }

    fn process_tokens(
        &self,
        tokens: &mut [Token],
        variables: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        // Simple implementation for demonstration
        if tokens.len() != 3 {
            return Err("Only simple expressions supported".to_string());
        }

        let left = match &tokens[0] {
            Token::Number(n) => *n,
            Token::Variable(name) => *variables
                .get(name)
                .ok_or_else(|| format!("Unknown variable: {}", name))?,
            _ => return Err("Expected number or variable".to_string()),
        };

        let right = match &tokens[2] {
            Token::Number(n) => *n,
            Token::Variable(name) => *variables
                .get(name)
                .ok_or_else(|| format!("Unknown variable: {}", name))?,
            _ => return Err("Expected number or variable".to_string()),
        };

        let operator = match &tokens[1] {
            Token::Operator(op) => *op,
            _ => return Err("Expected operator".to_string()),
        };

        match operator {
            '+' => Ok(left + right),
            '-' => Ok(left - right),
            '*' => Ok(left * right),
            '/' => {
                if right == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left / right)
                }
            }
            _ => Err("Unknown operator".to_string()),
        }
    }
}

// BETTER APPROACH: Using clear ownership
// BETTER: Direct ownership with Vec and HashMap avoids hidden borrow rules.
// Mutation is explicit via &mut self, making the code safer and easier to reason about.

struct ParsedExpression {
    tokens: Vec<Token>,
}

struct Calculation {
    expression: String,
    tokens: Vec<Token>,
    result: f64,
}

struct Calculator {
    variables: HashMap<String, f64>,
    history: Vec<Calculation>,
}

impl Calculator {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            history: Vec::new(),
        }
    }

    fn parse(&self, expr: &str) -> Result<ParsedExpression, String> {
        // Tokenize the expression
        let mut tokens = Vec::new();

        for part in expr.split_whitespace() {
            if let Ok(num) = part.parse::<f64>() {
                tokens.push(Token::Number(num));
            } else if "+-*/".contains(part) && part.len() == 1 {
                tokens.push(Token::Operator(part.chars().next().unwrap()));
            } else {
                tokens.push(Token::Variable(part.to_string()));
            }
        }

        Ok(ParsedExpression { tokens })
    }

    fn evaluate_parsed(&mut self, expr: String, parsed: ParsedExpression) -> Result<f64, String> {
        // Resolve variables
        let mut resolved_tokens = Vec::new();

        for token in &parsed.tokens {
            match token {
                Token::Variable(name) => {
                    let value = self
                        .variables
                        .get(name)
                        .ok_or_else(|| format!("Unknown variable: {}", name))?;
                    resolved_tokens.push(Token::Number(*value));
                }
                _ => resolved_tokens.push(token.clone()),
            }
        }

        // Simple evaluation for demonstration
        if resolved_tokens.len() != 3 {
            return Err("Only simple expressions supported".to_string());
        }

        let left = match &resolved_tokens[0] {
            Token::Number(n) => *n,
            _ => return Err("Expected number".to_string()),
        };

        let right = match &resolved_tokens[2] {
            Token::Number(n) => *n,
            _ => return Err("Expected number".to_string()),
        };

        let operator = match &resolved_tokens[1] {
            Token::Operator(op) => *op,
            _ => return Err("Expected operator".to_string()),
        };

        let result = match operator {
            '+' => left + right,
            '-' => left - right,
            '*' => left * right,
            '/' => {
                if right == 0.0 {
                    return Err("Division by zero".to_string());
                }
                left / right
            }
            _ => return Err("Unknown operator".to_string()),
        };

        // Store in history
        self.history.push(Calculation {
            expression: expr,
            tokens: parsed.tokens,
            result,
        });

        Ok(result)
    }

    fn evaluate(&mut self, expr: String) -> Result<f64, String> {
        let parsed = self.parse(&expr)?;
        self.evaluate_parsed(expr, parsed)
    }

    fn set_variable(&mut self, name: String, value: f64) {
        self.variables.insert(name, value);
    }

    fn history(&self) -> &[Calculation] {
        &self.history
    }

    fn last_result(&self) -> Option<f64> {
        self.history.last().map(|calc| calc.result)
    }
}

// Thread-safe version when truly needed
// BETTER: Only use Arc<Mutex<>> when actual cross-thread sharing is required.
// For single-threaded use, direct ownership is simpler and faster.

struct ThreadSafeCalculator {
    inner: Arc<Mutex<Calculator>>,
}

impl ThreadSafeCalculator {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Calculator::new())),
        }
    }

    fn evaluate(&self, expr: String) -> Result<f64, String> {
        let mut calc = self.inner.lock().map_err(|_| "Lock poisoned")?;
        calc.evaluate(expr)
    }

    fn set_variable(&self, name: String, value: f64) -> Result<(), String> {
        let mut calc = self.inner.lock().map_err(|_| "Lock poisoned")?;
        calc.set_variable(name, value);
        Ok(())
    }
}

fn main() {
    println!("--- BAD APPROACH: Overusing Rc<RefCell<>> ---");
    let bad_calc = BadCalculator::new();

    bad_calc.set_variable("x", 10.0);
    bad_calc.set_variable("y", 5.0);

    bad_calc.set_expression("x + y");
    match bad_calc.evaluate() {
        Ok(result) => println!("x + y = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n--- BETTER APPROACH: Clear Ownership ---");
    let mut calc = Calculator::new();

    calc.set_variable("x".to_string(), 10.0);
    calc.set_variable("y".to_string(), 5.0);

    match calc.evaluate("x + y".to_string()) {
        Ok(result) => println!("x + y = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    println!("Last result: {:?}", calc.last_result());

    println!("\n--- THREAD-SAFE VERSION ---");
    let thread_safe = ThreadSafeCalculator::new();

    thread_safe.set_variable("a".to_string(), 7.0).unwrap();
    thread_safe.set_variable("b".to_string(), 3.0).unwrap();

    match thread_safe.evaluate("a - b".to_string()) {
        Ok(result) => println!("a - b = {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numbers_and_operators() {
        let calc = Calculator::new();
        let parsed = calc.parse("1 + 2").unwrap();
        assert_eq!(parsed.tokens.len(), 3);
        assert_eq!(parsed.tokens[0], Token::Number(1.0));
        assert_eq!(parsed.tokens[1], Token::Operator('+'));
        assert_eq!(parsed.tokens[2], Token::Number(2.0));
    }

    #[test]
    fn parse_variables() {
        let calc = Calculator::new();
        let parsed = calc.parse("x + y").unwrap();
        assert_eq!(parsed.tokens.len(), 3);
        assert_eq!(parsed.tokens[0], Token::Variable("x".to_string()));
        assert_eq!(parsed.tokens[1], Token::Operator('+'));
        assert_eq!(parsed.tokens[2], Token::Variable("y".to_string()));
    }

    #[test]
    fn parse_complex_expression() {
        let calc = Calculator::new();
        let parsed = calc.parse("10 + 20 * 3").unwrap();
        assert_eq!(parsed.tokens.len(), 5);
        assert_eq!(parsed.tokens[0], Token::Number(10.0));
        assert_eq!(parsed.tokens[1], Token::Operator('+'));
        assert_eq!(parsed.tokens[2], Token::Number(20.0));
        assert_eq!(parsed.tokens[3], Token::Operator('*'));
        assert_eq!(parsed.tokens[4], Token::Number(3.0));
    }

    #[test]
    fn parse_mixed_variables_and_numbers() {
        let calc = Calculator::new();
        let parsed = calc.parse("x + 5 - y").unwrap();
        assert_eq!(parsed.tokens.len(), 5);
        assert_eq!(parsed.tokens[0], Token::Variable("x".to_string()));
        assert_eq!(parsed.tokens[1], Token::Operator('+'));
        assert_eq!(parsed.tokens[2], Token::Number(5.0));
        assert_eq!(parsed.tokens[3], Token::Operator('-'));
        assert_eq!(parsed.tokens[4], Token::Variable("y".to_string()));
    }

    #[test]
    fn parse_empty_string() {
        let calc = Calculator::new();
        let parsed = calc.parse("").unwrap();
        assert!(parsed.tokens.is_empty());
    }

    #[test]
    fn parse_multiple_operators() {
        let calc = Calculator::new();
        let parsed = calc.parse("1 + 2 + 3").unwrap();
        assert_eq!(parsed.tokens.len(), 5);
        assert_eq!(parsed.tokens[0], Token::Number(1.0));
        assert_eq!(parsed.tokens[1], Token::Operator('+'));
        assert_eq!(parsed.tokens[2], Token::Number(2.0));
        assert_eq!(parsed.tokens[3], Token::Operator('+'));
        assert_eq!(parsed.tokens[4], Token::Number(3.0));
    }

    #[test]
    fn parse_negative_numbers_as_numbers() {
        // "-5" parses as a number, not a variable
        let calc = Calculator::new();
        let parsed = calc.parse("-5 + 3").unwrap();
        assert_eq!(parsed.tokens.len(), 3);
        assert_eq!(parsed.tokens[0], Token::Number(-5.0));
        assert_eq!(parsed.tokens[1], Token::Operator('+'));
        assert_eq!(parsed.tokens[2], Token::Number(3.0));
    }

    #[test]
    fn evaluate_parsed_success_numbers() {
        let mut calc = Calculator::new();
        let parsed = calc.parse("10 + 5").unwrap();
        let result = calc.evaluate_parsed("10 + 5".to_string(), parsed).unwrap();
        assert_eq!(result, 15.0);
    }

    #[test]
    fn evaluate_parsed_success_variables() {
        let mut calc = Calculator::new();
        calc.set_variable("x".to_string(), 10.0);
        calc.set_variable("y".to_string(), 3.0);
        let parsed = calc.parse("x * y").unwrap();
        let result = calc.evaluate_parsed("x * y".to_string(), parsed).unwrap();
        assert_eq!(result, 30.0);
    }

    #[test]
    fn evaluate_parsed_division_by_zero() {
        let mut calc = Calculator::new();
        let parsed = calc.parse("10 / 0").unwrap();
        let result = calc.evaluate_parsed("10 / 0".to_string(), parsed);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Division by zero"));
    }

    #[test]
    fn evaluate_parsed_unknown_variable() {
        let mut calc = Calculator::new();
        let parsed = calc.parse("z + 1").unwrap();
        let result = calc.evaluate_parsed("z + 1".to_string(), parsed);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown variable"));
    }

    #[test]
    fn evaluate_parsed_history_recorded() {
        let mut calc = Calculator::new();
        let parsed = calc.parse("2 + 3").unwrap();
        calc.evaluate_parsed("2 + 3".to_string(), parsed).unwrap();
        assert_eq!(calc.history().len(), 1);
        assert_eq!(calc.history()[0].expression, "2 + 3");
        assert_eq!(calc.history()[0].result, 5.0);
        assert_eq!(calc.last_result(), Some(5.0));
    }

    #[test]
    fn evaluate_parsed_too_many_tokens() {
        let mut calc = Calculator::new();
        let parsed = calc.parse("1 + 2 + 3").unwrap();
        let result = calc.evaluate_parsed("1 + 2 + 3".to_string(), parsed);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Only simple expressions supported"));
    }
}
