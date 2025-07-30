use icn_common::{Cid, Did, FixedTimeProvider};
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, ResolutionAction,
    ResolutionProposal, VoteOption,
};
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[test]
fn execute_resolution_proposal() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let paused = Arc::new(AtomicBool::new(false));
    let frozen = Arc::new(AtomicBool::new(false));
    let pause_f = paused.clone();
    let freeze_f = frozen.clone();
    let mut gov = GovernanceModule::new();
    gov.set_callback(move |p: &icn_governance::Proposal| {
        if let ProposalType::Resolution(res) = &p.proposal_type {
            for a in &res.actions {
                match a {
                    ResolutionAction::PauseCredential(_) => pause_f.store(true, Ordering::SeqCst),
                    ResolutionAction::FreezeReputation(_) => freeze_f.store(true, Ordering::SeqCst),
                }
            }
        }
        Ok(())
    });
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);
    let cid = Cid::new_v1_sha256(0x55, b"c");
    let pid = gov
        .submit_proposal(
            ProposalSubmission {
                proposer: Did::from_str("did:example:alice").unwrap(),
                proposal_type: ProposalType::Resolution(ResolutionProposal {
                    actions: vec![
                        ResolutionAction::PauseCredential(cid.clone()),
                        ResolutionAction::FreezeReputation(
                            Did::from_str("did:example:bob").unwrap(),
                        ),
                    ],
                }),
                description: "dispute".into(),
                duration_secs: 1,
                quorum: None,
                threshold: None,
                content_cid: None,
            },
            &time_provider,
        )
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
        &time_provider,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
        &time_provider,
    )
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();
    assert!(paused.load(Ordering::SeqCst));
    assert!(frozen.load(Ordering::SeqCst));
}
