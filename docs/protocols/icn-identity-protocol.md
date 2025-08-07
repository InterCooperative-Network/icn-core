# InterCooperative Network Identity & Credential Protocol
## Definitive Specification

---

## Executive Summary

The ICN Identity & Credential Protocol establishes a **self-sovereign identity system** where individuals and organizations control their own identities without reliance on centralized authorities. Built on W3C Decentralized Identifiers (DIDs) and Verifiable Credentials (VCs), the protocol enables **privacy-preserving verification** through zero-knowledge proofs while maintaining the democratic principle that identity and membership cannot be bought or sold.

Every identity operation—from DID creation to credential issuance to revocation—creates an immutable DAG record, ensuring accountability while preserving privacy through selective disclosure and encryption. The system supports recovery mechanisms, delegation, and multi-party control while preventing Sybil attacks through mana-based rate limiting and proof-of-personhood verification.

---

## 0 · Scope and Implementation Alignment (Normative)

### 0.1 Entities
- DID and DIDDocument (W3C-aligned with ICN extensions)
- VerifiableCredential (basic issue/verify)
- Delegated/Capability credentials (experimental)

### 0.2 Operations
- Register DID, resolve DID
- Issue credential, verify credential
- Optional: basic revocation via credential status field (global registries deferred)

### 0.3 Security
- DID-signed HTTP requests (node APIs)
- Message signatures for P2P
- ZK verification: experimental; non-normative in v1

### 0.4 Pending Extensions
- Soul-bound membership credential schema finalization
- ZK selective disclosure flows as defaults
- Social recovery workflows and DID key custody policies
- Network-wide revocation registries and proofs of non-revocation

### 0.5 Mappings
- Crates: `icn-identity`, `icn-api::identity_trait`, `icn-protocol`
- Auth: See ICN Core API Contracts Guide for DID auth header format

---

## 1. Core Design Principles

### 1.1 Self-Sovereignty
- Individuals control their own identity
- No central authority can revoke core identity
- Portable across organizations and federations

### 1.2 Privacy by Design
- Minimal disclosure by default
- Zero-knowledge proofs for sensitive attributes
- Selective revelation of credentials

### 1.3 Sybil Resistance
- DID creation requires mana burn
- Proof-of-personhood for human identities
- Rate limiting and pattern detection

### 1.4 Recovery & Resilience
- Social recovery mechanisms
- Key rotation without identity loss
- Disaster recovery procedures

---

## 2. Decentralized Identifier (DID) System

### 2.1 DID Structure

```rust
pub struct DID {
    // DID format: did:icn:<type>:<unique-identifier>
    method: String,                    // Always "icn"
    identifier_type: IdentifierType,   // Type of entity
    unique_id: String,                  // Unique identifier
    
    // Full DID string
    uri: String,                        // e.g., "did:icn:person:alice123"
}

pub enum IdentifierType {
    Person,                             // Individual human
    Organization,                       // Cooperative, community, etc.
    Device,                            // IoT device, phone, server
    Service,                           // API endpoint, application
    Ephemeral,                         // Temporary identity
}

pub struct DIDDocument {
    // Core fields (W3C compliant)
    id: DID,
    controller: Vec<DID>,              // Who controls this DID
    
    // Cryptographic material
    verification_method: Vec<VerificationMethod>,
    authentication: Vec<String>,       // References to verification methods
    assertion_method: Vec<String>,
    key_agreement: Vec<String>,
    capability_invocation: Vec<String>,
    capability_delegation: Vec<String>,
    
    // Service endpoints
    service: Vec<ServiceEndpoint>,
    
    // ICN-specific extensions
    icn_metadata: ICNMetadata,
    
    // Versioning
    version: u64,
    created: Timestamp,
    updated: Timestamp,
    
    // Proof
    proof: DocumentProof,
}

pub struct VerificationMethod {
    id: String,                        // e.g., "did:icn:person:alice#key1"
    method_type: VerificationMethodType,
    controller: DID,
    public_key: PublicKeyMaterial,
    
    // Key metadata
    created: Timestamp,
    expires: Option<Timestamp>,
    revoked: Option<RevocationInfo>,
}

pub enum VerificationMethodType {
    Ed25519VerificationKey2020,
    EcdsaSecp256k1VerificationKey2019,
    RsaVerificationKey2018,
    X25519KeyAgreementKey2020,
    BLS12381G2Key2020,              // For aggregated signatures
}

pub struct ICNMetadata {
    // Organization membership
    organizations: Vec<OrganizationId>,
    primary_org: Option<OrganizationId>,
    
    // Network participation
    node_type: Option<NodeClass>,
    compute_score: Option<ComputeScore>,
    
    // Trust and reputation
    trust_score: f64,
    reputation_cid: Option<CID>,
    
    // Recovery information
    recovery_contacts: Vec<RecoveryContact>,
    recovery_threshold: u32,
}
```

### 2.2 DID Lifecycle

