use icn_runtime::abi::*;

#[test]
fn abi_constants_are_stable() {
    assert_eq!(ABI_HOST_ACCOUNT_GET_MANA, 10);
    assert_eq!(ABI_HOST_ACCOUNT_SPEND_MANA, 11);
    assert_eq!(ABI_HOST_ACCOUNT_CREDIT_MANA, 12);
    assert_eq!(ABI_HOST_SUBMIT_MESH_JOB, 16);
    assert_eq!(ABI_HOST_GET_PENDING_MESH_JOBS, 22);
    assert_eq!(ABI_HOST_ANCHOR_RECEIPT, 23);
    assert_eq!(ABI_HOST_GET_REPUTATION, 24);
    assert_eq!(ABI_HOST_VERIFY_ZK_PROOF, 25);
    assert_eq!(ABI_HOST_GENERATE_ZK_PROOF, 26);
    assert_eq!(ABI_HOST_CREATE_GOVERNANCE_PROPOSAL, 17);
    assert_eq!(ABI_HOST_OPEN_GOVERNANCE_VOTING, 18);
    assert_eq!(ABI_HOST_CAST_GOVERNANCE_VOTE, 19);
    assert_eq!(ABI_HOST_CLOSE_VOTING_AND_VERIFY, 20);
    assert_eq!(ABI_HOST_EXECUTE_GOVERNANCE_PROPOSAL, 21);
}
