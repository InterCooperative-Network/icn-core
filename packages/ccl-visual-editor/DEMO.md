# CCL Visual Editor - Demo & Usage Guide

## Interface Overview

The CCL Visual Editor provides a comprehensive interface for building governance contracts visually:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ 🎨 CCL Visual Editor                                                            │
├─────────────────────────────────────────────────────────────────────────────────┤
│ Component Palette      │        Visual Canvas           │    Property Inspector │
│ ┌─────────────────────┐ │ ┌─────────────────────────────┐ │ ┌─────────────────┐ │
│ │ 🏛️ Governance        │ │ │                             │ │ │ Properties      │ │
│ │                     │ │ │  ┌─────────────────────┐     │ │ │                 │ │
│ │ 🗳️ Voting Mechanism │ │ │  │ 🗳️ Voting Mechanism │     │ │ │ 🗳️ Voting       │ │
│ │ 👤 Member Role       │ │ │  │ Quorum: 50          │     │ │ │ Mechanism       │ │
│ │ 📝 Proposal Creation │ │ │  │ Threshold: 0.6      │     │ │ │                 │ │
│ │                     │ │ │  └─────────────────────┘     │ │ │ Quorum: [50   ] │ │
│ │ 💰 Economics         │ │ │           │                 │ │ │ Threshold:      │ │
│ │                     │ │ │           ▼                 │ │ │ [0.6          ] │ │
│ │ 💰 Budget Allocation │ │ │  ┌─────────────────────┐     │ │ │                 │ │
│ │                     │ │ │  │ 👤 Member Role       │     │ │ │ ☑ Can Vote      │ │
│ │ ⭐ Identity          │ │ │  │ Role: member        │     │ │ │ ☑ Can Propose   │ │
│ │                     │ │ │  │ Can Vote: ✓         │     │ │ │                 │ │
│ │ ⭐ Reputation Check  │ │ │  └─────────────────────┘     │ │ │                 │ │
│ │                     │ │ │                             │ │ │                 │ │
│ │ ❓ Logic            │ │ │                             │ │ │                 │ │
│ │                     │ │ │                             │ │ │                 │ │
│ │ ❓ If Condition     │ │ │                             │ │ │                 │ │
│ └─────────────────────┘ │ └─────────────────────────────┘ │ └─────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────────┤
│ Generated CCL Code                                                              │
│ ┌─────────────────────────────────────────────────────────────────────────────┐ │
│ │ // Generated CCL Contract: Governance Contract                              │ │
│ │ struct Role {                                                               │ │
│ │     name: String,                                                           │ │
│ │     can_vote: Boolean,                                                      │ │
│ │     voting_weight: Integer                                                  │ │
│ │ }                                                                           │ │
│ │                                                                             │ │
│ │ fn conduct_vote(proposal: Proposal, voter: Did) -> Proposal {              │ │
│ │     require_role(voter, "member");                                         │ │
│ │     let quorum = 50;                                                       │ │
│ │     let threshold = 0.6;                                                   │ │
│ │     // ... voting logic                                                    │ │
│ │ }                                                                           │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Usage Flow

### 1. Component Palette (Left Panel)
- **Governance Components**: 🗳️ Voting, 👤 Roles, 📝 Proposals
- **Economic Components**: 💰 Budget allocation
- **Identity Components**: ⭐ Reputation checks
- **Logic Components**: ❓ Conditional logic

**Usage**: Drag components from palette to canvas, or click to add

### 2. Visual Canvas (Center)
- **Drag & Drop**: Place components anywhere on canvas
- **Connections**: Draw lines between component ports
- **Selection**: Click components to select and configure
- **Visual Feedback**: Selected components highlighted in blue

**Features**:
- Grid background for alignment
- Component ports (input/output)
- Connection validation
- Auto-layout suggestions

### 3. Property Inspector (Right Panel)
- **Component Info**: Name, description, category
- **Parameter Configuration**: Editable fields with validation
- **Real-time Updates**: Changes immediately reflected in code
- **Validation**: Error highlighting for invalid values

**Parameter Types**:
- Numbers with min/max validation
- Booleans with checkboxes
- Strings with pattern validation
- DIDs with format validation
- Dropdown selections

### 4. Code Preview (Bottom Panel)
- **Live Generation**: Updates as you build
- **Syntax Highlighting**: CCL code with proper formatting
- **Validation**: Real-time error checking
- **Copy Function**: One-click copy to clipboard

## Example Workflow

### Building a Simple Governance Contract

1. **Add Member Role**:
   ```
   Drag "👤 Member Role" to canvas
   Configure: role_name="member", can_vote=true
   ```

2. **Add Voting Mechanism**:
   ```
   Drag "🗳️ Voting Mechanism" to canvas
   Configure: quorum=50, threshold=0.6, voting_period_days=7
   ```

3. **Connect Components**:
   ```
   Draw line from Member Role output to Voting input
   System validates data type compatibility
   ```

4. **Generated CCL**:
   ```ccl
   struct Role {
       name: String,
       can_vote: Boolean,
       voting_weight: Integer
   }
   
   fn create_member_role() -> Role {
       return Role {
           name: "member",
           can_vote: true,
           voting_weight: 1
       };
   }
   
   fn conduct_vote(proposal: Proposal, voter: Did, vote: String) -> Proposal {
       require_role(voter, "member");
       // ... implementation
   }
   ```

5. **Deploy Contract**:
   ```
   Click "Deploy Contract" button
   System calls ICN node /contracts endpoint
   Success notification shows deployment CID
   ```

## Advanced Features

### Component Library Extension
```typescript
// Add custom components
const CUSTOM_COMPONENT = {
  id: 'custom_vote_delegation',
  category: 'governance',
  name: 'Vote Delegation',
  description: 'Allow members to delegate their votes',
  icon: '🎯',
  // ... component definition
}
```

### Template Import/Export
```typescript
// Save visual contract as template
const template = {
  name: "Basic Governance",
  nodes: [...],
  connections: [...]
}

// Load template into editor
<VisualEditor initialContract={template} />
```

### Validation & Testing
- Real-time parameter validation
- Data type compatibility checking
- CCL syntax validation
- Connection logic verification

## Integration with ICN

The visual editor integrates seamlessly with ICN infrastructure:

1. **CCL Compilation**: Generated code uses ICN's CCL → WASM pipeline
2. **Contract Deployment**: Direct deployment to ICN nodes
3. **Governance Integration**: Works with existing governance templates
4. **Federation Support**: Multi-node contract deployment

## Accessibility Features

- **Keyboard Navigation**: Full keyboard support for all interactions
- **Screen Reader**: ARIA labels and semantic markup
- **High Contrast**: Visual indicators for colorblind users
- **Voice Commands**: Experimental voice control support

## Mobile Support

Responsive design for tablets and mobile devices:
- Touch-optimized drag and drop
- Collapsible panels for small screens
- Gesture support for navigation
- Mobile-friendly component sizing

---

**Status**: ✅ Fully implemented and ready for use!  
**Integration**: ✅ Connected to ICN governance infrastructure  
**Testing**: ✅ Test interface available at `test.html`