```rust
pub struct DIDLifecycle {
    // DID Creation
    pub fn create_did(
        identity_type: IdentifierType,
        initial_keys: Vec<KeyPair>,
        metadata: ICNMetadata
    ) -> Result<(DID, DIDDocument)> {
        // 1. Generate unique identifier
        let unique_id = generate_unique_id()?;
        let did = DID {
            method: "icn".to_string(),
            identifier_type: identity_type.clone(),
            unique_id: unique_id.clone(),
            uri: format!("did:icn:{}:{}", identity_type.to_string(), unique_id),
        };
        
        // 2. Charge mana for Sybil resistance
        let creation_cost = calculate_did_creation_cost(&identity_type);
        burn_mana(&msg.sender, creation_cost)?;
        
        // 3. Verify proof-of-personhood if human
        if identity_type == IdentifierType::Person {
            require(verify_proof_of_personhood(&msg.sender)?);
        }
        
        // 4. Create verification methods
        let verification_methods = initial_keys.iter()
            .enumerate()
            .map(|(i, key)| create_verification_method(&did, &key, i))
            .collect();
        
        // 5. Build DID document
        let did_document = DIDDocument {
            id: did.clone(),
            controller: vec![did.clone()],
            verification_method: verification_methods,
            authentication: vec![format!("{}#key-1", did.uri)],
            assertion_method: vec![format!("{}#key-1", did.uri)],
            key_agreement: vec![format!("{}#key-2", did.uri)],
            capability_invocation: vec![format!("{}#key-1", did.uri)],
            capability_delegation: vec![],
            service: vec![],
            icn_metadata: metadata,
            version: 1,
            created: now(),
            updated: now(),
            proof: generate_document_proof(&did_document, &initial_keys[0])?,
        };
        
        // 6. Anchor in DAG
        let doc_cid = put_dag(&did_document)?;
        register_did(&did, &doc_cid)?;
        
        emit DIDCreated(did.clone(), doc_cid);
        Ok((did, did_document))
    }
    
    // Key Rotation
    pub fn rotate_keys(
        did: &DID,
        old_key: &KeyPair,
        new_keys: Vec<KeyPair>
    ) -> Result<()> {
        // 1. Verify ownership
        let doc = resolve_did(did)?;
        require(verify_key_ownership(&doc, old_key)?);
        
        // 2. Create new document version
        let mut new_doc = doc.clone();
        new_doc.version += 1;
        new_doc.updated = now();
        
        // 3. Revoke old keys
        for method in &mut new_doc.verification_method {
            if method.public_key == old_key.public_key() {
                method.revoked = Some(RevocationInfo {
                    reason: RevocationReason::KeyRotation,
                    timestamp: now(),
                    replacement: Some(format!("{}#key-{}", did.uri, new_doc.version)),
                });
            }
        }
        
        // 4. Add new keys
        for (i, key) in new_keys.iter().enumerate() {
            new_doc.verification_method.push(
                create_verification_method(did, key, new_doc.version as usize + i)
            );
        }
        
        // 5. Update references
        new_doc.authentication = vec![format!("{}#key-{}", did.uri, new_doc.version)];
        // ... update other method references
        
        // 6. Sign with old key (proves continuity)
        new_doc.proof = generate_document_proof(&new_doc, old_key)?;
        
        // 7. Anchor in DAG
        let new_cid = put_dag(&new_doc)?;
        update_did_registry(did, &new_cid)?;
        
        emit KeyRotation(did.clone(), new_cid);
        Ok(())
    }
    
    // DID Recovery
    pub fn recover_did(
        did: &DID,
        recovery_signatures: Vec<RecoverySignature>,
        new_keys: Vec<KeyPair>
    ) -> Result<()> {
        // 1. Get recovery configuration
        let doc = resolve_did(did)?;
        let recovery_config = &doc.icn_metadata.recovery_contacts;
        let threshold = doc.icn_metadata.recovery_threshold;
        
        // 2. Verify recovery signatures
        let valid_signatures = recovery_signatures.iter()
            .filter(|sig| verify_recovery_signature(sig, recovery_config).is_ok())
            .count();
        
        require(valid_signatures >= threshold as usize);
        
        // 3. Create recovery document
        let mut recovery_doc = doc.clone();
        recovery_doc.version += 1;
        recovery_doc.updated = now();
        
        // 4. Revoke all old keys
        for method in &mut recovery_doc.verification_method {
            method.revoked = Some(RevocationInfo {
                reason: RevocationReason::Recovery,
                timestamp: now(),
                replacement: None,
            });
        }
        
        // 5. Add new keys
        recovery_doc.verification_method.clear();
        for (i, key) in new_keys.iter().enumerate() {
            recovery_doc.verification_method.push(
                create_verification_method(did, key, i)
            );
        }
        
        // 6. Create recovery proof
        recovery_doc.proof = generate_recovery_proof(&recovery_doc, &recovery_signatures)?;
        
        // 7. Anchor with recovery flag
        let recovery_cid = put_dag(&recovery_doc)?;
        update_did_registry_recovery(did, &recovery_cid)?;
        
        emit DIDRecovered(did.clone(), recovery_cid);
        Ok(())
    }
}
```

