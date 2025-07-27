/**
 * React Hooks Integration Example
 * 
 * This example demonstrates:
 * - ICNProvider setup
 * - Using ICN React hooks
 * - Real-time data management
 * - Error handling in React
 */

import React, { useEffect, useState } from 'react';
import { 
  ICNProvider, 
  useICNClient, 
  useICNConnection,
  useICNJobs,
  useICNGovernance,
  useICNTrust,
  useICNCredentials,
  useICNTokens,
  useICNMutualAid,
  ICNError,
  ErrorUtils 
} from '@icn/ts-sdk';

// Main App Component with ICN Provider
function App() {
  return (
    <ICNProvider 
      options={{
        nodeEndpoint: process.env.REACT_APP_ICN_ENDPOINT || 'http://localhost:8080',
        network: 'devnet',
        timeout: 30000,
        encryptionConfig: {
          enableEncryption: true,
        }
      }}
    >
      <div className="app">
        <header>
          <h1>ICN React Example</h1>
          <ConnectionStatus />
        </header>
        
        <main>
          <JobsSection />
          <GovernanceSection />
          <TrustSection />
          <CredentialsSection />
          <TokensSection />
          <MutualAidSection />
        </main>
      </div>
    </ICNProvider>
  );
}

// Connection Status Component
function ConnectionStatus() {
  const { connected, connecting, error } = useICNConnection();

  return (
    <div className={`connection-status ${connected ? 'connected' : 'disconnected'}`}>
      {connecting && <span>🔄 Connecting...</span>}
      {connected && <span>✅ Connected to ICN</span>}
      {error && <span>❌ Connection Error: {error}</span>}
    </div>
  );
}

// Jobs Management Section
function JobsSection() {
  const { jobs, loading, submitJob, refreshJobs } = useICNJobs();
  const [jobSpec, setJobSpec] = useState({
    image: 'alpine:latest',
    command: ['echo', 'Hello ICN!'],
    resources: {
      cpu_cores: 1,
      memory_mb: 512,
      storage_mb: 1024
    }
  });

  useEffect(() => {
    refreshJobs();
  }, []);

  const handleSubmitJob = async () => {
    try {
      const jobId = await submitJob({
        job_spec: jobSpec,
        submitter_did: 'did:key:example',
        max_cost: 100
      });
      console.log('Job submitted:', jobId);
    } catch (error) {
      console.error('Failed to submit job:', ErrorUtils.getErrorMessage(error));
    }
  };

  return (
    <section className="jobs-section">
      <h2>🚀 Mesh Computing Jobs</h2>
      
      <div className="job-submission">
        <h3>Submit New Job</h3>
        <div className="form-group">
          <label>Docker Image:</label>
          <input 
            value={jobSpec.image}
            onChange={(e) => setJobSpec({...jobSpec, image: e.target.value})}
          />
        </div>
        <button onClick={handleSubmitJob} disabled={loading}>
          {loading ? 'Submitting...' : 'Submit Job'}
        </button>
      </div>

      <div className="jobs-list">
        <h3>Recent Jobs ({jobs.length})</h3>
        {loading && <p>Loading jobs...</p>}
        {jobs.map((job) => (
          <div key={job.id} className={`job-item status-${job.status.toLowerCase()}`}>
            <strong>{job.id.slice(0, 8)}...</strong>
            <span className="status">{job.status}</span>
            <span className="cost">Cost: {job.cost}</span>
            {job.progress && <span className="progress">{job.progress}%</span>}
          </div>
        ))}
      </div>
    </section>
  );
}

