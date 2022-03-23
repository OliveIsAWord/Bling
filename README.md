# Bling
Bling is a small, dynamic, expression-oriented programming language with no garbage collection. This language is in initial development but is currently in a working state. 

```
x := 5
while( () => {x}
    () => {
        print(x)
        x = sub(x 1)
    }
)
```
Output:
```
5
4
3
2
1
```
(Hopefully we'll have more impressive examples once the appropriate functionality is implemented.)

## Features
- 🔰 **Dead simple.** From the bottom up, Bling was designed to be simple and elegant. In its syntax, its bytecode, its types, and more, the goal was to be as clean and understandable as possible.
- 📢 **Expressive.** Expression-oriented semantics, clean lambda syntax, and functions as a first class value make Bling a language that can say a lot with very little.
- 🚯 **No garbage collection, no leaks.** Built on top of Rust's robust static memory management, all values once out of scope will be freed or returned to the parent scope as appropriate.
- 🔬 **Lexically scoped.**  Variables are scoped to the block they are declared in, and assignment and declaration are two separate operators. This greatly benefits local reasoning and avoids unintuitive edge cases in languages like Python (which is otherwise a fantastic language).

## Roadmap
Note: Goals further in the future are more likely to change.

### Essential
- [x] Parsing
- [x] IR Generation / Interpreter

### Important
- [ ] Language Documentation
- [ ] Command Line Interface
- [ ] IR Optimization
- [ ] Compilation to WASM

### Beneficial
- [ ] Linter
- [ ] Auto-Formatter
- [ ] Debugger
