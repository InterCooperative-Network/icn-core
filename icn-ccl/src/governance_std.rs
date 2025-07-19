use std::collections::HashMap;

/// Standard macros for cooperative governance patterns.
pub fn governance_macros() -> HashMap<String, String> {
    let mut m = HashMap::new();

    m.insert(
        "is_quorum_met".to_string(),
        "fn is_quorum_met(participant_count: Integer, total_members: Integer, quorum_percent: Integer) -> Bool {\n    let required = (total_members * quorum_percent) / 100;\n    return participant_count >= required;\n}".to_string(),
    );

    m.insert(
        "conduct_delegated_vote".to_string(),
        "fn conduct_delegated_vote(\n    direct_votes: Integer,\n    delegated_votes: Integer,\n    delegation_weight: Integer\n) -> Integer {\n    let weighted_delegated = (delegated_votes * delegation_weight) / 100;\n    return direct_votes + weighted_delegated;\n}".to_string(),
    );

    m.insert(
        "select_council".to_string(),
        "fn select_council(cycle: Integer, members: Array<Integer>, seats: Integer) -> Array<Integer> {\n    let count = array_len(members);\n    let i = 0;\n    let council = [];\n    while i < seats {\n        let idx = (cycle + i) % count;\n        array_push(council, members[idx]);\n        let i = i + 1;\n    }\n    return council;\n}".to_string(),
    );

    m
}
