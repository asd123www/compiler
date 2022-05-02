# compiler
compiler

Why we use `Rust` instead of `c/c++`? https://www.abetterinternet.org/docs/memory-safety/


### how to compile constant value?

Calculate a value no matter whether it's constant or not, make sure arithmetic is legal.

Then at the specific constant point the value is there.

### For if-else:
see `example/if-else.koopa`, just alloc space for that result.

Short-circuit evaluation by `jump`.

### For while 
After every `while`, we can substitute the `<replace_me_with_break>` `<replace_me_with_continue>` to corresponding jump.

### Multi-function grammar
The original grammar is shit, change to vector.


### Function
好像这个return value有点恶心啊.