# Contributing to Sharira

Thank you for your interest in contributing to Sharira.

## Development Workflow

1. Fork and clone the repository
2. Create a feature branch from `main`
3. Make your changes
4. Run `make check` to validate
5. Open a pull request

## Prerequisites

- Rust stable (MSRV 1.89)
- Components: `rustfmt`, `clippy`
- Optional: `cargo-audit`, `cargo-deny`, `cargo-llvm-cov`

## Makefile Targets

| Command | Description |
|---------|-------------|
| `make check` | fmt + clippy + test + audit |
| `make fmt` | Check formatting |
| `make clippy` | Lint with `-D warnings` |
| `make test` | Run test suite |
| `make audit` | Security audit |
| `make deny` | Supply chain checks |
| `make bench` | Run benchmarks with history tracking |
| `make coverage` | Generate coverage report |
| `make doc` | Build documentation |

## Adding a Module

1. Create `src/module_name.rs` with module doc comment
2. Add `pub mod module_name;` to `src/lib.rs`
3. Re-export key types from `lib.rs`
4. Add unit tests in the module (`#[cfg(test)] mod tests`)
5. Add integration tests in `tests/integration.rs` if cross-module
6. Update README feature list

If the module requires an external dependency, gate it behind a feature flag.

## Code Style

- `cargo fmt` — mandatory
- `cargo clippy -- -D warnings` — zero warnings
- Doc comments on all public items
- `#[non_exhaustive]` on public enums
- `#[must_use]` on all pure functions
- `#[inline]` on hot-path functions
- No `unsafe` code
- No `unwrap()` or `panic!()` in library code
- Use `tracing` for structured logging, not `println!`
- Units in comments: always annotate physical quantities (meters, kg, Newtons, radians)

## Testing

- Unit tests colocated in modules (`#[cfg(test)] mod tests`)
- Integration tests in `tests/`
- Feature-gated tests with `#[cfg(feature = "...")]`
- Target: 80%+ line coverage
- Mathematical property tests for physics formulas (round-trip, conservation laws)
- Use small epsilon for f32 comparisons (`< 0.01` or `< 1e-5`)

## Commits

- Use conventional-style messages
- One logical change per commit

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
