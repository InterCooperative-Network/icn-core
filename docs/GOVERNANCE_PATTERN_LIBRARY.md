# Governance Pattern Library

ICN Core ships with a small library of Cooperative Contract Language templates. These serve as starting points for common governance structures.

| Template | Description |
|----------|-------------|
| `rotating_stewards.ccl` | Rotates stewardship duties each week |
| `cooperative_council.ccl` | Council approves proposals by majority vote |
| `general_assembly.ccl` | All members vote with equal weight |

You can access the same source text programmatically through the `icn-templates` crate:

```rust
use icn_templates::GENERAL_ASSEMBLY;
```

The pattern library will expand over time with additional examples. Contributions are welcome!
