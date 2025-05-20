# Parallelized Programming Language Scanner

## EBNF Grammar for the Scanner and Functions

```
program      = { statement } ;
statement    = assignment | expression | function_def | parallel | sync | barrier | control_flow ;
assignment   = identifier '=' expression ';' ;
function_def = 'fn' identifier '(' [ parameters ] ')' block ;
parameters   = identifier { ',' identifier } ;
block        = '{' { statement } '}' ;
expression   = call | term { ('+' | '-' | '*' | '/') term } ;
call        = identifier '(' [ arguments ] ')' ;
arguments   = expression { ',' expression } ;
term         = number | identifier | '(' expression ')' | '-' term ;
parallel     = 'spawn' statement ;
sync         = 'sync' ';' ;
barrier      = 'barrier' ';' ;
control_flow = jump | jump_if_zero | jump_if_not_zero ;
jump         = 'jump' number ';' ;
jump_if_zero = 'jz' number ';' ;
jump_if_not_zero = 'jnz' number ';' ;
identifier   = letter { letter | digit | '_' } ;
number       = digit { digit } ;
letter       = 'a'..'z' | 'A'..'Z' ;
digit        = '0'..'9' ;
whitespace   = ' ' | '\t' | '\n' | '\r' ;
comment      = '//' { any character except '\n' } ;
```

## Tokens
- Identifiers: variable/function names
- Numbers: integer literals
- Operators: +, -, *, /
- Assignment: =
- Delimiters: ;, (, )
- Keywords: spawn, sync, barrier, jump, jz, jnz
- Comments: // ...

## Scanner Responsibilities
- Skip whitespace and comments
- Tokenize input into the above tokens
- Report errors for invalid characters
