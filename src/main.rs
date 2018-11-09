use std::io::{self, BufRead, Write};
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::collections::VecDeque;
use std::f64::consts;

// We need a VecDeque because we need to also push to the back.
// Using a regular vec would require dissolving the entire stack, 
// just to push one element, and then add everything back.
// This is more efficient.
// A LinkedList could also be used, but the VecDeque has better locality.
type Stack = VecDeque<f64>;

const PHI: f64 = 1.61803398875;

/// Every available operation in the calculator
enum StackOp {
    // binary operations
    Add, // addition
    Sub, // subtraction
    Mul, // multiplication
    Div, // division
    Pow, // power
    // unary operations
    Sqrt, // square root
    Neg,  // negation
    Abs,  // absolute
    Ln, Log, Lg, // log-e, log-10, and log-2
    Sin, Asin,   // sin and its inverse
    Cos, Acos,   // cos and its inverse
    Tan, Atan,   // tan and its inverse
    ToDeg, // converts a (radian) number to degrees
    ToRad, // converts a (degree) number to radians
    // stack operations
    Sum,       // Sums the entire stack
    Prod,      // Multiplies the entire stack
    Pop,       // pops an item off the stack
    Clear,     // clears the stack
    Swap,      // swaps the two topmost elements
    Rotate,    // pushes the front to the back
    Duplicate, // duplicates the topmost element
    // other
    NoOp, // no operation (error case)
    Num(f64), // a number
}

/// Prints the list of available commands to the console
fn print_help() -> StackOp {
    println!("List of available commands: ");
    println!("help, ? -- print this help");
    println!("<number> -- Pushes a number to the stack");
    println!("pi -- Pushes π onto the stack");
    println!("e -- Pushes e onto the stack");
    println!("phi -- Pushes the golden ratio onto the stack");
    println!("+, -, *, /, ^ -- Applies the respective binary operation");
    println!("sqrt -- Takes the square root of the last number");
    println!("neg -- Negates the last number");
    println!("abs -- Makes the last number positive");
    println!("ln -- Applies the natural log to the last number");
    println!("lg, log2 -- Applies the base-2 log to the last number");
    println!("log, log10 -- Applies the base-10 log to the last number");
    println!("sin -- Applies the sine of the last number (in radians)");
    println!("cos -- Applies the cosine of the last number (in radians)");
    println!("tan -- Applies the tangent of the last number (in radians)");
    println!("to deg -- Converts a number (in radians) to degrees");
    println!("to rad -- Converts a number (in degrees) to radians");
    println!("sum -- Add the entire stack together");
    println!("prod -- Multiplies the entire stack together");
    println!("pop -- Removes the topmost number");
    println!("clear -- Clears the stack");
    println!("swap -- Swaps the two topmost numbers");
    println!("rotate -- Moves the first number to the end of the stack");
    StackOp::NoOp
}

/// Parses a string and returns a stack-operator
fn parse_string(input: &str) -> StackOp {
    use StackOp::*;

    match input.trim() {
        // binary operations
        "+" | "add" => Add,
        "-" | "sub" | "subtract" => Sub,
        "*" | "mul" | "multiply" => Mul,
        "/" | "div" | "divide" => Div,
        "^" | "pow" | "power" => Pow,
        // unary operations
        "abs" | "absolute" => Abs,
        "sqrt" | "root" => Sqrt,
        "neg" | "negate" | "~" => Neg,
        "ln" | "loge" => Ln,
        "log" | "log10" => Log,
        "lg" | "log2" => Lg,
        "sin" => Sin,
        "asin" | "sin^-1" => Asin,
        "cos" => Cos,
        "acos" | "cos^-1" => Acos,
        "tan" => Tan,
        "atan" | "tan^-1" => Atan,
        "deg" | "to deg" => ToDeg,
        "rad" | "to rad" => ToRad,
        // constants
        "pi" | "π" => Num(consts::PI),
        "e" => Num(consts::E),
        "phi" | "φ" | "ϕ" => Num(PHI),
        // stack operations
        "sum" => Sum,
        "prod" => Prod,
        "pop" => Pop,
        "clear" | "cls" => Clear,
        "swap" => Swap,
        "rotate" | "rot" => Rotate,
        "copy" | "clone" | "duplicate" => Duplicate,
        // other
        "help" | "?" => print_help(),
        "quit" | "q" | "end" => {
            println!("To quit, press ctrl+c");
            NoOp
        },
        // number
        str => {
            if let Ok(num) = str.parse::<f64>() {
                Num(num)
            } else {
                println!("Error! Couldn't parse {}", str);
                NoOp
            }
        }
    }
}

