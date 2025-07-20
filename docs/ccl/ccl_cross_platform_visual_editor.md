# 🎨 CCL Cross-Platform Visual Editor

> **Universal Design**: Visual contract editor that works seamlessly across AgoraNet (mobile/desktop) and Web UI (browser) with comprehensive accessibility and input method support

> **✅ PHASE 1 COMPLETED**: Core shared package `@icn/ccl-visual-editor` implemented with Tamagui components, complete TypeScript architecture, governance component library, and real-time CCL code generation

---

## 🌐 **Cross-Platform Architecture**

### **Shared Component Library**
```
@icn/ccl-visual-editor (new package)
├── Core Editor Components (Tamagui-based)
├── Input Handlers (Touch, Mouse, Keyboard, Voice)
├── Accessibility Features (Screen readers, navigation)
└── Platform Adapters (React Native vs Web optimizations)

Apps Integration:
├── AgoraNet (React Native + Tamagui)
│   ├── Mobile: Touch-first with gesture support
│   ├── Desktop: Mouse/keyboard with shortcuts
│   └── Accessibility: Voice navigation, screen readers
└── Web UI (React + Tailwind + Tamagui)
    ├── Browser: Mouse/keyboard/touch hybrid
    ├── Mobile Web: Touch-optimized responsive
    └── Accessibility: Full ARIA support
```

### **Multi-Modal Input Matrix**

| Input Method | AgoraNet Mobile | AgoraNet Desktop | Web UI Browser | Web UI Mobile |
|--------------|-----------------|------------------|----------------|---------------|
| **Touch** | ✅ Primary | ✅ Secondary | ✅ Secondary | ✅ Primary |
| **Mouse** | ❌ N/A | ✅ Primary | ✅ Primary | ❌ N/A |
| **Keyboard** | ✅ External | ✅ Primary | ✅ Primary | ✅ Virtual |
| **Voice** | ✅ Native | ✅ Native | ✅ Web Speech | ✅ Web Speech |
| **Gestures** | ✅ Multi-touch | ✅ Trackpad | ✅ Trackpad | ✅ Touch |
| **Screen Reader** | ✅ Native | ✅ Native | ✅ ARIA | ✅ Mobile |

---

## 🧩 **Component Architecture**

### **1. Shared Core Package**
```bash
# New package: @icn/ccl-visual-editor
packages/ccl-visual-editor/
├── src/
│   ├── components/           # Core visual editor components
│   │   ├── VisualEditor.tsx  # Main editor container
│   │   ├── ComponentPalette.tsx
│   │   ├── CanvasArea.tsx
│   │   ├── PropertyInspector.tsx
│   │   └── CodePreview.tsx
│   ├── input/               # Input method handlers
│   │   ├── TouchHandler.tsx
│   │   ├── MouseHandler.tsx
│   │   ├── KeyboardHandler.tsx
│   │   ├── VoiceHandler.tsx
│   │   └── GestureHandler.tsx
│   ├── accessibility/       # Accessibility features
│   │   ├── ScreenReaderSupport.tsx
│   │   ├── KeyboardNavigation.tsx
│   │   └── VoiceCommands.tsx
│   └── platform/           # Platform-specific adapters
│       ├── NativeAdapter.tsx
│       └── WebAdapter.tsx
```

### **2. Multi-Modal Input Architecture**

#### **Touch Input (Mobile Primary)**
```tsx
// Touch-optimized component placement and sizing
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
          <HStack space="$3" alignItems="center">
            <Text fontSize={24}>{component.icon}</Text>
            <VStack flex={1}>
              <Heading size="$4">{component.name}</Heading>
              <Body size="$2" color="$gray10">{component.description}</Body>
            </VStack>
          </HStack>
        </Card>
      ))}
    </VStack>
  )
}

// Drag and drop for touch
const useTouchDragDrop = () => {
  const [isDragging, setIsDragging] = useState(false)
  const [dragData, setDragData] = useState(null)
  
  const handleTouchStart = (event, data) => {
    setIsDragging(true)
    setDragData(data)
    // Visual feedback
    Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
  }
  
  const handleTouchMove = (event) => {
    if (isDragging) {
      // Update drag preview position
      updateDragPreview(event.touches[0])
    }
  }
  
  const handleTouchEnd = (event) => {
    if (isDragging) {
      const dropTarget = findDropTarget(event.changedTouches[0])
      if (dropTarget) {
        onDrop(dragData, dropTarget)
        Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success)
      }
      setIsDragging(false)
      setDragData(null)
    }
  }
  
  return { isDragging, handleTouchStart, handleTouchMove, handleTouchEnd }
}
```

