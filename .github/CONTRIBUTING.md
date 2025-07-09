# Contributing to the InterCooperative Network (ICN)

First off, thank you for considering contributing to the ICN! It's people like you that make the ICN such a great community.

Following these guidelines helps to communicate that you respect the time of the developers managing and developing this open source project. In return, they should reciprocate that respect in addressing your issue, assessing changes, and helping you finalize your pull requests.

## How Can I Contribute?

There are many ways to contribute, from writing tutorials or blog posts, improving the documentation, submitting bug reports and feature requests or writing code which can be incorporated into the ICN itself.

### Reporting Bugs

If you find a bug, please ensure the bug was not already reported by searching on GitHub under [Issues](https://github.com/InterCooperative-Network/icn-core/issues). If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/InterCooperative-Network/icn-core/issues/new?assignees=&labels=bug&template=bug_report.md&title=%5BBUG%5D). Be sure to include a **title and clear description**, as much relevant information as possible, and a **code sample or an executable test case** demonstrating the expected behavior that is not occurring.

### Suggesting Enhancements

If you have an idea for a new feature or an improvement to an existing one, please open an issue with the "feature request" label. Provide a clear and detailed explanation of the feature, why it's needed, and how it would work. You can use the [Feature Request template](https://github.com/InterCooperative-Network/icn-core/issues/new?assignees=&labels=enhancement&template=feature_request.md&title=%5BFEAT%5D).

### Your First Code Contribution

Unsure where to begin contributing to ICN? You can start by looking through `good first issue` and `help wanted` issues:

*   [Good first issues](https://github.com/InterCooperative-Network/icn-core/labels/good%20first%20issue) - issues which should only require a few lines of code, and a test or two.
*   [Help wanted issues](https://github.com/InterCooperative-Network/icn-core/labels/help%20wanted) - issues which should be a bit more involved than `good first issue` issues.

### Setting Up Your Development Environment (`icn-devnet`)

The quickest way to spin up a local federation is the **icn-devnet** Docker
environment contained in this repository. You will need:

1. **Docker** and **Docker Compose** installed and running.
2. The **stable Rust toolchain** if you intend to build the crates locally
   (`rustup toolchain install stable` and `rustup override set stable`).

Launch the federation from the repo root:

```bash
cd icn-devnet
./launch_federation.sh
```

The script builds all images, starts three nodes and waits for them to join the
mesh before submitting a test job. Once complete the nodes are accessible at
`http://localhost:5001`, `http://localhost:5002` and `http://localhost:5003`.

Configuration is driven by the Docker compose file. You can override values such
as `ICN_NODE_NAME`, `ICN_HTTP_LISTEN_ADDR`, `ICN_P2P_LISTEN_ADDR`,
`ICN_ENABLE_P2P`, `ICN_BOOTSTRAP_PEERS`, `ICN_STORAGE_BACKEND` or `RUST_LOG` to
control logging verbosity. Stop the federation with `docker-compose down`.

### Running Tests and Linters

Before submitting code, ensure the workspace passes formatting, linting and
tests. From the repository root run:

```bash
# Check formatting (apply with `cargo fmt --all` if needed)
cargo fmt --all -- --check

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features --workspace
```

Setting `RUST_LOG=debug` can provide more verbose output while developing. These
are the same checks executed by CI.

### How to Claim an Issue

If you see an issue you'd like to work on, please comment on the issue expressing your interest. This helps avoid duplicated effort. If the issue is already assigned or someone has already expressed interest, consider looking for another issue or offering to collaborate.

## Pull Request Process

1.  Ensure any install or build dependencies are removed before the end of the layer when doing a build.
2.  Update the README.md with details of changes to the interface, this includes new environment variables, exposed ports, useful file locations and container parameters.
3.  Increase the version numbers in any examples and the README.md to the new version that this Pull Request would represent. The Githu Maintainers will have the final say on versioning.
4.  You may merge the Pull Request in once you have the sign-off of two other developers, or if you do not have permission to do that, you may request the second reviewer to merge it for you.

## Code of Conduct

This project and everyone participating in it is governed by the [ICN Code of Conduct](./CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [INSERT MATRIX CONTACT METHOD HERE]. 