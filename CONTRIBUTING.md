# Contributing to Sierpinski Triangle Blockchain

Thank you for your interest in contributing to **TrinityChain**! This is a unique project combining fractal geometry with blockchain technology, and we're excited to have you here.

## 🌟 Welcome New Contributors!

This project is currently in **early development** (v0.1.0), which means:
- ✅ Lots of opportunities to make significant contributions
- ✅ Your ideas and feedback are highly valued
- ✅ Many features are still being designed
- ✅ Room to shape the direction of the project

**Never contributed to blockchain before?** That's totally fine! We welcome developers of all experience levels.

---

## 🚀 Quick Start

### 1. Fork & Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/trinitychain.git
cd trinitychain
```

### 2. Build & Test

```bash
# Build the project
cargo build --release

# Run tests (all 33 should pass)
cargo test

# Try the miner
cargo run --bin trinity-wallet-new
cargo run --release --bin trinity-miner <your_address>
```

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

---

## 🎯 Ways to Contribute

### 1. **Code Contributions**

#### High-Priority Areas:
- **Performance**: Mining optimization, parallel validation
- **Testing**: Integration tests, fuzzing, stress tests
- **Documentation**: Code comments, API docs, tutorials
- **Bug Fixes**: See GitHub Issues

#### Feature Ideas:
- Transaction fee market improvements
- Web-based block explorer
- Wallet encryption
- Mining pool support
- Mobile wallet
- GPU mining kernels

### 2. **Non-Code Contributions**

- **Documentation**: Write tutorials, guides, or explainers
- **Testing**: Run the software, report bugs
- **Community**: Answer questions, help onboard new users
- **Design**: UI/UX for block explorer, logos, branding
- **Research**: Economics, game theory, fractal algorithms

### 3. **Research & Ideas**

This is a novel blockchain design! We welcome research on:
- Fractal economics and game theory
- Novel DeFi primitives using geometric shapes
- Scalability solutions for triangle-based UTXO
- Cross-chain compatibility
- Privacy features

---

## 📝 Contribution Process

### Step 1: Pick an Issue (or Create One)

Browse [GitHub Issues](https://github.com/littlekickoffkittie/trinitychain/issues) for:
- `good first issue` - Perfect for newcomers
- `help wanted` - We need your expertise
- `bug` - Something's broken
- `enhancement` - New features

**Don't see your idea?** Create a new issue to discuss it first!

### Step 2: Develop Your Changes

```bash
# Create a feature branch
git checkout -b feature/my-awesome-feature

# Make your changes
# ... edit code ...

# Test your changes
cargo test
cargo build --release

# Test mining/wallet functionality
cargo run --release --bin trinity-miner <address>
```

### Step 3: Write Good Commit Messages

```bash
# Good commit message format:
git commit -m "feat: Add GPU mining support

- Implement CUDA kernels for SHA-256
- Add GPU device detection
- Benchmark shows 100x speedup on RTX 3080

Closes #42"
```

**Commit message types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `test:` - Adding tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvement
- `chore:` - Maintenance tasks

### Step 4: Submit a Pull Request

1. Push your branch:
   ```bash
   git push origin feature/my-awesome-feature
   ```

2. Go to GitHub and click "New Pull Request"

3. Fill out the PR template:
   - **Title**: Clear, descriptive (e.g., "Add GPU mining support")
   - **Description**: What changed and why
   - **Testing**: How you tested it
   - **Breaking Changes**: If any

4. Wait for review (usually within 24-48 hours)

---

## 🧪 Testing Guidelines

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test blockchain
cargo test geometry
cargo test transaction

# With output
cargo test -- --nocapture

# Integration tests
cargo test --test integration
```

### Writing Tests

