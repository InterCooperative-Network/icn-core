# CCL Library - Reusable Governance Primitives

This directory contains pre-built CCL contracts that implement common governance patterns for cooperative organizations.

## Contracts

### Assembly Governance (`assembly_governance.ccl`)
- Large-scale democratic assemblies with delegation
- Supports delegated voting and quorum requirements
- Implements time-bound proposals with weighted voting

**Features:**
- Delegation of voting power within limits
- Assembly quorum requirements
- Time-bounded voting periods
- Reputation-based delegation weights

### Budgeting (`budgeting.ccl`)
- Multi-tier budget approval system
- Category-based fund allocation
- Progressive approval requirements based on amount

**Features:**
- Three-tier approval system (Simple/Committee/Assembly)
- Budget category management
- Deadline-based request processing
- Reputation-based approval authority

### Reputation-Weighted Voting (`reputation_voting.ccl`)
- Voting system with reputation-based weights
- Supports both linear and quadratic scaling
- Includes reputation management features

**Features:**
- Quadratic or linear reputation scaling
- Minimum reputation requirements
- Vote weight transparency
- Reputation adjustment mechanisms

## Usage

These contracts can be imported and used as building blocks for more complex governance systems:

```ccl
import "ccl-lib/assembly_governance.ccl" as Assembly;
import "ccl-lib/budgeting.ccl" as Budget;
import "ccl-lib/reputation_voting.ccl" as ReputationVote;

fn create_cooperative_governance() -> Bool {
    let proposal = Assembly.create_assembly_proposal(
        host_get_caller(),
        "Budget Allocation for Q4",
        WEEK
    );
    
    let budget_request = Budget.create_budget_request(
        host_get_caller(),
        5000,
        "operations",
        "Quarterly operational expenses"
    );
    
    return true;
}
```

## Design Principles

1. **Modularity**: Each contract focuses on a specific governance aspect
2. **Composability**: Contracts can be combined to create complex systems
3. **Transparency**: All voting and approval processes are auditable
4. **Flexibility**: Parameters can be adjusted for different cooperative needs
5. **Anti-Extraction**: Built-in safeguards prevent wealth concentration

## Integration with ICN

These contracts integrate with the ICN runtime through host functions:
- `host_get_reputation()` - Get caller's reputation score
- `host_get_current_time()` - Get current timestamp
- `host_get_caller()` - Get the caller's DID
- `host_submit_mesh_job()` - Submit work to the mesh
- `host_anchor_receipt()` - Anchor execution receipts

## Testing

Each contract includes a `run()` function that demonstrates basic usage and can be used for testing.

## Contributing

To add new governance primitives:
1. Follow the existing naming and structure patterns
2. Include comprehensive comments
3. Implement a `run()` function for testing
4. Update this README with contract details