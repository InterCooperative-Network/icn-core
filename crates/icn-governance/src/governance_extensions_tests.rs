//! Tests for governance protocol extensions: time-locks, vetoes, and sponsorship

#[cfg(test)]
mod tests {
    use super::super::*;
    use icn_common::{Did, FixedTimeProvider};
    use std::collections::HashSet;
    use std::str::FromStr;

    fn create_test_governance() -> GovernanceModule {
        let mut config = GovernanceConfig::default();
        config.min_sponsors = 2;
        config.timelock_delay_secs = 86400; // 1 day
        config.veto_grace_period_secs = 172800; // 2 days

        let mut veto_members = HashSet::new();
        veto_members.insert(Did::default());
        config.veto_members = veto_members;

        GovernanceModule::with_config(config)
    }

    #[test]
    fn test_sponsorship_requirement() {
        let mut gov = create_test_governance();
        let proposer = Did::default();
        let sponsor1 = Did::from_str("did:key:sponsor1").unwrap();
        let sponsor2 = Did::from_str("did:key:sponsor2").unwrap();

        // Add members
        gov.add_member(proposer.clone());
        gov.add_member(sponsor1.clone());
        gov.add_member(sponsor2.clone());

        let time_provider = FixedTimeProvider::new(1000);

        // Submit proposal - should be in PendingSponsorship state
        let submission = ProposalSubmission {
            proposer: proposer.clone(),
            proposal_type: ProposalType::GenericText("Test proposal".to_string()),
            description: "A test proposal requiring sponsors".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
            timelock_delay: None,
        };

        let proposal_id = gov.submit_proposal(submission, &time_provider).unwrap();
        let proposal = gov.get_proposal(&proposal_id).unwrap().unwrap();
        assert_eq!(proposal.status, ProposalStatus::PendingSponsorship);

        // First sponsor
        gov.sponsor_proposal(&proposal_id, sponsor1.clone(), &time_provider)
            .unwrap();
        let proposal = gov.get_proposal(&proposal_id).unwrap().unwrap();
        assert_eq!(proposal.status, ProposalStatus::PendingSponsorship);
        assert_eq!(proposal.sponsorship.sponsors.len(), 1);

        // Second sponsor - should advance to Deliberation
        gov.sponsor_proposal(&proposal_id, sponsor2.clone(), &time_provider)
            .unwrap();
        let proposal = gov.get_proposal(&proposal_id).unwrap().unwrap();
        assert_eq!(proposal.status, ProposalStatus::Deliberation);
        assert_eq!(proposal.sponsorship.sponsors.len(), 2);
        assert!(proposal.sponsorship.sponsorship_complete_at.is_some());
    }

    #[test]
    fn test_governance_config_updates() {
        let mut gov = GovernanceModule::new();

        // Test default config
        assert_eq!(gov.config().min_sponsors, 1);
        assert_eq!(gov.config().timelock_delay_secs, 0);
        assert_eq!(gov.config().veto_grace_period_secs, 0);
        assert!(gov.config().veto_members.is_empty());

        // Update config
        let mut new_config = GovernanceConfig::default();
        new_config.min_sponsors = 3;
        new_config.timelock_delay_secs = 86400;
        new_config.veto_grace_period_secs = 172800;
        new_config.veto_members.insert(Did::default());

        gov.set_config(new_config.clone());

        assert_eq!(gov.config().min_sponsors, 3);
        assert_eq!(gov.config().timelock_delay_secs, 86400);
        assert_eq!(gov.config().veto_grace_period_secs, 172800);
        assert_eq!(gov.config().veto_members.len(), 1);
    }
}
