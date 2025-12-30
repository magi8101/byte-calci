# Bytecode Calculator

A GUI calculator built in Rust with a complete bytecode compilation pipeline. Runs natively on Windows/Mac/Linux and in web browsers via WebAssembly.

## Features

- **Bytecode Compiler Pipeline**: Tokenizer → Parser → AST → CodeGenerator → Bytecode → VM
- **GUI Interface**: Built with egui/eframe
- **Cross-Platform**: Native desktop + WebAssembly

### Math Operations
- Basic: `+`, `-`, `*`, `/`, `%`, `^` (power), `**` (power)
- Functions: `sin`, `cos`, `tan`, `sqrt`, `abs`, `ln`, `log`, `exp`, `floor`, `ceil`, `round`
- Extended: `sinh`, `cosh`, `tanh`, `factorial`, `gcd`, `lcm`, `nPr`, `nCr`

### Arrays
```
[1, 2, 3, 4, 5]
sum([1,2,3])    → 6
avg([1,2,3])    → 2
min([1,2,3])    → 1
max([1,2,3])    → 3
len([1,2,3])    → 3
```

## Build

### Native
```bash
cargo build --release
cargo run --release
```

### Web (WebAssembly)

Prerequisites:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Development server:
```bash
trunk serve
```

Production build:
```bash
trunk build --release
```
Output in `dist/` folder - deploy to any static hosting.

## Architecture

```
src/
├── main.rs          # Entry point (native + wasm)
├── lib.rs           # Library exports
├── tokenizer.rs     # Lexical analysis
├── ast.rs           # Abstract Syntax Tree
├── parser.rs        # Expression parser
├── bytecode.rs      # Bytecode definitions
├── codegen.rs       # Bytecode generator
├── vm.rs            # Virtual machine
├── disassembler.rs  # Bytecode disassembly
└── gui.rs           # egui interface
```

## License

MIT
