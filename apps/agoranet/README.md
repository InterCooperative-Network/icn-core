# ICN AgoraNet

Cross-platform governance deliberation and voting interface for the InterCooperative Network.

## Overview

AgoraNet is ICN's democratic governance platform, providing tools for proposal creation, community deliberation, and participatory decision-making. It enables cooperatives, federations, and communities to coordinate democratically.

## Technology Stack

- **React Native**: Cross-platform mobile development
- **Expo**: Development and build toolchain  
- **Tamagui**: Cross-platform UI components
- **Expo Router**: File-based navigation
- **Tauri**: Desktop application wrapper
- **TypeScript**: Type safety

## Platform Support

| Platform | Status | Build Command | Notes |
|----------|--------|---------------|-------|
| **Web** | ✅ | `pnpm web` | PWA-enabled browser app |
| **iOS** | ✅ | `pnpm ios` | Native iOS app via Expo |
| **Android** | ✅ | `pnpm android` | Native Android app via Expo |
| **Desktop** | ✅ | `pnpm tauri:dev` | Desktop app via Tauri |

## Features

### Governance Functions
- ✅ Proposal creation and editing
- ✅ Community deliberation forums
- ✅ Voting and consensus mechanisms
- ✅ Proposal status tracking
- ✅ Delegation and proxy voting

### Deliberation Tools
- ✅ Real-time collaborative editing
- ✅ Threaded discussions
- ✅ Consensus-building tools
- ✅ Argument mapping
- ✅ Decision trees

### Community Features
- ✅ Member profiles and reputation
- ✅ Working groups and committees
- ✅ Notification system
- ✅ Event calendar
- ✅ Meeting coordination

### Recent Updates
- **CRDT Real-Time Sync** for proposals and deliberation notes across devices
- **Advanced Governance Features** including dynamic quorum rules and multi-level delegation
- **Updated API Integrations** with ICN Node v0.7 for job management and identity verification
- **Accessibility Additions** such as screen reader navigation and improved color contrast

## Development

### Start Development Server
```bash
# Web development
pnpm dev
pnpm web

# Mobile development  
pnpm ios    # iOS simulator
pnpm android # Android emulator

# Desktop development
pnpm tauri:dev
```

### Build for Production
```bash
# Web (PWA)
pnpm build:web

# Mobile (App Stores)
pnpm build:ios
pnpm build:android

# Desktop
pnpm tauri:build
```

## Key Features

### Proposal Management
- Draft, review, and submit proposals
- Version control and collaborative editing
- Impact assessment tools
- Timeline and milestone tracking

### Voting Systems
- Multiple voting mechanisms (FPTP, ranked choice, quadratic)
- Delegation and liquid democracy
- Quorum and threshold management
- Transparent tallying and results

### Deliberation Spaces
- Structured discussion forums
- Real-time collaborative document editing
- Argument visualization and mapping
- Consensus-building workflows

### Community Coordination
- Working group management
- Meeting scheduling and facilitation
- Member directory and messaging
- Event planning and coordination

## Configuration

### Environment Variables
```bash
# ICN Node Configuration
EXPO_PUBLIC_ICN_NODE_ENDPOINT=http://localhost:8080
EXPO_PUBLIC_ICN_NETWORK=devnet

# Governance Configuration
EXPO_PUBLIC_VOTING_PERIOD_DAYS=7
EXPO_PUBLIC_QUORUM_THRESHOLD=0.51
EXPO_PUBLIC_ENABLE_DELEGATION=true
```

## Usage Examples

### Proposal Creation
```typescript
import { useGovernance } from '@/hooks/useGovernance'

function CreateProposal() {
  const { submitProposal } = useGovernance()
  
  const handleSubmit = async (proposal) => {
    await submitProposal({
      title: 'Network Upgrade Proposal',
      description: 'Proposal to upgrade ICN protocol',
      changes: [{ parameter: 'block_time', value: 5 }],
      votingPeriod: 7
    })
  }
}
```

### Voting Interface
```typescript
import { useVoting } from '@/hooks/useVoting'

function VotingCard({ proposal }) {
  const { vote } = useVoting()
  
  return (
    <Card>
      <VStack space="$3">
        <Heading>{proposal.title}</Heading>
        <Body>{proposal.description}</Body>
        <HStack space="$2">
          <Button onPress={() => vote(proposal.id, 'yes')}>
            Vote Yes
          </Button>
          <Button onPress={() => vote(proposal.id, 'no')}>
            Vote No
          </Button>
        </HStack>
      </VStack>
    </Card>
  )
}
```

## Project Structure

```
apps/agoranet/
├── src/
│   ├── app/                    # Expo Router pages
│   │   ├── (tabs)/            # Tab navigation
│   │   ├── proposals/         # Proposal management
│   │   ├── voting/            # Voting interface
│   │   ├── deliberation/      # Discussion forums
│   │   └── community/         # Community features
│   ├── components/            # Reusable components
│   ├── hooks/                 # Custom React hooks
│   └── utils/                 # Utility functions
├── assets/                    # Static assets
└── .tauri/                   # Tauri desktop config
```

## Contributing

1. Follow React Native and Expo best practices
2. Implement accessible governance interfaces
3. Test democratic workflows thoroughly
4. Ensure cross-platform compatibility
5. Document governance patterns

## License

Apache-2.0 