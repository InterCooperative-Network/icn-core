#!/bin/bash

# ICN QR Code Action Examples
# These examples demonstrate the new QR/NFC functionality for seamless ICN interactions

echo "🔗 ICN QR Code Action Examples"
echo "=============================="
echo ""

# Example 1: Share Identity
echo "📱 Example 1: Share Identity"
echo "Generate a QR code to share your DID:"
echo "$ icn-cli qr share-identity did:icn:alice"
echo ""

# Example 2: Transfer Tokens
echo "💰 Example 2: Transfer Tokens"
echo "Generate a QR code for token transfer:"
echo "$ icn-cli qr transfer seed 100 did:icn:bob --memo 'Payment for services'"
echo ""

# Example 3: Vote on Proposal
echo "🗳️ Example 3: Vote on Proposal"
echo "Generate a QR code to vote on a governance proposal:"
echo "$ icn-cli qr vote bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi approve"
echo ""

# Example 4: Join Federation
echo "🤝 Example 4: Join Federation"
echo "Generate a QR code for federation onboarding:"
echo "$ icn-cli qr join 'Cooperative Network' --code 'INVITE123'"
echo ""

# Example 5: Verify Credential
echo "🔐 Example 5: Verify Credential"
echo "Generate a QR code to verify a credential:"
echo "$ icn-cli qr verify-credential bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
echo ""

# Example 6: Save to File
echo "💾 Example 6: Save QR Code to File"
echo "Save QR code as PNG image:"
echo "$ icn-cli qr share-identity did:icn:alice --output alice_qr.png --size 512"
echo ""

# Example 7: Decode URL
echo "🔍 Example 7: Decode Action URL"
echo "Decode an ICN action URL to see its contents:"
echo "$ icn-cli qr decode 'icn://share?did=did%3Aicn%3Aalice'"
echo ""

# Example 8: Encode Custom URL
echo "🔧 Example 8: Encode Custom URL"
echo "Generate QR code from a custom ICN action URL:"
echo "$ icn-cli qr encode 'icn://transfer?token=seed&amount=50&to=did:icn:charlie'"
echo ""

echo "🎯 Use Cases:"
echo "============="
echo ""
echo "• Business Cards: Share your DID with NFC tap or QR scan"
echo "• Cooperative Meetings: Vote on proposals by scanning QR codes"
echo "• Resource Sharing: Transfer tokens with phone-to-phone NFC"
echo "• Onboarding: Join federations by scanning invitation QR codes"
echo "• Verification: Prove credentials with tap-to-verify NFC"
echo "• Mobile Payments: Send resources with QR code scanning"
echo ""

echo "📖 For more information, see the ICN Action documentation."