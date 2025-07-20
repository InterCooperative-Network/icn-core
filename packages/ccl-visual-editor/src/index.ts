// Main export for @icn/ccl-visual-editor package

// Export main component
export { VisualEditor } from './components/VisualEditor'

// Export all subcomponents
export { ComponentPalette } from './components/ComponentPalette'
export { CanvasArea } from './components/CanvasArea'
export { PropertyInspector } from './components/PropertyInspector'
export { CodePreview } from './components/CodePreview'

// Export types
export type {
  Platform,
  InputMethod,
  Position,
  Size,
  PaletteComponent,
  ParameterDef,
  PortDef,
  ComponentConfig,
  CanvasNode,
  Connection,
  EditorState,
  DragState,
  TouchEvent,
  MouseEvent,
  KeyboardEvent,
  VoiceCommand,
  EditorConfig,
  EditorEventHandlers,
  CCLGenerationResult,
  PlatformAdapter,
  ContextMenuItem,
  AccessibilityAnnouncement,
  KeyboardNavigationState,
  VisualEditorProps,
} from './types'

// Export component definitions
export { 
  GOVERNANCE_COMPONENTS,
  GOVERNANCE_CATEGORIES,
  getComponentById,
  getComponentsByCategory,
  validateComponentConfig,
} from './palette/governanceComponents'

// Helper functions and utilities
export { default as createDefaultConfig } from './utils/createDefaultConfig'
export { default as generateCCLFromNodes } from './utils/generateCCL'

// Version
export const VERSION = '0.1.0' 