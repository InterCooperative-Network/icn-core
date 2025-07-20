# @icn/ccl-visual-editor

> Cross-platform visual editor for CCL (Cooperative Contract Language) contracts. Works seamlessly in React Native and Web environments.

## 🎨 **Universal Visual Contract Authoring**

A world-class visual editor that enables intuitive creation of CCL contracts through drag-and-drop components, with full support for mobile, desktop, and web platforms.

### **✨ Key Features**

- **🌐 Cross-Platform**: Works in React Native (iOS/Android), React Web, and Desktop (via Tauri)
- **📱 Multi-Modal Input**: Touch, mouse, keyboard, voice commands, and gesture support
- **♿ Accessibility First**: Screen reader support, keyboard navigation, and high contrast modes
- **🎯 Live Code Generation**: Real-time CCL code generation with validation and compilation
- **🔧 Rich Components**: Pre-built governance, economics, and identity components
- **⚡ Live Preview**: See your contract logic as you build
- **🚀 Direct Deployment**: Compile and deploy to ICN network

## 📦 **Installation**

```bash
# Install the package
pnpm add @icn/ccl-visual-editor

# Install peer dependencies
pnpm add react react-native @tamagui/core @tamagui/config
```

## 🚀 **Quick Start**

### **AgoraNet Integration (React Native)**

```tsx
import React from 'react'
import { VisualEditor } from '@icn/ccl-visual-editor'
import { Platform } from 'react-native'

export default function ContractEditorScreen() {
  const platform = Platform.OS === 'web' ? 'web' : 'mobile'
  const inputMethods = ['touch', 'voice']
  
  if (Platform.OS === 'web') {
    inputMethods.push('mouse', 'keyboard')
  }
  
  return (
    <VisualEditor
      config={{
        platform,
        inputMethods,
        features: {
          voiceCommands: true,
          gestures: true,
          collaboration: true,
          autoSave: true,
        }
      }}
      onCodeGenerated={handleCCLGeneration}
      onContractDeploy={handleContractDeploy}
    />
  )
}
```

### **Web UI Integration (React Web)**

```tsx
import React from 'react'
import { VisualEditor } from '@icn/ccl-visual-editor'

export function ContractEditorPage() {
  return (
    <div className="h-screen bg-gray-50">
      <VisualEditor
        config={{
          platform: 'web',
          inputMethods: ['mouse', 'keyboard', 'touch'],
          layout: {
            showPalette: true,
            showInspector: true,
            showCode: true,
            panelSizes: {
              palette: 300,
              inspector: 350,
              code: 280,
            }
          }
        }}
        onCodeGenerated={handleCCLGeneration}
        onContractDeploy={deployToICN}
      />
    </div>
  )
}
```

## 🧩 **Component Library**

### **Governance Components**

- **🗳️ Voting Mechanism**: Quorum, threshold, and delegation settings
- **👥 Member Role**: Role assignment and permissions
- **💰 Budget Request**: Budget allocation with approval workflows
- **📝 Proposal Creation**: Governance proposal management
- **⭐ Reputation Weighting**: Reputation-based voting weights
- **🏛️ Assembly Governance**: Large-scale democratic assemblies

### **Economics Components** *(Coming Soon)*

- **💎 Token Creation**: Create and manage token classes
- **🔄 Token Transfer**: Transfer logic with conditions
- **⏰ Time Banking**: Labor time tokens and exchange
- **🤝 Mutual Credit**: Community credit systems

### **Identity Components** *(Coming Soon)*

- **🆔 DID Management**: Create and manage decentralized identities
- **📜 Credential Issuance**: Issue verifiable credentials
- **🔗 Federation Joining**: Cross-federation verification

## 🎯 **Multi-Modal Input Support**

### **Touch (Mobile Primary)**

```tsx
// Touch-optimized components with haptic feedback
const TouchOptimizedPalette = () => {
  return (
    <VStack space="$3" padding="$4">
      {components.map((component) => (
        <Card
          key={component.id}
          pressStyle={{ scale: 0.95 }}
          animation="bouncy"
          onPress={() => handleTouch(component)}
          minHeight={60} // Touch-friendly minimum
          padding="$3"
        >
          <Text fontSize={24}>{component.icon}</Text>
          <Heading size="$4">{component.name}</Heading>
        </Card>
      ))}
    </VStack>
  )
}
```

### **Voice Commands**

```tsx
// Voice command system
const voiceCommands = {
  'add voting mechanism': () => addComponent('voting_mechanism'),
  'add member role': () => addComponent('member_role'),
  'add budget request': () => addComponent('budget_request'),
  'generate code': () => showCodePreview(),
  'compile contract': () => handleCompile(),
  'save contract': () => handleSave(),
}
```

