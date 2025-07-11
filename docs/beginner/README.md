# ICN Core Beginner Quickstart

This short guide gets you up and running with the InterCooperative Network core repository as fast as possible. It covers only the minimal steps. For full details see the [Developer Onboarding Guide](../ONBOARDING.md).

## 1. Clone the Repository
```bash
git clone <repository-url>
cd icn-core
```

## 2. Install Rust Toolchain
```bash
rustup toolchain install stable
rustup override set stable
```

## 3. Install Helpers and Run Checks
```bash
cargo install just            # optional command runner
just setup                    # install dependencies and hooks
just validate                 # format, lint and test
```

## 4. Explore Further
- [Developer Onboarding Guide](../ONBOARDING.md)
- [ICN Feature Overview](../ICN_FEATURE_OVERVIEW.md)
- [Project Context](../../CONTEXT.md)
