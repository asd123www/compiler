# compiler
compiler

Why we use `Rust` instead of `c/c++`? https://www.abetterinternet.org/docs/memory-safety/


### how to compile constant value?

Calculate a value no matter whether it's constant or not, make sure arithmetic is legal.

Then at the specific constant point the value is there.

Last bit is `1` indicate it's constant, `0` variable whose value is not defined when compiling.

### For if-else:
see `example/if-else.koopa`, just alloc space for that result.

Short-circuit evaluation by `jump`.

### For while 
After every `while`, we can substitute the `<replace_me_with_break>` `<replace_me_with_continue>` to corresponding jump.

### Multi-function grammar
The original grammar is shit, change to vector.


### Function
Store function info also in scope.

### Global Declaration
```
Initializer ::= INT | "undef" | Aggregate | "zeroinit";
```

Therefore I assume it's value is fixed, not in run-time.


### Parser problem

1. Variable Type only `int`, substitute the Btype.
2. Integrate the small parts to a whole, eliminate the shift and reduce conflicts.

```
  /home/compiler/src/sysy.lalrpop:161:16: 161:34: Local ambiguity detected

    The problem arises after having observed the following symbols in the input:
      "int"
    At that point, if the next token is a `r#"[_a-zA-Z][_a-zA-Z0-9]*"#`, then the parser can proceed in two different ways.

    First, the parser could execute the production at /home/compiler/src/sysy.lalrpop:161:16: 161:34, which would consume the top 1 token(s) from the stack and produce a `BType`. This might then yield a parse tree like
      "int"   ╷ VarDef ";"
      ├─BType─┘          │
      └─VarDecl──────────┘

    Alternatively, the parser could shift the `r#"[_a-zA-Z][_a-zA-Z0-9]*"#` token and later use it to construct a `Ident`. This might then yield a parse tree like
      "int" r#"[_a-zA-Z][_a-zA-Z0-9]*"# "(" FuncFParams ")" Block
      │     └─Ident───────────────────┘                         │
      └─FuncDef─────────────────────────────────────────────────┘

    See the LALRPOP manual for advice on making your grammar LR(1).
```


### Constant Array
We don't have to evaluate it during compiling!

Just treat it as an ordinary array.

### Function Parameter type is `pointer` or `int` ?

use the biset, maximum parameter number is 29.



## risc-v

### how to reference variable
scope use the Value(pointer) to address, not the inherit `variable name`.
you should accept the API instead of your own convention to code easier.


### Lesson learned

Write as many asserts as you can