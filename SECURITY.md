# Security Policy

## Supported Versions

We are committed to ensuring the security of the InterCooperative Network (ICN) Core project. Security patches and updates will be provided for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

The ICN Core team and community take all security vulnerabilities seriously. Thank you for improving the security of our project. We appreciate your efforts and responsible disclosure and will make every effort to acknowledge your contributions.

To report a security vulnerability, please email the lead maintainers at [security@intercooperative.network](mailto:security@intercooperative.network).

Please include the following details with your report:

*   A description of the vulnerability and its potential impact.
*   Steps to reproduce the vulnerability.
*   Any (anonymized) proof-of-concept or exploit code.
*   Information about the affected versions.

We will acknowledge your email within 48 hours and will send a more detailed response within 72 hours indicating the next steps in handling your report.

**Please do not report security vulnerabilities through public GitHub issues.**

## Disclosure Policy

When the team receives a security bug report, they will assign it to a primary handler. This person will coordinate the fix and release process, involving the following steps:

1.  **Confirm the problem** and determine the affected versions.
2.  **Audit code** to find any similar problems.
3.  **Prepare fixes** for all releases still under maintenance. These fixes will be applied to the main branch and any necessary release branches.
4.  **Release new versions** with the fix.
5.  Once the fix is publicly available, **publicly disclose the vulnerability** through a security advisory on GitHub and potentially other channels if deemed necessary.

We aim to handle disclosures responsibly and will coordinate with you on the timing of public announcements.

We appreciate your help in keeping ICN Core secure. 
## Federation Security Protocols and Key Management

Federated deployments rely on libp2p for encrypted transport and DID-based authentication. Each node should store its private keys in a secure hardware module or encrypted keystore. Rotate federation signing keys on a regular schedule and distribute the updated public keys to all peers via the existing federation discovery channels.

## Zero-Knowledge Proof Security Implications

Zero-knowledge circuits are used for selective disclosure and contract verification. Ensure that proof generation happens in a trusted execution environment and that verification keys are distributed over authenticated channels. Compromised proving keys can lead to forged proofs and network instability.

## CRDT Conflict Resolution Impacts on Security

CRDT-based data synchronization protects against data loss but can hide malicious updates if conflict rules are not audited. Monitor merge events and maintain an audit log of state transitions. Regularly verify that the CRDT state matches the intended policy outcomes.
