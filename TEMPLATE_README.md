# Ashforge-RS Template Starter

A ready-to-use Rust project template derived from the ash-rpc project structure.

## What's Inside

The `ashforge-rs-template-starter/` directory contains a complete, buildable Rust project template with:

### Structure
- **src/lib.rs** - Minimal library starter with placeholder code
- **Cargo.toml** - Pre-configured with TODO comments for customization
- **.github/** - CI/CD workflows and repository configuration with placeholders
- **README.md** - Template documentation with sections to fill in
- **LICENSE** - Apache 2.0 license (original copy)
- **SECURITY.md** - Security policy (original copy)
- **Makefile** - Build and development commands (original copy)
- **.gitignore** - Standard Rust gitignore (original copy)
- **.commitlintrc.json** - Commit linting configuration (original copy)
- **TEMPLATE_USAGE.md** - Comprehensive guide on how to use this template

### What Was Excluded
- **examples/** - Not copied (add your own as needed)
- **fuzz/** - Not copied (add your own as needed)
- **src/** contents - Only a placeholder lib.rs is included

### GitHub Actions
All workflows have been simplified with placeholder content:
- **rust.yml** - Basic CI pipeline with test, clippy, and formatting checks
- **release.yml** - Release automation placeholder
- **commitlint.yml** - Commit message validation placeholder
- **CODEOWNERS** - Code ownership template
- **dependabot.yml** - Dependency update configuration

## Using the Template

See [TEMPLATE_USAGE.md](ashforge-rs-template-starter/TEMPLATE_USAGE.md) for detailed instructions.

### Quick Start

```bash
# Copy template to new location
cp -r ashforge-rs-template-starter /path/to/your-new-project
cd /path/to/your-new-project

# Update Cargo.toml with your project details
# Update README.md with your project information
# Update .github/CODEOWNERS with your GitHub username

# Start developing
cargo build
cargo test
```

## Template Status

✅ **Ready to use** - The template builds successfully and passes tests  
✅ **No warnings** - Clean compilation  
✅ **Documented** - Includes comprehensive usage guide  
✅ **Minimal** - Only essential structure, ready for expansion

## Making This a GitHub Template

To use this as a GitHub template repository:

1. Create a new repository on GitHub
2. Copy the contents of `ashforge-rs-template-starter/` to the new repository
3. Enable "Template repository" in the repository settings
4. Users can then click "Use this template" to create new projects

---
