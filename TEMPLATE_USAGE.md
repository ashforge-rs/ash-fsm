# Template Usage Guide

This directory contains a template starter for creating new Rust projects with the ashforge-rs structure.

## What's Included

- ✅ **Cargo.toml** - Pre-configured with placeholders for project metadata
- ✅ **src/lib.rs** - Basic library structure with placeholder code
- ✅ **.github/workflows/** - CI/CD workflows (rust.yml, release.yml, commitlint.yml) with placeholders
- ✅ **.github/CODEOWNERS** - Code ownership template
- ✅ **.github/dependabot.yml** - Automated dependency updates
- ✅ **LICENSE** - Apache 2.0 license
- ✅ **README.md** - Template readme with sections to fill out
- ✅ **SECURITY.md** - Security policy
- ✅ **Makefile** - Build and development commands
- ✅ **.gitignore** - Standard Rust gitignore
- ✅ **.commitlintrc.json** - Commit message linting configuration

## What's NOT Included

- ❌ **examples/** - Add your own examples as needed
- ❌ **fuzz/** - Add fuzzing tests if needed
- ❌ **src content** - Only a placeholder lib.rs is provided

## How to Use This Template

### Option 1: Manual Setup

1. **Copy the template directory:**
   ```bash
   cp -r ashforge-rs-template-starter /path/to/your-new-project
   cd /path/to/your-new-project
   ```

2. **Update Cargo.toml:**
   - Change `name` from "your-project-name" to your actual project name
   - Update `version`, `description`, `authors`
   - Update `repository`, `documentation`, `homepage` URLs
   - Update `keywords` and `categories`
   - Add your dependencies

3. **Update README.md:**
   - Replace "Your Project Name" with your actual name
   - Fill in the overview and features
   - Update installation and quick start sections
   - Remove the "Template Information" section at the bottom

4. **Update .github/CODEOWNERS:**
   - Replace `@OWNER_USERNAME` with your GitHub username or team

5. **Update .github/workflows:**
   - Review and customize CI/CD workflows as needed
   - Update branch names if not using master/main

6. **Implement your library:**
   - Replace the placeholder content in `src/lib.rs`
   - Add additional modules and files as needed

7. **Initialize git (if not already a git repo):**
   ```bash
   git init
   git add .
   git commit -m "Initial commit from ashforge-rs template"
   ```

### Option 2: GitHub Template Repository

If this template is set up as a GitHub template repository:

1. Go to the repository on GitHub
2. Click "Use this template"
3. Create your new repository
4. Clone and follow steps 2-6 above

## Checklist Before Publishing

- [ ] Updated Cargo.toml with correct project metadata
- [ ] Updated README.md with project-specific information
- [ ] Updated CODEOWNERS with correct usernames
- [ ] Implemented actual library code in src/
- [ ] Added tests
- [ ] Reviewed and customized CI/CD workflows
- [ ] Added examples (if applicable)
- [ ] Updated SECURITY.md if needed
- [ ] Removed template-specific documentation

## Features to Consider Adding

- **Examples directory** - If you want to showcase usage
- **Documentation** - Consider using `cargo doc` and docs.rs
- **Benchmarks** - Add `benches/` directory for performance testing
- **Integration tests** - Add `tests/` directory
- **More dependencies** - Based on your project needs

## Support

For questions about this template, please refer to the main ash-rpc repository.
