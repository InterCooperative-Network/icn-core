// Define governance types
export interface Proposal {
  id: string;
  title: string;
  description: string;
  author: string;
  status: 'draft' | 'active' | 'passed' | 'failed' | 'executed';
  created_at: string;
  voting_deadline: string;
  votes: {
    yes: number;
    no: number;
    abstain: number;
  };
  quorum_required: number;
  threshold_required: number;
  changes?: ProposalChange[];
}

export interface ProposalChange {
  parameter: string;
  current_value: any;
  proposed_value: any;
  description?: string;
}

export interface Vote {
  proposal_id: string;
  voter: string;
  choice: 'yes' | 'no' | 'abstain';
  weight?: number;
  timestamp: string;
}

export interface Member {
  id: string;
  name: string;
  avatar?: string;
  reputation: number;
  voting_power: number;
  delegation?: string;
  joined_at: string;
}

export interface Discussion {
  id: string;
  proposal_id: string;
  author: string;
  content: string;
  timestamp: string;
  replies: Discussion[];
  reactions: {
    [emoji: string]: string[];
  };
}

export interface GovernanceState {
  proposals: Proposal[];
  members: Member[];
  discussions: Discussion[];
  votes: Vote[];
  current_user?: Member;
}