/// Prompts the user for an input from the console.
fn get_input() -> io::Result<StackOp> {
    let mut buff = String::new();
    let stdin = io::stdin();

    print!("> ");
    io::stdout().flush()?;
    stdin.lock().read_line(&mut buff)?;
    buff = buff.to_lowercase(); // ensure lowercase

    Ok(parse_string(&buff))
}

/// Applies a binary operation if the stack has enough elements.
/// If not, nothing happens.
/// NB The top of the stack holds the SECOND operator, not the first
/// So if we push 2 1 - the operation becomes 2 - 1, not 1 - 2
fn eval_binop<F>(stack: &mut Stack, fun: F)
where
    F: FnOnce(f64, f64) -> f64,
{
    if stack.len() >= 2 {
        // we know it's safe to unwrap, because the stack has at least 2 numbers
        let a = stack.pop_back().unwrap();
        let b = stack.pop_back().unwrap();
        stack.push_back(fun(b, a));
    }
}

/// Applies a unary operation if the stack has enough elements.
/// If not, nothing happens.
fn eval_unop<F>(stack: &mut Stack, fun: F)
where
    F: FnOnce(f64) -> f64,
{
    if let Some(a) = stack.pop_back() {
        stack.push_back(fun(a));
    }
}

// Folds the stack over fun, then pushes the result.
fn eval_stackop<F>(stack: &mut Stack, start: f64, fun: F)
where
    F: FnMut(f64, &f64) -> f64,
{
    let result = stack.iter().fold(start, fun);
    stack.clear();
    stack.push_back(result);
}

/// Swaps the two topmost elements of the stack
fn swap(stack: &mut Stack) {
    if stack.len() >= 2 {
        let a = stack.pop_back().unwrap();
        let b = stack.pop_back().unwrap();
        stack.push_back(a);
        stack.push_back(b);
    }
}

/// Moves the topmost element to the bottom
fn rotate(stack: &mut Stack) {
    if let Some(num) = stack.pop_back() {
        stack.push_front(num); // Important! This must be the opposite of the pop
    }
}

/// Duplicates the topmost element of the stack
fn duplicate(stack: &mut Stack) {
    if let Some(&num) = stack.back() {
        stack.push_back(num);
    }
}

/// Determines what to do given a StackOp, and applies its effect to the stack
fn eval(stack: &mut Stack, last_op: StackOp) {
    use StackOp::*;

    match last_op {
        // binary operators
        Add => eval_binop(stack, f64::add),
        Sub => eval_binop(stack, f64::sub),
        Mul => eval_binop(stack, f64::mul),
        Div => eval_binop(stack, f64::div),
        Pow => eval_binop(stack, f64::powf),
        // unary operators
        Sqrt  => eval_unop(stack, f64::sqrt),
        Abs   => eval_unop(stack, f64::abs),
        Neg   => eval_unop(stack, f64::neg),
        Ln    => eval_unop(stack, f64::ln),
        Lg    => eval_unop(stack, f64::log2),
        Log   => eval_unop(stack, f64::log10),
        Sin   => eval_unop(stack, f64::sin),
        Asin  => eval_unop(stack, f64::asin),
        Cos   => eval_unop(stack, f64::cos),
        Acos  => eval_unop(stack, f64::acos),
        Tan   => eval_unop(stack, f64::tan),
        Atan  => eval_unop(stack, f64::atan),
        ToDeg => eval_unop(stack, f64::to_degrees),
        ToRad => eval_unop(stack, f64::to_radians),
        // stack operations
        Sum       => eval_stackop(stack, 0.0, |acc, x| acc + x),
        Prod      => eval_stackop(stack, 1.0, |acc, x| acc * x),
        Pop       => { stack.pop_back(); }, // brackets required to ignore result of pop_back
        Clear     => stack.clear(),
        Swap      => swap(stack),
        Rotate    => rotate(stack),
        Duplicate => duplicate(stack),
        // number
        Num(n) => stack.push_back(n),
        // other
        NoOp => return, // do nothing
    }
}

fn main() -> io::Result<()> {
    println!("Welcome to the stack calculator!");
    println!("Type \"help\" and hit return to view available commands.");
    let mut stack = VecDeque::new();
    loop {
        let input = get_input()?;
        eval(&mut stack, input);

        if stack.len() >= 1 {
            println!("Stack: {:.2?}", stack);
        }
    }
}