### 2.3 DID Resolution

```rust
pub struct DIDResolver {
    cache: LRUCache<DID, DIDDocument>,
    
    pub fn resolve(&self, did: &DID) -> Result<DIDDocument> {
        // 1. Check cache
        if let Some(cached) = self.cache.get(did) {
            if !is_expired(&cached) {
                return Ok(cached.clone());
            }
        }
        
        // 2. Query registry
        let doc_cid = get_did_registry_entry(did)?;
        
        // 3. Fetch from DAG
        let document = get_dag::<DIDDocument>(&doc_cid)?;
        
        // 4. Verify document integrity
        verify_document_proof(&document)?;
        
        // 5. Check for updates
        if let Some(update_cid) = check_for_updates(&document) {
            let updated = get_dag::<DIDDocument>(&update_cid)?;
            verify_document_update(&document, &updated)?;
            self.cache.insert(did.clone(), updated.clone());
            return Ok(updated);
        }
        
        // 6. Cache and return
        self.cache.insert(did.clone(), document.clone());
        Ok(document)
    }
    
    pub fn resolve_key(&self, did: &DID, key_id: &str) -> Result<PublicKey> {
        let doc = self.resolve(did)?;
        
        for method in &doc.verification_method {
            if method.id == key_id {
                if method.revoked.is_some() {
                    return Err(Error::KeyRevoked);
                }
                return Ok(method.public_key.clone());
            }
        }
        
        Err(Error::KeyNotFound)
    }
}
```

---

## 3. Verifiable Credentials

### 3.1 Credential Structure

```rust
pub struct VerifiableCredential {
    // Standard W3C fields
    context: Vec<String>,              // JSON-LD contexts
    id: Option<String>,                // Unique credential ID
    credential_type: Vec<String>,      // Types of credential
    
    // Parties
    issuer: DID,                       // Who issued this
    holder: DID,                       // Who holds this
    
    // Claims
    credential_subject: CredentialSubject,
    
    // Validity
    issuance_date: Timestamp,
    expiration_date: Option<Timestamp>,
    
    // Status
    credential_status: Option<CredentialStatus>,
    
    // Evidence
    evidence: Vec<Evidence>,
    
    // Proof
    proof: CredentialProof,
    
    // ICN extensions
    icn_metadata: CredentialMetadata,
}

pub struct CredentialSubject {
    id: DID,                           // Subject of the credential
    claims: HashMap<String, Claim>,    // Actual claims
}

pub enum Claim {
    // Simple claims
    Boolean(bool),
    Number(f64),
    String(String),
    
    // Complex claims
    Date(Timestamp),
    Duration(Duration),
    Amount(u64, String),               // Value and unit
    
    // Composite claims
    Object(HashMap<String, Claim>),
    Array(Vec<Claim>),
    
    // Privacy-preserving claims
    Hash(Hash),                        // Hashed value
    Commitment(Commitment),            // Cryptographic commitment
    Range(RangeProof),                // Prove value in range without revealing
}

pub struct CredentialMetadata {
    // Credential classification
    privacy_level: PrivacyLevel,
    transferable: bool,               // Most are non-transferable
    
    // Usage restrictions
    usage_limit: Option<u32>,         // Max number of uses
    usage_count: u32,                 // Current usage count
    valid_contexts: Vec<OrganizationId>, // Where it's valid
    
    // Issuance metadata
    issuance_cost: Option<Mana>,      // Cost to issue
    verification_cost: Option<Mana>,   // Cost to verify
    
    // Governance
    revocable_by: Vec<DID>,          // Who can revoke
    disputes_allowed: bool,           // Can be disputed
}

pub enum CredentialType {
    // Identity credentials
    PersonalIdentity,
    OrganizationalIdentity,
    
    // Membership credentials
    CooperativeMember,
    CommunityMember,
    FederationDelegate,
    
    // Capability credentials
    ComputeProvider,
    StorageProvider,
    ValidatorNode,
    
    // Qualification credentials
    SkillCertificate,
    TrainingCompletion,
    ExperienceRecord,
    
    // Economic credentials
    CreditScore,
    PaymentCapability,
    ResourceOwnership,
    
    // Governance credentials
    VotingEligibility,
    ProposalRight,
    VetoAuthority,
}
```

### 3.2 Credential Issuance

