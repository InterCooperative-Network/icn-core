/**
 * Trust Networks Management Example
 * 
 * This example demonstrates:
 * - Trust relationship creation and management
 * - Trust path discovery
 * - Trust score calculation
 * - Trust graph analytics
 * - Federation trust statistics
 */

import { 
  ICNClient, 
  createStorage, 
  TrustLevel, 
  TrustContext,
  ICNTrustError,
  ErrorUtils 
} from '@icn/ts-sdk';

async function trustNetworkExample() {
  console.log('🤝 Starting Trust Networks Management Example\n');

  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet',
    storage: createStorage('@trust-example:'),
  });

  try {
    await client.connect();
    console.log('✅ Connected to ICN node\n');

    // Example DIDs for demonstration
    const aliceDid = 'did:key:alice123';
    const bobDid = 'did:key:bob456';
    const charlieDid = 'did:key:charlie789';

    // 1. Create Trust Relationships
    console.log('🔗 Creating trust relationships...');
    
    // Alice trusts Bob in technical context
    await client.trust.updateTrustRelationship({
      from: aliceDid,
      to: bobDid,
      trust_level: 'high',
      context: 'technical',
      metadata: {
        reason: 'successful_collaboration',
        project: 'rust_development',
        duration: '6_months'
      }
    });
    console.log(`✅ Alice → Bob: high trust (technical)`);

    // Bob trusts Charlie in financial context
    await client.trust.updateTrustRelationship({
      from: bobDid,
      to: charlieDid,
      trust_level: 'medium',
      context: 'financial',
      metadata: {
        reason: 'loan_repayment',
        amount: '1000',
        currency: 'MANA'
      }
    });
    console.log(`✅ Bob → Charlie: medium trust (financial)`);

    // Charlie trusts Alice in governance context
    await client.trust.updateTrustRelationship({
      from: charlieDid,
      to: aliceDid,
      trust_level: 'high',
      context: 'governance',
      metadata: {
        reason: 'proposal_quality',
        votes_aligned: '12'
      }
    });
    console.log(`✅ Charlie → Alice: high trust (governance)\n`);

    // 2. Get Trust Relationships
    console.log('📊 Retrieving trust relationships...');
    
    const aliceRelationships = await client.trust.getEntityTrustRelationships(aliceDid, {
      include_inherited: true,
      include_cross_federation: true
    });
    
    console.log(`🔍 Alice has ${aliceRelationships.length} trust relationships:`);
    aliceRelationships.forEach((rel, index) => {
      console.log(`   ${index + 1}. ${rel.from} → ${rel.to}`);
      console.log(`      Level: ${rel.trust_level}, Context: ${rel.context}`);
      console.log(`      Created: ${new Date(rel.created_at * 1000).toLocaleDateString()}`);
    });
    console.log();

    // 3. Calculate Trust Scores
    console.log('⭐ Calculating trust scores...');
    
    const entities = [aliceDid, bobDid, charlieDid];
    const trustScores = await client.trust.getTrustScores(entities);
    
    console.log('📈 Trust Scores:');
    trustScores.forEach(score => {
      console.log(`   ${score.did}: ${(score.score * 100).toFixed(1)}%`);
      console.log(`      Incoming: ${score.incoming_trust_count}, Outgoing: ${score.outgoing_trust_count}`);
      console.log(`      Federations: ${score.federations.length}`);
      
      // Show context-specific scores
      Object.entries(score.context_scores).forEach(([context, contextScore]) => {
        console.log(`      ${context}: ${(contextScore * 100).toFixed(1)}%`);
      });
      console.log();
    });

    // 4. Find Trust Paths
    console.log('🛤️  Finding trust paths...');
    
    const trustPaths = await client.trust.findTrustPaths({
      from: aliceDid,
      to: charlieDid,
      context: 'general',
      max_length: 5,
      max_paths: 3,
      min_trust_level: 'low'
    });
    
    console.log(`🔍 Found ${trustPaths.length} trust paths from Alice to Charlie:`);
    trustPaths.forEach((path, index) => {
      console.log(`   Path ${index + 1}: ${path.from} → ${path.path.join(' → ')} → ${path.to}`);
      console.log(`      Length: ${path.length}, Weight: ${path.weight.toFixed(2)}`);
      console.log(`      Effective Trust: ${path.effective_trust}`);
      console.log(`      Contexts: ${path.contexts.join(', ')}`);
    });
    console.log();

    // 5. Trust Graph Analytics
    console.log('📊 Analyzing trust graph...');
    
    const graphStats = await client.trust.getTrustGraphStats();
    console.log('🌐 Trust Graph Statistics:');
    console.log(`   Total Entities: ${graphStats.total_entities}`);
    console.log(`   Total Relationships: ${graphStats.total_relationships}`);
    console.log(`   Average Trust Score: ${(graphStats.average_trust_score * 100).toFixed(1)}%`);
    console.log(`   Connected Components: ${graphStats.connected_components}`);
    
    console.log('   Relationships by Context:');
    Object.entries(graphStats.relationships_by_context).forEach(([context, count]) => {
      console.log(`      ${context}: ${count}`);
    });
    
    console.log('   Trust Distribution:');
    Object.entries(graphStats.trust_distribution).forEach(([level, count]) => {
      console.log(`      ${level}: ${count}`);
    });
    console.log();

    // 6. Search by Trust Criteria
    console.log('🔍 Searching entities by trust criteria...');
    
    const highTrustEntities = await client.trust.searchByTrust({
      min_trust_level: 'high',
      context: 'technical',
      include_inherited: true
    }, 10, 0);
    
    console.log(`📋 Found ${highTrustEntities.length} entities with high technical trust:`);
    highTrustEntities.forEach((entity, index) => {
      console.log(`   ${index + 1}. ${entity.did}: ${(entity.score * 100).toFixed(1)}%`);
      if (entity.context_scores.technical) {
        console.log(`      Technical Score: ${(entity.context_scores.technical * 100).toFixed(1)}%`);
      }
    });
    console.log();

    // 7. Validate Trust for Operations
    console.log('✅ Validating trust for operations...');
    
    const canDelegate = await client.trust.validateTrustOperation(
      aliceDid,
      bobDid,
      'governance',
      'delegate_vote'
    );
    
    const canTransfer = await client.trust.validateTrustOperation(
      bobDid,
      charlieDid,
      'financial',
      'transfer_tokens'
    );
    
    console.log(`🗳️  Alice can delegate vote to Bob: ${canDelegate ? 'Yes' : 'No'}`);
    console.log(`💰 Bob can transfer tokens to Charlie: ${canTransfer ? 'Yes' : 'No'}`);
    console.log();

    // 8. Federation Trust Statistics
    console.log('🏛️  Analyzing federation trust...');
    
    try {
      const federationStats = await client.trust.getFederationTrustStats('federation_1');
      console.log('📊 Federation Trust Statistics:');
      console.log(`   Member Count: ${federationStats.member_count}`);
      console.log(`   Average Internal Trust: ${(federationStats.average_internal_trust * 100).toFixed(1)}%`);
      console.log(`   Active Contexts: ${federationStats.active_contexts.join(', ')}`);
      console.log(`   Bridge Count: ${federationStats.bridge_count}`);
    } catch (error) {
      console.log('⚠️  Federation statistics not available (federation may not exist)');
    }
    console.log();

    // 9. Trust Relationship Management
    console.log('🔧 Managing trust relationships...');
    
    // Update existing relationship
    await client.trust.updateTrustRelationship({
      from: aliceDid,
      to: bobDid,
      trust_level: 'absolute',
      context: 'technical',
      metadata: {
        reason: 'exceptional_work',
        project: 'icn_core_contribution'
      }
    });
    console.log('✅ Updated Alice → Bob trust to absolute level');

    // Get updated trust score
    const bobUpdatedScore = await client.trust.getTrustScore(bobDid);
    console.log(`📈 Bob's updated trust score: ${(bobUpdatedScore.score * 100).toFixed(1)}%`);
    
    // Demonstrate trust relationship removal (commented to preserve example data)
    // await client.trust.removeTrustRelationship(aliceDid, bobDid, 'technical');
    // console.log('🗑️  Removed Alice → Bob trust relationship');

    console.log('\n🎉 Trust Networks example completed successfully!');
    console.log('\n💡 Key Insights:');
    console.log('   • Trust relationships are context-specific');
    console.log('   • Trust paths enable transitive trust calculation');
    console.log('   • Trust scores aggregate multiple relationships');
    console.log('   • Trust validation enables secure operations');
    console.log('   • Federation analytics provide network insights');

  } catch (error) {
    console.error('❌ Error during trust network example:');
    
    if (ErrorUtils.isErrorType(error, ICNTrustError)) {
      console.error('🤝 Trust Error:', error.message);
      console.error('💡 Tip: Check trust relationship parameters and permissions');
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
  trustNetworkExample().catch(console.error);
}

export { trustNetworkExample };