### **Keyboard Shortcuts (Desktop)**

- **Ctrl/Cmd + Z**: Undo
- **Ctrl/Cmd + Y**: Redo
- **Ctrl/Cmd + C**: Copy selected
- **Ctrl/Cmd + V**: Paste
- **Ctrl/Cmd + A**: Select all
- **Ctrl/Cmd + S**: Save contract
- **Delete**: Delete selected components
- **Tab**: Navigate between areas
- **Arrow keys**: Navigate within areas

## ♿ **Accessibility Features**

### **Screen Reader Support**

```tsx
const AccessibleCanvasNode = ({ node }) => {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`${node.component.name}: ${node.component.description}`}
      aria-describedby={`node-${node.id}-details`}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          onSelect(node)
        }
      }}
    >
      <Card>
        <Heading id={`node-${node.id}-title`}>{node.component.name}</Heading>
        <Body id={`node-${node.id}-details`}>{node.component.description}</Body>
      </Card>
    </div>
  )
}
```

### **Keyboard Navigation**

- **Tab Navigation**: Cycle through all interactive elements
- **Arrow Key Navigation**: Move within component areas
- **Focus Management**: Clear visual focus indicators
- **Skip Links**: Jump to main content areas

## 🔧 **API Reference**

### **VisualEditor Props**

```tsx
interface VisualEditorProps {
  config: EditorConfig
  initialState?: Partial<EditorState>
  onNodeCreate?: (component: PaletteComponent, position: Position) => void
  onNodeUpdate?: (nodeId: string, updates: Partial<CanvasNode>) => void
  onNodeDelete?: (nodeId: string) => void
  onNodeSelect?: (nodeIds: string[]) => void
  onConnectionCreate?: (connection: Omit<Connection, 'id'>) => void
  onConnectionDelete?: (connectionId: string) => void
  onCodeGenerated?: (code: string) => void
  onContractDeploy?: (code: string, metadata: any) => void
  onSave?: (editorState: EditorState) => void
  onLoad?: () => EditorState | null
}
```

### **EditorConfig**

```tsx
interface EditorConfig {
  platform: 'web' | 'mobile' | 'desktop'
  inputMethods: ('touch' | 'mouse' | 'keyboard' | 'voice' | 'gesture')[]
  accessibility: {
    screenReader: boolean
    highContrast: boolean
    largeText: boolean
    keyboardOnly: boolean
  }
  features: {
    voiceCommands: boolean
    gestures: boolean
    collaboration: boolean
    autoSave: boolean
  }
  layout: {
    showPalette: boolean
    showInspector: boolean
    showCode: boolean
    panelSizes: {
      palette: number
      inspector: number
      code: number
    }
  }
}
```

## 🎨 **Customization**

### **Custom Components**

```tsx
import { PaletteComponent } from '@icn/ccl-visual-editor'

const customComponent: PaletteComponent = {
  id: 'custom_voting',
  category: 'governance',
  name: 'Custom Voting',
  description: 'Custom voting mechanism',
  icon: '🗳️',
  defaultConfig: {
    customOption: 'value'
  },
  parameters: [
    {
      name: 'customOption',
      type: 'string',
      description: 'Custom parameter',
      required: true,
      default: 'value'
    }
  ],
  ports: {
    inputs: [
      { id: 'input', name: 'Input', type: 'data', dataType: 'string' }
    ],
    outputs: [
      { id: 'output', name: 'Output', type: 'data', dataType: 'string' }
    ]
  }
}
```

### **Custom Themes**

```tsx
import { Theme } from '@tamagui/core'

<Theme name="custom_dark">
  <VisualEditor config={config} />
</Theme>
```

## 🔨 **Development**

```bash
# Install dependencies
pnpm install

# Start development build
pnpm dev

# Build for production
pnpm build

# Type checking
pnpm type-check

# Linting
pnpm lint

# Format code
pnpm format
```

## 🤝 **Contributing**

1. Follow the existing component patterns
2. Ensure all components work across platforms
3. Add comprehensive TypeScript types
4. Test on web, iOS, and Android
5. Include accessibility features
6. Update documentation

## 📄 **License**

Apache-2.0 - See [LICENSE](../../LICENSE) for details.

## 🔗 **Related Packages**

- [`@icn/ui-kit`](../ui-kit) - Shared UI components
- [`@icn/ts-sdk`](../ts-sdk) - TypeScript SDK for ICN
- [`icn-core`](../../) - Core ICN protocol implementation

---

**Built with ❤️ for the InterCooperative Network** 