# CCL Developer Onboarding Guide

Welcome to CCL (Cooperative Contract Language) development! This guide will help you get started with building governance contracts for cooperative organizations.

## 🚀 Quick Start

### Prerequisites

1. **Rust**: CCL is built with Rust. Install it from [rustup.rs](https://rustup.rs/)
2. **ICN Core**: Clone the ICN Core repository
3. **VSCode** (recommended): With the ICN CCL Tools extension

### Installation

```bash
# Clone the repository
git clone https://github.com/InterCooperative-Network/icn-core
cd icn-core

# Install CCL tools
cargo install --path icn-ccl --bin ccl-cli
cargo install --path icn-ccl --bin ccl-lsp

# Set up development environment
just setup  # or cargo build if you don't have just
```

### Your First Contract

Create a new CCL package:

```bash
ccl-cli package init my-coop "Your Name"
cd my-coop
```

This creates a basic governance contract structure:

```ccl
// CCL Version: 0.2.0
contract my_coop {
    state {
        // Add your state variables here
    }

    proposal CreateProposal {
        title: string,
        description: string,
        // Add proposal fields here
    }

    function initialize() {
        // Initialize your contract here
        log("Contract my_coop initialized");
    }

    policy vote_on_proposal {
        quorum: 50%,
        threshold: 66%,
        deadline: 7d,
    }
}
```

## 📚 Learning Path

### 1. Understanding CCL Basics

CCL is designed specifically for cooperative governance. Key concepts:

- **Contracts**: Define the governance structure
- **Proposals**: Democratic decision-making mechanisms
- **Policies**: Rules for voting and decision execution
- **Roles**: Permission-based access control
- **State**: Persistent data storage

### 2. Language Fundamentals

#### Data Types

```ccl
// Primitive types
let member_count: u32 = 100;
let budget: mana = 50000;
let active: bool = true;
let name: string = "Alice";

// Cooperative-specific types
let member_id: did = get_caller();
let content_hash: cid = calculate_cid(data);
let address: address = "0x742d35Cc...";
```

#### Functions

```ccl
function calculate_dividend(total_profit: mana, member_count: u32) -> mana {
    require(member_count > 0, "No members to distribute to");
    return total_profit / member_count;
}
```

#### Control Flow

```ccl
function process_vote(vote_type: string) {
    if vote_type == "approve" {
        log("Vote approved");
    } else if vote_type == "reject" {
        log("Vote rejected");
    } else {
        log("Vote abstained");
    }
}
```

### 3. Governance Patterns

#### Simple Voting

```ccl
contract simple_voting {
    state {
        proposals: Map<string, Proposal>,
        votes: Map<string, Map<did, VoteType>>,
    }

    proposal SimpleProposal {
        id: string,
        title: string,
        description: string,
        created_at: timestamp,
        deadline: timestamp,
    }

    function create_proposal(title: string, description: string) -> string {
        let proposal_id = hash(concat(title, now().to_string()));
        let proposal = SimpleProposal {
            id: proposal_id,
            title: title,
            description: description,
            created_at: now(),
            deadline: now() + days(7),
        };
        
        proposals.insert(proposal_id, proposal);
        log(concat("Proposal created: ", title));
        return proposal_id;
    }

    function cast_vote(proposal_id: string, vote: VoteType) {
        require(proposals.contains(proposal_id), "Proposal not found");
        require(now() < proposals.get(proposal_id).deadline, "Voting deadline passed");
        
        let voter = get_caller();
        votes.get_mut(proposal_id).insert(voter, vote);
        
        log("Vote cast successfully");
    }

    policy voting_rules {
        quorum: 50%,
        threshold: 66%,
        deadline: 7d,
    }
}
```

#### Ranked Choice Voting

```ccl
contract ranked_choice_voting {
    state {
        elections: Map<string, Election>,
        ballots: Map<string, Array<Ballot>>,
    }

    proposal Election {
        id: string,
        title: string,
        candidates: Array<string>,
        start_time: timestamp,
        end_time: timestamp,
    }

    struct Ballot {
        voter: did,
        rankings: Array<string>, // Candidate IDs in order of preference
    }

    function create_election(title: string, candidates: Array<string>) -> string {
        let election_id = hash(concat(title, now().to_string()));
        let election = Election {
            id: election_id,
            title: title,
            candidates: candidates,
            start_time: now(),
            end_time: now() + days(14),
        };
        
        elections.insert(election_id, election);
        ballots.insert(election_id, []);
        
        return election_id;
    }

    function submit_ballot(election_id: string, rankings: Array<string>) {
        require(elections.contains(election_id), "Election not found");
        let election = elections.get(election_id);
        require(now() >= election.start_time, "Election not started");
        require(now() <= election.end_time, "Election ended");
        
        let ballot = Ballot {
            voter: get_caller(),
            rankings: rankings,
        };
        
        ballots.get_mut(election_id).push(ballot);
        log("Ballot submitted");
    }
}
```

#### Budget Allocation

```ccl
contract participatory_budget {
    state {
        total_budget: mana,
        projects: Map<string, Project>,
        allocations: Map<string, mana>,
        member_budgets: Map<did, mana>,
    }

    struct Project {
        id: string,
        title: string,
        description: string,
        requested_amount: mana,
        category: string,
    }

    function set_budget(amount: mana) {
        require(get_caller() == get_contract(), "Only contract can set budget");
        total_budget = amount;
        
        // Distribute equal voting tokens to all members
        let member_count = get_member_count();
        let tokens_per_member = amount / member_count;
        
        for member in get_members() {
            member_budgets.insert(member, tokens_per_member);
        }
    }

    function submit_project(title: string, description: string, amount: mana, category: string) -> string {
        let project_id = hash(concat(title, get_caller().to_string()));
        let project = Project {
            id: project_id,
            title: title,
            description: description,
            requested_amount: amount,
            category: category,
        };
        
        projects.insert(project_id, project);
        allocations.insert(project_id, 0);
        
        return project_id;
    }

    function allocate_budget(project_id: string, amount: mana) {
        require(projects.contains(project_id), "Project not found");
        let voter = get_caller();
        require(member_budgets.get(voter) >= amount, "Insufficient budget tokens");
        
        member_budgets.get_mut(voter) -= amount;
        allocations.get_mut(project_id) += amount;
        
        log(concat("Allocated ", amount.to_string(), " to project"));
    }
}
```

## 🛠️ Development Tools

### VSCode Extension

Install the ICN CCL Tools extension for:
- Syntax highlighting
- Autocompletion
- Error checking
- Go-to-definition
- Integrated compilation

### Command Line Tools

```bash
# Compile a contract
ccl-cli compile my_contract.ccl

# Debug a contract
ccl-cli debug my_contract.ccl --interactive

# Format code
ccl-cli format src/*.ccl

# Migrate to newer version
ccl-cli migrate upgrade old_contract.ccl --target-version 0.2.0

# Convert from Solidity
ccl-cli migrate convert my_contract.sol --output my_contract.ccl --source-language solidity
```

### Package Management

```bash
# Create new package
ccl-cli package init voting-system "Alice Cooper"

# Add dependency
ccl-cli package add governance-lib "^1.0.0"

# Install dependencies
ccl-cli package install
```

## 🎯 Best Practices

### 1. Security

- Always use `require()` for input validation
- Be careful with arithmetic operations (check for overflow)
- Validate all external inputs
- Use access controls for sensitive functions

```ccl
function withdraw_funds(amount: mana) {
    require(is_member(get_caller()), "Only members can withdraw");
    require(amount <= get_balance(), "Insufficient funds");
    require(amount > 0, "Amount must be positive");
    
    transfer(get_caller(), amount);
}
```

### 2. Gas Efficiency

- Minimize storage operations
- Use local variables for repeated calculations
- Avoid unnecessary loops

```ccl
function batch_process(items: Array<string>) {
    let processed_count = 0; // Local variable
    
    for item in items {
        if process_item(item) {
            processed_count += 1;
        }
    }
    
    // Single storage write
    total_processed += processed_count;
}
```

### 3. Testing

Write comprehensive tests for your contracts:

```ccl
// In your test file
#[test]
function test_voting_process() {
    let contract = deploy_test_contract();
    let proposal_id = contract.create_proposal("Test Proposal", "Description");
    
    contract.cast_vote(proposal_id, VoteType::Approve);
    let result = contract.tally_votes(proposal_id);
    
    assert(result.approved == 1, "Vote should be recorded");
}
```

### 4. Documentation

Document your contracts thoroughly:

```ccl
/// Manages membership in a cooperative organization
contract membership_manager {
    /// Total number of active members
    state member_count: u32;
    
    /// Map of member DIDs to their status
    state members: Map<did, MemberStatus>;
    
    /// Add a new member to the cooperative
    /// @param member_did The DID of the new member
    /// @param initial_stake The initial stake amount
    function add_member(member_did: did, initial_stake: mana) {
        require(!members.contains(member_did), "Member already exists");
        require(initial_stake >= MIN_STAKE, "Stake too low");
        
        members.insert(member_did, MemberStatus::Active);
        member_count += 1;
        
        log("New member added successfully");
    }
}
```

## 🔄 Migration Guide

### Upgrading from CCL v0.1.x to v0.2.x

The migration tool can automatically update most syntax:

```bash
ccl-cli migrate upgrade old_contract.ccl --target-version 0.2.0
```

Key changes:
- `rule` → `policy`
- `when...then` → `if { }`
- `charge()` → `require_payment()`

### Converting from Solidity

```bash
ccl-cli migrate convert MyContract.sol --output my_contract.ccl --source-language solidity
```

Note: Automatic conversion requires manual review for:
- Event handling
- Modifier logic
- Complex inheritance patterns

## 📖 Advanced Topics

### Custom Types

```ccl
enum VoteType {
    Approve,
    Reject,
    Abstain,
}

struct Member {
    did: did,
    join_date: timestamp,
    reputation: u32,
    stake: mana,
}
```

### Error Handling

```ccl
function safe_divide(a: u64, b: u64) -> Result<u64, string> {
    if b == 0 {
        return Err("Division by zero");
    }
    return Ok(a / b);
}
```

### Async Operations

```ccl
async function fetch_external_data(url: string) -> Result<string, string> {
    // Fetch data from external source
    let response = http_get(url).await?;
    return Ok(response.body);
}
```

## 🤝 Community

- **GitHub**: [InterCooperative-Network/icn-core](https://github.com/InterCooperative-Network/icn-core)
- **Documentation**: [ICN Developer Docs](https://docs.intercooperative.network)
- **Examples**: Browse the `examples/` directory for real-world contracts

## 🐛 Troubleshooting

### Common Issues

1. **Compilation Errors**
   ```
   Error: Unexpected token 'rule'
   ```
   Solution: Update to CCL v0.2.x syntax or use migration tool

2. **Missing Dependencies**
   ```
   Error: Package 'governance-lib' not found
   ```
   Solution: Run `ccl-cli package install`

3. **LSP Not Working**
   - Check VSCode settings
   - Verify `ccl-lsp` is installed
   - Restart VSCode

### Getting Help

1. Check the error message carefully
2. Look for similar issues in GitHub
3. Use `ccl-cli migrate detect` to identify version issues
4. Ask for help in the community forums

---

**Next Steps**: Try building your first governance contract using the examples above, then explore the advanced patterns in the `examples/` directory!