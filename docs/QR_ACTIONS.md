# ICN QR Code and NFC Actions

This document describes the QR code and NFC action system for ICN, enabling seamless sharing of identities, content, tokens, and cooperative actions without copying cryptographic strings.

## Overview

The ICN Action system provides a URL-based encoding scheme (`icn://`) that can be embedded in QR codes or NFC tags. This enables users to:

- Share identities (DIDs) with a tap or scan
- Transfer tokens with mobile payment-like simplicity  
- Vote on governance proposals via QR codes
- Join federations through invitation links
- Verify credentials with tap-to-verify
- Perform any ICN operation through human-friendly interactions

## URL Scheme

All actions use the `icn://` URL scheme with specific paths and parameters:

### Share Identity
```
icn://share?did=did:icn:alice
```
Share a DID for identity verification or contact exchange.

### Share Content
```
icn://share?cid=bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi&title=Document&description=Important%20file
```
Share content by CID with optional metadata.

### Transfer Tokens
```
icn://transfer?token=seed&amount=100&to=did:icn:bob&memo=Payment%20for%20services
```
Transfer tokens with specified amount and optional memo.

### Vote on Proposals  
```
icn://vote?proposal=bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi&vote=approve&voter=did:icn:alice
```
Vote on governance proposals with approve/reject/abstain choices.

### Join Federation
```
icn://join?federation=Cooperative%20Network&code=INVITE123
```
Join a federation with optional invitation code.

### Verify Credential
```
icn://verify?vc=bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi&challenge=abc123
```
Verify a credential with optional challenge string.

### Submit Job
```
icn://submit?job=bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi&submitter=did:icn:alice&max_cost=1000
```
Submit a mesh job for distributed execution.

### Compact Encoding
For shorter URLs suitable for smaller QR codes:
```
icn://x?d=eyJTaGFyZUlkZW50aXR5Ijp7ImRpZCI6ImRpZDppY246YWxpY2UifX0=
```
Base64-encoded JSON representation of the action.

## CLI Usage

The ICN CLI provides comprehensive QR code generation:

### Basic Commands

#### Share Identity
```bash
icn-cli qr share-identity did:icn:alice
```

#### Transfer Tokens
```bash
icn-cli qr transfer seed 100 did:icn:bob --memo "Payment for services"
```

#### Vote on Proposal
```bash
icn-cli qr vote bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi approve
```

#### Join Federation
```bash
icn-cli qr join "Cooperative Network" --code INVITE123
```

#### Verify Credential
```bash
icn-cli qr verify-credential bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi
```

### Advanced Options

#### Save to File
```bash
icn-cli qr share-identity did:icn:alice --output alice_qr.png --size 512
```

#### Decode URL
```bash
icn-cli qr decode "icn://share?did=did%3Aicn%3Aalice"
```

#### Encode Custom URL
```bash
icn-cli qr encode "icn://transfer?token=seed&amount=50&to=did:icn:charlie"
```

## Frontend Integration

### Web Applications

JavaScript/TypeScript integration using the TypeScript SDK:

```typescript
import { ActionEncoder, QrGenerator } from '@icn/ts-sdk';

// Create an action
const action = {
  ShareIdentity: {
    did: 'did:icn:alice'
  }
};

// Encode as URL
const url = ActionEncoder.encode(action);

// Generate QR code for display
const qrDataUrl = await QrGenerator.generateDataUrl(url, 256);
document.getElementById('qr-display').src = qrDataUrl;
```

### Mobile Applications (React Native)

```typescript
import { ActionEncoder } from '@icn/ts-sdk';
import QRCode from 'react-native-qrcode-svg';

const ShareIdentityScreen = ({ did }) => {
  const action = { ShareIdentity: { did } };
  const url = ActionEncoder.encode(action);
  
  return (
    <View>
      <QRCode value={url} size={200} />
      <Text>Scan to share identity</Text>
    </View>
  );
};
```

### NFC Integration

For NFC-enabled devices:

