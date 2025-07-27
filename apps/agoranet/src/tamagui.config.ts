import { createTamagui } from '@tamagui/core';
import { config as configBase } from '@tamagui/config';
import { createInterFont } from '@tamagui/font-inter';

const interFont = createInterFont();

const config = createTamagui({
  ...configBase,
  fonts: {
    ...configBase.fonts,
    body: interFont,
    heading: interFont,
  },
  themes: {
    ...configBase.themes,
    // Governance-themed colors
    governance_light: {
      ...configBase.themes.light,
      primary: '#1e40af',
      secondary: '#3b82f6',
      accent: '#10b981',
      destructive: '#ef4444',
    },
    governance_dark: {
      ...configBase.themes.dark,
      primary: '#3b82f6',
      secondary: '#1e40af',
      accent: '#10b981',
      destructive: '#f87171',
    },
  },
});

export type Conf = typeof config;

declare module '@tamagui/core' {
  interface TamaguiCustomConfig extends Conf {}
}

export default config;