Every new feature should include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_new_feature() {
        // Arrange
        let blockchain = Blockchain::new();

        // Act
        let result = blockchain.my_new_feature();

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Test Coverage Goals

- **Unit tests**: All public functions
- **Integration tests**: End-to-end workflows
- **Edge cases**: Invalid inputs, boundary conditions
- **Performance tests**: Benchmarks for critical paths

---

## 📋 Code Style

### Rust Formatting

We use `rustfmt` for consistent code style:

```bash
# Format your code before committing
cargo fmt

# Check formatting without changing files
cargo fmt -- --check
```

### Clippy Lints

We use Clippy to catch common mistakes:

```bash
# Run Clippy
cargo clippy -- -D warnings

# Fix suggestions automatically
cargo clippy --fix
```

### Code Guidelines

1. **Use descriptive names**:
   ```rust
   // Good
   let mining_reward = calculate_block_reward(height);

   // Bad
   let r = calc(h);
   ```

2. **Add documentation comments**:
   ```rust
   /// Calculates the block reward for a given height, accounting for halvings.
   ///
   /// # Arguments
   /// * `height` - The block height
   ///
   /// # Returns
   /// The reward in area units (u64)
   pub fn calculate_block_reward(height: BlockHeight) -> u64 {
       // ...
   }
   ```

3. **Handle errors properly**:
   ```rust
   // Use Result types
   pub fn load_blockchain(&self) -> Result<Blockchain, ChainError> {
       // ...
   }

   // Avoid unwrap() in library code
   ```

4. **Keep functions small**: <50 lines when possible

5. **Prefer explicit over clever**: Clarity > brevity

---

## 🏗️ Project Structure

```
TrinityChain/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── blockchain.rs       # Core blockchain logic
│   ├── geometry.rs         # Sierpinski triangle math
│   ├── transaction.rs      # Transaction types
│   ├── crypto.rs           # Cryptography (ECDSA)
│   ├── persistence.rs      # Database (SQLite)
│   ├── network.rs          # P2P networking
│   ├── error.rs            # Error types
│   └── bin/                # CLI tools
│       ├── trinity-miner.rs
│       ├── trinity-wallet.rs
│       └── ...
├── tests/                  # Integration tests
├── Cargo.toml              # Dependencies
├── README.md               # Main documentation
├── BITCOIN_FEATURES.md     # Economics documentation
└── CONTRIBUTING.md         # This file!
```

### Key Modules

- **blockchain.rs**: Blocks, chain, validation, difficulty, supply
- **geometry.rs**: Triangle subdivision, area calculation
- **transaction.rs**: Subdivision, transfer, coinbase transactions
- **crypto.rs**: Key generation, signing, verification
- **persistence.rs**: SQLite database operations
- **network.rs**: P2P communication, block propagation

---

## 🐛 Reporting Bugs

Found a bug? Please report it!

### Bug Report Template

```markdown
**Description**
Clear description of the bug

**Steps to Reproduce**
1. Run `cargo run --bin trinity-miner <address>`
2. Wait for 10 blocks
3. See error: ...

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: Ubuntu 22.04
- Rust version: 1.90.0
- Commit: abc123

**Logs**
```
Paste relevant error messages
```

**Additional Context**
Any other relevant information
```

---

## 💡 Feature Requests

Have an idea? We'd love to hear it!

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
A clear description of the problem

**Describe the solution you'd like**
What you want to happen

**Describe alternatives you've considered**
Other solutions you've thought about

**Use Cases**
Who would use this and why?

**Additional Context**
Mock-ups, research, examples, etc.
```

---

## 📚 Resources

### Learning Resources

**Blockchain Basics:**
- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)
- [Mastering Bitcoin (free ebook)](https://github.com/bitcoinbook/bitcoinbook)

**Rust:**
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

**Sierpinski Triangles:**
- [Wikipedia: Sierpinski Triangle](https://en.wikipedia.org/wiki/Sierpi%C5%84ski_triangle)
- [Fractal Geometry](https://en.wikipedia.org/wiki/Fractal)

### Project Documentation

- [README.md](README.md) - Getting started
- [BITCOIN_FEATURES.md](BITCOIN_FEATURES.md) - Economics & supply
- [PROJECT_STATUS.md](PROJECT_STATUS.md) - Roadmap (200+ features)

### Get Help

- **GitHub Issues**: Ask questions, report bugs
- **Code Comments**: Most functions have documentation
- **Tests**: Look at tests for usage examples

---

## 🤝 Community Guidelines

### Be Respectful

- **Inclusive**: Welcome all backgrounds and skill levels
- **Patient**: Help newcomers learn
- **Constructive**: Give helpful feedback
- **Professional**: Keep discussions on-topic

### Communication

- **Be clear**: Explain your ideas thoroughly
- **Ask questions**: No question is too basic
- **Provide context**: Include logs, code, environment details
- **Follow up**: Respond to feedback on your PRs

---

## 🎖️ Recognition

All contributors will be:
- ✅ Listed in project documentation
- ✅ Credited in release notes
- ✅ Given a shout-out on the main README (if you wish)

Significant contributors may be offered:
- Core maintainer status
- Architecture decision input
- Early preview of roadmap items

---

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License (same as the project).

---

## 🚧 Work in Progress Areas

These areas need help but may change significantly:

### High Priority:
1. **Mining Optimization** - Current miner is single-threaded
2. **Wallet Encryption** - Keys stored in plaintext
3. **Transaction Fees** - Need proper fee market
4. **Block Explorer** - Web UI for viewing chain

### Medium Priority:
5. **GPU Mining** - CUDA/OpenCL kernels
6. **REST API** - Complete API server
7. **Mobile Wallet** - iOS/Android apps
8. **Documentation** - Tutorials, videos

### Research Needed:
9. **Fractal Economics** - Game theory analysis
10. **Scalability** - Sharding for triangle UTXO
11. **Privacy** - zk-SNARKs for triangles
12. **Smart Contracts** - Geometric contract primitives

---

## 📞 Contact

- **GitHub Issues**: [TrinityChain/issues](https://github.com/littlekickoffkittie/trinitychain/issues)
- **Repository**: [github.com/littlekickoffkittie/trinitychain](https://github.com/littlekickoffkittie/trinitychain)

---

## 🎯 First Contribution Ideas

Perfect for newcomers:

1. **Add tests** - Pick a function in `src/` and add unit tests
2. **Improve docs** - Add comments to undocumented functions
3. **Fix clippy warnings** - Run `cargo clippy` and fix warnings
4. **Write tutorial** - Document how to do something (mining, transfers, etc.)
5. **Optimize code** - Profile and improve performance
6. **Add error handling** - Replace `unwrap()` with proper error handling

Look for issues tagged `good first issue`!

---

**Thank you for contributing to the Sierpinski Triangle Blockchain! 🔺⛓️**

*Where geometry meets blockchain, powered by community.*
