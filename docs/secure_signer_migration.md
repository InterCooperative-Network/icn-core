# Migrating from `StubSigner`

`RuntimeContext::new_with_real_libp2p` and the default node startup now use
`Ed25519Signer` which performs real cryptographic signing and zeroizes the
private key on drop. Tests relying on `StubSigner` continue to work because the
stub type is still available under `icn_runtime::context::StubSigner`.

For existing tests simply keep constructing `StubSigner` directly. Production
code should prefer `Ed25519Signer`.