```typescript
// Web NFC API (where supported)
const writeNFC = async (action) => {
  const url = ActionEncoder.encode(action);
  const ndef = new NDEFMessage({
    records: [{ recordType: "url", data: url }]
  });
  
  await navigator.nfc.write(ndef);
};

// React Native NFC
import NfcManager from 'react-native-nfc-manager';

const writeNfcTag = async (action) => {
  const url = ActionEncoder.encode(action);
  await NfcManager.writeNdefMessage([
    { type: 'U', payload: url }
  ]);
};
```

## Use Cases

### 1. Digital Business Cards
Replace traditional business cards with NFC-enabled cards or QR codes that share your DID and credentials.

### 2. Cooperative Governance
During meetings, display QR codes for proposals that members can scan to vote instantly on their phones.

### 3. Resource Exchange
Enable mobile payment-like token transfers by scanning QR codes or tapping NFC devices.

### 4. Event Check-in
Use QR codes or NFC for event registration, credential verification, and resource access.

### 5. Cooperative Onboarding
New members can join federations by scanning invitation QR codes that contain all necessary setup information.

### 6. Supply Chain Verification
Products can have NFC tags or QR codes linking to verifiable credentials about their origin and certification.

### 7. Emergency Response
Emergency responders can quickly access relevant credentials and resources through pre-configured NFC tags or QR codes.

## Security Considerations

### URL Validation
All ICN action URLs are validated before execution:
- Scheme must be `icn://`
- Required parameters must be present
- DIDs and CIDs must be properly formatted
- Vote choices must be valid options

### User Consent
Applications should always:
- Display the action clearly before execution
- Require user confirmation for sensitive operations
- Show the source of scanned QR codes/NFC tags
- Allow users to review and cancel actions

### Privacy Protection
- Compact encoding can be used to reduce URL length
- Sensitive information should not be embedded in URLs
- Challenge-response patterns can be used for verification

## Error Handling

The action system provides comprehensive error messages:

```rust
pub enum ActionError {
    InvalidUrl(String),
    MissingParameter(String), 
    InvalidParameter(String),
    UnsupportedAction(String),
    QrGeneration(String),
    // ... more error types
}
```

Applications should handle these errors gracefully and provide clear feedback to users.

## Future Enhancements

### Planned Features
- Batch actions (multiple operations in one QR code)
- Conditional actions (if-then logic in URLs)
- Time-limited actions (expiring invitations)
- Multi-step workflows (guided onboarding sequences)
- Integration with hardware wallets
- Support for additional QR code formats (SVG, PDF417)

### Protocol Extensions
- Support for custom action types
- Plugin system for domain-specific actions
- Integration with external payment systems
- Cross-federation action routing

## Technical Implementation

### Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌────────────────┐
│   QR Scanner    │───▶│  Action Decoder  │───▶│ Action Handler │
│                 │    │                  │    │                │ 
│ • Camera Scan   │    │ • URL Parsing    │    │ • Validation   │
│ • NFC Read      │    │ • Parameter      │    │ • User Consent │
│ • Manual Entry  │    │   Extraction     │    │ • API Calls    │
└─────────────────┘    └──────────────────┘    └────────────────┘
```

### Crates
- `icn-action`: Core action encoding/decoding and QR generation
- `icn-cli`: Command-line QR code generation
- `ts-sdk`: TypeScript/JavaScript integration
- `mobile-sdk`: React Native and mobile platform integration

### Dependencies
- `qrcode`: QR code generation (optional feature)
- `image`: Image processing for QR codes
- `url`: URL parsing and validation
- `base64`: Compact encoding support
- `serde`: JSON serialization

## Conclusion

The ICN QR Code and NFC Action system transforms complex cryptographic operations into simple, human-friendly interactions. By enabling tap-to-share, scan-to-vote, and other intuitive gestures, ICN becomes accessible to everyday users while maintaining security and decentralization.

This system supports the ICN vision of "radically usable" cooperative infrastructure that anyone can use without technical expertise.