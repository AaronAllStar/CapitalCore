# ADR-0002: Decimal / Money Representation

## Context
Per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md) Invariant #1: *"No f32/f64 for money. Use integer minor units or a decimal type, explicit rounding mode, per-currency precision. Every numeric decision is documented in an ADR."*
The legacy Python application improperly used IEEE 754 floating-point numbers (`float`) across billing, marketplace pricing, backtesting balances, and database columns.

## Decision
Represent all financial amounts using fixed integer minor units (`i64`) tied to an explicit `Currency` enum:
- `Money` struct encapsulating an `i64` amount and a `Currency` enum variant.
- Currency metadata defines precision (e.g. `USD` = 2 decimal places, `JPY` = 0, `BTC` = 8).
- Rounding mode is always explicitly passed to division operations via an enum (`HalfUp`, `Floor`, `Ceiling`, etc.).
- Arithmetic operations are checked (`checked_add`, `checked_sub`, `checked_mul`) and return `Result<Money, NumericError>` or panic on currency mismatch.
- `rust_decimal::Decimal` is reserved exclusively for exchange rate conversions or non-integer multiplier formulas, never as primary storage for account balances.

## Alternatives Considered
1. **`rust_decimal::Decimal` across all models**: Rejected because it adds unnecessary heap/runtime overhead and obscures strict per-currency minor-unit bounds.
2. **`f64` with rounding wrappers**: Rejected because it violates hard invariant #1 and allows cumulative precision drift.
3. **Custom arbitrary-precision big rational type**: Rejected as unnecessary complexity for real-time transaction processing.

## Consequences
- **Positive**: Complete elimination of precision loss and non-deterministic floating-point behavior; fast integer arithmetic suitable for high-frequency fraud checks; compile-time prevention of cross-currency arithmetic without explicit conversion.
- **Negative / Risks**: Division operations require explicit handling of remainder and rounding; legacy database values stored as floats must undergo sanitization and conversion.

## Test Strategy
- Property-based testing using `proptest`: verify algebraic identities (e.g., `(a + b) - b == a`) across all valid `i64` ranges.
- Explicit unit test vectors for edge cases (zero, maximum minor unit, currency mismatch errors, negative amounts).
- Differential comparisons against legacy pricing to ensure correct conversion.

## Migration Impact
- Any legacy floats in exported seed data or migrations will be parsed through exact decimal string parsers into integer minor units.
