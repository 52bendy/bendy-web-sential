---
name: testing
description: Testing for bendy-web-sential. Use when writing tests, running test suites, or verifying code changes.
when_to_use: Adding new features, fixing bugs, ensuring code quality, test coverage analysis.
---

# Testing Guide

## Backend Testing (Rust)

### Run Tests
```bash
cargo test
cargo test -- --nocapture    # Show output
cargo test test_name          # Run specific test
```

### Test Organization
- Unit tests: Inline in source files
- Integration tests: `tests/` directory
- Module tests: `#[cfg(test)]` blocks

### Test Coverage
- **Security modules**: ≥80% coverage required
- Key files to test:
  - `src/security/jwt.rs`
  - `src/security/totp.rs`
  - `src/security/token_blacklist.rs`
  - `src/middleware/auth.rs`
  - `src/error.rs`

### Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_validation() {
        // test code
    }
}
```

## Frontend Testing

### Run Tests
```bash
cd frontend
npm test                    # Run tests
npm test -- --coverage     # With coverage
npm test -- --watch        # Watch mode
```

### Test Framework
- Framework: Vitest
- Coverage: ≥80% for critical paths

## Linting

### Rust
```bash
cargo fmt                # Format code
cargo clippy             # Lint
cargo clippy -- -D warnings  # Strict mode
```

### Frontend
```bash
cd frontend
npm run lint             # ESLint
npm run lint -- --fix    # Auto-fix
```

## Integration Tests

### BWS Test Skill
Project includes `bws-test` skill for integration testing:
- Tests gateway authentication
- Tests API key authentication
- Tests route proxying

## CI/CD Testing

### Automated Checks
1. `cargo test`
2. `cargo fmt --check`
3. `cargo clippy`
4. Build verification

### Pre-commit Checklist
- [ ] All tests pass
- [ ] Code formatted
- [ ] No clippy warnings
- [ ] New tests for new features
