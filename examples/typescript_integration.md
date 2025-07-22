# ICN Action TypeScript SDK Integration

This example shows how the ICN QR/NFC action layer will integrate with the TypeScript SDK for frontend applications.

```typescript
// packages/ts-sdk/src/action.ts

export interface IcnAction {
  ShareIdentity?: { did: string };
  ShareContent?: { cid: string; title?: string; description?: string };
  TransferToken?: { token: string; amount: number; to: string; memo?: string };
  Vote?: { proposal: string; vote: 'approve' | 'reject' | 'abstain'; voter?: string };
  JoinFederation?: { federation_id: string; invitation_code?: string };
  VerifyCredential?: { credential: string; challenge?: string };
  SubmitJob?: { job_spec: string; submitter: string; max_cost?: number };
}

export class ActionEncoder {
  static encode(action: IcnAction): string {
    // Implementation will call Rust WASM bindings or pure TS implementation
    // Returns icn:// URL
  }
  
  static decode(url: string): IcnAction {
    // Parse icn:// URL back to action object
  }
  
  static encodeCompact(action: IcnAction): string {
    // Returns base64 encoded compact URL: icn://x?d=...
  }
}

export class QrGenerator {
  static async generateDataUrl(url: string, size: number = 256): Promise<string> {
    // Generate QR code as data URL for web display
  }
  
  static async generateSvg(url: string, size: number = 256): Promise<string> {
    // Generate QR code as SVG string
  }
  
  static generateTerminal(url: string): string {
    // Generate ASCII art QR code for terminal display
  }
}

// React component example
import React from 'react';
import { ActionEncoder, QrGenerator } from '@icn/ts-sdk';

export const ShareIdentityComponent: React.FC<{ did: string }> = ({ did }) => {
  const [qrDataUrl, setQrDataUrl] = React.useState<string>('');
  
  React.useEffect(() => {
    const action = { ShareIdentity: { did } };
    const url = ActionEncoder.encode(action);
    
    QrGenerator.generateDataUrl(url, 256).then(setQrDataUrl);
  }, [did]);
  
  return (
    <div className="share-identity">
      <h3>Share Your Identity</h3>
      <img src={qrDataUrl} alt="QR code for sharing identity" />
      <p>Scan this QR code to get my DID: {did}</p>
    </div>
  );
};

// React Native component example
import React from 'react';
import { View, Text } from 'react-native';
import QRCode from 'react-native-qrcode-svg';
import { ActionEncoder } from '@icn/ts-sdk';

export const TransferTokenComponent: React.FC<{
  token: string;
  amount: number;
  to: string;
  memo?: string;
}> = ({ token, amount, to, memo }) => {
  const action = { TransferToken: { token, amount, to, memo } };
  const url = ActionEncoder.encode(action);
  
  return (
    <View style={{ alignItems: 'center', padding: 20 }}>
      <Text style={{ fontSize: 18, fontWeight: 'bold', marginBottom: 10 }}>
        Transfer {amount} {token}
      </Text>
      <QRCode value={url} size={200} />
      <Text style={{ marginTop: 10, textAlign: 'center' }}>
        Scan to receive {amount} {token} tokens
      </Text>
      {memo && <Text style={{ fontStyle: 'italic' }}>Memo: {memo}</Text>}
    </View>
  );
};

// NFC integration example
export class NfcHandler {
  static async writeAction(action: IcnAction): Promise<void> {
    const url = ActionEncoder.encode(action);
    
    if ('nfc' in navigator) {
      // Web NFC API
      const ndef = new NDEFMessage({
        records: [{ recordType: "url", data: url }]
      });
      await navigator.nfc.write(ndef);
    } else {
      // Fallback or React Native NFC
      throw new Error('NFC not supported on this platform');
    }
  }
  
  static async readAction(): Promise<IcnAction> {
    if ('nfc' in navigator) {
      // Web NFC API
      const reader = new NDEFReader();
      const reading = await reader.scan();
      
      for (const record of reading.message.records) {
        if (record.recordType === 'url') {
          const url = new TextDecoder().decode(record.data);
          return ActionEncoder.decode(url);
        }
      }
    }
    
    throw new Error('No ICN action found on NFC tag');
  }
}

// Wallet UI integration
export const WalletActions: React.FC = () => {
  const [currentAction, setCurrentAction] = React.useState<IcnAction | null>(null);
  
  const handleScan = async () => {
    try {
      // This would use camera API or NFC
      const action = await scanQrOrNfc();
      setCurrentAction(action);
    } catch (error) {
      console.error('Failed to scan action:', error);
    }
  };
  
  const executeAction = async () => {
    if (!currentAction) return;
    
    // Show confirmation dialog
    const confirmed = await showActionConfirmation(currentAction);
    if (!confirmed) return;
    
    // Execute the action via ICN API
    try {
      await executeIcnAction(currentAction);
      setCurrentAction(null);
    } catch (error) {
      console.error('Failed to execute action:', error);
    }
  };
  
  return (
    <div className="wallet-actions">
      <button onClick={handleScan}>Scan QR Code or NFC</button>
      
      {currentAction && (
        <div className="action-preview">
          <h3>Action Preview</h3>
          <pre>{JSON.stringify(currentAction, null, 2)}</pre>
          <button onClick={executeAction}>Execute Action</button>
          <button onClick={() => setCurrentAction(null)}>Cancel</button>
        </div>
      )}
    </div>
  );
};

// Helper functions
async function scanQrOrNfc(): Promise<IcnAction> {
  // Implementation would use camera or NFC APIs
  // Returns decoded action
}

async function showActionConfirmation(action: IcnAction): Promise<boolean> {
  // Show user-friendly confirmation dialog
  return confirm(`Execute action: ${JSON.stringify(action, null, 2)}?`);
}

async function executeIcnAction(action: IcnAction): Promise<void> {
  // Make appropriate API calls to ICN node based on action type
  if (action.TransferToken) {
    // Call token transfer API
  } else if (action.Vote) {
    // Call governance voting API
  }
  // etc.
}
```

