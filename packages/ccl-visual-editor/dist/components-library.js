export const GOVERNANCE_COMPONENTS = [
    {
        id: 'voting_mechanism',
        category: 'governance',
        name: 'Voting Mechanism',
        description: 'Create voting rules with quorum and thresholds',
        icon: '🗳️',
        ports: [
            { id: 'proposal_input', label: 'Proposal', type: 'input', dataType: 'proposal' },
            { id: 'vote_output', label: 'Vote Result', type: 'output', dataType: 'vote' }
        ],
        defaultConfig: {
            quorum: 50,
            threshold: 0.6,
            voting_period_days: 7
        },
        parameters: [
            {
                name: 'quorum',
                type: 'number',
                description: 'Minimum number of votes required',
                required: true,
                validation: { min: 1, max: 1000 }
            },
            {
                name: 'threshold',
                type: 'number',
                description: 'Percentage required for approval (0.5-1.0)',
                required: true,
                validation: { min: 0.5, max: 1.0 }
            },
            {
                name: 'voting_period_days',
                type: 'number',
                description: 'Number of days voting remains open',
                required: true,
                validation: { min: 1, max: 365 }
            }
        ]
    },
    {
        id: 'member_role',
        category: 'governance',
        name: 'Member Role',
        description: 'Define member roles and permissions',
        icon: '👤',
        ports: [
            { id: 'member_input', label: 'Member', type: 'input', dataType: 'data' },
            { id: 'role_output', label: 'Role Assignment', type: 'output', dataType: 'data' }
        ],
        defaultConfig: {
            role_name: 'member',
            can_vote: true,
            can_propose: true,
            voting_weight: 1
        },
        parameters: [
            {
                name: 'role_name',
                type: 'string',
                description: 'Name of the role',
                required: true,
                validation: { pattern: '^[a-zA-Z_][a-zA-Z0-9_]*$' }
            },
            {
                name: 'can_vote',
                type: 'boolean',
                description: 'Can members with this role vote?',
                required: true
            },
            {
                name: 'can_propose',
                type: 'boolean',
                description: 'Can members with this role create proposals?',
                required: true
            },
            {
                name: 'voting_weight',
                type: 'number',
                description: 'Weight of votes from this role',
                required: true,
                validation: { min: 0, max: 10 }
            }
        ]
    },
    {
        id: 'proposal_creation',
        category: 'governance',
        name: 'Proposal Creation',
        description: 'Create and submit new proposals',
        icon: '📝',
        ports: [
            { id: 'proposer_input', label: 'Proposer', type: 'input', dataType: 'data' },
            { id: 'proposal_output', label: 'New Proposal', type: 'output', dataType: 'proposal' }
        ],
        defaultConfig: {
            require_role: 'member',
            min_description_length: 10,
            max_description_length: 1000
        },
        parameters: [
            {
                name: 'require_role',
                type: 'string',
                description: 'Role required to create proposals',
                required: true,
                validation: { options: ['member', 'admin', 'council'] }
            },
            {
                name: 'min_description_length',
                type: 'number',
                description: 'Minimum proposal description length',
                required: true,
                validation: { min: 1, max: 500 }
            },
            {
                name: 'max_description_length',
                type: 'number',
                description: 'Maximum proposal description length',
                required: true,
                validation: { min: 100, max: 5000 }
            }
        ]
    },
    {
        id: 'budget_allocation',
        category: 'economics',
        name: 'Budget Allocation',
        description: 'Allocate budget based on proposals',
        icon: '💰',
        ports: [
            { id: 'proposal_input', label: 'Proposal', type: 'input', dataType: 'proposal' },
            { id: 'budget_output', label: 'Budget Decision', type: 'output', dataType: 'data' }
        ],
        defaultConfig: {
            max_budget: 10000,
            require_approval: true,
            approval_threshold: 0.7
        },
        parameters: [
            {
                name: 'max_budget',
                type: 'number',
                description: 'Maximum budget amount available',
                required: true,
                validation: { min: 1, max: 1000000 }
            },
            {
                name: 'require_approval',
                type: 'boolean',
                description: 'Require governance approval for budget allocation',
                required: true
            },
            {
                name: 'approval_threshold',
                type: 'number',
                description: 'Approval threshold for budget requests',
                required: true,
                validation: { min: 0.5, max: 1.0 }
            }
        ]
    },
    {
        id: 'reputation_check',
        category: 'identity',
        name: 'Reputation Check',
        description: 'Verify member reputation before actions',
        icon: '⭐',
        ports: [
            { id: 'member_input', label: 'Member', type: 'input', dataType: 'data' },
            { id: 'reputation_output', label: 'Reputation Status', type: 'output', dataType: 'data' }
        ],
        defaultConfig: {
            min_reputation: 0.5,
            reputation_weight: 1.0,
            check_recency: true
        },
        parameters: [
            {
                name: 'min_reputation',
                type: 'number',
                description: 'Minimum reputation score required',
                required: true,
                validation: { min: 0.0, max: 1.0 }
            },
            {
                name: 'reputation_weight',
                type: 'number',
                description: 'Weight of reputation in decisions',
                required: true,
                validation: { min: 0.0, max: 5.0 }
            },
            {
                name: 'check_recency',
                type: 'boolean',
                description: 'Consider recency of reputation updates',
                required: true
            }
        ]
    },
    {
        id: 'if_condition',
        category: 'logic',
        name: 'If Condition',
        description: 'Conditional logic based on criteria',
        icon: '❓',
        ports: [
            { id: 'condition_input', label: 'Condition', type: 'input', dataType: 'data' },
            { id: 'true_output', label: 'True Branch', type: 'output', dataType: 'control' },
            { id: 'false_output', label: 'False Branch', type: 'output', dataType: 'control' }
        ],
        defaultConfig: {
            operator: 'equals',
            comparison_value: ''
        },
        parameters: [
            {
                name: 'operator',
                type: 'string',
                description: 'Comparison operator',
                required: true,
                validation: { options: ['equals', 'not_equals', 'greater_than', 'less_than', 'contains'] }
            },
            {
                name: 'comparison_value',
                type: 'string',
                description: 'Value to compare against',
                required: true
            }
        ]
    }
];
export const COMPONENT_CATEGORIES = [
    { id: 'governance', name: 'Governance', icon: '🏛️', color: '#3B82F6' },
    { id: 'economics', name: 'Economics', icon: '💰', color: '#10B981' },
    { id: 'identity', name: 'Identity', icon: '🔑', color: '#8B5CF6' },
    { id: 'logic', name: 'Logic', icon: '⚡', color: '#F59E0B' }
];
export function getComponentsByCategory(category) {
    return GOVERNANCE_COMPONENTS.filter(component => component.category === category);
}
export function getComponentById(id) {
    return GOVERNANCE_COMPONENTS.find(component => component.id === id);
}
export function searchComponents(searchTerm) {
    const term = searchTerm.toLowerCase();
    return GOVERNANCE_COMPONENTS.filter(component => component.name.toLowerCase().includes(term) ||
        component.description.toLowerCase().includes(term) ||
        component.category.toLowerCase().includes(term));
}
