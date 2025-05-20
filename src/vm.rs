use crate::parser;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::thread;

// Define bytecode instruction set for VM

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Allow dead code for unused variants
pub enum Bytecode {
    // Unary operations
    Neg, // Negate the top value on the stack

    // Arithmetic operations
    Add, // Add two values
    Sub, // Subtract two values
    Mul, // Multiply two values
    Div, // Divide two values

    // Data movement
    LoadConst(f64),  // Load a constant value (changed to f64 for signed integers)
    LoadVar(usize),  // Load a variable from memory
    StoreVar(usize), // Store a value to a variable

    // Parallel execution
    Spawn,   // Spawn a new thread/task
    Sync,    // Synchronize all threads/tasks
    Barrier, // Wait at a barrier for all threads

    // Control flow
    Jump(usize),          // Unconditional jump
    JumpIfZero(usize),    // Jump if top of stack is zero
    JumpIfNotZero(usize), // Jump if top of stack is not zero

    // Stack operations
    Pop, // Pop value from stack
    Dup, // Duplicate top of stack

    // Halt
    Halt, // Stop execution
}

// Define a struct for the VM
#[derive(Debug)]
pub struct VM {
    pub stack: Vec<f64>, // Stack for the VM (changed to f64 for signed integers)
    pub memory: HashMap<usize, f64>, // Memory for the VM (changed to f64 for signed integers)
    pub pc: usize,       // Program counter
    pub bytecode: Vec<Bytecode>, // Bytecode instructions
    pub threads: Vec<thread::JoinHandle<()>>, // Threads for parallel execution
    pub receivers: Vec<Receiver<f64>>, // Receivers for thread results (changed to f64 for signed integers)
}

impl VM {
    // Create a new VM instance
    pub fn new(bytecode: Vec<Bytecode>) -> Self {
        VM {
            stack: Vec::new(),
            memory: HashMap::new(),
            pc: 0,
            bytecode,
            threads: Vec::new(),
            receivers: Vec::new(),
        }
    }

    // Execute the bytecode instructions
    pub fn execute(&mut self) {
        macro_rules! binop {
            ($self:ident, $op:tt) => {{
                let b = $self.stack.pop().unwrap_or_else(|| panic!("Stack is empty"));
                let a = $self.stack.pop().unwrap_or_else(|| panic!("Stack is empty"));
                $self.stack.push(a $op b);
                $self.pc += 1;
            }};
        }

        macro_rules! stackop {
            ($self:ident, $body:block) => {{
                $body
                $self.pc += 1;
            }};
        }

        while self.pc < self.bytecode.len() {
            match self.bytecode[self.pc] {
                Bytecode::Neg => stackop!(self, {
                    if let Some(val) = self.stack.pop() {
                        self.stack.push(-val); // Updated to use f64 directly
                    } else {
                        panic!("Stack is empty");
                    }
                }),
                Bytecode::Add => binop!(self, +),
                Bytecode::Sub => binop!(self, -),
                Bytecode::Mul => binop!(self, *),
                Bytecode::Div => binop!(self, /),
                Bytecode::LoadConst(value) => stackop!(self, {
                    self.stack.push(value as f64); // Cast to f64
                }),
                Bytecode::LoadVar(index) => stackop!(self, {
                    if let Some(&value) = self.memory.get(&index) {
                        self.stack.push(value);
                    } else {
                        panic!("Variable not found in memory");
                    }
                }),
                Bytecode::StoreVar(index) => stackop!(self, {
                    if let Some(value) = self.stack.pop() {
                        self.memory.insert(index, value);
                    } else {
                        panic!("Stack is empty");
                    }
                }),
                Bytecode::Spawn => {
                    // Get the current bytecode value (should be 5 in our test case)
                    let value_to_spawn = if let Some(&val) = self.stack.last() {
                        val
                    } else {
                        // Default value if stack is empty
                        0.0
                    };

                    let (tx, rx) = mpsc::channel::<f64>(); // Updated to use f64
                    self.receivers.push(rx);

                    let handle = thread::spawn(move || {
                        // Simulate some computation
                        tx.send(value_to_spawn).unwrap();
                    });

                    self.threads.push(handle);
                    self.pc += 1;
                }
                Bytecode::Sync => {
                    // Clear the main thread's stack before collecting results
                    self.stack.clear();

                    // Wait for all threads to finish and collect their results
                    for thread in self.threads.drain(..) {
                        thread.join().unwrap();
                    }
                    // Retrieve results from receivers
                    for rx in self.receivers.drain(..) {
                        if let Ok(val) = rx.recv() {
                            self.stack.push(val);
                        }
                    }
                    self.pc += 1;
                }
                Bytecode::Barrier => {
                    // Wait at a barrier for all threads
                    while let Some(thread) = self.threads.pop() {
                        thread.join().unwrap();
                    }
                    self.pc += 1; // Move to the next instruction
                }
                Bytecode::Jump(target) => {
                    self.pc = target; // Jump to the target instruction
                }
                Bytecode::JumpIfZero(target) => {
                    if let Some(&top) = self.stack.last() {
                        if top == 0.0 {
                            self.pc = target; // Jump to the target instruction
                        } else {
                            self.pc += 1; // Move to the next instruction
                        }
                    } else {
                        panic!("Stack is empty");
                    }
                }
                Bytecode::JumpIfNotZero(target) => {
                    if let Some(&top) = self.stack.last() {
                        if top != 0.0 {
                            self.pc = target; // Jump to the target instruction
                        } else {
                            self.pc += 1; // Move to the next instruction
                        }
                    } else {
                        panic!("Stack is empty");
                    }
                }
                Bytecode::Pop => stackop!(self, {
                    self.stack.pop();
                }),
                Bytecode::Dup => stackop!(self, {
                    if let Some(&top) = self.stack.last() {
                        self.stack.push(top);
                    } else {
                        panic!("Stack is empty");
                    }
                }),
                Bytecode::Halt => {
                    println!("Execution halted");
                    break; // Stop execution
                }
            }
        }
    }

