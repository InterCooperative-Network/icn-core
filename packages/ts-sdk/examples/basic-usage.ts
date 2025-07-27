/**
 * Basic Usage Example for ICN TypeScript SDK
 * 
 * This example demonstrates:
 * - Client initialization
 * - Connection management
 * - Basic API operations
 * - Error handling
 */

import { 
  ICNClient, 
  createStorage, 
  ICNConnectionError, 
  ICNNetworkError,
  ErrorUtils 
} from '@icn/ts-sdk';

async function basicUsageExample() {
  console.log('🚀 Starting ICN TypeScript SDK Basic Usage Example\n');

  // 1. Create client with configuration
  console.log('📊 Creating ICN client...');
  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet',
    timeout: 30000,
    storage: createStorage('@icn-example:'),
    encryptionConfig: {
      enableEncryption: true,
      passphrase: 'example-secret-key'
    }
  });

  try {
    // 2. Connect to the ICN node
    console.log('🔗 Connecting to ICN node...');
    await client.connect();
    
    const connectionState = client.getConnectionState();
    console.log('✅ Connected successfully!');
    console.log(`   Network: ${connectionState.network}`);
    console.log(`   Endpoint: ${connectionState.nodeEndpoint}`);
    console.log(`   Status: ${connectionState.connected ? 'Connected' : 'Disconnected'}\n`);

    // 3. Get node information
    console.log('ℹ️  Fetching node information...');
    const nodeInfo = await client.system.getInfo();
    console.log('📋 Node Info:');
    console.log(`   Name: ${nodeInfo.name}`);
    console.log(`   Version: ${nodeInfo.version}`);
    console.log(`   Status: ${nodeInfo.status_message}\n`);

    // 4. Get node status
    console.log('📊 Fetching node status...');
    const nodeStatus = await client.system.getStatus();
    console.log('🔍 Node Status:');
    console.log(`   Online: ${nodeStatus.is_online}`);
    console.log(`   Peers: ${nodeStatus.peer_count}`);
    console.log(`   Block Height: ${nodeStatus.current_block_height}\n`);

    // 5. Check health
    console.log('🏥 Checking node health...');
    const health = await client.system.getHealth();
    console.log(`💚 Health Status: ${health.status}`);
    if (health.details) {
      console.log('   Details:', health.details);
    }
    console.log();

    // 6. List federation peers
    console.log('👥 Fetching federation peers...');
    const peers = await client.federation.listPeers();
    console.log(`🌐 Found ${peers.length} federation peers:`);
    peers.slice(0, 3).forEach((peer, index) => {
      console.log(`   ${index + 1}. ${peer}`);
    });
    if (peers.length > 3) {
      console.log(`   ... and ${peers.length - 3} more\n`);
    } else {
      console.log();
    }

    // 7. Get federation status
    console.log('🏛️  Fetching federation status...');
    const federationStatus = await client.federation.getStatus();
    console.log('🔗 Federation Status:');
    console.log(`   Peer Count: ${federationStatus.peer_count}`);
    console.log(`   Connected Peers: ${federationStatus.peers.length}\n`);

    // 8. Test account operations (if we have a DID)
    if (connectionState.did) {
      console.log('💰 Fetching account information...');
      try {
        const manaBalance = await client.account.getManaBalance(connectionState.did);
        console.log(`💎 Mana Balance: ${manaBalance.balance}`);
        
        const reputation = await client.reputation.getScore(connectionState.did);
        console.log(`⭐ Reputation Score: ${reputation.score}`);
        console.log();
      } catch (error) {
        console.log('⚠️  Account operations require authentication\n');
      }
    } else {
      console.log('⚠️  No DID configured - skipping account operations\n');
    }

    // 9. List jobs
    console.log('⚙️  Fetching mesh computing jobs...');
    try {
      const jobs = await client.mesh.listJobs();
      console.log(`📋 Found ${jobs.length} jobs:`);
      jobs.slice(0, 3).forEach((job, index) => {
        console.log(`   ${index + 1}. ${job.id} - ${job.status}`);
      });
      if (jobs.length > 3) {
        console.log(`   ... and ${jobs.length - 3} more\n`);
      } else {
        console.log();
      }
    } catch (error) {
      console.log('⚠️  Job listing may require authentication\n');
    }

    // 10. Test storage
    console.log('💾 Testing local storage...');
    const storage = client.getStorage();
    
    // Store some test data
    await storage.setCachedData('test-key', { 
      message: 'Hello ICN!', 
      timestamp: Date.now() 
    }, 60000); // 1 minute TTL
    
    const storedData = await storage.getCachedData('test-key');
    console.log('✅ Storage test successful:');
    console.log(`   Stored: ${JSON.stringify(storedData)}\n`);

    console.log('🎉 Basic usage example completed successfully!');

  } catch (error) {
    console.error('❌ Error during example execution:');
    
    if (ErrorUtils.isErrorType(error, ICNConnectionError)) {
      console.error('🔌 Connection Error:', error.message);
      console.error('💡 Tip: Make sure ICN node is running at the specified endpoint');
    } else if (ErrorUtils.isErrorType(error, ICNNetworkError)) {
      console.error('🌐 Network Error:', error.message);
      if (error.statusCode) {
        console.error(`   Status Code: ${error.statusCode}`);
      }
    } else {
      console.error('🔍 Unexpected Error:', ErrorUtils.getErrorMessage(error));
    }
    
    console.error('\n📖 For troubleshooting, check:');
    console.error('   1. ICN node is running and accessible');
    console.error('   2. Network connectivity');
    console.error('   3. API endpoint URL is correct');
    console.error('   4. Required permissions for operations');
  } finally {
    // Clean up connection
    try {
      await client.disconnect();
      console.log('\n🔌 Disconnected from ICN node');
    } catch (error) {
      console.error('⚠️  Error during disconnect:', ErrorUtils.getErrorMessage(error));
    }
  }
}

// Run the example
if (require.main === module) {
  basicUsageExample().catch(console.error);
}

export { basicUsageExample };