```rust
pub struct CredentialIssuance {
    pub fn issue_credential(
        issuer: DID,
        holder: DID,
        credential_type: CredentialType,
        claims: HashMap<String, Claim>,
        evidence: Vec<Evidence>
    ) -> Result<VerifiableCredential> {
        // 1. Verify issuer authority
        require(can_issue_credential(&issuer, &credential_type)?);
        
        // 2. Verify holder eligibility
        require(is_eligible_for_credential(&holder, &credential_type)?);
        
        // 3. Validate claims against schema
        validate_claims(&claims, &credential_type)?;
        
        // 4. Check evidence if required
        if requires_evidence(&credential_type) {
            verify_evidence(&evidence, &claims)?;
        }
        
        // 5. Create credential
        let credential = VerifiableCredential {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://icn.coop/credentials/v1".to_string(),
            ],
            id: Some(generate_credential_id()),
            credential_type: vec![
                "VerifiableCredential".to_string(),
                credential_type.to_string(),
            ],
            issuer: issuer.clone(),
            holder: holder.clone(),
            credential_subject: CredentialSubject {
                id: holder.clone(),
                claims,
            },
            issuance_date: now(),
            expiration_date: calculate_expiration(&credential_type),
            credential_status: Some(CredentialStatus {
                id: generate_status_id(),
                status_type: StatusType::RevocationList2020,
                revocation_list_index: assign_revocation_index()?,
                revocation_list_credential: get_revocation_list_id(&issuer),
            }),
            evidence,
            proof: generate_credential_proof(&credential, &issuer)?,
            icn_metadata: CredentialMetadata {
                privacy_level: determine_privacy_level(&credential_type),
                transferable: false,  // Non-transferable by default
                usage_limit: None,
                usage_count: 0,
                valid_contexts: vec![],
                issuance_cost: calculate_issuance_cost(&credential_type),
                verification_cost: Some(1),  // 1 mana per verification
                revocable_by: vec![issuer.clone()],
                disputes_allowed: true,
            },
        };
        
        // 6. Charge issuance cost
        if let Some(cost) = credential.icn_metadata.issuance_cost {
            charge_mana(&holder, cost)?;
        }
        
        // 7. Anchor in DAG
        let credential_cid = put_dag(&credential)?;
        
        // 8. Update credential registry
        register_credential(&credential.id.unwrap(), &credential_cid)?;
        
        emit CredentialIssued(holder, issuer, credential_type, credential_cid);
        Ok(credential)
    }
    
    pub fn issue_membership_credential(
        org: OrganizationId,
        member: DID,
        membership_type: MembershipType
    ) -> Result<VerifiableCredential> {
        // Special handling for membership credentials
        let issuer = get_org_did(&org)?;
        
        // Membership must be approved by governance
        require(has_membership_approval(&org, &member)?);
        
        let claims = hashmap! {
            "organization".to_string() => Claim::String(org.to_string()),
            "membershipType".to_string() => Claim::String(membership_type.to_string()),
            "votingRights".to_string() => Claim::Boolean(true),
            "joinedDate".to_string() => Claim::Date(now()),
        };
        
        let credential = issue_credential(
            issuer,
            member.clone(),
            CredentialType::CooperativeMember,
            claims,
            vec![]
        )?;
        
        // Register member in organization
        add_member_to_org(&org, &member, &credential.id.unwrap())?;
        
        Ok(credential)
    }
}
```

### 3.3 Credential Verification

```rust
pub struct CredentialVerification {
    pub fn verify_credential(
        credential: &VerifiableCredential,
        options: VerificationOptions
    ) -> Result<VerificationResult> {
        // 1. Check expiration
        if let Some(expiry) = credential.expiration_date {
            if now() > expiry {
                return Ok(VerificationResult::Expired);
            }
        }
        
        // 2. Verify issuer signature
        let issuer_doc = resolve_did(&credential.issuer)?;
        if !verify_credential_proof(&credential.proof, &issuer_doc)? {
            return Ok(VerificationResult::InvalidSignature);
        }
        
        // 3. Check revocation status
        if let Some(status) = &credential.credential_status {
            if is_revoked(status)? {
                return Ok(VerificationResult::Revoked);
            }
        }
        
        // 4. Verify holder (if required)
        if options.verify_holder {
            let holder_doc = resolve_did(&credential.holder)?;
            if !verify_holder(&credential, &holder_doc)? {
                return Ok(VerificationResult::InvalidHolder);
            }
        }
        
        // 5. Check usage limits
        if let Some(limit) = credential.icn_metadata.usage_limit {
            if credential.icn_metadata.usage_count >= limit {
                return Ok(VerificationResult::UsageLimitExceeded);
            }
        }
        
        // 6. Verify claims (if schema provided)
        if let Some(schema) = options.schema {
            if !validate_against_schema(&credential.credential_subject.claims, &schema)? {
                return Ok(VerificationResult::SchemaViolation);
            }
        }
        
        // 7. Charge verification cost
        if let Some(cost) = credential.icn_metadata.verification_cost {
            charge_mana(&msg.sender, cost)?;
        }
        
        // 8. Update usage count
        increment_usage_count(&credential.id.unwrap())?;
        
        Ok(VerificationResult::Valid)
    }
    
    pub fn verify_membership(
        did: &DID,
        org: &OrganizationId
    ) -> Result<bool> {
        // Find membership credential
        let credentials = get_credentials_for_holder(did)?;
        
        for cred_id in credentials {
            let credential = get_credential(&cred_id)?;
            
            // Check if it's a membership credential for this org
            if credential.credential_type.contains(&"CooperativeMember".to_string()) ||
               credential.credential_type.contains(&"CommunityMember".to_string()) {
                
                if let Some(Claim::String(org_claim)) = 
                    credential.credential_subject.claims.get("organization") {
                    
                    if org_claim == &org.to_string() {
                        // Verify the credential
                        let result = verify_credential(&credential, VerificationOptions::default())?;
                        return Ok(result == VerificationResult::Valid);
                    }
                }
            }
        }
        
        Ok(false)
    }
}
```

