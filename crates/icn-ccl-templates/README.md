# ICN CCL Templates

This crate ships reusable Cooperative Contract Language (CCL) templates.

These templates capture common governance patterns so cooperatives can
quickly bootstrap new policies. Copy a template and modify the functions
or thresholds to fit your needs.

Available templates:
- `VOTING_TEMPLATE` – basic voting workflow
- `TREASURY_TEMPLATE` – simple treasury accounting

The template source is embedded at compile time and exposed as `&'static str`.
You can write the string to a `.ccl` file or pass it directly to the
`icn-ccl` compiler.
