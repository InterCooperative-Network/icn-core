import { EditorConfig, Platform, InputMethod } from '../types'

export default function createDefaultConfig(
  platform: Platform = 'web',
  inputMethods: InputMethod[] = ['mouse', 'keyboard', 'touch']
): EditorConfig {
  return {
    platform,
    inputMethods,
    accessibility: {
      screenReader: false,
      highContrast: false,
      largeText: false,
      keyboardOnly: false,
    },
    features: {
      voiceCommands: inputMethods.includes('voice'),
      gestures: inputMethods.includes('gesture'),
      collaboration: false,
      autoSave: true,
    },
    layout: {
      showPalette: true,
      showInspector: true,
      showCode: true,
      panelSizes: {
        palette: platform === 'mobile' ? 200 : 280,
        inspector: platform === 'mobile' ? 250 : 320,
        code: platform === 'mobile' ? 180 : 250,
      }
    }
  }
} 