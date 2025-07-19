# ICN Explorer

DAG viewer, job browser, and network activity visualizer for the InterCooperative Network.

## Overview

ICN Explorer is a web-based interface for browsing and visualizing ICN network activity. It provides tools for exploring the DAG structure, tracking job execution, monitoring network health, and analyzing economic activity.

## Technology Stack

- **React**: Modern React 18 with hooks
- **Vite**: Fast development and build tooling
- **TypeScript**: Type safety and developer experience
- **Tailwind CSS**: Utility-first styling
- **D3.js**: Data visualization and DAG rendering
- **React Router**: Client-side routing

## Platform Support

| Platform | Status | Build Command | Notes |
|----------|--------|---------------|-------|
| **Web** | ✅ | `pnpm build` | Modern browser support |
| **PWA** | ✅ | `pnpm build` | Progressive Web App |
| **Mobile** | ✅ | Responsive design | Mobile-optimized views |

## Features

### DAG Visualization
- ✅ Interactive DAG explorer
- ✅ Block and receipt visualization
- ✅ Content-addressed navigation
- ✅ DAG statistics and metrics
- ✅ Search and filtering

### Job Monitoring
- ✅ Real-time job tracking
- ✅ Job execution timelines
- ✅ Resource usage analytics
- ✅ Cost analysis and billing
- ✅ Performance benchmarks

### Network Analytics
- ✅ Network topology visualization
- ✅ Peer discovery and connectivity
- ✅ Message flow tracking
- ✅ Consensus monitoring
- ✅ Health and uptime metrics

### Economic Analysis
- ✅ Mana flow visualization
- ✅ Economic activity tracking
- ✅ Market analytics
- ✅ Resource pricing trends
- ✅ Economic health indicators

### Search and Discovery
- ✅ Advanced search interface
- ✅ Content-based filtering
- ✅ Transaction history
- ✅ Address and DID lookup
- ✅ Cross-reference navigation

## Development

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

# Explorer Configuration
VITE_APP_TITLE="ICN Network Explorer"
VITE_ENABLE_ANALYTICS=true
VITE_ENABLE_EXPERIMENTAL_FEATURES=false

# Visualization Settings
VITE_MAX_DAG_NODES=1000
VITE_REFRESH_INTERVAL=5000
VITE_ENABLE_REALTIME=true
```

## Key Features

### DAG Explorer
- Interactive node-link diagrams
- Zoomable and pannable interface
- Content-addressed navigation
- Block and receipt details
- DAG traversal and search

### Job Browser
- Job execution tracking
- Resource usage visualization
- Performance analytics
- Cost analysis and billing
- Execution timeline views

### Network Monitor
- Real-time network status
- Peer connectivity graphs
- Message flow visualization
- Consensus progress tracking
- Health monitoring dashboards

### Economic Dashboard
- Mana flow visualization
- Economic activity tracking
- Market trend analysis
- Resource pricing data
- Economic health metrics

## Usage Examples

### DAG Navigation
```tsx
import { DAGExplorer } from '@/components/dag/DAGExplorer'
import { useDAG } from '@/hooks/useDAG'

function DAGPage() {
  const { dag, loading, error } = useDAG()
  
  return (
    <div className="h-screen">
      <DAGExplorer 
        data={dag}
        loading={loading}
        onNodeClick={handleNodeClick}
      />
    </div>
  )
}
```

### Job Tracking
```tsx
import { JobTimeline } from '@/components/jobs/JobTimeline'
import { useJobs } from '@/hooks/useJobs'

function JobsPage() {
  const { jobs, filters, setFilters } = useJobs()
  
  return (
    <div>
      <JobFilters filters={filters} onChange={setFilters} />
      <JobTimeline jobs={jobs} />
    </div>
  )
}
```

### Network Visualization
```tsx
import { NetworkGraph } from '@/components/network/NetworkGraph'
import { useNetwork } from '@/hooks/useNetwork'

function NetworkPage() {
  const { peers, connections } = useNetwork()
  
  return (
    <NetworkGraph 
      nodes={peers}
      links={connections}
      onPeerSelect={handlePeerSelect}
    />
  )
}
```

## Visualization Components

### DAG Renderer
- Force-directed graph layout
- Hierarchical DAG visualization
- Interactive node exploration
- Zoom and pan controls
- Content preview on hover

### Timeline Views
- Job execution timelines
- Network activity timelines
- Economic event tracking
- Performance trend charts
- Real-time data streams

### Network Topology
- Peer connection graphs
- Geographic distribution maps
- Network health indicators
- Connectivity matrices
- Message flow diagrams

## Data Sources

### ICN SDK Integration
```tsx
import { useICNClient } from '@icn/ts-sdk'

function useDAGData() {
  const { client } = useICNClient()
  
  const fetchDAG = async () => {
    // Fetch DAG data from ICN node
    const response = await client.getDAGBlocks()
    return response.blocks
  }
  
  return useQuery(['dag'], fetchDAG)
}
```

### Real-time Updates
```tsx
import { useWebSocket } from '@/hooks/useWebSocket'

function useRealTimeJobs() {
  const { data, connected } = useWebSocket('ws://localhost:8080/jobs')
  
  return {
    jobs: data,
    connected,
    isRealTime: true
  }
}
```

## Testing

```bash
# Run unit tests
pnpm test

# Run tests with coverage
pnpm test --coverage

# Run visual regression tests
pnpm test:visual
```

## Performance Optimization

### Large Dataset Handling
- Virtual scrolling for large lists
- Data pagination and lazy loading
- Canvas-based rendering for complex visualizations
- Web Workers for heavy computations
- Optimized D3.js rendering

### Caching Strategy
- Browser-based data caching
- Progressive data loading
- Background data prefetching
- Optimistic UI updates
- Efficient state management

## Deployment

### Static Hosting
```bash
# Build and deploy
pnpm build

# Deploy to hosting services:
# - Vercel, Netlify, GitHub Pages
# - IPFS for decentralized hosting
# - CDN for global distribution
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
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

## API Endpoints

### DAG Data
- `GET /api/dag/blocks` - Get DAG blocks
- `GET /api/dag/receipts` - Get execution receipts
- `GET /api/dag/search` - Search DAG content

### Job Data
- `GET /api/jobs` - List jobs with filters
- `GET /api/jobs/:id` - Get job details
- `GET /api/jobs/stats` - Job statistics

### Network Data
- `GET /api/network/peers` - Network peers
- `GET /api/network/stats` - Network statistics
- `GET /api/network/health` - Network health

## Browser Compatibility

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Contributing

1. Follow React and TypeScript best practices
2. Use D3.js for data visualizations
3. Ensure responsive design
4. Write comprehensive tests
5. Optimize for performance

## License

Apache-2.0 