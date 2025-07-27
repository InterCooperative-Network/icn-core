/**
 * Token Operations Example
 * 
 * This example demonstrates:
 * - Token class creation
 * - Token minting and burning
 * - Token transfers between accounts
 * - Balance management
 * - Token economics
 */

import { 
  ICNClient, 
  createStorage, 
  ICNTokenError,
  ICNValidationError,
  ErrorUtils 
} from '@icn/ts-sdk';

async function tokenOperationsExample() {
  console.log('🪙 Starting Token Operations Example\n');

  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet',
    storage: createStorage('@token-example:'),
  });

  try {
    await client.connect();
    console.log('✅ Connected to ICN node\n');

    // Example DIDs for token operations
    const treasuryDid = 'did:key:treasury123';
    const aliceDid = 'did:key:alice456';
    const bobDid = 'did:key:bob789';
    const charlieDid = 'did:key:charlie012';

    // 1. Create Token Classes
    console.log('🏭 Creating token classes...');
    
    // Skill recognition token
    const skillToken = await client.tokens.createTokenClass({
      id: 'SKILL_TOKEN',
      name: 'Skill Recognition Token',
      symbol: 'SKILL',
      decimals: 2
    });
    console.log(`✅ Created ${skillToken.name} (${skillToken.symbol})`);
    
    // Reputation token
    const reputationToken = await client.tokens.createTokenClass({
      id: 'REP_TOKEN',
      name: 'Reputation Token',
      symbol: 'REP',
      decimals: 0
    });
    console.log(`✅ Created ${reputationToken.name} (${reputationToken.symbol})`);
    
    // Governance voting token
    const governanceToken = await client.tokens.createTokenClass({
      id: 'VOTE_TOKEN',
      name: 'Governance Voting Token',
      symbol: 'VOTE',
      decimals: 0
    });
    console.log(`✅ Created ${governanceToken.name} (${governanceToken.symbol})\n`);

    // 2. Initial Token Minting
    console.log('💰 Minting initial token supplies...');
    
    // Mint skill tokens to treasury
    await client.tokens.mintTokens({
      class_id: 'SKILL_TOKEN',
      to_did: treasuryDid,
      amount: 1000000 // 10,000.00 with 2 decimals
    });
    console.log('✅ Minted 10,000.00 SKILL tokens to treasury');
    
    // Mint reputation tokens to treasury
    await client.tokens.mintTokens({
      class_id: 'REP_TOKEN',
      to_did: treasuryDid,
      amount: 500000
    });
    console.log('✅ Minted 500,000 REP tokens to treasury');
    
    // Mint governance tokens to treasury
    await client.tokens.mintTokens({
      class_id: 'VOTE_TOKEN',
      to_did: treasuryDid,
      amount: 100000
    });
    console.log('✅ Minted 100,000 VOTE tokens to treasury\n');

    // 3. Check Treasury Balances
    console.log('📊 Checking treasury balances...');
    
    const treasuryBalances = await client.tokens.listBalances(treasuryDid);
    console.log('💼 Treasury Token Balances:');
    treasuryBalances.forEach(balance => {
      const formattedAmount = balance.class_id === 'SKILL_TOKEN' 
        ? (balance.amount / 100).toFixed(2) 
        : balance.amount.toString();
      console.log(`   ${balance.class_id}: ${formattedAmount}`);
    });
    console.log();

    // 4. Distribute Tokens to Users
    console.log('🎁 Distributing tokens to users...');
    
    // Give Alice skill tokens for her contributions
    await client.tokens.transferTokens({
      class_id: 'SKILL_TOKEN',
      from_did: treasuryDid,
      to_did: aliceDid,
      amount: 50000 // 500.00 SKILL
    });
    console.log('✅ Transferred 500.00 SKILL tokens to Alice');
    
    // Give Alice reputation tokens
    await client.tokens.transferTokens({
      class_id: 'REP_TOKEN',
      from_did: treasuryDid,
      to_did: aliceDid,
      amount: 1000
    });
    console.log('✅ Transferred 1,000 REP tokens to Alice');
    
    // Give Bob skill tokens
    await client.tokens.transferTokens({
      class_id: 'SKILL_TOKEN',
      from_did: treasuryDid,
      to_did: bobDid,
      amount: 30000 // 300.00 SKILL
    });
    console.log('✅ Transferred 300.00 SKILL tokens to Bob');
    
    // Give Bob governance tokens
    await client.tokens.transferTokens({
      class_id: 'VOTE_TOKEN',
      from_did: treasuryDid,
      to_did: bobDid,
      amount: 500
    });
    console.log('✅ Transferred 500 VOTE tokens to Bob\n');

    // 5. User Token Balances
    console.log('👥 Checking user token balances...');
    
    const aliceBalances = await client.tokens.listBalances(aliceDid);
    console.log('👩 Alice\'s Token Balances:');
    aliceBalances.forEach(balance => {
      const formattedAmount = balance.class_id === 'SKILL_TOKEN' 
        ? (balance.amount / 100).toFixed(2) 
        : balance.amount.toString();
      console.log(`   ${balance.class_id}: ${formattedAmount}`);
    });
    
    const bobBalances = await client.tokens.listBalances(bobDid);
    console.log('👨 Bob\'s Token Balances:');
    bobBalances.forEach(balance => {
      const formattedAmount = balance.class_id === 'SKILL_TOKEN' 
        ? (balance.amount / 100).toFixed(2) 
        : balance.amount.toString();
      console.log(`   ${balance.class_id}: ${formattedAmount}`);
    });
    console.log();

    // 6. Peer-to-Peer Token Transfers
    console.log('↔️  Demonstrating peer-to-peer transfers...');
    
    // Alice pays Bob for a service using skill tokens
    await client.tokens.transferTokens({
      class_id: 'SKILL_TOKEN',
      from_did: aliceDid,
      to_did: bobDid,
      amount: 15000 // 150.00 SKILL
    });
    console.log('✅ Alice paid Bob 150.00 SKILL tokens for services');
    
    // Bob transfers some governance tokens to Alice for her participation
    await client.tokens.transferTokens({
      class_id: 'VOTE_TOKEN',
      from_did: bobDid,
      to_did: aliceDid,
      amount: 100
    });
    console.log('✅ Bob transferred 100 VOTE tokens to Alice for governance participation\n');

    // 7. Check Updated Balances
    console.log('📈 Updated balances after transfers...');
    
    const aliceUpdatedBalances = await client.tokens.listBalances(aliceDid);
    console.log('👩 Alice\'s Updated Balances:');
    aliceUpdatedBalances.forEach(balance => {
      const formattedAmount = balance.class_id === 'SKILL_TOKEN' 
        ? (balance.amount / 100).toFixed(2) 
        : balance.amount.toString();
      console.log(`   ${balance.class_id}: ${formattedAmount}`);
    });
    
    const bobUpdatedBalances = await client.tokens.listBalances(bobDid);
    console.log('👨 Bob\'s Updated Balances:');
    bobUpdatedBalances.forEach(balance => {
      const formattedAmount = balance.class_id === 'SKILL_TOKEN' 
        ? (balance.amount / 100).toFixed(2) 
        : balance.amount.toString();
      console.log(`   ${balance.class_id}: ${formattedAmount}`);
    });
    console.log();

    // 8. Token Class Information
    console.log('ℹ️  Token class information...');
    
    const skillTokenInfo = await client.tokens.getTokenClass('SKILL_TOKEN');
    if (skillTokenInfo) {
      console.log('🏆 Skill Token Details:');
      console.log(`   Name: ${skillTokenInfo.name}`);
      console.log(`   Symbol: ${skillTokenInfo.symbol}`);
      console.log(`   Decimals: ${skillTokenInfo.decimals}`);
      console.log(`   ID: ${skillTokenInfo.id}`);
    }
    
    const repTokenInfo = await client.tokens.getTokenClass('REP_TOKEN');
    if (repTokenInfo) {
      console.log('⭐ Reputation Token Details:');
      console.log(`   Name: ${repTokenInfo.name}`);
      console.log(`   Symbol: ${repTokenInfo.symbol}`);
      console.log(`   Decimals: ${repTokenInfo.decimals}`);
      console.log(`   ID: ${repTokenInfo.id}`);
    }
    console.log();

    // 9. Token Burning (Reducing Supply)
    console.log('🔥 Demonstrating token burning...');
    
    // Burn some tokens from treasury to reduce supply
    await client.tokens.burnTokens({
      class_id: 'SKILL_TOKEN',
      from_did: treasuryDid,
      amount: 100000 // 1,000.00 SKILL
    });
    console.log('✅ Burned 1,000.00 SKILL tokens from treasury (reducing total supply)');
    
    // Check treasury balance after burning
    const treasuryAfterBurn = await client.tokens.listBalances(treasuryDid);
    const skillBalance = treasuryAfterBurn.find(b => b.class_id === 'SKILL_TOKEN');
    if (skillBalance) {
      console.log(`📉 Treasury SKILL balance after burn: ${(skillBalance.amount / 100).toFixed(2)}`);
    }
    console.log();

    // 10. Advanced Token Operations
    console.log('🚀 Advanced token operations...');
    
    // Multi-token transfer simulation (using multiple transfers)
    console.log('📦 Simulating multi-token package transfer...');
    
    // Package: 50 SKILL + 100 REP to Charlie
    await client.tokens.transferTokens({
      class_id: 'SKILL_TOKEN',
      from_did: treasuryDid,
      to_did: charlieDid,
      amount: 5000 // 50.00 SKILL
    });
    
    await client.tokens.transferTokens({
      class_id: 'REP_TOKEN',
      from_did: treasuryDid,
      to_did: charlieDid,
      amount: 100
    });
    
    console.log('✅ Transferred welcome package to Charlie: 50.00 SKILL + 100 REP');
    
    // Check Charlie's balances
    const charlieBalances = await client.tokens.listBalances(charlieDid);
    console.log('👤 Charlie\'s Token Balances:');
    charlieBalances.forEach(balance => {
      const formattedAmount = balance.class_id === 'SKILL_TOKEN' 
        ? (balance.amount / 100).toFixed(2) 
        : balance.amount.toString();
      console.log(`   ${balance.class_id}: ${formattedAmount}`);
    });

    console.log('\n🎉 Token Operations example completed successfully!');
    console.log('\n💡 Key Features Demonstrated:');
    console.log('   • Token class creation with different decimal precision');
    console.log('   • Initial token minting to treasury');
    console.log('   • Token distribution to users');
    console.log('   • Peer-to-peer token transfers');
    console.log('   • Balance tracking and management');
    console.log('   • Token burning for supply management');
    console.log('   • Multi-token operations');
    
    console.log('\n📊 Token Economy Summary:');
    console.log('   • SKILL tokens: Reward contributions and services');
    console.log('   • REP tokens: Track reputation and standing');
    console.log('   • VOTE tokens: Enable governance participation');
    console.log('   • Treasury: Manages token distribution and economy');

  } catch (error) {
    console.error('❌ Error during token operations example:');
    
    if (ErrorUtils.isErrorType(error, ICNTokenError)) {
      console.error('🪙 Token Error:', error.message);
      console.error('💡 Tip: Check token parameters and account permissions');
    } else if (ErrorUtils.isErrorType(error, ICNValidationError)) {
      console.error('📝 Validation Error:', error.message);
      if (error.field) {
        console.error(`   Field: ${error.field}`);
      }
    } else {
      console.error('🔍 Unexpected Error:', ErrorUtils.getErrorMessage(error));
    }
  } finally {
    await client.disconnect();
    console.log('\n🔌 Disconnected from ICN node');
  }
}

// Run the example
if (require.main === module) {
  tokenOperationsExample().catch(console.error);
}

export { tokenOperationsExample };