# Phase 3: Semantic Analysis for C--

## Data flow and interfaces

`analyze_source(&str)` processes one snapshot of the current editor buffer through the existing lexer, parser and semantic analyzer. Both Syntax Analysis and Semantic Analysis refresh this snapshot, selecting the original or annotated tree respectively. Unsaved documents work without a `.tokens` file. Lexical errors stop subsequent phases. Recoverable syntax errors produce a partial AST and a persistent warning; partial results never report unconditional success.

`SemanticAnalyzer::analyze_with_details(&AstNode) -> SemanticResult` returns:

- `annotated_ast`: a copy with semantic attributes; the parser AST is unchanged.
- `symbols`: the declaration registry actually used for name lookup and type checking, in declaration order.
- `errors`: semantic diagnostics with source line and column.

`SemanticAnalyzer::analyze(&AstNode)` remains available for diagnostics-only callers. Results are cleared when the editor changes, when another document is opened and before a new analysis.

## Attributes and annotated tree

| Attribute | Direction | Rule |
| --- | --- | --- |
| Declared type | Inherited | A declaration distributes its `int`, `float`/`real` or `bool` type to each declared identifier. |
| Identifier type | From context | Each use resolves the nearest active declaration in the symbol table. |
| Expression type | Synthesized | Operators compute a result type from their operand types. Expression-statement wrappers propagate it upward. |
| Constant value | Synthesized | Literal-only subexpressions are evaluated when safe. Identifiers, input and increment/decrement have no compile-time value. |
| Conversion target | Contextual | An integer operand in a mixed numeric expression, or the integer RHS of a float assignment, records conversion to `float`. |

`Syntax` shows the original AST. `Semantic` shows a separate expandable tree with labels such as `AddOp(+) [type: int -> float, value: 14]`. Both retain source positions and support click-to-code navigation. The annotated tree can be copied as indented text.

Constants use checked signed 64-bit integer arithmetic and finite 64-bit floating-point values. Integer division truncates toward zero. Integer powers require a nonnegative exponent representable by `u32`; an out-of-range result is rejected. Floating-point arithmetic follows IEEE 754 rounding. Constant division/remainder by zero, overflow and non-finite results produce diagnostics rather than panics. No variable-value propagation, execution of branches or runtime evaluation is attempted.

## Symbols and scopes

A stack of hash maps resolves names to a persistent declaration registry. Leaving a block removes its active lookup scope but retains its symbols for display. Shadowing is allowed in child scopes; duplicate declarations in the same scope are rejected. An undeclared use produces an error and does not create a fake declaration.

The `Hash Table` tab displays:

- identifier name and declared type;
- unique scope ID;
- declaration line and column;
- sorted unique declaration/use lines;
- logical storage offset;
- individual reference positions, including reads, writes and input destinations.

Scope IDs start at zero and increase when blocks are entered. Offsets are sequential logical storage slots, beginning at zero, one slot per successful declaration. They are not byte offsets, machine addresses or an ABI layout. Symbols from closed scopes retain their original slots. Rows navigate to declarations; the copy button exports a table separated by tabs. Every run starts a fresh registry, so ordering is deterministic for the same source.

## Type rules

| Construction | Accepted operands / result |
| --- | --- |
| Assignment | Equal types; also `int` to `float`. `float` to `int`, boolean/numeric conversions and other mismatches are rejected. |
| `+`, `-`, `*`, `/`, `%`, `^` | Numeric operands; `float` if either operand is float, otherwise `int`. |
| `<`, `>`, `<=`, `>=` | Numeric operands; boolean result. |
| `==`, `!=` | Numeric pairs or boolean pairs; boolean result. |
| `&&`, `||`, `!` | Boolean operands; boolean result. |
| `++`, `--` | An initialized numeric identifier; same type as its declaration. Literals and other expressions are rejected as targets. |
| `if`, `while`, `do`/`until` conditions | Must have boolean type. |
| `cin` | Destination must be declared; successful input initializes it. |
| `cout` | Check each expression. String literals are permitted as output values, without adding string declarations to the grammar. |

An unresolved or invalid expression has unknown type. Traversal continues to report independent errors while avoiding unnecessary type-error cascades.

## Definite initialization

Declarations start uninitialized. A compatible assignment with an error-free RHS initializes its target; invalid assignments do not. Input initializes a declared destination without first reading it. Reads and updates require initialization.

Each `if` branch starts from the same incoming state. Only identifiers initialized on both branches are definitely initialized afterward; a missing `else` preserves the incoming guarantees. A `while` body may execute zero times, so its assignments do not add guarantees after the loop. A `do` body runs before its condition, so valid assignments to enclosing variables remain initialized. Block-local variables never escape their scope.

## Demonstration and validation

Keep the original `semantic.c--` sample unchanged. It intentionally includes syntax and semantic errors: its output is partial and should demonstrate undeclared variables, incompatible assignments and use before initialization.

A small valid example for the annotated tree and symbol table:

```c--
main {
    float total;
    int count;
    count = 2;
    total = 2 + 3 * 4;
    if (count > 0) {
        bool ready;
        ready = true;
        cout << ready;
    }
    cout << total;
}
```

Here the constant expression synthesizes integer value `14` and records promotion to float for assignment. `ready` remains visible in the table after its scope closes; `total` has no assumed runtime value when read by `cout`.

Run unit tests with `cargo test --locked`. The GTK integration test runs separately with a display:

```sh
xvfb-run -a cargo test --locked gtk_semantic_views -- --ignored --test-threads=1
```

It checks the current-buffer action, original/annotated trees, table, error updates and stale-result invalidation. Additional tests cover preserved AST data, scopes, conversions, constants, initialization and partial syntax recovery.
