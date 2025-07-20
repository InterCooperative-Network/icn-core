// Core types for the CCL Visual Editor

export type Platform = 'web' | 'mobile' | 'desktop'
export type InputMethod = 'touch' | 'mouse' | 'keyboard' | 'voice' | 'gesture'

export interface Position {
  x: number
  y: number
}

export interface Size {
  width: number
  height: number
}

// Component palette types
export interface PaletteComponent {
  id: string
  category: 'governance' | 'economics' | 'identity' | 'logic' | 'flow'
  name: string
  description: string
  icon: string
  defaultConfig: ComponentConfig
  parameters: ParameterDef[]
  ports: {
    inputs: PortDef[]
    outputs: PortDef[]
  }
}

export interface ParameterDef {
  name: string
  type: 'string' | 'number' | 'boolean' | 'did' | 'duration' | 'select'
  description: string
  required: boolean
  default?: any
  validation?: {
    min?: number
    max?: number
    pattern?: string
    options?: string[]
  }
}

export interface PortDef {
  id: string
  name: string
  type: 'data' | 'control' | 'event'
  dataType?: 'string' | 'number' | 'boolean' | 'object' | 'array'
}

export interface ComponentConfig {
  [key: string]: any
}

// Canvas node types
export interface CanvasNode {
  id: string
  type: 'component' | 'logic' | 'data'
  position: Position
  size: Size
  component: PaletteComponent
  config: ComponentConfig
  connections: string[] // Connection IDs
  selected: boolean
  zIndex: number
}

export interface Connection {
  id: string
  sourceNodeId: string
  targetNodeId: string
  sourcePortId: string
  targetPortId: string
  type: 'data' | 'control' | 'event'
}

// Editor state types
export interface EditorState {
  nodes: CanvasNode[]
  connections: Connection[]
  selectedNodes: string[]
  dragState: DragState | null
  canvasOffset: Position
  canvasZoom: number
  isDirty: boolean
}

export interface DragState {
  type: 'node' | 'connection' | 'selection'
  startPosition: Position
  currentPosition: Position
  data: any
}

// Input handling types
export interface TouchEvent {
  touches: Touch[]
  changedTouches: Touch[]
  type: 'start' | 'move' | 'end' | 'cancel'
}

export interface MouseEvent {
  clientX: number
  clientY: number
  button: number
  ctrlKey: boolean
  shiftKey: boolean
  altKey: boolean
  type: 'down' | 'move' | 'up' | 'click' | 'dblclick'
}

export interface KeyboardEvent {
  key: string
  code: string
  ctrlKey: boolean
  shiftKey: boolean
  altKey: boolean
  metaKey: boolean
  type: 'down' | 'up' | 'press'
}

export interface VoiceCommand {
  command: string
  confidence: number
  alternatives: string[]
}

// Editor configuration
export interface EditorConfig {
  platform: Platform
  inputMethods: InputMethod[]
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

// Event handlers
export interface EditorEventHandlers {
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

// CCL generation types
export interface CCLGenerationResult {
  code: string
  metadata: {
    version: string
    generator: string
    timestamp: string
    nodeCount: number
    connectionCount: number
  }
  errors: string[]
  warnings: string[]
}

// Platform adapter interface
export interface PlatformAdapter {
  createDragPreview: (node: CanvasNode) => any
  showContextMenu: (position: Position, items: ContextMenuItem[]) => void
  showToast: (message: string, type: 'success' | 'error' | 'info') => void
  vibrate: (pattern?: number[]) => void
  playSound: (sound: string) => void
  getClipboard: () => Promise<string>
  setClipboard: (text: string) => Promise<void>
}

export interface ContextMenuItem {
  id: string
  label: string
  icon?: string
  action: () => void
  disabled?: boolean
  separator?: boolean
}

// Accessibility types
export interface AccessibilityAnnouncement {
  message: string
  priority: 'low' | 'medium' | 'high'
  type: 'announcement' | 'alert' | 'status'
}

export interface KeyboardNavigationState {
  focusedArea: 'palette' | 'canvas' | 'inspector' | 'code'
  focusedIndex: number
  focusHistory: string[]
}

// Export main props interface
export interface VisualEditorProps extends EditorEventHandlers {
  config: EditorConfig
  initialState?: Partial<EditorState>
  className?: string
  style?: any
} 