use crate::parser::Expr;
use crate::vm::Bytecode;

/// A trait for compiling AST nodes into instructions.
pub trait Compiler {
    /// The type of instruction emitted by the compiler.
    type Instruction;

    /// Compile an AST expression into a sequence of instructions.
    fn compile(expr: &Expr) -> Vec<Self::Instruction>;
}

/// A compiler that emits `Bytecode` instructions from AST expressions.
pub struct BytecodeCompiler;

impl Compiler for BytecodeCompiler {
    type Instruction = Bytecode;

    fn compile(expr: &Expr) -> Vec<Bytecode> {
        let mut code = Vec::new();
        Bytecode::compile_expr(expr, &mut code);
        code.push(Bytecode::Halt);
        code
    }
}

impl BytecodeCompiler {
    /// Inherent method to compile expressions into bytecode via the Compiler trait.
    pub fn compile(expr: &Expr) -> Vec<Bytecode> {
        <Self as Compiler>::compile(expr)
    }
}
