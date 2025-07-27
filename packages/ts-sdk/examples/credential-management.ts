/**
 * Enhanced Credential Management Example
 * 
 * This example demonstrates:
 * - Credential issuance with rich metadata
 * - Credential presentations with selective disclosure
 * - Credential verification workflows
 * - Credential revocation and status tracking
 * - Trust attestations and anchoring
 */

import { 
  ICNClient, 
  createStorage, 
  ICNCredentialError,
  ICNValidationError,
  ErrorUtils 
} from '@icn/ts-sdk';

async function credentialManagementExample() {
  console.log('🆔 Starting Enhanced Credential Management Example\n');

  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080',
    network: 'devnet',
    storage: createStorage('@credential-example:'),
  });

  try {
    await client.connect();
    console.log('✅ Connected to ICN node\n');

    // Example participants
    const universityDid = 'did:key:university123';
    const employerDid = 'did:key:employer456';
    const studentDid = 'did:key:student789';
    const verifierDid = 'did:key:verifier012';

    // 1. Issue Academic Credential
    console.log('🎓 Issuing academic credential...');
    
    const academicCredential = await client.credentials.issueCredential({
      credential_type: 'academic_degree',
      holder: studentDid,
      issuer: universityDid,
      claims: {
        degree_type: 'Bachelor of Science',
        major: 'Computer Science',
        gpa: '3.85',
        graduation_date: '2024-05-15',
        honors: 'Magna Cum Laude',
        thesis_title: 'Distributed Systems in Blockchain Networks',
        specializations: ['Cryptography', 'Distributed Systems', 'AI'],
        total_credits: '128',
        university_name: 'ICN University',
        accreditation: 'AACSB'
      },
      evidence: [
        'https://university.edu/transcripts/student789',
        'https://university.edu/thesis/distributed-systems-blockchain',
        'https://university.edu/verification/gpa-3.85'
      ],
      validity_period: 10 * 365 * 24 * 3600 // 10 years
    });
    
    console.log('✅ Academic credential issued successfully');
    console.log(`   Credential CID: ${academicCredential.credential_cid}`);
    console.log(`   Valid until: ${new Date(academicCredential.valid_until! * 1000).toLocaleDateString()}`);
    console.log();

    // 2. Issue Professional Skill Credential
    console.log('💼 Issuing professional skill credential...');
    
    const skillCredential = await client.credentials.issueCredential({
      credential_type: 'professional_skill',
      holder: studentDid,
      issuer: employerDid,
      claims: {
        skill_name: 'Full-Stack Development',
        proficiency_level: 'Expert',
        technologies: ['React', 'TypeScript', 'Node.js', 'PostgreSQL', 'Docker'],
        years_experience: '3',
        project_count: '15',
        certification_date: '2024-06-01',
        performance_rating: '4.8/5.0',
        mentor_feedback: 'Exceptional technical skills and team collaboration',
        remote_work_capable: 'true'
      },
      evidence: [
        'https://employer.com/portfolio/student789',
        'https://github.com/student789/projects',
        'https://employer.com/performance-reviews/2024'
      ],
      validity_period: 2 * 365 * 24 * 3600 // 2 years
    });
    
    console.log('✅ Professional skill credential issued successfully');
    console.log(`   Credential CID: ${skillCredential.credential_cid}`);
    console.log();

    // 3. List Credentials for Student
    console.log('📋 Listing student credentials...');
    
    const studentCredentials = await client.credentials.listCredentials({
      holder: studentDid,
      status: 'active'
    });
    
    console.log(`🎯 Found ${studentCredentials.credentials.length} active credentials for student:`);
    studentCredentials.credentials.forEach((cred, index) => {
      console.log(`   ${index + 1}. ${cred.credential_type} (${cred.cid.slice(0, 12)}...)`);
      console.log(`      Issuer: ${cred.issuer}`);
      console.log(`      Issued: ${new Date(cred.issued_at * 1000).toLocaleDateString()}`);
      if (cred.valid_until) {
        console.log(`      Expires: ${new Date(cred.valid_until * 1000).toLocaleDateString()}`);
      }
      console.log(`      Presentations: ${cred.presentation_count}`);
    });
    console.log();

    // 4. Present Academic Credential for Job Application
    console.log('💼 Presenting academic credential for job application...');
    
    const academicPresentation = await client.credentials.presentCredential({
      credential_proof: academicCredential.credential_proof,
      context: 'job_application_tech_company',
      disclosed_fields: [
        'degree_type',
        'major', 
        'gpa',
        'graduation_date',
        'honors',
        'specializations',
        'university_name'
      ],
      challenge: 'job_app_challenge_2024_' + Date.now()
    });
    
    console.log('✅ Academic credential presented successfully');
    console.log(`   Presentation ID: ${academicPresentation.presentation_id}`);
    console.log(`   Verification Status: ${academicPresentation.verification_result.valid ? 'Valid' : 'Invalid'}`);
    console.log(`   Trust Score: ${academicPresentation.verification_result.trust_score?.toFixed(2) || 'N/A'}`);
    console.log();

    // 5. Verify the Presentation
    console.log('🔍 Verifying credential presentation...');
    
    const verificationResult = await client.credentials.verifyCredential({
      presentation_id: academicPresentation.presentation_id,
      verification_level: 'enhanced',
      required_claims: ['degree_type', 'major', 'gpa']
    });
    
    console.log('📊 Verification Results:');
    console.log(`   Valid: ${verificationResult.valid ? '✅ Yes' : '❌ No'}`);
    console.log(`   Verification Level: ${verificationResult.verification_level}`);
    console.log(`   Trust Score: ${verificationResult.trust_score?.toFixed(2) || 'N/A'}`);
    
    console.log('   Verified Claims:');
    Object.entries(verificationResult.verified_claims).forEach(([claim, value]) => {
      console.log(`      ${claim}: ${value}`);
    });
    
    if (verificationResult.warnings.length > 0) {
      console.log('   ⚠️  Warnings:');
      verificationResult.warnings.forEach(warning => {
        console.log(`      • ${warning}`);
      });
    }
    
    if (verificationResult.errors.length > 0) {
      console.log('   ❌ Errors:');
      verificationResult.errors.forEach(error => {
        console.log(`      • ${error}`);
      });
    }
    console.log();

    // 6. Anchor Disclosure to DAG
    console.log('⚓ Anchoring credential disclosure to DAG...');
    
    const anchoredDisclosure = await client.credentials.anchorDisclosure({
      credential_cid: academicCredential.credential_cid,
      disclosed_fields: ['degree_type', 'major', 'graduation_date'],
      presentation_context: 'public_verification_portal',
      verifier: verifierDid,
      metadata: {
        verification_purpose: 'employment_background_check',
        requester_organization: 'Tech Startup Inc',
        verification_date: new Date().toISOString(),
        compliance_standard: 'SOC2_Type2'
      }
    });
    
    console.log('✅ Disclosure anchored to DAG successfully');
    console.log(`   Disclosure CID: ${anchoredDisclosure.disclosure_cid}`);
    console.log(`   DAG Block CID: ${anchoredDisclosure.dag_block_cid}`);
    console.log(`   Anchored at: ${new Date(anchoredDisclosure.anchored_at * 1000).toLocaleString()}`);
    console.log();

    // 7. Get Comprehensive Credential Status
    console.log('📈 Checking comprehensive credential status...');
    
    const credentialStatus = await client.credentials.getCredentialStatus(
      academicCredential.credential_cid
    );
    
    console.log('📊 Academic Credential Status:');
    console.log(`   CID: ${credentialStatus.cid}`);
    console.log(`   Type: ${credentialStatus.credential_type}`);
    console.log(`   Status: ${credentialStatus.status}`);
    console.log(`   Revoked: ${credentialStatus.revoked ? '❌ Yes' : '✅ No'}`);
    console.log(`   Presentations: ${credentialStatus.presentations.length}`);
    console.log(`   Anchored Disclosures: ${credentialStatus.anchored_disclosures.length}`);
    console.log(`   Trust Attestations: ${credentialStatus.trust_attestations.length}`);
    
    if (credentialStatus.presentations.length > 0) {
      console.log('   Recent Presentations:');
      credentialStatus.presentations.slice(0, 3).forEach((presentation, index) => {
        console.log(`      ${index + 1}. ${presentation.presentation_id.slice(0, 12)}...`);
        console.log(`         Context: ${presentation.context}`);
        console.log(`         Date: ${new Date(presentation.presented_at * 1000).toLocaleDateString()}`);
        if (presentation.verification_result) {
          console.log(`         Valid: ${presentation.verification_result.valid ? 'Yes' : 'No'}`);
        }
      });
    }
    console.log();

    // 8. Present Skill Credential with Selective Disclosure
    console.log('🛡️  Presenting skill credential with selective disclosure...');
    
    const skillPresentation = await client.credentials.presentCredential({
      credential_proof: skillCredential.credential_proof,
      context: 'freelance_project_application',
      disclosed_fields: [
        'skill_name',
        'proficiency_level',
        'technologies',
        'years_experience',
        'performance_rating'
      ], // Not disclosing sensitive info like specific project count or mentor feedback
      challenge: 'freelance_challenge_' + Date.now()
    });
    
    console.log('✅ Skill credential presented with selective disclosure');
    console.log(`   Presentation ID: ${skillPresentation.presentation_id}`);
    console.log('   Disclosed only: skill level, technologies, experience, rating');
    console.log('   Protected: project count, mentor feedback, personal details');
    console.log();

    // 9. Credential Lifecycle Management
    console.log('🔄 Demonstrating credential lifecycle management...');
    
    // Update credential status example (would be done by issuer)
    console.log('📝 Checking if credential needs renewal...');
    
    const now = Date.now() / 1000;
    const skillExpirationTime = skillCredential.valid_until!;
    const timeUntilExpiration = skillExpirationTime - now;
    const daysUntilExpiration = timeUntilExpiration / (24 * 3600);
    
    console.log(`⏰ Skill credential expires in ${daysUntilExpiration.toFixed(0)} days`);
    
    if (daysUntilExpiration < 90) { // Less than 3 months
      console.log('⚠️  Credential renewal recommended');
    } else {
      console.log('✅ Credential is current');
    }
    console.log();

    // 10. Advanced Query Operations
    console.log('🔍 Advanced credential queries...');
    
    // Query credentials by type
    const allSkillCredentials = await client.credentials.listCredentials({
      credential_type: 'professional_skill',
      limit: 10
    });
    
    console.log(`🎯 Found ${allSkillCredentials.total_count} skill credentials in system`);
    console.log(`   Showing ${allSkillCredentials.credentials.length} results`);
    console.log(`   Has more: ${allSkillCredentials.has_more ? 'Yes' : 'No'}`);
    
    // Query credentials by issuer
    const universityCredentials = await client.credentials.listCredentials({
      issuer: universityDid,
      status: 'active'
    });
    
    console.log(`🏫 University has issued ${universityCredentials.total_count} active credentials`);
    
    // Demonstrate revocation (commented to preserve example data)
    console.log('\n🚫 Credential revocation (simulated)...');
    console.log('   Note: In a real scenario, revocation would be performed by the issuer');
    console.log('   Reasons for revocation might include:');
    console.log('     • Fraudulent information discovered');
    console.log('     • Credential holder violation');
    console.log('     • Institutional policy change');
    console.log('     • Security compromise');
    
    // Example revocation call (commented):
    // const revocationResult = await client.credentials.revokeCredential({
    //   credential_cid: skillCredential.credential_cid,
    //   reason: 'No longer employed - credential invalid',
    //   revoked_by: employerDid
    // });

    console.log('\n🎉 Enhanced Credential Management example completed successfully!');
    console.log('\n💡 Key Features Demonstrated:');
    console.log('   • Rich credential issuance with comprehensive metadata');
    console.log('   • Selective disclosure for privacy protection');
    console.log('   • Enhanced verification with trust scoring');
    console.log('   • DAG anchoring for immutable records');
    console.log('   • Comprehensive status tracking');
    console.log('   • Credential lifecycle management');
    console.log('   • Advanced querying capabilities');
    
    console.log('\n🔒 Privacy & Security Benefits:');
    console.log('   • Zero-knowledge proofs protect sensitive data');
    console.log('   • Selective disclosure minimizes data exposure');
    console.log('   • Immutable anchoring prevents tampering');
    console.log('   • Trust scoring validates credential reliability');
    console.log('   • Revocation system maintains integrity');

  } catch (error) {
    console.error('❌ Error during credential management example:');
    
    if (ErrorUtils.isErrorType(error, ICNCredentialError)) {
      console.error('🆔 Credential Error:', error.message);
      console.error('💡 Tip: Check credential parameters and issuer permissions');
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
  credentialManagementExample().catch(console.error);
}

export { credentialManagementExample };