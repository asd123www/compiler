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