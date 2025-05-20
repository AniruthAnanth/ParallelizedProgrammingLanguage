# Parallelized Programming Language Scanner

## EBNF Grammar for the Scanner

```
program      = { statement } ;
statement    = assignment | expression | parallel | sync | barrier | control_flow ;
assignment   = identifier '=' expression ';' ;
expression   = term { ('+' | '-' | '*' | '/') term } ;
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
