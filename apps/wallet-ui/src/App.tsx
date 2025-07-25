import React from 'react'
import { StatusBar } from 'expo-status-bar'
import { TamaguiProvider, Theme } from '@tamagui/core'
import { ICNProvider } from '@icn/ts-sdk'
import { I18NProvider, useTranslation } from '@icn/i18n'
import { Button, VStack, Heading, Body, icnConfig } from '@icn/ui-kit'
import { SafeAreaProvider } from 'react-native-safe-area-context'
import './i18n' // Initialize i18n

export default function App() {
  const icnOptions = {
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet' as const,
  }

  return (
    <SafeAreaProvider>
      <TamaguiProvider config={icnConfig}>
        <Theme name="light_icn">
          <I18NProvider>
            <ICNProvider options={icnOptions}>
              <WalletScreen />
              <StatusBar style="auto" />
            </ICNProvider>
          </I18NProvider>
        </Theme>
      </TamaguiProvider>
    </SafeAreaProvider>
  )
}

function WalletScreen() {
  const { t } = useTranslation('wallet')
  
  return (
    <VStack 
      flex={1} 
      justifyContent="center" 
      alignItems="center" 
      padding="$4" 
      space="$4"
    >
      <Heading>{t('title')}</Heading>
      <Body textAlign="center">
        {t('subtitle')}
      </Body>
      
      <VStack space="$3" width="100%" maxWidth={300}>
        <Button variant="primary" size="lg">
          {t('actions.createWallet')}
        </Button>
        
        <Button variant="secondary" size="lg">
          {t('actions.importWallet')}
        </Button>
        
        <Button variant="ghost" size="md">
          {t('actions.connectNode')}
        </Button>
      </VStack>
      
      <Body fontSize="$2" color="$gray10" textAlign="center">
        {t('version', { version: '0.1.0' })} • {t('description')}
      </Body>
    </VStack>
  )
} 