---

## 4. Zero-Knowledge Proofs

### 4.1 ZK Credential System

```rust
pub struct ZKCredential {
    // Public parameters
    public_key: ZKPublicKey,
    credential_commitment: Commitment,
    
    // Hidden attributes
    hidden_claims: Vec<HiddenClaim>,
    
    // Selective disclosure
    disclosed_claims: HashMap<String, Claim>,
    
    // Proof system
    proof_system: ProofSystem,
}

pub enum ProofSystem {
    // BBS+ signatures for selective disclosure
    BBSPlus {
        params: BBSPlusParams,
        signature: BBSPlusSignature,
    },
    
    // CL signatures (Camenisch-Lysyanskaya)
    CLSignature {
        params: CLParams,
        signature: CLSig,
    },
    
    // zk-SNARKs for complex proofs
    Groth16 {
        proving_key: ProvingKey,
        verification_key: VerifyingKey,
    },
    
    // Bulletproofs for range proofs
    Bulletproofs {
        generators: Generators,
        pedersen_commitment: PedersenCommitment,
    },
}

pub struct ZKProofGeneration {
    pub fn generate_zk_credential_proof(
        credential: &VerifiableCredential,
        attributes_to_reveal: Vec<String>,
        predicates: Vec<Predicate>
    ) -> Result<ZKProof> {
        // 1. Create commitment to all attributes
        let commitment = commit_to_attributes(&credential.credential_subject.claims)?;
        
        // 2. Generate selective disclosure proof
        let disclosure_proof = match determine_proof_system(&credential) {
            ProofSystem::BBSPlus { .. } => {
                generate_bbs_plus_proof(
                    &credential,
                    &attributes_to_reveal,
                    &predicates
                )?
            },
            ProofSystem::Bulletproofs { .. } => {
                generate_bulletproof(
                    &credential,
                    &predicates
                )?
            },
            _ => return Err(Error::UnsupportedProofSystem),
        };
        
        // 3. Create proof of knowledge
        let pok = prove_knowledge_of_credential(
            &credential,
            &commitment,
            &disclosure_proof
        )?;
        
        Ok(ZKProof {
            commitment,
            disclosed_attributes: attributes_to_reveal,
            predicates,
            disclosure_proof,
            proof_of_knowledge: pok,
            timestamp: now(),
        })
    }
    
    pub fn verify_zk_proof(
        proof: &ZKProof,
        issuer_public_key: &PublicKey
    ) -> Result<bool> {
        // 1. Verify commitment
        if !verify_commitment(&proof.commitment) {
            return Ok(false);
        }
        
        // 2. Verify selective disclosure
        if !verify_disclosure_proof(
            &proof.disclosure_proof,
            &proof.disclosed_attributes,
            issuer_public_key
        )? {
            return Ok(false);
        }
        
        // 3. Verify predicates
        for predicate in &proof.predicates {
            if !verify_predicate(predicate, &proof.disclosure_proof)? {
                return Ok(false);
            }
        }
        
        // 4. Verify proof of knowledge
        if !verify_proof_of_knowledge(&proof.proof_of_knowledge)? {
            return Ok(false);
        }
        
        Ok(true)
    }
}

pub enum Predicate {
    // Range proofs
    GreaterThan { attribute: String, value: u64 },
    LessThan { attribute: String, value: u64 },
    InRange { attribute: String, min: u64, max: u64 },
    
    // Set membership
    InSet { attribute: String, set: HashSet<String> },
    NotInSet { attribute: String, set: HashSet<String> },
    
    // Boolean
    IsTrue { attribute: String },
    IsFalse { attribute: String },
    
    // Equality (without revealing value)
    Equals { attribute: String, commitment: Commitment },
    NotEquals { attribute: String, commitment: Commitment },
}
```

### 4.2 Privacy-Preserving Verification