// Governance Section
function GovernanceSection() {
  const { proposals, loading, submitProposal, castVote, refreshProposals } = useICNGovernance();
  const [proposalText, setProposalText] = useState('');

  useEffect(() => {
    refreshProposals();
  }, []);

  const handleSubmitProposal = async () => {
    try {
      const proposalId = await submitProposal({
        proposer_did: 'did:key:example',
        proposal: {
          type: 'GenericText',
          data: { text: proposalText }
        },
        description: 'Community proposal',
        duration_secs: 7 * 24 * 3600 // 7 days
      });
      console.log('Proposal submitted:', proposalId);
      setProposalText('');
    } catch (error) {
      console.error('Failed to submit proposal:', ErrorUtils.getErrorMessage(error));
    }
  };

  const handleVote = async (proposalId: string, vote: string) => {
    try {
      await castVote(proposalId, vote);
      console.log('Vote cast successfully');
    } catch (error) {
      console.error('Failed to cast vote:', ErrorUtils.getErrorMessage(error));
    }
  };

  return (
    <section className="governance-section">
      <h2>🏛️ Governance</h2>
      
      <div className="proposal-submission">
        <h3>Submit Proposal</h3>
        <textarea
          value={proposalText}
          onChange={(e) => setProposalText(e.target.value)}
          placeholder="Enter your proposal text..."
          rows={3}
        />
        <button onClick={handleSubmitProposal} disabled={loading || !proposalText.trim()}>
          {loading ? 'Submitting...' : 'Submit Proposal'}
        </button>
      </div>

      <div className="proposals-list">
        <h3>Active Proposals ({proposals.length})</h3>
        {loading && <p>Loading proposals...</p>}
        {proposals.map((proposal) => (
          <div key={proposal.id} className="proposal-item">
            <h4>{proposal.description}</h4>
            <p className="proposal-type">{proposal.proposal_type?.type || 'Unknown'}</p>
            <div className="votes">
              <span>Yes: {proposal.votes?.yes || 0}</span>
              <span>No: {proposal.votes?.no || 0}</span>
              <span>Abstain: {proposal.votes?.abstain || 0}</span>
            </div>
            <div className="vote-buttons">
              <button onClick={() => handleVote(proposal.id, 'Yes')}>👍 Yes</button>
              <button onClick={() => handleVote(proposal.id, 'No')}>👎 No</button>
              <button onClick={() => handleVote(proposal.id, 'Abstain')}>🤷 Abstain</button>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

// Trust Management Section
function TrustSection() {
  const { trustScore, loading, getTrustScore } = useICNTrust();
  const [targetDid, setTargetDid] = useState('');

  const handleGetTrustScore = async () => {
    if (!targetDid.trim()) return;
    
    try {
      await getTrustScore(targetDid);
    } catch (error) {
      console.error('Failed to get trust score:', ErrorUtils.getErrorMessage(error));
    }
  };

  return (
    <section className="trust-section">
      <h2>🤝 Trust Networks</h2>
      
      <div className="trust-lookup">
        <h3>Look up Trust Score</h3>
        <input
          value={targetDid}
          onChange={(e) => setTargetDid(e.target.value)}
          placeholder="Enter DID to check trust score..."
        />
        <button onClick={handleGetTrustScore} disabled={loading || !targetDid.trim()}>
          {loading ? 'Loading...' : 'Get Trust Score'}
        </button>
      </div>

      {trustScore && (
        <div className="trust-score">
          <h3>Trust Score for {trustScore.did}</h3>
          <div className="score-display">
            <div className="overall-score">
              Overall: {(trustScore.score * 100).toFixed(1)}%
            </div>
            <div className="context-scores">
              {Object.entries(trustScore.context_scores).map(([context, score]) => (
                <div key={context} className="context-score">
                  {context}: {(score * 100).toFixed(1)}%
                </div>
              ))}
            </div>
          </div>
          <div className="trust-stats">
            <span>Incoming: {trustScore.incoming_trust_count}</span>
            <span>Outgoing: {trustScore.outgoing_trust_count}</span>
            <span>Federations: {trustScore.federations.length}</span>
          </div>
        </div>
      )}
    </section>
  );
}

// Credentials Section
function CredentialsSection() {
  const { credentials, loading, refreshCredentials } = useICNCredentials();

  useEffect(() => {
    refreshCredentials();
  }, []);

  return (
    <section className="credentials-section">
      <h2>🆔 Credentials</h2>
      
      <div className="credentials-list">
        <h3>My Credentials ({credentials.length})</h3>
        <button onClick={() => refreshCredentials()} disabled={loading}>
          {loading ? 'Loading...' : 'Refresh'}
        </button>
        
        {credentials.map((credential) => (
          <div key={credential.cid} className="credential-item">
            <h4>{credential.credential_type}</h4>
            <p>Issuer: {credential.issuer}</p>
            <p>Status: {credential.status}</p>
            <p>Issued: {new Date(credential.issued_at * 1000).toLocaleDateString()}</p>
            {credential.valid_until && (
              <p>Expires: {new Date(credential.valid_until * 1000).toLocaleDateString()}</p>
            )}
          </div>
        ))}
        
        {credentials.length === 0 && !loading && (
          <p>No credentials found</p>
        )}
      </div>
    </section>
  );
}

// Tokens Section
function TokensSection() {
  const { balances, loading, getBalances } = useICNTokens();
  const [userDid, setUserDid] = useState('');

  const handleGetBalances = async () => {
    if (!userDid.trim()) return;
    
    try {
      await getBalances(userDid);
    } catch (error) {
      console.error('Failed to get balances:', ErrorUtils.getErrorMessage(error));
    }
  };

  return (
    <section className="tokens-section">
      <h2>🪙 Token Balances</h2>
      
      <div className="balance-lookup">
        <input
          value={userDid}
          onChange={(e) => setUserDid(e.target.value)}
          placeholder="Enter DID to check token balances..."
        />
        <button onClick={handleGetBalances} disabled={loading || !userDid.trim()}>
          {loading ? 'Loading...' : 'Get Balances'}
        </button>
      </div>

      {balances.length > 0 && (
        <div className="balances-list">
          <h3>Token Balances</h3>
          {balances.map((balance) => (
            <div key={balance.class_id} className="balance-item">
              <span className="token-class">{balance.class_id}</span>
              <span className="amount">{balance.amount}</span>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

// Mutual Aid Section
function MutualAidSection() {
  const { resources, loading, refreshResources } = useICNMutualAid();

  useEffect(() => {
    refreshResources();
  }, []);

  return (
    <section className="mutual-aid-section">
      <h2>🤲 Mutual Aid Resources</h2>
      
      <div className="resources-list">
        <h3>Available Resources ({resources.length})</h3>
        <button onClick={() => refreshResources()} disabled={loading}>
          {loading ? 'Loading...' : 'Refresh'}
        </button>
        
        {resources.map((resource) => (
          <div key={resource.id} className="resource-item">
            <h4>{resource.name}</h4>
            <p>{resource.description}</p>
            <p>Category: {resource.category}</p>
            <p>Status: {resource.availability}</p>
            <p>Provider: {resource.provider_did}</p>
          </div>
        ))}
        
        {resources.length === 0 && !loading && (
          <p>No resources available</p>
        )}
      </div>
    </section>
  );
}

export default App;