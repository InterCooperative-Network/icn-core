import { Slot } from 'expo-router';
import { TamaguiProvider } from '@tamagui/core';
import { StatusBar } from 'expo-status-bar';
import { GovernanceProvider } from '@/hooks/useGovernance';
import config from '@/tamagui.config';

export default function RootLayout() {
  return (
    <TamaguiProvider config={config}>
      <GovernanceProvider>
        <StatusBar style="auto" />
        <Slot />
      </GovernanceProvider>
    </TamaguiProvider>
  );
}