# CCL Visual Editor

A React-based visual contract editor for ICN (InterCooperative Network) governance contracts. This package provides a drag-and-drop interface for building CCL (Cooperative Contract Language) contracts visually.

## Features

✨ **Visual Contract Building**: Drag and drop governance components to build contracts
🎨 **Live Code Generation**: Real-time CCL code generation as you build
🔧 **Property Inspector**: Configure component parameters with validation
📱 **Cross-Platform**: Works in web browsers with React
🎯 **Governance Focus**: Pre-built components for voting, roles, budgets, and more
🔗 **ICN Integration**: Direct integration with ICN node for contract deployment

## Components

### 🗳️ Governance Components
- **Voting Mechanism**: Configure quorum, thresholds, and voting periods
- **Member Role**: Define roles with permissions and voting weights
- **Proposal Creation**: Set up proposal submission requirements
- **Budget Allocation**: Configure budget limits and approval processes

### ⭐ Identity Components  
- **Reputation Check**: Verify member reputation before actions

### ❓ Logic Components
- **If Condition**: Conditional logic based on criteria

## Installation

```bash
# Install in your React project
npm install @icn/ccl-visual-editor @icn/ts-sdk

# Or with pnpm
pnpm add @icn/ccl-visual-editor @icn/ts-sdk
```

## Quick Start

```tsx
import React from 'react'
import { VisualEditor } from '@icn/ccl-visual-editor'

function MyApp() {
  const handleCodeGenerated = (result) => {
    console.log('Generated CCL:', result.code)
  }

  const handleDeploy = async (cclCode) => {
    // Deploy to ICN node
    const response = await fetch('/contracts', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: cclCode })
    })
    return response.json()
  }

  return (
    <VisualEditor
      onCodeGenerated={handleCodeGenerated}
      onContractDeploy={handleDeploy}
    />
  )
}
```

## API Reference

### `<VisualEditor />`

The main visual editor component.

**Props:**
- `initialContract?: VisualContract` - Initial contract to load
- `readOnly?: boolean` - Make editor read-only
- `onContractChange?: (contract: VisualContract) => void` - Called when contract changes
- `onCodeGenerated?: (result: CCLGenerationResult) => void` - Called when code is generated
- `onContractDeploy?: (code: string) => Promise<void>` - Called when deploying contract

### `<ComponentPalette />`

Component palette for dragging governance building blocks.

### `<CanvasArea />`

Visual canvas for assembling contracts with drag-and-drop.

### `<PropertyInspector />`

Property inspector for configuring selected components.

### `<CodePreview />`

Live preview of generated CCL code with syntax highlighting.

## Generated CCL Example

When you build a contract visually, it generates CCL code like this:

```ccl
// Generated CCL Contract: Governance Contract
// Description: Generated from visual editor

struct Role {
    name: String,
    can_vote: Boolean,
    can_propose: Boolean,
    voting_weight: Integer
}

struct Proposal {
    id: String,
    proposer: Did,
    title: String,
    description: String,
    votes_yes: Integer,
    votes_no: Integer,
    status: String
}

// Role: member
fn create_member_role() -> Role {
    return Role {
        name: "member",
        can_vote: true,
        can_propose: true,
        voting_weight: 1
    };
}

// Voting mechanism
fn conduct_vote(proposal: Proposal, voter: Did, vote: String) -> Proposal {
    require_role(voter, "member");
    
    let updated_proposal = proposal;
    if (vote == "yes") {
        updated_proposal.votes_yes = updated_proposal.votes_yes + 1;
    } else if (vote == "no") {
        updated_proposal.votes_no = updated_proposal.votes_no + 1;
    }
    
    let total_votes = updated_proposal.votes_yes + updated_proposal.votes_no;
    let quorum = 50;
    let threshold = 0.6;
    
    if (total_votes >= quorum) {
        let approval_rate = updated_proposal.votes_yes / total_votes;
        if (approval_rate >= threshold) {
            updated_proposal.status = "approved";
        } else {
            updated_proposal.status = "rejected";
        }
    }
    
    return updated_proposal;
}
```

## Development

```bash
# Build the package
npm run build

# Watch for changes
npm run dev

# Clean build artifacts
npm run clean
```

## Architecture

The visual editor follows a component-based architecture:

1. **Component Library**: Pre-defined governance components with parameters
2. **Visual Canvas**: Drag-and-drop interface for building contracts
3. **Code Generator**: Converts visual structure to CCL code
4. **Property System**: Configure component parameters with validation
5. **Live Preview**: Real-time code generation and validation

## Contributing

1. Add new components to `src/components-library.ts`
2. Update the code generator in `src/ccl-generator.ts`
3. Test with the visual editor
4. Submit a pull request

## License

Apache-2.0 - see LICENSE file for details.