#### **Mouse Input (Desktop Primary)**
```tsx
// Mouse-optimized with hover states and precise positioning
const MouseOptimizedCanvas = () => {
  const [hoveredComponent, setHoveredComponent] = useState(null)
  const [selectedComponents, setSelectedComponents] = useState([])
  
  return (
    <div 
      className="canvas-area"
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onContextMenu={handleRightClick}
    >
      {nodes.map((node) => (
        <CanvasNode
          key={node.id}
          node={node}
          isHovered={hoveredComponent === node.id}
          isSelected={selectedComponents.includes(node.id)}
          onMouseEnter={() => setHoveredComponent(node.id)}
          onMouseLeave={() => setHoveredComponent(null)}
          onMouseDown={(e) => handleMouseDown(e, node)}
          style={{
            cursor: isDragging ? 'grabbing' : 'grab',
            transform: `translate(${node.position.x}px, ${node.position.y}px)`,
            transition: isDragging ? 'none' : 'all 0.2s ease'
          }}
        />
      ))}
      
      {/* Connection lines */}
      <svg className="connections-overlay">
        {connections.map((conn) => (
          <ConnectionLine key={conn.id} connection={conn} />
        ))}
      </svg>
    </div>
  )
}

// Keyboard shortcuts for mouse users
const useKeyboardShortcuts = () => {
  useEffect(() => {
    const handleKeyboard = (e) => {
      if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
          case 'z': handleUndo(); break
          case 'y': handleRedo(); break
          case 'c': handleCopy(); break
          case 'v': handlePaste(); break
          case 'a': handleSelectAll(); break
          case 's': handleSave(); break
        }
      }
      
      switch (e.key) {
        case 'Delete': handleDelete(); break
        case 'Escape': handleCancelOperation(); break
        case 'Enter': handleConfirmOperation(); break
      }
    }
    
    document.addEventListener('keydown', handleKeyboard)
    return () => document.removeEventListener('keydown', handleKeyboard)
  }, [])
}
```

#### **Voice Input (Universal)**
```tsx
// Voice command system
const useVoiceCommands = () => {
  const [isListening, setIsListening] = useState(false)
  const [recognition, setRecognition] = useState(null)
  
  useEffect(() => {
    if ('webkitSpeechRecognition' in window || 'SpeechRecognition' in window) {
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition
      const recognition = new SpeechRecognition()
      
      recognition.continuous = false
      recognition.interimResults = false
      recognition.lang = 'en-US'
      
      recognition.onresult = (event) => {
        const command = event.results[0][0].transcript.toLowerCase()
        handleVoiceCommand(command)
      }
      
      setRecognition(recognition)
    }
  }, [])
  
  const voiceCommands = {
    'add voting mechanism': () => addComponent('voting_mechanism'),
    'add member role': () => addComponent('member_role'),
    'add budget request': () => addComponent('budget_request'),
    'select all': () => handleSelectAll(),
    'delete selected': () => handleDelete(),
    'generate code': () => showCodePreview(),
    'compile contract': () => handleCompile(),
    'save contract': () => handleSave(),
  }
  
  const handleVoiceCommand = (command) => {
    const action = voiceCommands[command]
    if (action) {
      action()
      // Provide audio feedback
      if ('speechSynthesis' in window) {
        const utterance = new SpeechSynthesisUtterance(`Executed: ${command}`)
        speechSynthesis.speak(utterance)
      }
    }
  }
  
  return { isListening, startListening, stopListening }
}
```

#### **Accessibility Features**
```tsx
// Screen reader support
const AccessibleCanvasNode = ({ node, ...props }) => {
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
      {...props}
    >
      <Card>
        <Heading id={`node-${node.id}-title`}>{node.component.name}</Heading>
        <Body id={`node-${node.id}-details`}>{node.component.description}</Body>
      </Card>
    </div>
  )
}

// Keyboard navigation
const useKeyboardNavigation = () => {
  const [focusedIndex, setFocusedIndex] = useState(0)
  const [focusedArea, setFocusedArea] = useState('palette') // palette, canvas, inspector
  
  const handleKeyboardNavigation = (e) => {
    switch (e.key) {
      case 'Tab':
        e.preventDefault()
        if (e.shiftKey) {
          cycleFocusBackward()
        } else {
          cycleFocusForward()
        }
        break
      case 'ArrowUp':
        e.preventDefault()
        moveFocusUp()
        break
      case 'ArrowDown':
        e.preventDefault()
        moveFocusDown()
        break
      case 'ArrowLeft':
        e.preventDefault()
        changeFocusArea('left')
        break
      case 'ArrowRight':
        e.preventDefault()
        changeFocusArea('right')
        break
    }
  }
  
  return { focusedIndex, focusedArea, handleKeyboardNavigation }
}
```

---

## 🎯 **Implementation Strategy**

### **Phase 1: Shared Component Foundation (Week 1-2)**

1. **Create `@icn/ccl-visual-editor` package**
```bash
pnpm create @icn/ccl-visual-editor
cd packages/ccl-visual-editor
```

