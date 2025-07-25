# CCL Governance Patterns Tutorial

This tutorial demonstrates common governance patterns implemented in CCL (Cooperative Contract Language).

## Table of Contents

1. [Simple Democracy](#simple-democracy)
2. [Liquid Democracy](#liquid-democracy) 
3. [Consensus Decision Making](#consensus-decision-making)
4. [Participatory Budgeting](#participatory-budgeting)
5. [Reputation-Based Voting](#reputation-based-voting)
6. [Multi-Stage Governance](#multi-stage-governance)
7. [Resource Allocation](#resource-allocation)
8. [Conflict Resolution](#conflict-resolution)

## Simple Democracy

The most basic form of cooperative governance - one member, one vote.

```ccl
// CCL Version: 0.2.0
contract simple_democracy {
    state {
        members: Map<did, Member>,
        proposals: Map<string, Proposal>,
        votes: Map<string, Map<did, VoteChoice>>,
        proposal_count: u32,
    }

    struct Member {
        did: did,
        joined_at: timestamp,
        is_active: bool,
    }

    struct Proposal {
        id: string,
        title: string,
        description: string,
        proposer: did,
        created_at: timestamp,
        voting_deadline: timestamp,
        execution_deadline: timestamp,
        status: ProposalStatus,
        required_quorum: u32,
        required_threshold: u32,
    }

    enum VoteChoice {
        Approve,
        Reject,
        Abstain,
    }

    enum ProposalStatus {
        Active,      // Currently accepting votes
        Passed,      // Approved and ready for execution
        Rejected,    // Failed to meet threshold
        Executed,    // Successfully executed
        Expired,     // Deadline passed without execution
    }

    function initialize() {
        proposal_count = 0;
        log("Simple democracy contract initialized");
    }

    function add_member(new_member: did) {
        require(!members.contains(new_member), "Member already exists");
        
        let member = Member {
            did: new_member,
            joined_at: now(),
            is_active: true,
        };
        
        members.insert(new_member, member);
        log("New member added to democracy");
    }

    function create_proposal(title: string, description: string) -> string {
        require(is_active_member(get_caller()), "Only active members can create proposals");
        
        proposal_count += 1;
        let proposal_id = concat("proposal_", proposal_count.to_string());
        
        let proposal = Proposal {
            id: proposal_id,
            title: title,
            description: description,
            proposer: get_caller(),
            created_at: now(),
            voting_deadline: now() + days(7),
            execution_deadline: now() + days(14),
            status: ProposalStatus::Active,
            required_quorum: get_active_member_count() / 2, // 50% quorum
            required_threshold: 66, // 66% approval threshold
        };
        
        proposals.insert(proposal_id, proposal);
        votes.insert(proposal_id, Map::new());
        
        log(concat("Proposal created: ", title));
        return proposal_id;
    }

    function vote(proposal_id: string, choice: VoteChoice) {
        require(proposals.contains(proposal_id), "Proposal not found");
        require(is_active_member(get_caller()), "Only active members can vote");
        
        let proposal = proposals.get(proposal_id);
        require(proposal.status == ProposalStatus::Active, "Proposal not accepting votes");
        require(now() <= proposal.voting_deadline, "Voting deadline passed");
        
        votes.get_mut(proposal_id).insert(get_caller(), choice);
        log("Vote recorded");
    }

    function tally_votes(proposal_id: string) -> VoteResult {
        require(proposals.contains(proposal_id), "Proposal not found");
        
        let proposal = proposals.get(proposal_id);
        require(now() > proposal.voting_deadline, "Voting still active");
        
        let votes_map = votes.get(proposal_id);
        let mut approve_count = 0;
        let mut reject_count = 0;
        let mut abstain_count = 0;
        
        for (_, vote_choice) in votes_map {
            match vote_choice {
                VoteChoice::Approve => approve_count += 1,
                VoteChoice::Reject => reject_count += 1,
                VoteChoice::Abstain => abstain_count += 1,
            }
        }
        
        let total_votes = approve_count + reject_count + abstain_count;
        let quorum_met = total_votes >= proposal.required_quorum;
        let threshold_met = (approve_count * 100) / (approve_count + reject_count) >= proposal.required_threshold;
        
        let passed = quorum_met && threshold_met;
        
        // Update proposal status
        if passed {
            proposals.get_mut(proposal_id).status = ProposalStatus::Passed;
        } else {
            proposals.get_mut(proposal_id).status = ProposalStatus::Rejected;
        }
        
        return VoteResult {
            approve_count: approve_count,
            reject_count: reject_count,
            abstain_count: abstain_count,
            quorum_met: quorum_met,
            threshold_met: threshold_met,
            passed: passed,
        };
    }

    function is_active_member(member_did: did) -> bool {
        return members.contains(member_did) && members.get(member_did).is_active;
    }

    function get_active_member_count() -> u32 {
        let mut count = 0;
        for (_, member) in members {
            if member.is_active {
                count += 1;
            }
        }
        return count;
    }

    struct VoteResult {
        approve_count: u32,
        reject_count: u32,
        abstain_count: u32,
        quorum_met: bool,
        threshold_met: bool,
        passed: bool,
    }
}
```

## Liquid Democracy

Allows members to delegate their voting power to trusted representatives.

```ccl
// CCL Version: 0.2.0
contract liquid_democracy {
    state {
        members: Map<did, Member>,
        delegations: Map<did, did>, // voter -> delegate
        proposals: Map<string, Proposal>,
        votes: Map<string, Map<did, VoteChoice>>,
    }

    struct Member {
        did: did,
        expertise_areas: Array<string>,
        is_active: bool,
    }

    function delegate_vote(delegate: did, topic_area: string) {
        require(is_active_member(get_caller()), "Only active members can delegate");
        require(is_active_member(delegate), "Delegate must be active member");
        require(delegate != get_caller(), "Cannot delegate to yourself");
        
        // Store delegation
        delegations.insert(get_caller(), delegate);
        
        log(concat("Vote delegated to ", delegate.to_string()));
    }

    function revoke_delegation() {
        require(delegations.contains(get_caller()), "No delegation to revoke");
        
        delegations.remove(get_caller());
        log("Delegation revoked");
    }

    function vote_with_delegation(proposal_id: string, choice: VoteChoice) {
        let voter = get_caller();
        
        // Calculate voting power (direct + delegated)
        let voting_power = calculate_voting_power(voter, proposal_id);
        
        // Record vote with weighted power
        record_weighted_vote(proposal_id, voter, choice, voting_power);
    }

    function calculate_voting_power(voter: did, proposal_id: string) -> u32 {
        let mut power = 1; // Base vote
        
        // Add delegated votes
        for (delegator, delegate) in delegations {
            if delegate == voter && !has_voted_directly(proposal_id, delegator) {
                power += 1;
            }
        }
        
        return power;
    }

    function has_voted_directly(proposal_id: string, voter: did) -> bool {
        return votes.get(proposal_id).contains(voter);
    }

    function record_weighted_vote(proposal_id: string, voter: did, choice: VoteChoice, power: u32) {
        // Implementation would record the weighted vote
        log(concat("Recorded vote with power: ", power.to_string()));
    }
}
```

## Consensus Decision Making

Implements consent-based decision making where proposals pass unless there are strong objections.

```ccl
// CCL Version: 0.2.0
contract consensus_democracy {
    state {
        proposals: Map<string, ConsensusProposal>,
        objections: Map<string, Array<Objection>>,
        support_levels: Map<string, Map<did, SupportLevel>>,
    }

    struct ConsensusProposal {
        id: string,
        title: string,
        description: string,
        proposer: did,
        discussion_deadline: timestamp,
        consent_deadline: timestamp,
        status: ConsensusStatus,
    }

    enum SupportLevel {
        StrongSupport,    // Enthusiastic yes
        Support,          // Willing to go along
        Neutral,          // No strong opinion
        Concern,          // Has concerns but won't block
        Objection,        // Strong objection, would block
    }

    enum ConsensusStatus {
        Discussion,       // Open for discussion and concerns
        ConsentRound,     // Checking for objections
        Consensus,        // No blocking objections
        Blocked,          // Has blocking objections
        Modified,         // Modified to address concerns
    }

    struct Objection {
        objector: did,
        concern: string,
        is_blocking: bool,
        proposed_solution: string,
    }

    function create_consensus_proposal(title: string, description: string) -> string {
        let proposal_id = hash(concat(title, now().to_string()));
        
        let proposal = ConsensusProposal {
            id: proposal_id,
            title: title,
            description: description,
            proposer: get_caller(),
            discussion_deadline: now() + days(14), // 2 weeks discussion
            consent_deadline: now() + days(21),    // 1 week for consent round
            status: ConsensusStatus::Discussion,
        };
        
        proposals.insert(proposal_id, proposal);
        objections.insert(proposal_id, []);
        support_levels.insert(proposal_id, Map::new());
        
        return proposal_id;
    }

    function express_support(proposal_id: string, level: SupportLevel) {
        require(proposals.contains(proposal_id), "Proposal not found");
        require(is_active_member(get_caller()), "Only members can express support");
        
        support_levels.get_mut(proposal_id).insert(get_caller(), level);
        
        if level == SupportLevel::Objection {
            start_consent_round(proposal_id);
        }
    }

    function raise_objection(proposal_id: string, concern: string, is_blocking: bool, solution: string) {
        require(proposals.contains(proposal_id), "Proposal not found");
        
        let objection = Objection {
            objector: get_caller(),
            concern: concern,
            is_blocking: is_blocking,
            proposed_solution: solution,
        };
        
        objections.get_mut(proposal_id).push(objection);
        
        if is_blocking {
            proposals.get_mut(proposal_id).status = ConsensusStatus::Blocked;
        }
        
        log("Objection raised - seeking resolution");
    }

    function resolve_objection(proposal_id: string, objection_index: u32, resolution: string) {
        require(proposals.contains(proposal_id), "Proposal not found");
        
        let objections_list = objections.get_mut(proposal_id);
        require(objection_index < objections_list.len(), "Invalid objection index");
        
        // Mark objection as resolved
        objections_list.remove(objection_index);
        
        // Check if all blocking objections are resolved
        let has_blocking_objections = objections_list.iter().any(|obj| obj.is_blocking);
        
        if !has_blocking_objections {
            proposals.get_mut(proposal_id).status = ConsensusStatus::Consensus;
        }
        
        log("Objection resolved through dialogue");
    }

    function start_consent_round(proposal_id: string) {
        let proposal = proposals.get_mut(proposal_id);
        if proposal.status == ConsensusStatus::Discussion {
            proposal.status = ConsensusStatus::ConsentRound;
            log("Consent round started - final check for objections");
        }
    }

    function check_consensus(proposal_id: string) -> bool {
        let objections_list = objections.get(proposal_id);
        let has_blocking = objections_list.iter().any(|obj| obj.is_blocking);
        
        return !has_blocking;
    }
}
```

## Participatory Budgeting

Democratic allocation of cooperative resources.

```ccl
// CCL Version: 0.2.0
contract participatory_budget {
    state {
        total_budget: mana,
        budget_cycles: Map<string, BudgetCycle>,
        projects: Map<string, Project>,
        allocations: Map<string, Map<string, mana>>, // cycle_id -> project_id -> amount
        member_allowances: Map<string, Map<did, mana>>, // cycle_id -> member -> remaining allowance
    }

    struct BudgetCycle {
        id: string,
        total_amount: mana,
        submission_deadline: timestamp,
        voting_deadline: timestamp,
        status: BudgetStatus,
        categories: Array<string>,
    }

    struct Project {
        id: string,
        title: string,
        description: string,
        category: string,
        requested_amount: mana,
        proposer: did,
        submitted_at: timestamp,
        cycle_id: string,
    }

    enum BudgetStatus {
        Submission,   // Accepting project proposals
        Voting,       // Members allocating their budgets
        Completed,    // Voting finished, results finalized
    }

    function create_budget_cycle(cycle_id: string, amount: mana, categories: Array<string>) -> string {
        require(!budget_cycles.contains(cycle_id), "Cycle already exists");
        
        let cycle = BudgetCycle {
            id: cycle_id,
            total_amount: amount,
            submission_deadline: now() + days(30), // 30 days to submit projects
            voting_deadline: now() + days(45),     // 15 days to vote
            status: BudgetStatus::Submission,
            categories: categories,
        };
        
        budget_cycles.insert(cycle_id, cycle);
        allocations.insert(cycle_id, Map::new());
        member_allowances.insert(cycle_id, Map::new());
        
        // Distribute equal voting tokens to all members
        distribute_voting_tokens(cycle_id, amount);
        
        return cycle_id;
    }

    function submit_project(
        cycle_id: string,
        title: string,
        description: string,
        category: string,
        requested_amount: mana
    ) -> string {
        require(budget_cycles.contains(cycle_id), "Budget cycle not found");
        let cycle = budget_cycles.get(cycle_id);
        require(cycle.status == BudgetStatus::Submission, "Not accepting submissions");
        require(now() <= cycle.submission_deadline, "Submission deadline passed");
        require(cycle.categories.contains(category), "Invalid category");
        
        let project_id = hash(concat(title, get_caller().to_string()));
        
        let project = Project {
            id: project_id,
            title: title,
            description: description,
            category: category,
            requested_amount: requested_amount,
            proposer: get_caller(),
            submitted_at: now(),
            cycle_id: cycle_id,
        };
        
        projects.insert(project_id, project);
        allocations.get_mut(cycle_id).insert(project_id, 0);
        
        log("Project submitted for participatory budgeting");
        return project_id;
    }

    function allocate_budget(cycle_id: string, project_id: string, amount: mana) {
        require(budget_cycles.contains(cycle_id), "Budget cycle not found");
        require(projects.contains(project_id), "Project not found");
        
        let cycle = budget_cycles.get(cycle_id);
        require(cycle.status == BudgetStatus::Voting, "Not in voting phase");
        require(now() <= cycle.voting_deadline, "Voting deadline passed");
        
        let voter = get_caller();
        let remaining_allowance = member_allowances.get(cycle_id).get(voter);
        require(remaining_allowance >= amount, "Insufficient voting tokens");
        
        // Deduct from member's allowance
        member_allowances.get_mut(cycle_id).get_mut(voter) -= amount;
        
        // Add to project allocation
        allocations.get_mut(cycle_id).get_mut(project_id) += amount;
        
        log(concat("Allocated ", amount.to_string(), " voting tokens to project"));
    }

    function finalize_budget_cycle(cycle_id: string) -> Array<ProjectResult> {
        require(budget_cycles.contains(cycle_id), "Budget cycle not found");
        
        let cycle = budget_cycles.get_mut(cycle_id);
        require(now() > cycle.voting_deadline, "Voting still active");
        
        cycle.status = BudgetStatus::Completed;
        
        // Calculate results
        let mut results = [];
        let cycle_allocations = allocations.get(cycle_id);
        
        for (project_id, allocated_amount) in cycle_allocations {
            let project = projects.get(project_id);
            let funded = allocated_amount >= project.requested_amount;
            
            let result = ProjectResult {
                project_id: project_id,
                title: project.title,
                requested: project.requested_amount,
                allocated: allocated_amount,
                funded: funded,
            };
            
            results.push(result);
        }
        
        return results;
    }

    function distribute_voting_tokens(cycle_id: string, total_budget: mana) {
        let member_count = get_active_member_count();
        let tokens_per_member = total_budget / member_count;
        
        for member in get_active_members() {
            member_allowances.get_mut(cycle_id).insert(member, tokens_per_member);
        }
    }

    struct ProjectResult {
        project_id: string,
        title: string,
        requested: mana,
        allocated: mana,
        funded: bool,
    }
}
```

## Reputation-Based Voting

Voting power based on member contributions and expertise.

```ccl
// CCL Version: 0.2.0
contract reputation_voting {
    state {
        members: Map<did, ReputationMember>,
        reputation_scores: Map<did, Map<string, u32>>, // member -> domain -> score
        proposals: Map<string, ReputationProposal>,
        weighted_votes: Map<string, Map<did, WeightedVote>>,
    }

    struct ReputationMember {
        did: did,
        total_reputation: u32,
        expertise_domains: Array<string>,
        contribution_history: Array<Contribution>,
    }

    struct ReputationProposal {
        id: string,
        title: string,
        description: string,
        domain: string, // Which expertise area is most relevant
        proposer: did,
        created_at: timestamp,
        voting_deadline: timestamp,
    }

    struct WeightedVote {
        voter: did,
        choice: VoteChoice,
        weight: u32, // Based on reputation in relevant domain
        timestamp: timestamp,
    }

    struct Contribution {
        description: string,
        domain: string,
        reputation_gained: u32,
        timestamp: timestamp,
    }

    function add_reputation(member: did, domain: string, points: u32, description: string) {
        require(is_authorized_to_award_reputation(get_caller()), "Not authorized");
        
        // Add to domain-specific reputation
        if !reputation_scores.contains(member) {
            reputation_scores.insert(member, Map::new());
        }
        
        let current_score = reputation_scores.get(member).get(domain).unwrap_or(0);
        reputation_scores.get_mut(member).insert(domain, current_score + points);
        
        // Update member's total reputation
        if members.contains(member) {
            members.get_mut(member).total_reputation += points;
            
            // Add to contribution history
            let contribution = Contribution {
                description: description,
                domain: domain,
                reputation_gained: points,
                timestamp: now(),
            };
            members.get_mut(member).contribution_history.push(contribution);
        }
        
        log(concat("Reputation awarded: ", points.to_string(), " in ", domain));
    }

    function create_reputation_proposal(title: string, description: string, domain: string) -> string {
        let proposal_id = hash(concat(title, now().to_string()));
        
        let proposal = ReputationProposal {
            id: proposal_id,
            title: title,
            description: description,
            domain: domain,
            proposer: get_caller(),
            created_at: now(),
            voting_deadline: now() + days(7),
        };
        
        proposals.insert(proposal_id, proposal);
        weighted_votes.insert(proposal_id, Map::new());
        
        return proposal_id;
    }

    function vote_with_reputation(proposal_id: string, choice: VoteChoice) {
        require(proposals.contains(proposal_id), "Proposal not found");
        let proposal = proposals.get(proposal_id);
        require(now() <= proposal.voting_deadline, "Voting deadline passed");
        
        let voter = get_caller();
        let voting_weight = calculate_voting_weight(voter, proposal.domain);
        
        let weighted_vote = WeightedVote {
            voter: voter,
            choice: choice,
            weight: voting_weight,
            timestamp: now(),
        };
        
        weighted_votes.get_mut(proposal_id).insert(voter, weighted_vote);
        
        log(concat("Reputation-weighted vote cast with weight: ", voting_weight.to_string()));
    }

    function calculate_voting_weight(voter: did, domain: string) -> u32 {
        let base_weight = 10; // Minimum weight for any member
        let domain_reputation = reputation_scores.get(voter).get(domain).unwrap_or(0);
        
        // Weight = base + (domain reputation / 10)
        return base_weight + (domain_reputation / 10);
    }

    function tally_reputation_votes(proposal_id: string) -> ReputationVoteResult {
        let votes_map = weighted_votes.get(proposal_id);
        let mut approve_weight = 0;
        let mut reject_weight = 0;
        let mut abstain_weight = 0;
        
        for (_, vote) in votes_map {
            match vote.choice {
                VoteChoice::Approve => approve_weight += vote.weight,
                VoteChoice::Reject => reject_weight += vote.weight,
                VoteChoice::Abstain => abstain_weight += vote.weight,
            }
        }
        
        let total_weight = approve_weight + reject_weight + abstain_weight;
        let approval_percentage = if total_weight > 0 {
            (approve_weight * 100) / total_weight
        } else { 0 };
        
        return ReputationVoteResult {
            approve_weight: approve_weight,
            reject_weight: reject_weight,
            abstain_weight: abstain_weight,
            total_weight: total_weight,
            approval_percentage: approval_percentage,
            passed: approval_percentage >= 60, // 60% threshold
        };
    }

    function is_authorized_to_award_reputation(caller: did) -> bool {
        // Only existing high-reputation members can award reputation
        if !members.contains(caller) {
            return false;
        }
        
        let member = members.get(caller);
        return member.total_reputation >= 100; // Minimum 100 reputation to award
    }

    struct ReputationVoteResult {
        approve_weight: u32,
        reject_weight: u32,
        abstain_weight: u32,
        total_weight: u32,
        approval_percentage: u32,
        passed: bool,
    }
}
```

## Best Practices

### 1. Always Validate Inputs

```ccl
function transfer_funds(recipient: did, amount: mana) {
    require(is_active_member(recipient), "Recipient must be active member");
    require(amount > 0, "Amount must be positive");
    require(amount <= get_available_balance(), "Insufficient funds");
    
    // Proceed with transfer
}
```

### 2. Use Events for Transparency

```ccl
function execute_proposal(proposal_id: string) {
    // ... execution logic ...
    
    log(concat("Proposal executed: ", proposal_id));
    emit ProposalExecuted(proposal_id, get_caller(), now());
}
```

### 3. Implement Time Locks for Critical Changes

```ccl
function change_governance_rules(new_rules: GovernanceRules) {
    require(proposal_passed_with_supermajority(), "Requires supermajority approval");
    
    // 7-day time lock before changes take effect
    schedule_change(new_rules, now() + days(7));
    
    log("Governance rule change scheduled");
}
```

### 4. Consider Gas Costs

```ccl
// Inefficient - loops through all members
function count_active_members_bad() -> u32 {
    let mut count = 0;
    for member in all_members {
        if member.is_active {
            count += 1;
        }
    }
    return count;
}

// Efficient - maintain counter
state active_member_count: u32;

function add_member(new_member: did) {
    members.insert(new_member, member_data);
    active_member_count += 1; // Update counter
}
```

## Conclusion

These patterns provide a foundation for building sophisticated governance systems in CCL. Mix and match them based on your cooperative's needs:

- **Simple Democracy**: Good for straightforward yes/no decisions
- **Liquid Democracy**: Useful when members have varying expertise
- **Consensus**: Best for decisions requiring broad agreement
- **Participatory Budgeting**: Essential for resource allocation
- **Reputation-Based**: Effective when expertise matters
- **Multi-Stage**: Good for complex decisions needing deliberation

Remember to test thoroughly and consider the specific needs and values of your cooperative when choosing governance patterns.