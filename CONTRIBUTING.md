# Contributing to vllama

Thank you for your interest in contributing to vllama!

## Development Status

vllama is currently in **experimental development (0.0.x)**. The core functionality is tested and working, but we're still validating deployment configurations and gathering real-world feedback.

See [ai/STATUS.md](ai/STATUS.md) and [ai/TODO.md](ai/TODO.md) for current priorities.

## How to Contribute

### Reporting Bugs

Before creating a bug report:
1. Check the [existing issues](https://github.com/nijaru/vllama/issues)
2. Review [TESTING_STATUS.md](TESTING_STATUS.md) to see what's been tested

When reporting bugs, please include:
- Your OS and GPU (e.g., "Fedora 40, RTX 4090")
- vllama version (`vllama --version`)
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs from `vllm.log`

### Suggesting Features

Feature requests are welcome! Please:
1. Check [ai/TODO.md](ai/TODO.md) to see if it's already planned
2. Open an issue explaining the use case
3. Discuss before implementing large changes

**Current focus:** Deployment validation and real-world testing before adding new features.

### Code Contributions

**Development setup:**
```bash
# Clone and build
git clone https://github.com/nijaru/vllama.git
cd vllama
cargo build --release

# Run tests
cargo test
cargo test --test api_tests -- --ignored  # Integration tests (requires server)

# Check code
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

**Before submitting:**
1. Run all tests (`cargo test`)
2. Fix clippy warnings (`cargo clippy --workspace -- -D warnings`)
3. Format code (`cargo fmt`)
4. Update documentation if needed
5. Add tests for new functionality

**Pull Request Process:**
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Test thoroughly
5. Submit PR with clear description of changes

### Testing Contributions

We especially need help with:
- Testing deployment configurations (see `deployment-configs` branch)
- Testing with different GPU models
- Testing with various LLM models
- Load testing and performance validation

See [docs/TESTING_DEPLOYMENT.md](docs/TESTING_DEPLOYMENT.md) for deployment testing guidelines.

## Development Guidelines

### Code Style
- Follow existing code patterns
- Use meaningful variable names
- Add comments only for non-obvious "why" (not "what")
- Keep functions focused and testable

### Testing
- Add unit tests for new functionality
- Add integration tests for API changes
- Manual testing for server behavior (startup, shutdown, cleanup)
- Document test results

### Documentation
- Update README.md for user-facing changes
- Update ai/STATUS.md for development status
- Add entries to ai/TODO.md for planned work
- Keep documentation honest about what's tested vs not tested

## Questions?

- Open an issue for questions
- Check [CLAUDE.md](CLAUDE.md) for project architecture and decisions
- Review [ai/DECISIONS.md](ai/DECISIONS.md) for architectural rationale

## License

By contributing to vllama, you agree that your contributions will be licensed under the Elastic License 2.0.