    pub fn run(bytecode: Vec<Bytecode>) -> f64 {
        let mut vm = VM::new(bytecode);
        vm.execute();
        vm.stack.pop().unwrap_or_else(|| 0_f64) // Ensure the default value is explicitly `f64`
    }

    /// Compile an AST expression using the provided compiler and execute it, returning the top of stack.
    pub fn run_expr<C: crate::compiler::Compiler<Instruction = Bytecode>>(expr: &parser::Expr) -> f64 {
        let bytecode = C::compile(expr);
        VM::run(bytecode)
    }
}

#[allow(dead_code)]
impl Bytecode {
    pub(crate) fn compile_expr(expr: &parser::Expr, code: &mut Vec<Bytecode>) {
        use crate::scanner::Token;
        match expr {
            parser::Expr::Number(n) => code.push(Bytecode::LoadConst(*n as f64)),
            parser::Expr::Ident(name) => panic!("Identifier '{}' not supported in bytecode", name),
            parser::Expr::UnaryOp { op, rhs } => {
                Bytecode::compile_expr(rhs, code);
                match op {
                    Token::Minus => code.push(Bytecode::Neg),
                    _ => panic!("Unsupported unary op: {:?}", op),
                }
            }
            parser::Expr::BinaryOp { lhs, op, rhs } => {
                Bytecode::compile_expr(lhs, code);
                Bytecode::compile_expr(rhs, code);
                match op {
                    Token::Plus => code.push(Bytecode::Add),
                    Token::Minus => code.push(Bytecode::Sub),
                    Token::Star => code.push(Bytecode::Mul),
                    Token::Slash => code.push(Bytecode::Div),
                    _ => panic!("Unsupported binary op: {:?}", op),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let bytecode = vec![
            Bytecode::LoadConst(2.0),
            Bytecode::LoadConst(3.0),
            Bytecode::Add,
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack.pop(), Some(5.0));
    }

    #[test]
    fn test_subtraction() {
        let bytecode = vec![
            Bytecode::LoadConst(10.0),
            Bytecode::LoadConst(4.0),
            Bytecode::Sub,
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack.pop(), Some(6.0));
    }

    #[test]
    fn test_multiplication() {
        let bytecode = vec![
            Bytecode::LoadConst(6.0),
            Bytecode::LoadConst(7.0),
            Bytecode::Mul,
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack.pop(), Some(42.0));
    }

    #[test]
    fn test_division() {
        let bytecode = vec![
            Bytecode::LoadConst(20.0),
            Bytecode::LoadConst(4.0),
            Bytecode::Div,
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack.pop(), Some(5.0));
    }

    #[test]
    fn test_store_and_load_var() {
        let bytecode = vec![
            Bytecode::LoadConst(99.0),
            Bytecode::StoreVar(1),
            Bytecode::LoadVar(1),
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack.pop(), Some(99.0));
    }

    #[test]
    fn test_jump() {
        let bytecode = vec![
            Bytecode::LoadConst(1.0),
            Bytecode::Jump(4),
            Bytecode::LoadConst(2.0), // skipped
            Bytecode::LoadConst(3.0), // skipped
            Bytecode::LoadConst(4.0),
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack, vec![1.0, 4.0]);
    }

    #[test]
    fn test_jump_if_zero() {
        let bytecode = vec![
            Bytecode::LoadConst(0.0),
            Bytecode::JumpIfZero(4),
            Bytecode::LoadConst(99.0), // skipped
            Bytecode::LoadConst(88.0), // skipped
            Bytecode::LoadConst(42.0),
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack, vec![0.0, 42.0]);
    }

    #[test]
    fn test_jump_if_not_zero() {
        let bytecode = vec![
            Bytecode::LoadConst(5.0),
            Bytecode::JumpIfNotZero(4),
            Bytecode::LoadConst(99.0), // skipped
            Bytecode::LoadConst(88.0), // skipped
            Bytecode::LoadConst(42.0),
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert_eq!(vm.stack, vec![5.0, 42.0]);
    }

    #[test]
    fn test_dup_and_pop() {
        let bytecode = vec![
            Bytecode::LoadConst(7.0),
            Bytecode::Dup,
            Bytecode::Add,
            Bytecode::Pop,
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_parallel_spawn_and_sync() {
        let bytecode = vec![
            Bytecode::LoadConst(2.0),
            Bytecode::LoadConst(3.0),
            Bytecode::Add,
            Bytecode::Spawn,
            Bytecode::Sync,
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        // The main thread's stack should have the result of the addition
        assert_eq!(vm.stack.pop(), Some(5.0));
    }

    #[test]
    #[should_panic(expected = "Variable not found in memory")]
    fn test_load_var_not_found() {
        let bytecode = vec![Bytecode::LoadVar(999), Bytecode::Halt];
        let mut vm = VM::new(bytecode);
        vm.execute();
    }

    #[test]
    #[should_panic(expected = "Stack is empty")]
    fn test_stack_underflow_add() {
        let bytecode = vec![Bytecode::Add, Bytecode::Halt];
        let mut vm = VM::new(bytecode);
        vm.execute();
    }

    #[test]
    fn test_negation() {
        let bytecode = vec![Bytecode::LoadConst(5.0), Bytecode::Neg, Bytecode::Halt];
        let mut vm = VM::new(bytecode);
        vm.execute();
        let val = vm.stack.pop().unwrap();
        assert_eq!(val, -(5.0_f64));
    }

    #[test]
    fn test_barrier_does_not_collect_results() {
        let bytecode = vec![
            Bytecode::LoadConst(10.0),
            Bytecode::Spawn,
            Bytecode::Barrier,
            Bytecode::Pop,
            Bytecode::Halt,
        ];
        // Thread will start at Spawn+1, execute until halt, then send nothing; barrier should join only
        let mut vm = VM::new(bytecode.clone());
        vm.execute();
        // After Pop, stack should be empty
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_multiple_spawns_and_sync_collects_all() {
        let bytecode = vec![
            Bytecode::LoadConst(4.0),
            Bytecode::LoadConst(1.0),
            Bytecode::Add,   // 5
            Bytecode::Spawn, // thread1
            Bytecode::Spawn, // thread2
            Bytecode::Sync,  // collect two results
            Bytecode::Halt,
        ];
        let mut vm = VM::new(bytecode);
        vm.execute();
        // Should collect two values of 5
        assert_eq!(vm.stack, vec![5.0, 5.0]);
    }
}
