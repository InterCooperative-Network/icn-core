import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { GovernanceState, Proposal, Vote, Member, Discussion } from '@/types/governance';

interface GovernanceContextType {
  state: GovernanceState;
  submitProposal: (proposal: Omit<Proposal, 'id' | 'created_at' | 'votes'>) => Promise<void>;
  vote: (proposalId: string, choice: 'yes' | 'no' | 'abstain') => Promise<void>;
  addDiscussion: (proposalId: string, content: string) => Promise<void>;
  isLoading: boolean;
  error: string | null;
}

const GovernanceContext = createContext<GovernanceContextType | undefined>(undefined);

export function useGovernance() {
  const context = useContext(GovernanceContext);
  if (!context) {
    throw new Error('useGovernance must be used within a GovernanceProvider');
  }
  return context;
}

interface GovernanceProviderProps {
  children: ReactNode;
}

export function GovernanceProvider({ children }: GovernanceProviderProps) {
  const [state, setState] = useState<GovernanceState>({
    proposals: [],
    members: [],
    discussions: [],
    votes: [],
    current_user: {
      id: 'user-1',
      name: 'Demo User',
      reputation: 100,
      voting_power: 1,
      joined_at: new Date().toISOString(),
    },
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Initialize with demo data
  useEffect(() => {
    setState({
      proposals: [
        {
          id: 'prop-1',
          title: 'Network Upgrade Proposal',
          description: 'Proposal to upgrade the ICN protocol to version 2.0 with improved consensus mechanisms and enhanced security features.',
          author: 'alice.icn',
          status: 'active',
          created_at: '2024-01-15T10:00:00Z',
          voting_deadline: '2024-01-22T10:00:00Z',
          votes: { yes: 12, no: 3, abstain: 2 },
          quorum_required: 0.51,
          threshold_required: 0.67,
          changes: [
            {
              parameter: 'consensus_algorithm',
              current_value: 'proof_of_cooperation_v1',
              proposed_value: 'proof_of_cooperation_v2',
              description: 'Upgrade to more efficient consensus algorithm'
            }
          ]
        },
        {
          id: 'prop-2',
          title: 'Community Fund Allocation',
          description: 'Allocate 10,000 ICN tokens from the community fund to support local cooperative development initiatives.',
          author: 'bob.icn',
          status: 'active',
          created_at: '2024-01-14T14:30:00Z',
          voting_deadline: '2024-01-21T14:30:00Z',
          votes: { yes: 8, no: 7, abstain: 1 },
          quorum_required: 0.51,
          threshold_required: 0.51,
        }
      ],
      members: [
        {
          id: 'member-1',
          name: 'Alice Cooper',
          reputation: 150,
          voting_power: 1.5,
          joined_at: '2023-06-01T00:00:00Z',
        },
        {
          id: 'member-2',
          name: 'Bob Wilson',
          reputation: 120,
          voting_power: 1.2,
          joined_at: '2023-07-15T00:00:00Z',
        },
      ],
      discussions: [
        {
          id: 'disc-1',
          proposal_id: 'prop-1',
          author: 'alice.icn',
          content: 'I believe this upgrade will significantly improve our network performance. The new consensus algorithm has been tested extensively.',
          timestamp: '2024-01-16T08:00:00Z',
          replies: [],
          reactions: { '👍': ['bob.icn', 'charlie.icn'], '💡': ['alice.icn'] }
        }
      ],
      votes: [],
      current_user: {
        id: 'user-1',
        name: 'Demo User',
        reputation: 100,
        voting_power: 1,
        joined_at: new Date().toISOString(),
      },
    });
  }, []);

  const submitProposal = async (proposalData: Omit<Proposal, 'id' | 'created_at' | 'votes'>) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const newProposal: Proposal = {
        ...proposalData,
        id: `prop-${Date.now()}`,
        created_at: new Date().toISOString(),
        votes: { yes: 0, no: 0, abstain: 0 },
      };

      setState(prev => ({
        ...prev,
        proposals: [...prev.proposals, newProposal],
      }));
    } catch (err) {
      setError('Failed to submit proposal');
    } finally {
      setIsLoading(false);
    }
  };

  const vote = async (proposalId: string, choice: 'yes' | 'no' | 'abstain') => {
    setIsLoading(true);
    setError(null);

    try {
      const newVote: Vote = {
        proposal_id: proposalId,
        voter: state.current_user?.id || 'anonymous',
        choice,
        weight: state.current_user?.voting_power || 1,
        timestamp: new Date().toISOString(),
      };

      setState(prev => ({
        ...prev,
        votes: [...prev.votes.filter(v => v.proposal_id !== proposalId || v.voter !== newVote.voter), newVote],
        proposals: prev.proposals.map(p => {
          if (p.id === proposalId) {
            const updatedVotes = { ...p.votes };
            updatedVotes[choice] += newVote.weight || 1;
            return { ...p, votes: updatedVotes };
          }
          return p;
        }),
      }));
    } catch (err) {
      setError('Failed to submit vote');
    } finally {
      setIsLoading(false);
    }
  };

  const addDiscussion = async (proposalId: string, content: string) => {
    setIsLoading(true);
    setError(null);

    try {
      const newDiscussion: Discussion = {
        id: `disc-${Date.now()}`,
        proposal_id: proposalId,
        author: state.current_user?.name || 'Anonymous',
        content,
        timestamp: new Date().toISOString(),
        replies: [],
        reactions: {},
      };

      setState(prev => ({
        ...prev,
        discussions: [...prev.discussions, newDiscussion],
      }));
    } catch (err) {
      setError('Failed to add discussion');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <GovernanceContext.Provider
      value={{
        state,
        submitProposal,
        vote,
        addDiscussion,
        isLoading,
        error,
      }}
    >
      {children}
    </GovernanceContext.Provider>
  );
}