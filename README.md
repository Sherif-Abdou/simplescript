# Simplescript

This is a simple programming language I made that is similar to C but with more python like syntax. This programming language currently uses llvm12 to compile the language to llvm IR.

## Dependencies
* LLVM 12

## Examples
### Hello world
There's still currently fixes needed on c string literals, but this works so far
```
extern def puts(str: &char): i64


def main(): i64 {
    s = "Hello world\n\0"
    puts(&s)

    return 0
}
```

### Basic structs
```
extern def malloc(size: i64): &char

struct String {
    internal: &char, 
    capacity: i64,
    size: i64,
}

def make_string(capacity: i64): String {
    s = String()
    s.capacity = capacity
    s.len = 0
    s.internal = malloc(capacity)

    return s
}

def main(): i64 {
    s = make_string(4)

    return 0
}
```

## Features implemented

What's currently implemented
* Variables
* very minimal data types(i64, f64, char)
* Pointers
* Arrays
* Expressions
* Functions
* C extern bindings
* If statements
* While loops

Essential items that are not currently implemented
* Compiler handles syntax errors properly, gives a nice compiler error
* A full set of data types (i32, f32, i16, bool)
* Fixes to arguments, there's still some weird behavior when directly dealing with function arguments
* Assignment operators (+=, -=, *=, /=)
* Boolean operations (&&, ||)
