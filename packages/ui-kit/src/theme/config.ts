import { createTamagui } from '@tamagui/core'
import { config as defaultConfig } from '@tamagui/config/v3'
import { lightTokens, darkTokens } from './tokens'
import { ICNTheme } from './types'

// Create ICN-specific theme configuration
export const icnLightTheme: ICNTheme = {
  name: 'light',
  tokens: lightTokens,
  isDark: false,
}

export const icnDarkTheme: ICNTheme = {
  name: 'dark',
  tokens: darkTokens,
  isDark: true,
}

// Tamagui configuration with ICN themes
export const icnConfig = createTamagui({
  ...defaultConfig,
  themes: {
    ...defaultConfig.themes,
    light_icn: {
      background: lightTokens.color.background,
      backgroundHover: lightTokens.color.surface,
      color: lightTokens.color.text,
      colorHover: lightTokens.color.textSecondary,
      borderColor: lightTokens.color.border,
      primary: lightTokens.color.primary,
      secondary: lightTokens.color.secondary,
      success: lightTokens.color.success,
      warning: lightTokens.color.warning,
      danger: lightTokens.color.danger,
    },
    dark_icn: {
      background: darkTokens.color.background,
      backgroundHover: darkTokens.color.surface,
      color: darkTokens.color.text,
      colorHover: darkTokens.color.textSecondary,
      borderColor: darkTokens.color.border,
      primary: darkTokens.color.primary,
      secondary: darkTokens.color.secondary,
      success: darkTokens.color.success,
      warning: darkTokens.color.warning,
      danger: darkTokens.color.danger,
    },
  },
})

export type ICNConfig = typeof icnConfig

export default icnConfig 