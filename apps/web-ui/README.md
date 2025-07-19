# ICN Web UI

Federation and cooperative management dashboard for the InterCooperative Network.

## Overview

ICN Web UI is a browser-based administration dashboard for managing ICN federations, cooperatives, and communities. It provides comprehensive tools for network administration, member management, and system monitoring.

## Technology Stack

- **React**: Modern React 18 with hooks
- **Vite**: Fast development and build tooling
- **TypeScript**: Type safety and developer experience
- **Tailwind CSS**: Utility-first styling
- **React Router**: Client-side routing
- **Headless UI**: Accessible component primitives

## Platform Support

| Platform | Status | Build Command | Notes |
|----------|--------|---------------|-------|
| **Web** | ✅ | `pnpm build` | Modern browser support |
| **PWA** | ✅ | `pnpm build` | Progressive Web App |
| **Desktop** | 🔄 | Future Tauri support | Optional desktop wrapper |

## Features

### Dashboard Overview
- ✅ Network status and health monitoring
- ✅ Real-time statistics and metrics
- ✅ System alerts and notifications
- ✅ Quick action buttons
- ✅ Recent activity feed

### Member Management
- ✅ Member directory and profiles
- ✅ Role and permission management
- ✅ DID verification and validation
- ✅ Reputation tracking
- ✅ Member onboarding workflows

### Job Management
- ✅ Mesh job monitoring
- ✅ Job queue management
- ✅ Resource allocation tracking
- ✅ Performance analytics
- ✅ Cost and billing analysis

### Governance Administration
- ✅ Proposal management
- ✅ Voting oversight
- ✅ Policy configuration
- ✅ Consensus monitoring
- ✅ Election administration

### System Configuration
- ✅ Network parameters
- ✅ Economic policy settings
- ✅ Security configurations
- ✅ API key management
- ✅ Audit logs

## Development

### Prerequisites
```bash
# Node.js 18+ and pnpm
node --version  # >= 18.0.0
pnpm --version  # >= 8.0.0
```

### Start Development Server
```bash
# Install dependencies
pnpm install

# Start development server
pnpm dev

# Access at http://localhost:3000
```

### Build for Production
```bash
# Build for production
pnpm build

# Preview production build
pnpm preview

# Output in dist/ directory
```

## Configuration

### Environment Variables
Create `.env.local` file:

```bash
# ICN Node Configuration
VITE_ICN_NODE_ENDPOINT=http://localhost:8080
VITE_ICN_NETWORK=devnet

# Dashboard Configuration
VITE_APP_TITLE="ICN Federation Dashboard"
VITE_FEDERATION_NAME="My Federation"
VITE_ENABLE_ANALYTICS=true

# Feature Flags
VITE_ENABLE_MEMBER_REGISTRATION=true
VITE_ENABLE_JOB_SUBMISSION=true
VITE_ENABLE_GOVERNANCE_VOTING=true
```

### Theme Customization
Update `tailwind.config.js`:

```javascript
export default {
  theme: {
    extend: {
      colors: {
        primary: {
          // Custom brand colors
          500: '#your-primary-color',
        },
        secondary: {
          // Custom secondary colors
        }
      }
    }
  }
}
```

## Project Structure

```
apps/web-ui/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── common/         # Generic components
│   │   ├── dashboard/      # Dashboard-specific
│   │   ├── members/        # Member management
│   │   ├── jobs/          # Job management
│   │   └── governance/     # Governance tools
│   ├── pages/              # Route components
│   ├── hooks/              # Custom React hooks
│   ├── utils/              # Utility functions
│   ├── types/              # TypeScript definitions
│   └── styles/             # Global styles
├── public/                 # Static assets
├── dist/                   # Build output
└── vite.config.ts         # Vite configuration
```

## Key Components

### Dashboard Widgets
```tsx
import { NetworkStatus } from '@/components/dashboard/NetworkStatus'
import { JobMetrics } from '@/components/dashboard/JobMetrics'
import { MemberActivity } from '@/components/dashboard/MemberActivity'

function Dashboard() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <NetworkStatus />
      <JobMetrics />
      <MemberActivity />
    </div>
  )
}
```

### Data Tables
```tsx
import { useMembers } from '@/hooks/useMembers'
import { DataTable } from '@/components/common/DataTable'

function MemberList() {
  const { members, loading } = useMembers()
  
  const columns = [
    { key: 'did', label: 'DID' },
    { key: 'name', label: 'Name' },
    { key: 'role', label: 'Role' },
    { key: 'reputation', label: 'Reputation' },
  ]
  
  return (
    <DataTable 
      data={members}
      columns={columns}
      loading={loading}
    />
  )
}
```

### Forms and Modals
```tsx
import { useForm } from 'react-hook-form'
import { Modal } from '@/components/common/Modal'
import { Button } from '@/components/common/Button'

function CreateJobModal({ isOpen, onClose }) {
  const { register, handleSubmit } = useForm()
  
  const onSubmit = async (data) => {
    // Submit job logic
  }
  
  return (
    <Modal isOpen={isOpen} onClose={onClose}>
      <form onSubmit={handleSubmit(onSubmit)}>
        {/* Form fields */}
        <Button type="submit">Create Job</Button>
      </form>
    </Modal>
  )
}
```

## API Integration

### ICN SDK Usage
```tsx
import { useICNClient, useICNConnection } from '@icn/ts-sdk'

function NetworkStatus() {
  const { connected, error } = useICNConnection()
  const { client } = useICNClient()
  
  return (
    <div className="bg-white p-6 rounded-lg shadow">
      <h3 className="text-lg font-medium">Network Status</h3>
      <div className={`mt-2 ${connected ? 'text-green-600' : 'text-red-600'}`}>
        {connected ? 'Connected' : 'Disconnected'}
      </div>
      {error && (
        <div className="mt-2 text-red-600 text-sm">{error}</div>
      )}
    </div>
  )
}
```

## Testing

```bash
# Run unit tests
pnpm test

# Run tests with UI
pnpm test:ui

# Run tests with coverage
pnpm test --coverage
```

## Deployment

### Static Hosting
```bash
# Build for production
pnpm build

# Deploy dist/ to:
# - Vercel, Netlify, GitHub Pages
# - AWS S3 + CloudFront
# - Your web server
```

### Docker Deployment
```dockerfile
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN pnpm install
COPY . .
RUN pnpm build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Environment-Specific Builds
```bash
# Development build
VITE_ICN_NETWORK=devnet pnpm build

# Staging build
VITE_ICN_NETWORK=testnet pnpm build

# Production build
VITE_ICN_NETWORK=mainnet pnpm build
```

## Security Considerations

### Authentication
- Secure session management
- Role-based access control
- API key protection
- DID-based authentication

### Data Protection
- Input validation and sanitization
- XSS protection
- CSRF protection
- Secure API communication

## Accessibility

- WCAG 2.1 AA compliance
- Keyboard navigation support
- Screen reader compatibility
- High contrast mode
- Responsive design

## Contributing

1. Follow React and TypeScript best practices
2. Use Tailwind CSS for styling
3. Write comprehensive tests
4. Ensure accessibility compliance
5. Update documentation

## License

Apache-2.0 