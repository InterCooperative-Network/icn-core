# ICN CLI Crate

This crate provides a command-line interface (CLI) for interacting with the InterCooperative Network (ICN).

## Purpose

The `icn-cli` is a binary application that allows users and administrators to:

*   Manage ICN node instances (e.g., start, stop, query status).
*   Interact with the ICN network (e.g., send messages, query data).
*   Manage identities and credentials.
*   Execute administrative commands.

It serves as a primary tool for direct interaction with ICN functionalities from a terminal.

## Public API Style

As a CLI application, its "API" is its set of commands, subcommands, arguments, and flags. The style emphasizes:

*   **Usability:** Clear, intuitive command structure and helpful messages.
*   **Discoverability:** Easy way to find available commands and options (e.g., `--help` flags).
*   **Scriptability:** Output formats suitable for scripting (e.g., JSON) where appropriate.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 