2. **Core components using Tamagui** (works in both React Native and Web)
```tsx
// packages/ccl-visual-editor/src/components/VisualEditor.tsx
import { VStack, HStack, Card, ScrollView } from '@tamagui/core'
import { ComponentPalette } from './ComponentPalette'
import { CanvasArea } from './CanvasArea'
import { PropertyInspector } from './PropertyInspector'
import { CodePreview } from './CodePreview'

export const VisualEditor = ({ platform, inputMethods, onCodeGenerated }) => {
  const [selectedNode, setSelectedNode] = useState(null)
  const [nodes, setNodes] = useState([])
  const [connections, setConnections] = useState([])
  
  // Platform-specific optimizations
  const isTouch = inputMethods.includes('touch')
  const isMobile = platform === 'mobile'
  
  return (
    <VStack flex={1} space="$2">
      {isMobile ? (
        // Mobile layout: Tabs for different areas
        <TabLayout>
          <Tab name="Components">
            <ComponentPalette 
              onComponentSelect={handleComponentSelect}
              touchOptimized={isTouch}
            />
          </Tab>
          <Tab name="Canvas">
            <CanvasArea
              nodes={nodes}
              connections={connections}
              onNodeUpdate={handleNodeUpdate}
              touchOptimized={isTouch}
            />
          </Tab>
          <Tab name="Properties">
            <PropertyInspector
              selectedNode={selectedNode}
              onPropertyChange={handlePropertyChange}
            />
          </Tab>
          <Tab name="Code">
            <CodePreview
              nodes={nodes}
              connections={connections}
              onCodeGenerated={onCodeGenerated}
            />
          </Tab>
        </TabLayout>
      ) : (
        // Desktop layout: Side panels
        <HStack flex={1} space="$2">
          <ComponentPalette width={250} />
          <VStack flex={1}>
            <CanvasArea flex={1} />
            <CodePreview height={200} />
          </VStack>
          <PropertyInspector width={300} />
        </HStack>
      )}
    </VStack>
  )
}
```

### **Phase 2: AgoraNet Integration (Week 2-3)**

1. **Add visual editor to AgoraNet**
```tsx
// apps/agoranet/src/app/contracts/editor.tsx
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
      platform={platform}
      inputMethods={inputMethods}
      onCodeGenerated={handleCCLGeneration}
      onContractDeploy={handleContractDeploy}
    />
  )
}
```

2. **Mobile-optimized gesture support**
```tsx
// Touch gestures for mobile
const useMobileGestures = () => {
  const panGestureHandler = PanGestureHandler.create()
    .onUpdate((event) => {
      // Pan to move components
      updateComponentPosition(event.translationX, event.translationY)
    })
  
  const pinchGestureHandler = PinchGestureHandler.create()
    .onUpdate((event) => {
      // Pinch to zoom canvas
      updateCanvasZoom(event.scale)
    })
  
  const doubleTapGestureHandler = TapGestureHandler.create()
    .numberOfTaps(2)
    .onEnd(() => {
      // Double tap to edit component
      openPropertyInspector()
    })
}
```

### **Phase 3: Web UI Integration (Week 3-4)**

1. **Add visual editor to Web UI**
```tsx
// apps/web-ui/src/pages/ContractEditorPage.tsx
import { VisualEditor } from '@icn/ccl-visual-editor'

export function ContractEditorPage() {
  const inputMethods = ['mouse', 'keyboard', 'touch'] // Hybrid support
  
  return (
    <div className="h-screen bg-gray-50">
      <div className="h-full max-w-7xl mx-auto">
        <VisualEditor
          platform="web"
          inputMethods={inputMethods}
          onCodeGenerated={handleCCLGeneration}
          onContractDeploy={deployToICN}
        />
      </div>
    </div>
  )
}

// Add route to existing navigation
const navigation = [
  { name: 'Demo', href: '/', icon: '🎯' },
  { name: 'Dashboard', href: '/dashboard', icon: '🏠' },
  { name: 'Federation', href: '/federation', icon: '🤝' },
  { name: 'Governance', href: '/governance', icon: '🗳️' },
  { name: 'Contract Editor', href: '/contracts/editor', icon: '🎨' }, // NEW
  { name: 'Cooperatives', href: '/cooperatives', icon: '🏢' },
  { name: 'Mesh Jobs', href: '/jobs', icon: '⚡' },
  { name: 'Settings', href: '/settings', icon: '⚙️' },
]
```

### **Phase 4: Cross-App Features (Week 4-5)**

1. **Shared contract templates**
2. **Real-time collaboration** (AgoraNet specialty)
3. **Cross-platform synchronization**
4. **Universal accessibility features**

---

## 🌟 **Key Benefits**

### **For AgoraNet Users**
- **Mobile-first design** with touch-optimized interactions
- **Voice commands** for hands-free contract editing
- **Real-time collaboration** for community contract drafting
- **Cross-platform sync** between mobile and desktop

### **For Web UI Users**
- **Powerful desktop tools** with mouse precision and keyboard shortcuts
- **Integration with federation management** workflows
- **Advanced debugging** and contract analysis tools
- **Professional development environment**

### **Universal Benefits**
- **Consistent experience** across all platforms
- **Accessibility-first design** supporting all users
- **Shared templates and components** between communities
- **Seamless handoff** between mobile discussion and desktop implementation

---

This creates a **world-class visual contract authoring experience** that works beautifully across all ICN applications and input methods!

Would you like me to start implementing Phase 1 - creating the shared `@icn/ccl-visual-editor` package with the core Tamagui components? 