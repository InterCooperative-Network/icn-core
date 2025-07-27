/**
 * Comprehensive Governance Example
 * 
 * This example demonstrates:
 * - Proposal creation with different types
 * - Voting workflows and delegation
 * - Proposal status tracking and analytics
 * - CCL template-based proposal generation
 * - Advanced governance operations
 */

import { 
  ICNClient, 
  createStorage, 
  ICNGovernanceError,
  ICNValidationError,
  ErrorUtils,
  GovernanceUtils,
  CCLUtils,
  EnhancedUtils
} from '@icn/ts-sdk';

async function governanceExample() {
  console.log('🏛️  Starting Comprehensive Governance Example\n');

  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet',
    storage: createStorage('@governance-example:'),
  });

  try {
    await client.connect();
    console.log('✅ Connected to ICN node\n');

    // Example participants
    const cooperativeDid = 'did:key:cooperative123';
    const memberAlice = 'did:key:alice456';
    const memberBob = 'did:key:bob789';
    const memberCharlie = 'did:key:charlie012';
    const applicantDid = 'did:key:applicant345';

    // 1. System Parameter Change Proposal
    console.log('⚙️  Creating system parameter change proposal...');
    
    const paramChangeProposal = await client.governance.submitProposal({
      proposer_did: memberAlice,
      proposal: {
        type: "SystemParameterChange",
        data: { 
          param: "min_stake_amount", 
          value: "1000" 
        }
      },
      description: "Increase minimum stake amount to improve network security and reduce spam. This change will require existing members to maintain higher stakes but will enhance overall federation stability.",
      duration_secs: 7 * 24 * 3600, // 7 days
      quorum: 3, // Need at least 3 votes
      threshold: 0.66 // Need 66% approval
    });
    
    console.log('✅ System parameter proposal created');
    console.log(`   Proposal ID: ${paramChangeProposal}`);
    console.log('   Description: Increase minimum stake amount');
    console.log('   Duration: 7 days');
    console.log('   Quorum: 3 votes required');
    console.log('   Threshold: 66% approval needed\n');

    // 2. Member Admission Proposal
    console.log('👥 Creating member admission proposal...');
    
    const admissionProposal = await client.governance.submitProposal({
      proposer_did: memberBob,
      proposal: {
        type: "MemberAdmission",
        data: { 
          did: applicantDid 
        }
      },
      description: `Proposal to admit ${applicantDid} as a new member. The applicant has demonstrated commitment to cooperative principles and has valuable technical skills in distributed systems.`,
      duration_secs: 5 * 24 * 3600, // 5 days
      quorum: 2,
      threshold: 0.5 // Simple majority
    });
    
    console.log('✅ Member admission proposal created');
    console.log(`   Proposal ID: ${admissionProposal}`);
    console.log(`   Candidate: ${applicantDid}`);
    console.log('   Duration: 5 days');
    console.log('   Threshold: Simple majority\n');

    // 3. Software Upgrade Proposal
    console.log('🔄 Creating software upgrade proposal...');
    
    const upgradeProposal = await client.governance.submitProposal({
      proposer_did: memberCharlie,
      proposal: {
        type: "SoftwareUpgrade",
        data: { 
          version: "v2.1.0" 
        }
      },
      description: "Upgrade to ICN Core v2.1.0 featuring enhanced credential privacy, improved mesh computing performance, and new trust network algorithms. This update includes security patches and performance optimizations.",
      duration_secs: 10 * 24 * 3600, // 10 days for major upgrade
      quorum: 4,
      threshold: 0.75 // 75% approval for major changes
    });
    
    console.log('✅ Software upgrade proposal created');
    console.log(`   Proposal ID: ${upgradeProposal}`);
    console.log('   Version: v2.1.0');
    console.log('   Duration: 10 days');
    console.log('   Threshold: 75% approval required\n');

    // 4. Resolution Proposal with Multiple Actions
    console.log('📋 Creating resolution proposal...');
    
    const resolutionProposal = await client.governance.submitProposal({
      proposer_did: memberAlice,
      proposal: {
        type: "Resolution",
        data: { 
          actions: [
            {
              action: "FreezeReputation",
              data: { did: "did:key:violator999" }
            }
          ]
        }
      },
      description: "Emergency resolution to freeze reputation of member who violated community guidelines. This action is temporary pending investigation of reported misconduct.",
      duration_secs: 3 * 24 * 3600, // 3 days for urgent matters
      quorum: 3,
      threshold: 0.6
    });
    
    console.log('✅ Resolution proposal created');
    console.log(`   Proposal ID: ${resolutionProposal}`);
    console.log('   Actions: Freeze reputation');
    console.log('   Urgency: 3-day voting period\n');

    // 5. List All Proposals
    console.log('📊 Listing all proposals...');
    
    const allProposals = await client.governance.listProposals();
    
    console.log(`🗳️  Found ${allProposals.length} total proposals:`);
    allProposals.slice(0, 5).forEach((proposal, index) => {
      const progress = GovernanceUtils.calculateVotingProgress(proposal);
      const outcome = GovernanceUtils.getProposalOutcome(proposal);
      const typeFormatted = GovernanceUtils.formatProposalType(proposal.proposal_type);
      const summary = GovernanceUtils.generateProposalSummary(proposal.proposal_type);
      
      console.log(`   ${index + 1}. ${proposal.id.slice(0, 12)}...`);
      console.log(`      Type: ${typeFormatted}`);
      console.log(`      Summary: ${summary.slice(0, 60)}${summary.length > 60 ? '...' : ''}`);
      console.log(`      Status: ${EnhancedUtils.formatProposalStatus(proposal.status)}`);
      console.log(`      Progress: ${progress.toFixed(1)}%`);
      console.log(`      Outcome: ${outcome}`);
      console.log(`      Votes: ${proposal.votes.yes} Yes, ${proposal.votes.no} No, ${proposal.votes.abstain} Abstain`);
    });
    
    if (allProposals.length > 5) {
      console.log(`   ... and ${allProposals.length - 5} more proposals`);
    }
    console.log();

    // 6. Voting on Proposals
    console.log('🗳️  Casting votes on proposals...');
    
    // Vote on parameter change proposal
    await client.governance.castVote({
      voter_did: memberBob,
      proposal_id: paramChangeProposal,
      vote_option: "Yes"
    });
    console.log(`✅ Bob voted YES on parameter change proposal`);
    
    await client.governance.castVote({
      voter_did: memberCharlie,
      proposal_id: paramChangeProposal,
      vote_option: "Yes"
    });
    console.log(`✅ Charlie voted YES on parameter change proposal`);
    
    // Vote on admission proposal
    await client.governance.castVote({
      voter_did: memberAlice,
      proposal_id: admissionProposal,
      vote_option: "Yes"
    });
    console.log(`✅ Alice voted YES on admission proposal`);
    
    await client.governance.castVote({
      voter_did: memberCharlie,
      proposal_id: admissionProposal,
      vote_option: "Yes"
    });
    console.log(`✅ Charlie voted YES on admission proposal`);
    
    // Vote on upgrade proposal
    await client.governance.castVote({
      voter_did: memberAlice,
      proposal_id: upgradeProposal,
      vote_option: "No"
    });
    console.log(`✅ Alice voted NO on upgrade proposal (needs more testing)`);
    
    await client.governance.castVote({
      voter_did: memberBob,
      proposal_id: upgradeProposal,
      vote_option: "Abstain"
    });
    console.log(`✅ Bob abstained on upgrade proposal`);
    console.log();

    // 7. Vote Delegation
    console.log('🤝 Setting up vote delegation...');
    
    try {
      // Bob delegates his voting power to Alice for governance expertise
      const delegationResult = await client.governance.delegateVote({
        from_did: memberBob,
        to_did: memberAlice
      });
      
      console.log('✅ Vote delegation established');
      console.log(`   Delegator: ${memberBob}`);
      console.log(`   Delegate: ${memberAlice}`);
      console.log(`   Delegation ID: ${delegationResult}`);
    } catch (error) {
      console.log('⚠️  Vote delegation may not be supported in current configuration');
    }
    console.log();

    // 8. Check Updated Proposal Status
    console.log('📊 Checking updated proposal status...');
    
    const updatedParamProposal = await client.governance.getProposal(paramChangeProposal);
    const paramProgress = EnhancedUtils.calculateVotingProgress(updatedParamProposal);
    
    console.log('📋 Parameter Change Proposal Status:');
    console.log(`   Total Votes: ${paramProgress.totalVotes}`);
    console.log(`   Yes: ${paramProgress.yesPercentage.toFixed(1)}%`);
    console.log(`   No: ${paramProgress.noPercentage.toFixed(1)}%`);
    console.log(`   Abstain: ${paramProgress.abstainPercentage.toFixed(1)}%`);
    console.log(`   Quorum Reached: ${paramProgress.quorumReached ? '✅ Yes' : '❌ No'}`);
    console.log(`   Progress: ${paramProgress.progressPercentage.toFixed(1)}%`);
    
    if (updatedParamProposal.voting_deadline) {
      const timeRemaining = ICNUtils.getTimeRemaining(updatedParamProposal.voting_deadline);
      console.log(`   Time Remaining: ${timeRemaining}`);
    }
    console.log();

    // 9. Detailed Vote Analysis
    console.log('📈 Detailed vote analysis...');
    
    if (updatedParamProposal.detailed_votes) {
      console.log('🗳️  Individual Votes:');
      updatedParamProposal.detailed_votes.forEach((vote, index) => {
        console.log(`   ${index + 1}. ${vote.voter.slice(0, 20)}... voted ${vote.option}`);
        console.log(`      Timestamp: ${new Date(vote.timestamp).toLocaleString()}`);
      });
    }
    console.log();

    // 10. CCL Template Example (Governance Extension)
    console.log('🔧 CCL Template Governance Example...');
    
    // Define a governance template for membership proposals
    const membershipTemplate = {
      id: 'membership_proposal',
      name: 'Standard Membership Proposal',
      description: 'Template for proposing new member admission',
      category: 'membership',
      template: `
        // Membership Proposal for {{candidate_did}}
        
        proposal membership_admission {
          candidate: DID = "{{candidate_did}}"
          sponsor: DID = "{{sponsor_did}}"
          evaluation_period: Duration = {{evaluation_days}}d
          required_endorsements: Number = {{endorsement_count}}
          
          requirements {
            background_check: {{background_check_required}}
            technical_assessment: {{technical_assessment_required}}
            community_interview: {{community_interview_required}}
          }
          
          voting {
            duration: Duration = {{voting_duration}}d
            quorum: Number = {{quorum}}
            threshold: Percentage = {{threshold}}%
          }
        }
      `,
      parameters: [
        {
          name: 'candidate_did',
          type: 'did',
          description: 'DID of the membership candidate',
          required: true
        },
        {
          name: 'sponsor_did',
          type: 'did',
          description: 'DID of the sponsoring member',
          required: true
        },
        {
          name: 'evaluation_days',
          type: 'number',
          description: 'Days for candidate evaluation',
          required: true,
          default: 14,
          validation: { min: 7, max: 30 }
        },
        {
          name: 'endorsement_count',
          type: 'number',
          description: 'Required endorsements from existing members',
          required: true,
          default: 2,
          validation: { min: 1, max: 5 }
        },
        {
          name: 'background_check_required',
          type: 'boolean',
          description: 'Whether background check is required',
          required: false,
          default: true
        },
        {
          name: 'technical_assessment_required',
          type: 'boolean',
          description: 'Whether technical assessment is required',
          required: false,
          default: true
        },
        {
          name: 'community_interview_required',
          type: 'boolean',
          description: 'Whether community interview is required',
          required: false,
          default: true
        },
        {
          name: 'voting_duration',
          type: 'number',
          description: 'Voting period in days',
          required: true,
          default: 7,
          validation: { min: 3, max: 14 }
        },
        {
          name: 'quorum',
          type: 'number',
          description: 'Minimum number of votes required',
          required: true,
          default: 3,
          validation: { min: 2, max: 10 }
        },
        {
          name: 'threshold',
          type: 'number',
          description: 'Percentage of yes votes required',
          required: true,
          default: 66,
          validation: { min: 50, max: 100 }
        }
      ]
    };
    
    // Use template to generate proposal
    const templateParameters = {
      candidate_did: applicantDid,
      sponsor_did: memberAlice,
      evaluation_days: 14,
      endorsement_count: 2,
      background_check_required: true,
      technical_assessment_required: true,
      community_interview_required: true,
      voting_duration: 7,
      quorum: 3,
      threshold: 66
    };
    
    // Validate parameters
    const validation = CCLUtils.validateTemplateParameters(membershipTemplate, templateParameters);
    
    if (validation.valid) {
      const generatedCCL = CCLUtils.generateCCLFromTemplate(membershipTemplate, templateParameters);
      
      console.log('✅ CCL Template Generated Successfully');
      console.log('📋 Generated CCL Proposal:');
      console.log('```ccl');
      console.log(generatedCCL.trim());
      console.log('```');
    } else {
      console.log('❌ Template validation failed:');
      validation.errors.forEach(error => {
        console.log(`   • ${error}`);
      });
    }
    console.log();

    // 11. Governance Analytics
    console.log('📊 Governance Analytics...');
    
    // Calculate participation metrics
    const allVotes = allProposals.flatMap(p => p.detailed_votes || []);
    const uniqueVoters = new Set(allVotes.map(v => v.voter));
    const voteDistribution = allVotes.reduce((acc, vote) => {
      acc[vote.option] = (acc[vote.option] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    console.log('🎯 Participation Metrics:');
    console.log(`   Total Proposals: ${allProposals.length}`);
    console.log(`   Active Proposals: ${allProposals.filter(p => p.status === 'Open').length}`);
    console.log(`   Unique Voters: ${uniqueVoters.size}`);
    console.log(`   Total Votes Cast: ${allVotes.length}`);
    
    console.log('🗳️  Vote Distribution:');
    Object.entries(voteDistribution).forEach(([option, count]) => {
      const percentage = ((count / allVotes.length) * 100).toFixed(1);
      console.log(`   ${option}: ${count} (${percentage}%)`);
    });
    
    console.log('📈 Proposal Success Rate:');
    const completedProposals = allProposals.filter(p => p.status === 'Closed' || p.status === 'Executed');
    const passedProposals = completedProposals.filter(p => GovernanceUtils.getProposalOutcome(p) === 'passed');
    const successRate = completedProposals.length > 0 ? (passedProposals.length / completedProposals.length) * 100 : 0;
    console.log(`   Success Rate: ${successRate.toFixed(1)}% (${passedProposals.length}/${completedProposals.length})`);
    console.log();

    // 12. Advanced Governance Features
    console.log('🔧 Advanced Governance Features...');
    
    console.log('🎯 Proposal Categories:');
    const proposalCategories = allProposals.reduce((acc, proposal) => {
      const category = GovernanceUtils.formatProposalType(proposal.proposal_type);
      acc[category] = (acc[category] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    Object.entries(proposalCategories).forEach(([category, count]) => {
      console.log(`   ${category}: ${count} proposal(s)`);
    });
    
    console.log('\n⏰ Proposal Timeline Analysis:');
    const now = new Date();
    const proposalsByAge = allProposals.map(p => {
      const created = new Date(p.created_at);
      const ageInDays = Math.floor((now.getTime() - created.getTime()) / (24 * 60 * 60 * 1000));
      return { ...p, ageInDays };
    }).sort((a, b) => a.ageInDays - b.ageInDays);
    
    proposalsByAge.slice(0, 3).forEach((proposal, index) => {
      console.log(`   ${index + 1}. ${proposal.id.slice(0, 12)}... (${proposal.ageInDays} days old)`);
      console.log(`      Status: ${proposal.status}`);
      if (proposal.voting_deadline) {
        const timeRemaining = ICNUtils.getTimeRemaining(proposal.voting_deadline);
        console.log(`      Deadline: ${timeRemaining}`);
      }
    });

    console.log('\n🎉 Comprehensive Governance example completed successfully!');
    console.log('\n💡 Key Features Demonstrated:');
    console.log('   • Multiple proposal types (parameter changes, admissions, upgrades, resolutions)');
    console.log('   • Voting workflows with different thresholds and quorums');
    console.log('   • Vote delegation for governance expertise');
    console.log('   • CCL template-based proposal generation');
    console.log('   • Comprehensive proposal analytics and metrics');
    console.log('   • Advanced governance utilities and helpers');
    
    console.log('\n🔒 Governance Benefits:');
    console.log('   • Democratic decision-making with transparent voting');
    console.log('   • Flexible quorum and threshold settings');
    console.log('   • Template-based proposal standardization');
    console.log('   • Comprehensive audit trail for all decisions');
    console.log('   • Advanced analytics for governance insights');

  } catch (error) {
    console.error('❌ Error during governance example:');
    
    if (ErrorUtils.isErrorType(error, ICNGovernanceError)) {
      console.error('🏛️  Governance Error:', error.message);
      console.error('💡 Tip: Check proposal parameters and voting permissions');
    } else if (ErrorUtils.isErrorType(error, ICNValidationError)) {
      console.error('📝 Validation Error:', error.message);
      if (error.field) {
        console.error(`   Field: ${error.field}`);
      }
    } else {
      console.error('🔍 Unexpected Error:', ErrorUtils.getErrorMessage(error));
    }
  } finally {
    await client.disconnect();
    console.log('\n🔌 Disconnected from ICN node');
  }
}

// Run the example
if (require.main === module) {
  governanceExample().catch(console.error);
}

export { governanceExample };