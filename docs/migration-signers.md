# Signer Migration Notes

RuntimeContext and the ICN node now default to `Ed25519Signer` when real libp2p networking is used. This signer keeps the private key in a `Zeroizing` wrapper to ensure the bytes are cleared on drop.

The previous `StubSigner` remains for tests. Existing tests and examples that construct `StubSigner` continue to work without modification.