```rust
pub struct PrivacyPreservingVerification {
    pub fn verify_age_over_18(credential: &VerifiableCredential) -> Result<bool> {
        // Generate range proof without revealing actual age
        let age_claim = credential.credential_subject.claims.get("birthdate")
            .ok_or(Error::ClaimNotFound)?;
        
        let age = calculate_age(age_claim)?;
        
        // Create range proof
        let proof = generate_range_proof(age, 18, 150)?;  // Between 18 and 150
        
        // Verify proof
        Ok(verify_range_proof(&proof, 18, 150)?)
    }
    
    pub fn verify_membership_without_revealing_identity(
        org: &OrganizationId
    ) -> Result<bool> {
        // Prove membership using ring signature
        let members = get_org_members(org)?;
        
        // Create ring of possible signers
        let ring = members.iter()
            .map(|m| get_member_public_key(m))
            .collect::<Result<Vec<_>>>()?;
        
        // Generate ring signature
        let signature = generate_ring_signature(
            &msg.sender_private_key,
            &ring,
            &org.to_bytes()
        )?;
        
        // Verify signature
        Ok(verify_ring_signature(&signature, &ring, &org.to_bytes())?)
    }
    
    pub fn verify_compute_capacity_threshold(
        min_compute: ComputeScore
    ) -> Result<bool> {
        // Prove compute capacity above threshold without revealing exact capacity
        let actual_compute = get_node_compute_score(&msg.sender)?;
        
        // Generate comparison proof
        let proof = generate_comparison_proof(
            &actual_compute,
            &min_compute,
            ComparisonType::GreaterThanOrEqual
        )?;
        
        Ok(verify_comparison_proof(&proof, &min_compute)?)
    }
}
```

---

## 5. Revocation System

### 5.1 Revocation Mechanisms

```rust
pub struct RevocationSystem {
    // Revocation registry for each issuer
    registries: HashMap<DID, RevocationRegistry>,
}

pub struct RevocationRegistry {
    issuer: DID,
    registry_type: RevocationType,
    
    // Revocation data
    revoked_credentials: HashSet<CredentialId>,
    revocation_list: MerkleTree,
    accumulator: Option<Accumulator>,
    
    // Metadata
    last_updated: Timestamp,
    update_frequency: Duration,
}

pub enum RevocationType {
    // Simple revocation list
    RevocationList2020,
    
    // Merkle tree based
    MerkleRevocation,
    
    // Cryptographic accumulator
    AccumulatorBased,
    
    // Privacy-preserving
    PrivacyPreserving,
}

impl RevocationSystem {
    pub fn revoke_credential(
        &mut self,
        credential_id: &CredentialId,
        reason: RevocationReason
    ) -> Result<()> {
        // 1. Get credential
        let credential = get_credential(credential_id)?;
        
        // 2. Verify revocation authority
        require(can_revoke(&msg.sender, &credential)?);
        
        // 3. Add to revocation registry
        let registry = self.registries.get_mut(&credential.issuer)
            .ok_or(Error::RegistryNotFound)?;
        
        registry.revoked_credentials.insert(credential_id.clone());
        
        // 4. Update revocation proof structures
        match registry.registry_type {
            RevocationType::MerkleRevocation => {
                registry.revocation_list.insert(credential_id.to_bytes());
            },
            RevocationType::AccumulatorBased => {
                if let Some(acc) = &mut registry.accumulator {
                    acc.add(credential_id.to_bytes());
                }
            },
            _ => {},
        }
        
        // 5. Record revocation in DAG
        let revocation_record = RevocationRecord {
            credential_id: credential_id.clone(),
            issuer: credential.issuer,
            reason,
            timestamp: now(),
            revoked_by: msg.sender.clone(),
        };
        
        let revocation_cid = put_dag(&revocation_record)?;
        
        emit CredentialRevoked(credential_id.clone(), revocation_cid);
        Ok(())
    }
    
    pub fn check_revocation_status(
        &self,
        credential_id: &CredentialId,
        proof: Option<RevocationProof>
    ) -> Result<bool> {
        let credential = get_credential(credential_id)?;
        let registry = self.registries.get(&credential.issuer)
            .ok_or(Error::RegistryNotFound)?;
        
        match registry.registry_type {
            RevocationType::RevocationList2020 => {
                Ok(registry.revoked_credentials.contains(credential_id))
            },
            
            RevocationType::MerkleRevocation => {
                if let Some(proof) = proof {
                    // Verify merkle proof
                    Ok(registry.revocation_list.verify_inclusion(
                        &credential_id.to_bytes(),
                        &proof.merkle_proof
                    )?)
                } else {
                    // Direct lookup
                    Ok(registry.revocation_list.contains(&credential_id.to_bytes()))
                }
            },
            
            RevocationType::AccumulatorBased => {
                if let Some(acc) = &registry.accumulator {
                    if let Some(proof) = proof {
                        // Non-membership proof
                        Ok(!acc.verify_non_membership(
                            &credential_id.to_bytes(),
                            &proof.accumulator_proof
                        )?)
                    } else {
                        Ok(false)  // Can't check without proof
                    }
                } else {
                    Ok(false)
                }
            },
            
            _ => Ok(false),
        }
    }
}
```

