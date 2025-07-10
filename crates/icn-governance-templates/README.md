# ICN Governance Templates

This crate provides reusable Cooperative Contract Language (CCL) snippets for
common governance patterns. Cooperatives can use these templates as a starting
point and modify them to fit local rules.

## Available Templates

- **Voting Logic** – Basic workflow for opening, casting, and closing votes.
- **Treasury Rules** – Simplified example of spending approvals from a shared
  treasury.

Each template is accessible through a helper function returning the CCL source
as a string.

## Customization

1. Add this crate as a dependency in your tools or tests.
2. Call the template function to retrieve the source.
3. Modify the source string or copy the template file to your cooperative's
   repository and adjust parameters such as thresholds or spending limits.

The templates themselves are intentionally minimal so that cooperatives can
extend them with additional checks or integrate them into larger contracts.
