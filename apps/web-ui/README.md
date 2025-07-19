# ICN Web UI - Federation Dashboard

A comprehensive web interface for managing InterCooperative Network (ICN) federations and participating in cooperative governance.

## Features

### 🎯 Demo Mode
- Interactive demonstration of all UI features
- Mock data showing real-world usage scenarios
- Technical architecture overview
- Navigation guide for new users

### 🏠 Dashboard
- Real-time federation health monitoring
- Key metrics: cooperatives, members, proposals, network peers
- Visual health indicators for network connectivity, governance activity
- Recent cooperatives and proposals overview
- System information display

### 🤝 Federation Management
- Create new federations with DID generation and metadata
- Join existing federations via peer addresses
- Trust configuration and peer management
- Real-time federation status monitoring
- Active cooperatives and members management

### 🗳️ Governance System
- CCL (Cooperative Contract Language) template-based proposal creation
- Interactive voting with real-time progress tracking
- Quorum and threshold monitoring
- Multiple proposal types:
  - Member admission with background checks
  - Budget allocation with accountability
  - Governance rule changes with impact assessment
- Execution receipt viewing and history
- Template parameter validation and CCL code generation

### 🏢 Cooperative Management
- Add and manage cooperatives within federations
- Capability-based filtering and search
- Health scoring and reputation tracking
- Member count and status monitoring
- Interactive cooperative cards with detailed information

## Technical Architecture

```
React + TypeScript UI
        ↓
   ICN TypeScript SDK
        ↓
   ICN Client SDK (@icn/client-sdk)
        ↓
   ICN API Traits (Rust)
        ↓
   ICN Core Protocol
```

### Key Components

- **React Context Providers**: FederationContext, GovernanceContext for state management
- **TypeScript SDK**: Extended with Federation and Governance utilities
- **CCL Integration**: Template system with parameter validation
- **Responsive Design**: Mobile-first with Tailwind CSS
- **Mock Data Support**: Development-friendly with realistic scenarios

## Development Setup

### Prerequisites
- Node.js 18+
- pnpm 8.0+

### Installation
```bash
# From the ICN core repository root
pnpm install

# Development mode
pnpm dev:web-ui

# Build for production
pnpm build:web-ui
```

### Environment Variables
Create a `.env` file in the web-ui directory:

```env
VITE_ICN_NODE_ENDPOINT=http://localhost:8080
VITE_ICN_NETWORK=devnet
```

## Usage

### Demo Mode (Recommended for First Time)
1. Visit the root URL to see the demo page
2. Explore features overview and technical architecture
3. Follow the navigation guide to understand each section
4. Review CCL templates and governance workflows

### Dashboard
1. Monitor federation health and key metrics
2. View active cooperatives and recent proposals
3. Check system status and network connectivity
4. Track governance activity and mesh job statistics

### Federation Management
1. **Create Federation**: Use the form to set up a new federation with metadata
2. **Join Federation**: Enter peer address to connect to existing federations
3. **Manage Peers**: Add/remove trusted peers and configure trust relationships
4. **Monitor Status**: Track peer count, sync status, and federation health

### Governance Workflows
1. **Create Proposals**: 
   - Choose from CCL templates or create custom proposals
   - Fill in template parameters with validation
   - Set voting duration, quorum, and threshold requirements
2. **Vote on Proposals**:
   - View active proposals with progress indicators
   - Cast votes (Yes/No/Abstain) with real-time updates
   - Monitor quorum achievement and time remaining
3. **Track History**: Review all proposals with outcomes and detailed votes

### Cooperative Management
1. **Add Cooperatives**: Create new cooperative entries with capabilities
2. **Filter and Search**: Find cooperatives by status, capabilities, or name
3. **Monitor Health**: Track reputation scores and member counts
4. **Manage Relationships**: Edit cooperative details and manage memberships

## CCL Templates

The system includes three pre-built CCL templates:

### Member Admission Template
```ccl
cooperative "{{cooperative_name}}" {
  propose admission {
    candidate: "{{candidate_did}}"
    sponsor: "{{sponsor_did}}"
    requirements {
      background_check: {{require_background_check}}
      commitment_period: {{commitment_months}} months
    }
    voting {
      threshold: {{approval_threshold}}
      quorum: {{quorum_percentage}}
    }
  }
}
```

### Budget Allocation Template
```ccl
cooperative "{{cooperative_name}}" {
  propose budget_allocation {
    purpose: "{{purpose}}"
    amount: {{amount}} mana
    category: "{{category}}"
    timeline {
      start_date: "{{start_date}}"
      end_date: "{{end_date}}"
    }
    accountability {
      responsible_member: "{{responsible_did}}"
      reporting_frequency: "{{reporting_frequency}}"
    }
  }
}
```

### Governance Change Template
```ccl
cooperative "{{cooperative_name}}" {
  propose governance_change {
    rule_name: "{{rule_name}}"
    current_value: "{{current_value}}"
    proposed_value: "{{proposed_value}}"
    rationale: "{{rationale}}"
    impact_assessment {
      affected_members: {{affected_members}}
      implementation_complexity: "{{complexity}}"
      reversible: {{reversible}}
    }
    voting {
      threshold: {{approval_threshold}}
      super_majority: true
    }
  }
}
```

## API Integration

The UI integrates with ICN APIs through the TypeScript SDK:

- **Federation API**: Peer management, status monitoring, join/leave operations
- **Governance API**: Proposal submission, voting, proposal retrieval
- **Identity API**: Credential management and verification
- **Mesh API**: Job submission and status tracking
- **System API**: Node information and health status

## Project Structure

```
apps/web-ui/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── Dashboard.tsx    # Main dashboard component
│   │   └── Navigation.tsx   # Navigation component
│   ├── pages/               # Route components
│   │   ├── FederationPage.tsx    # Federation management
│   │   ├── GovernancePage.tsx    # Governance interface
│   │   ├── CooperativesPage.tsx  # Cooperative management
│   │   └── DemoPage.tsx          # Demo and documentation
│   ├── contexts/            # React context providers
│   │   ├── FederationContext.tsx # Federation state management
│   │   └── GovernanceContext.tsx # Governance state management
│   ├── hooks/               # Custom React hooks
│   ├── utils/               # Utility functions
│   ├── types/               # TypeScript definitions
│   └── index.css           # Global styles
├── public/                  # Static assets
├── dist/                    # Build output
└── vite.config.ts          # Vite configuration
```

## Contributing

1. Follow the existing component structure and naming conventions
2. Use TypeScript strictly with proper type definitions
3. Implement responsive design patterns with Tailwind CSS
4. Add comprehensive error handling and loading states
5. Include mock data for development scenarios
6. Document new features and API integrations

## Related Documentation

- [ICN Core Documentation](../../README.md)
- [TypeScript SDK](../../packages/ts-sdk/README.md)
- [ICN API Reference](../../ICN_API_REFERENCE.md)
- [Governance Documentation](../../docs/governance.md)
- [Federation Guide](../../FEDERATION_CLI_EXAMPLES.md)

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details. 