---

## 6. Recovery Mechanisms

### 6.1 Social Recovery

```rust
pub struct SocialRecovery {
    pub fn setup_recovery(
        did: &DID,
        guardians: Vec<DID>,
        threshold: u32
    ) -> Result<()> {
        require(guardians.len() >= threshold as usize);
        require(threshold >= 3);  // Minimum 3 guardians
        
        // 1. Get consent from guardians
        for guardian in &guardians {
            let consent = request_guardian_consent(did, guardian)?;
            require(consent.accepted);
        }
        
        // 2. Create recovery shares
        let recovery_secret = generate_recovery_secret();
        let shares = shamir_split(&recovery_secret, threshold, guardians.len() as u32)?;
        
        // 3. Encrypt shares for guardians
        let encrypted_shares: Vec<_> = guardians.iter()
            .zip(shares.iter())
            .map(|(guardian, share)| {
                encrypt_for_did(guardian, share)
            })
            .collect::<Result<_>>()?;
        
        // 4. Store recovery configuration
        let recovery_config = RecoveryConfiguration {
            did: did.clone(),
            guardians: guardians.clone(),
            threshold,
            encrypted_shares,
            setup_date: now(),
            last_test: None,
        };
        
        store_recovery_config(&recovery_config)?;
        
        emit RecoverySetup(did.clone(), guardians.len(), threshold);
        Ok(())
    }
    
    pub fn initiate_recovery(
        did: &DID,
        guardian: DID,
        recovery_share: RecoveryShare
    ) -> Result<()> {
        // 1. Verify guardian
        let config = get_recovery_config(did)?;
        require(config.guardians.contains(&guardian));
        
        // 2. Store recovery share
        let recovery_session = get_or_create_recovery_session(did)?;
        recovery_session.add_share(guardian, recovery_share)?;
        
        // 3. Check if threshold met
        if recovery_session.shares.len() >= config.threshold as usize {
            // Reconstruct secret
            let shares: Vec<_> = recovery_session.shares.values().cloned().collect();
            let recovery_secret = shamir_reconstruct(&shares)?;
            
            // Generate new keys
            let new_keys = derive_keys_from_secret(&recovery_secret)?;
            
            // Perform recovery
            perform_recovery(did, &new_keys)?;
            
            emit RecoveryCompleted(did.clone());
        } else {
            emit RecoveryShareReceived(did.clone(), recovery_session.shares.len(), config.threshold);
        }
        
        Ok(())
    }
}
```

### 6.2 Delegated Recovery

```rust
pub struct DelegatedRecovery {
    pub fn setup_delegated_recovery(
        did: &DID,
        recovery_providers: Vec<RecoveryProvider>
    ) -> Result<()> {
        require(recovery_providers.len() >= 2);
        
        for provider in &recovery_providers {
            // Verify provider is legitimate
            require(verify_recovery_provider(&provider)?);
            
            // Create recovery delegation
            let delegation = RecoveryDelegation {
                did: did.clone(),
                provider: provider.clone(),
                activation_delay: Duration::from_secs(7 * 24 * 3600),  // 7 days
                evidence_required: EvidenceRequirement::High,
                created: now(),
            };
            
            store_recovery_delegation(&delegation)?;
        }
        
        Ok(())
    }
}
```

---

## 7. Multi-Party Control

### 7.1 Multi-Signature DIDs

```rust
pub struct MultiSigDID {
    did: DID,
    signers: Vec<DID>,
    threshold: u32,
    
    pub fn create_multisig_did(
        signers: Vec<DID>,
        threshold: u32
    ) -> Result<(DID, DIDDocument)> {
        require(signers.len() >= threshold as usize);
        require(threshold >= 2);
        
        // 1. Generate multisig DID
        let did = DID {
            method: "icn".to_string(),
            identifier_type: IdentifierType::Organization,
            unique_id: generate_multisig_id(&signers),
            uri: format!("did:icn:multisig:{}", generate_multisig_id(&signers)),
        };
        
        // 2. Create aggregate public key
        let public_keys: Vec<_> = signers.iter()
            .map(|s| get_public_key(s))
            .collect::<Result<_>>()?;
        
        let aggregate_key = create_threshold_key(&public_keys, threshold)?;
        
        // 3. Build DID document
        let did_document = DIDDocument {
            id: did.clone(),
            controller: signers.clone(),
            verification_method: vec![
                VerificationMethod {
                    id: format!("{}#threshold", did.uri),
                    method_type: VerificationMethodType::BLS12381G2Key2020,
                    controller: did.clone(),
                    public_key: aggregate_key,
                    created: now(),
                    expires: None,
                    revoked: None,
                }
            ],
            // ... other fields
        };
        
        // 4. Get threshold signatures
        let signatures = collect_threshold_signatures(&did_document, &signers, threshold)?;
        
        // 5. Create aggregate proof
        let proof = create_aggregate_proof(&signatures)?;
        did_document.proof = proof;
        
        // 6. Anchor in DAG
        let doc_cid = put_dag(&did_document)?;
        register_multisig_did(&did, &doc_cid, &signers, threshold)?;
        
        Ok((did, did_document))
    }
}
```