## Usage Examples

### Web Application
```html
<!DOCTYPE html>
<html>
<head>
  <title>ICN Cooperative Dashboard</title>
</head>
<body>
  <div id="share-identity">
    <!-- Share Identity QR code component -->
  </div>
  
  <div id="scan-actions">
    <button onclick="scanQrCode()">Scan QR Code</button>
    <div id="action-preview" style="display: none;">
      <!-- Action preview and confirmation -->
    </div>
  </div>
  
  <script src="@icn/ts-sdk"></script>
  <script>
    function generateIdentityQr() {
      const action = { ShareIdentity: { did: 'did:icn:alice' } };
      const url = IcnSdk.ActionEncoder.encode(action);
      
      IcnSdk.QrGenerator.generateDataUrl(url).then(dataUrl => {
        document.getElementById('qr-display').src = dataUrl;
      });
    }
  </script>
</body>
</html>
```

### Mobile App (React Native)
```tsx
import React from 'react';
import { View, Button, Alert } from 'react-native';
import { ActionEncoder, NfcHandler } from '@icn/ts-sdk';

export const CooperativeApp: React.FC = () => {
  const handleNfcTap = async () => {
    try {
      const action = await NfcHandler.readAction();
      
      Alert.alert(
        'ICN Action Detected',
        `Action: ${JSON.stringify(action, null, 2)}`,
        [
          { text: 'Cancel', style: 'cancel' },
          { text: 'Execute', onPress: () => executeAction(action) }
        ]
      );
    } catch (error) {
      Alert.alert('Error', 'Failed to read NFC tag');
    }
  };
  
  return (
    <View style={{ flex: 1, justifyContent: 'center', padding: 20 }}>
      <Button title="Tap NFC Device" onPress={handleNfcTap} />
    </View>
  );
};
```

This TypeScript SDK integration will make ICN actions accessible to web and mobile developers, enabling the creation of user-friendly cooperative applications.