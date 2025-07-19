import React from 'react'
import { StatusBar } from 'expo-status-bar'
import { TamaguiProvider, Theme } from '@tamagui/core'
import { ICNProvider } from '@icn/ts-sdk'
import { Button, VStack, Heading, Body, icnConfig } from '@icn/ui-kit'
import { SafeAreaProvider } from 'react-native-safe-area-context'

export default function App() {
  const icnOptions = {
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet' as const,
  }

  return (
    <SafeAreaProvider>
      <TamaguiProvider config={icnConfig}>
        <Theme name="light_icn">
          <ICNProvider options={icnOptions}>
            <WalletScreen />
            <StatusBar style="auto" />
          </ICNProvider>
        </Theme>
      </TamaguiProvider>
    </SafeAreaProvider>
  )
}

function WalletScreen() {
  return (
    <VStack 
      flex={1} 
      justifyContent="center" 
      alignItems="center" 
      padding="$4" 
      space="$4"
    >
      <Heading>ICN Wallet</Heading>
      <Body textAlign="center">
        Secure DID and key management for the InterCooperative Network
      </Body>
      
      <VStack space="$3" width="100%" maxWidth={300}>
        <Button variant="primary" size="lg">
          Create New Wallet
        </Button>
        
        <Button variant="secondary" size="lg">
          Import Existing Wallet
        </Button>
        
        <Button variant="ghost" size="md">
          Connect to Node
        </Button>
      </VStack>
      
      <Body fontSize="$2" color="$gray10" textAlign="center">
        Version 0.1.0 • Cross-platform DID wallet
      </Body>
    </VStack>
  )
} 