---

## 8. Monitoring & Analytics

### 8.1 Identity Metrics

```rust
pub struct IdentityMetrics {
    // DID metrics
    total_dids: Counter,
    dids_by_type: HashMap<IdentifierType, Counter>,
    key_rotations: Counter,
    recoveries: Counter,
    
    // Credential metrics
    credentials_issued: Counter,
    credentials_verified: Counter,
    credentials_revoked: Counter,
    
    // Privacy metrics
    zk_proofs_generated: Counter,
    selective_disclosures: Counter,
    
    // Security metrics
    failed_verifications: Counter,
    suspicious_patterns: Counter,
}

pub struct IdentityAnalytics {
    pub fn analyze_credential_usage(
        credential_type: &CredentialType
    ) -> CredentialAnalysis {
        CredentialAnalysis {
            total_issued: count_credentials_by_type(credential_type),
            active_credentials: count_active_credentials(credential_type),
            average_lifetime: calculate_average_lifetime(credential_type),
            revocation_rate: calculate_revocation_rate(credential_type),
            verification_frequency: calculate_verification_frequency(credential_type),
        }
    }
    
    pub fn detect_sybil_patterns() -> Vec<SuspiciousPattern> {
        let mut patterns = Vec::new();
        
        // Check for rapid DID creation
        if detect_rapid_did_creation() {
            patterns.push(SuspiciousPattern::RapidCreation);
        }
        
        // Check for credential farming
        if detect_credential_farming() {
            patterns.push(SuspiciousPattern::CredentialFarming);
        }
        
        // Check for identity cycling
        if detect_identity_cycling() {
            patterns.push(SuspiciousPattern::IdentityCycling);
        }
        
        patterns
    }
}
```

---

## 9. Implementation Roadmap

### 9.1 Phase 1: Core Identity (Months 1-2)
- [ ] DID creation and resolution
- [ ] Basic key management
- [ ] DAG integration
- [ ] Simple credentials

### 9.2 Phase 2: Advanced Credentials (Months 3-4)
- [ ] Credential schemas
- [ ] Issuance workflows
- [ ] Verification engine
- [ ] Revocation system

### 9.3 Phase 3: Privacy Features (Months 5-6)
- [ ] Zero-knowledge proofs
- [ ] Selective disclosure
- [ ] Anonymous credentials
- [ ] Ring signatures

### 9.4 Phase 4: Recovery & Security (Months 7-8)
- [ ] Social recovery
- [ ] Multi-party control
- [ ] Sybil detection
- [ ] Security auditing

---

## Appendix A: Configuration

```yaml
identity:
  # DID settings
  did:
    creation_cost: 10  # Mana
    identifier_length: 32
    key_algorithm: "Ed25519"
    proof_algorithm: "EdDSA"
    
  # Credential settings
  credentials:
    default_validity_period: 31536000  # 1 year in seconds
    max_claims_per_credential: 100
    verification_cost: 1  # Mana
    issuance_cost_base: 10  # Mana
    
  # Privacy settings
  privacy:
    zk_proof_system: "BBS+"
    commitment_scheme: "Pedersen"
    ring_signature_size: 16
    
  # Recovery settings
  recovery:
    min_guardians: 3
    max_guardians: 9
    recovery_delay: 604800  # 7 days
    guardian_consent_timeout: 86400  # 24 hours
    
  # Security settings
  security:
    max_did_per_hour: 10
    credential_rate_limit: 100
    proof_of_personhood_required: true
    sybil_detection_threshold: 0.8
```

---

## Appendix B: Error Codes

| Code | Error | Description |
|------|-------|-------------|
| I001 | InvalidDID | DID format incorrect |
| I002 | KeyRevoked | Key has been revoked |
| I003 | CredentialExpired | Credential past expiry |
| I004 | InvalidSignature | Signature verification failed |
| I005 | NotAuthorized | Lacks issuance authority |
| I006 | RecoveryFailed | Recovery process failed |
| I007 | ProofInvalid | ZK proof verification failed |
| I008 | RevocationCheckFailed | Could not verify revocation |
| I009 | SybilDetected | Potential Sybil attack |
| I010 | ThresholdNotMet | Insufficient signatures |

---

*This completes the Identity & Credential Protocol specification. The system provides self-sovereign identity with privacy-preserving verification while maintaining democratic principles.*

**Protocol Status**: DEFINITIVE  
**Dependencies**: DAG Protocol, Economic Protocol  
**Implementation Complexity**: Very High (cryptography, ZK proofs, recovery)  
**Estimated Development**: 8 months for full implementation