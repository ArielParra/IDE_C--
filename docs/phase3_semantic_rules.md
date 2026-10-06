# Phase 3: Semantic Analysis for C--

This project now includes a semantic-analysis pass consistent with the language grammar and the Phase 3 objectives:

## Implemented checks

1. Declaration validation
   - variables must be declared before use
   - duplicate declarations in the same scope are rejected
   - declaration type is stored in the symbol table

2. Type checking
   - assignment compatibility is enforced
   - numeric promotion from int to float is allowed
   - invalid assignments like `int x; x = true;` are flagged

3. Boolean conditions
   - `if`, `while`, and repetition conditions must evaluate to `bool`
   - non-boolean expressions are rejected

4. Arithmetic validation
   - arithmetic operators require numeric operands
   - mixed int/float expressions are allowed with float result

5. Relational and logical validation
   - relational operators require comparable values
   - logical operators require boolean operands

6. Scope management
   - block scopes are tracked with a symbol table stack
   - variables declared in child blocks do not leak into parent scopes

7. Input/output validation
   - input statements require identifiers that are already declared
   - output expressions are checked for valid operand types

## Core analyzer

The analyzer is implemented in:
- src/compiler/semantic.rs

## Test coverage

Semantic tests live in:
- src/tests/semantic.rs
