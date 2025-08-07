# InterCooperative Network Organizational Role & Association Protocol
## Definitive Specification v2.0

---

## Executive Summary

The Organizational Role & Association Protocol establishes the **social and structural foundation** of the InterCooperative Network, defining how cooperatives, communities, and federations function as distinct yet interconnected organizational forms. This protocol bridges human organizational patterns with cryptographic infrastructure, ensuring that **economic democracy** and **mutual aid** operate seamlessly through intuitive interfaces while maintaining adversarial resistance at the technical layer.

Unlike traditional platforms that extract value through centralized control, ICN's organizational structure ensures that **cooperatives generate economic value democratically**, **communities nurture civic life and culture**, and **federations coordinate without domination**. Every organizational interaction—from a new member scanning a QR code to join, to federations coordinating disaster response—creates cryptographically verifiable records while preserving local autonomy and human dignity.

**Revolutionary Principle**: This protocol implements true economic democracy where membership grants equal voice regardless of wealth, where organizations federate voluntarily without hierarchical control, and where technology serves human cooperation rather than capital accumulation.

---

## 0 · Scope and Implementation Alignment (Normative)

This section defines the minimal, implementation-aligned v1 of the Organizational Role & Association Protocol as it exists in the codebase today. More expansive (aspirational) material remains below and is treated as planned v2 extensions.

### 0.1 Entities
- **Federation**: Organizational aggregation unit with membership policy and basic metadata. Backed by `icn-identity`.
- **CooperativeProfile**: Public profile for cooperative organizations (searchable metadata), stored via identity/DAG services.
- **DID**: All entities identified by DIDs.

### 0.2 Membership Policies
- Normative (implemented): `Open` membership.
- Reserved (spec only, not yet implemented): `InviteOnly`, `AdminApproval`, `Consensus`, `Custom{...}`.

### 0.3 Member Status
- Defined enums exist for `Active`, `Pending`, `Suspended`, `Left`, `Removed`. Flows beyond `Active` for open membership are non-normative in v1.

### 0.4 HTTP API Surface
- Federation Management:
  - `POST /api/v1/federation/join` with `{ peer: string }` → join an open federation via a known peer.
  - `POST /api/v1/federation/leave` with `{ peer: string }` → leave the federation.
- Cooperative Registry:
  - Register/search/get profile endpoints are provided for cooperative discovery and capability lookup.

These map to the `icn-api` traits and TypeScript SDK. Nodes MUST require DID-based authentication as defined in the global API guide.

### 0.5 Minimal Flows
- Federation creation by admin DID with metadata and `membership_policy: Open`.
- Open join: DID resolves → policy check passes → member is added with default role `member` and default reputation, capabilities recorded.
- Leave federation: DID removed from member set.

### 0.6 Pending Extensions
- Invitation workflows (issue/accept/revoke invites).
- Admin or consensus approval for membership.
- Role elevation, committee structures, and complex voting-gated admission.
- P2P association/announcement message types beyond HTTP join/leave.
- Capability-based access contracts for resources beyond basic recording.

### 0.7 Backward/Forward Compatibility
- v1 defines only Open membership as normative. Future versions must extend via versioned DTOs and additive endpoints to preserve compatibility.

---

## 1. Core Design Principles

### 1.1 Organizational Sovereignty with Mutual Aid
- Each organization maintains complete internal autonomy
- Voluntary federation enables resource sharing without control
- Mutual aid obligations activate automatically during crises
- No organization can dominate another through economic or technical means

### 1.2 Progressive Decentralization
- Start with trusted seeds (founding cooperatives/communities)
- Gradually expand through democratic invitation and sponsorship
- Devices and members earn capabilities through participation
- Eventually achieve permissionless interaction with stake requirements

### 1.3 Human-Centered Cryptography
- QR codes and NFC eliminate passwords while maintaining security
- Device-bound credentials prevent phishing and identity theft
- Social recovery mechanisms respect human relationships
- Cryptographic proofs invisible to end users

### 1.4 Post-Capitalist Economic Relations
- Organizations cannot be bought, sold, or owned
- Membership represents belonging, not property
- Resources flow based on need and contribution, not market forces
- Surplus redistributed democratically, not extracted as profit

---

## 2. Organizational Taxonomy & Functions

### 2.1 Cooperatives: Democratic Economic Engines

```rust
pub struct Cooperative {
    // Core Identity
    id: CooperativeId,
    did: DID,  // did:icn:coop:<unique-id>
    name: String,
    charter: CooperativeCharter,
    
    // Democratic Membership
    worker_members: HashSet<DID>,          // Equal voting rights
    consumer_members: Option<HashSet<DID>>, // If multi-stakeholder
    probationary_members: HashSet<DID>,    // Earning full membership
    membership_credentials: HashMap<DID, MembershipCredential>,
    
    // Economic Production
    primary_function: EconomicFunction,
    resource_pools: HashMap<ResourceType, ResourcePool>,
    surplus_policy: SurplusDistribution,
    
    // Governance Structure
    governance_model: CooperativeGovernance,
    current_proposals: Vec<ProposalId>,
    decision_records: Vec<DecisionCID>,
    
    // Network Contribution
    compute_contribution: ComputeScore,
    storage_provision: StorageCapacity,
    mana_multiplier: f64,  // κ_org = 1.00 (baseline)
    
    // Federation Participation
    federation_memberships: Vec<FederationId>,
    inter_coop_agreements: Vec<AgreementId>,
    
    // Revolutionary Metrics
    democratic_health: DemocraticHealth,
    wealth_equality_index: f64,  // Gini coefficient (lower = better)
    member_satisfaction: f64,
}

pub enum EconomicFunction {
    DigitalInfrastructure {
        compute_capacity: ComputeSpec,
        storage_capacity: StorageSpec,
        network_bandwidth: BandwidthSpec,
        services_provided: Vec<ServiceType>,
    },
    
    Agricultural {
        land_stewarded: Hectares,  // Not "owned" - stewarded
        crops: Vec<CropType>,
        distribution_network: Vec<CommunityId>,
        regenerative_practices: Vec<Practice>,
    },
    
    Manufacturing {
        products: Vec<Product>,
        capacity: ProductionCapacity,
        materials_sourcing: Vec<SourceAgreement>,
        repair_capability: bool,  // Right-to-repair support
    },
    
    CareWork {
        care_types: Vec<CareType>,  // Childcare, eldercare, healthcare
        capacity: CareCapacity,
        community_integration: bool,
    },
    
    Knowledge {
        education_programs: Vec<Program>,
        research_areas: Vec<ResearchArea>,
        open_source_contributions: Vec<ProjectId>,
    },
}

pub struct CooperativeCharter {
    // Foundational Principles
    founding_principles: Vec<Principle>,
    commitment_to_cooperation: CooperationCommitment,
    
    // Membership Rules
    membership_requirements: MembershipCriteria,
    onboarding_process: OnboardingSteps,
    expulsion_procedure: ExpulsionProcess,  // Democratic, rare
    
    // Economic Policies
    surplus_distribution: SurplusPolicy,
    patronage_calculation: PatronageFormula,
    reserve_requirements: ReservePolicy,
    
    // Democratic Governance
    voting_method: VotingMethod,
    proposal_process: ProposalRules,
    meeting_frequency: Schedule,
}
```

### 2.2 Communities: Civic & Cultural Hearts

```rust
pub struct Community {
    // Core Identity
    id: CommunityId,
    did: DID,  // did:icn:community:<unique-id>
    name: String,
    charter: CommunityCharter,
    
    // Inclusive Membership
    full_members: HashSet<DID>,        // Voting rights
    provisional_members: HashSet<DID>,  // Earning membership
    guests: HashSet<DID>,               // Temporary participants
    
    // Civic Infrastructure
    public_goods: Vec<PublicGood>,
    commons_resources: Vec<CommonsResource>,
    cultural_spaces: Vec<CulturalSpace>,
    
    // Mutual Aid Systems
    mutual_aid_pool: MutualAidPool,
    emergency_resources: EmergencyCache,
    care_networks: Vec<CareNetwork>,
    
    // Governance & Deliberation
    governance_model: CommunityGovernance,
    agora_spaces: Vec<AgoraNetSpace>,  // Deliberation forums
    consensus_processes: Vec<ConsensusProcess>,
    
    // Cultural Expression
    cultural_programs: Vec<CulturalProgram>,
    traditions_maintained: Vec<Tradition>,
    languages_supported: Vec<Language>,
    
    // Network Participation
    mana_multiplier: f64,  // κ_org = 0.95 (civic bonus)
    federation_memberships: Vec<FederationId>,
    sister_communities: Vec<CommunityId>,
}

pub enum CommunityGovernance {
    DirectDemocracy {
        quorum: f64,
        consensus_threshold: f64,
    },
    
    Consensus {
        blocking_concerns: bool,  // Can one person block?
        modification_process: ModificationRules,
    },
    
    Sociocracy {
        circles: Vec<Circle>,
        linking: DoubleLinkning,
    },
    
    Indigenous {
        tradition: String,
        elders_council: Option<EldersCouncil>,
        ceremony_integration: bool,
    },
    
    Hybrid {
        components: Vec<GovernanceComponent>,
        decision_routing: DecisionRouter,
    },
}
```

### 2.3 Federations: Coordination Without Domination

```rust
pub struct Federation {
    // Core Identity
    id: FederationId,
    did: DID,  // did:icn:federation:<unique-id>
    name: String,
    charter: FederationCharter,
    
    // Organizational Members (not individuals)
    member_cooperatives: HashSet<CooperativeId>,
    member_communities: HashSet<CommunityId>,
    observer_organizations: HashSet<OrganizationId>,
    
    // Coordination Functions
    resource_balancer: ResourceBalancer,
    dispute_mediator: DisputeMediator,
    emergency_coordinator: EmergencyCoordinator,
    
    // Shared Infrastructure
    shared_resources: Vec<SharedResource>,
    bridge_protocols: HashMap<(OrgId, OrgId), BridgeProtocol>,
    translation_services: TranslationLayer,  // Inter-org communication
    
    // Democratic Safeguards
    rotation_schedule: RotationSchedule,  // Prevent entrenchment
    recall_mechanism: RecallProcess,
    transparency_requirements: TransparencyRules,
    
    // Network Economics
    mana_multiplier: f64,  // κ_org = 1.25 (coordination bonus)
    internal_currency: Option<MutualCreditSystem>,
    trade_agreements: Vec<TradeAgreement>,
    
    // Hierarchy Prevention
    max_delegation_depth: u8,  // Prevent deep hierarchies
    power_distribution_index: f64,  // Must stay balanced
    autonomy_guarantees: Vec<AutonomyGuarantee>,
}

pub struct FederationCharter {
    // Purpose & Principles
    shared_purpose: String,
    solidarity_principles: Vec<Principle>,
    
    // Membership Governance
    admission_process: AdmissionProcess,
    voting_weight: VotingWeight,  // Equal vs proportional
    exit_rights: ExitRights,  // Unconditional right to leave
    
    // Resource Coordination
    resource_sharing: ResourceSharingAgreement,
    emergency_obligations: EmergencyObligations,
    surplus_redistribution: RedistributionFormula,
    
    // Dispute Resolution
    mediation_process: MediationProcess,
    arbitration_option: Option<ArbitrationRules>,
    restorative_justice: RestorativeProcess,
}
```

---

## 3. Membership & Association Lifecycle

### 3.0 Simplified State Model (Aligned with Implementation)

To reduce complexity while matching existing code paths, membership progresses through a minimal, implementation-aligned state machine:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipState {
    Pending,    // Application received; awaiting automatic or governed activation
    Active,     // Full rights within org scope per policy
    Suspended,  // Temporarily disabled; no rights until reinstated
    Departed,   // Voluntary leave or removal (Left/Removed)
}

// Mapping to icn-identity::MembershipStatus
// Pending  -> Pending
// Active   -> Active
// Suspended-> Suspended
// Departed -> Left | Removed

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberRole {
    Member,
    Coordinator,
    Committee(String),
    Elder,            // Honorary role; not a lifecycle state
}
```

Notes:
- “Curious/Provisional/Probationary/Full/Elder” are consolidated. “Elder” is a role, not a state.
- Admission gating (automatic, invite, admin, consensus) is expressed via policy that transitions `Pending → Active`.
- Additional nuances (probation windows, participation checks) are policy rules that do not require distinct lifecycle states.

### 3.1 Human-Centered Onboarding

```rust
pub struct MembershipLifecycle;

impl MembershipLifecycle {
    // Initial contact → start onboarding session
    pub fn start_onboarding(
        person: &Person,
        organization: &Organization,
        method: ContactMethod,
    ) -> Result<OnboardingSession> {
        match method {
            ContactMethod::InPerson(event) => create_warm_onboarding(person, find_volunteer_sponsor(&event)?, organization),
            ContactMethod::QRCode(invitation) => { validate_invitation(&invitation)?; create_sponsored_onboarding(person, invitation.sponsor, organization) }
            ContactMethod::WebPortal => create_provisional_onboarding(person, organization),
            ContactMethod::Referral(referrer) => { verify_referrer_membership(&referrer, organization)?; create_referred_onboarding(person, referrer, organization) }
        }
    }

    // State transitions governed by policy
    pub fn transition(
        member: &Member,
        org: &Organization,
        event: MembershipEvent,
    ) -> Result<MembershipState> {
        match (member.state, event) {
            // Admission
            (MembershipState::Pending, MembershipEvent::AutoApprove) => Ok(MembershipState::Active),
            (MembershipState::Pending, MembershipEvent::InviteApproved) => Ok(MembershipState::Active),
            (MembershipState::Pending, MembershipEvent::ConsensusApproved) => Ok(MembershipState::Active),

            // Governance enforcement
            (MembershipState::Active, MembershipEvent::Suspend) => Ok(MembershipState::Suspended),
            (MembershipState::Suspended, MembershipEvent::Reinstate) => Ok(MembershipState::Active),

            // Exit
            (MembershipState::Active, MembershipEvent::Leave)
            | (MembershipState::Pending, MembershipEvent::Withdraw)
            | (_, MembershipEvent::Expel) => Ok(MembershipState::Departed),

            // No-op for unsupported transitions
            (state, _) => Ok(state),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipEvent {
    // Admission paths
    AutoApprove, InviteApproved, ConsensusApproved,
    // Governance actions
    Suspend, Reinstate,
    // Exit actions
    Leave, Withdraw, Expel,
}
```

### 3.2 Device & Node Association

```rust
pub struct DeviceAssociation {
    // Mobile Device Onboarding
    pub fn onboard_mobile_device(
        device: &MobileDevice,
        owner: &Member
    ) -> Result<DeviceCredential> {
        // 1. Generate device DID locally
        let device_did = device.generate_did_locally()?;
        
        // 2. Bind to owner's identity
        let binding = create_device_binding(owner.did, device_did)?;
        
        // 3. Progressive trust building
        let initial_trust = TrustLevel::Minimal;
        let capabilities = minimal_capabilities();
        
        // 4. Issue device credential
        let credential = DeviceCredential {
            device_did,
            owner_did: owner.did,
            organization: owner.organization,
            trust_level: initial_trust,
            capabilities,
            issued_at: now(),
            expires_at: now() + Duration::days(30),
        };
        
        Ok(credential)
    }
    
    // Collective Mobile Compute
    pub fn form_mobile_collective(
        devices: Vec<MobileDevice>,
        task: CollectiveTask
    ) -> Result<MobileCollective> {
        // Only trusted devices participate
        let trusted = devices.into_iter()
            .filter(|d| d.trust_level >= TrustLevel::Verified)
            .collect::<Vec<_>>();
        
        if trusted.len() < MIN_COLLECTIVE_SIZE {
            return Err(Error::InsufficientDevices);
        }
        
        // Form collective with fair work distribution
        let collective = MobileCollective {
            devices: trusted,
            task_distribution: distribute_fairly(&task, &trusted)?,
            coordination_protocol: Protocol::EqualPeer,
            compensation_model: Model::EqualShare,
        };
        
        Ok(collective)
    }
}
```

---

## 4. Intuitive Authentication & Interaction

### 4.1 QR Code Protocols

```rust
pub struct QRAuthentication {
    // Universal Login (No Passwords Ever)
    pub fn generate_login_qr(
        service: &Service,
        context: &Context
    ) -> VisualQRCode {
        let challenge = LoginChallenge {
            service_did: service.did,
            context: context.clone(),
            timestamp: now(),
            expires: now() + Duration::minutes(5),
            required_credentials: determine_required_credentials(service),
            accessibility_options: get_accessibility_options(),
        };
        
        VisualQRCode {
            payload: challenge,
            visual_design: accessible_high_contrast(),
            size_options: vec![Size::Small, Size::Medium, Size::Large],
            alternative_text: generate_alt_text(&challenge),
        }
    }
    
    // Mutual Aid Request QR
    pub fn generate_aid_request_qr(
        requester: &Member,
        need: &AidRequest
    ) -> AidQRCode {
        let request = MutualAidRequest {
            requester_did: requester.did,
            need_type: need.category,
            urgency: need.urgency,
            location: need.approximate_location(),  // Privacy-preserving
            accepts: vec![
                AidType::DirectHelp,
                AidType::Resources,
                AidType::Knowledge,
            ],
            privacy_level: need.privacy_preference,
        };
        
        AidQRCode {
            payload: request,
            visual_indicator: urgency_color(need.urgency),
            expiry: calculate_expiry(&need),
        }
    }
    
    // Resource Sharing QR
    pub fn generate_sharing_qr(
        sharer: &Organization,
        resource: &Resource
    ) -> SharingQRCode {
        let offer = ResourceOffer {
            provider: sharer.did,
            resource_type: resource.classification,
            quantity: resource.available_amount,
            conditions: resource.sharing_conditions,
            duration: resource.availability_window,
            preference: SharingPreference::LocalFirst,
        };
        
        SharingQRCode {
            payload: offer,
            verification_endpoint: get_verification_endpoint(sharer),
            terms: resource.terms_of_sharing,
        }
    }
}
```

### 4.2 NFC Touch Interactions

```rust
pub struct NFCInteraction {
    // Tap to Connect (Member Meeting)
    pub fn handle_member_tap(
        initiator_device: &NFCDevice,
        responder_device: &NFCDevice
    ) -> Result<Connection> {
        // 1. Exchange DIDs
        let initiator_did = initiator_device.broadcast_did()?;
        let responder_did = responder_device.broadcast_did()?;
        
        // 2. Verify membership credentials
        let initiator_creds = initiator_device.present_credentials()?;
        let responder_creds = responder_device.present_credentials()?;
        
        // 3. Establish secure channel
        let shared_secret = perform_diffie_hellman(
            initiator_device.ephemeral_key(),
            responder_device.ephemeral_key()
        )?;
        
        // 4. Create connection record
        let connection = Connection {
            participants: vec![initiator_did, responder_did],
            established_at: now(),
            trust_level: calculate_mutual_trust(&initiator_creds, &responder_creds),
            capabilities: determine_shared_capabilities(),
        };
        
        // 5. Optional: Exchange contact info
        if both_consent_to_contact_exchange() {
            exchange_contact_information(&connection)?;
        }
        
        Ok(connection)
    }
    
    // Tap to Pay (Mutual Economy)
    pub fn handle_payment_tap(
        payer_device: &NFCDevice,
        payee_terminal: &NFCTerminal
    ) -> Result<PaymentResult> {
        // 1. Terminal presents invoice
        let invoice = payee_terminal.current_invoice()?;
        
        // 2. Device shows to user for confirmation
        let user_decision = payer_device.display_and_confirm(&invoice)?;
        
        if !user_decision.approved {
            return Ok(PaymentResult::Declined);
        }
        
        // 3. Select payment method
        let payment_method = select_best_method(
            &payer_device.available_methods(),
            &invoice.accepted_methods
        )?;
        
        // 4. Execute payment
        let result = match payment_method {
            Method::MutualCredit => {
                execute_mutual_credit_transfer(&payer_device, &payee_terminal, &invoice)
            },
            Method::ResourceToken => {
                execute_token_transfer(&payer_device, &payee_terminal, &invoice)
            },
            Method::TimeBank => {
                execute_time_transfer(&payer_device, &payee_terminal, &invoice)
            },
            Method::Mana => {
                execute_mana_transfer(&payer_device, &payee_terminal, &invoice)
            },
        }?;
        
        // 5. Both parties receive receipts
        let receipt = generate_receipt(&result);
        payer_device.store_receipt(&receipt)?;
        payee_terminal.store_receipt(&receipt)?;
        
        Ok(PaymentResult::Complete(receipt))
    }
}
```

---

## 5. Inter-Organizational Coordination

### 5.1 Mutual Aid Networks

```rust
pub struct MutualAidCoordination {
    // Automated Mutual Aid Activation
    pub fn activate_mutual_aid(
        trigger: AidTrigger,
        affected_orgs: Vec<OrganizationId>
    ) -> Result<MutualAidResponse> {
        
        match trigger {
            AidTrigger::NaturalDisaster(disaster) => {
                // Immediate response without bureaucracy
                let nearby = find_organizations_within_radius(&disaster.location, 200_km);
                
                for org in &nearby {
                    // Automatic activation
                    activate_emergency_mode(org)?;
                    waive_all_transaction_costs(org)?;
                    mobilize_resources(org)?;
                }
                
                // Create temporary coordination federation
                let emergency_fed = create_emergency_federation(
                    affected_orgs,
                    nearby,
                    Duration::days(60)
                )?;
                
                Ok(MutualAidResponse::EmergencyActivated(emergency_fed))
            },
            
            AidTrigger::EconomicHardship(crisis) => {
                // Activate mutual credit and resource sharing
                for org in &affected_orgs {
                    extend_credit_lines(org)?;
                    share_surplus_resources(org)?;
                    offer_work_opportunities(org)?;
                }
                
                Ok(MutualAidResponse::EconomicSupport)
            },
            
            AidTrigger::HealthCrisis(health) => {
                // Coordinate care resources
                mobilize_care_workers(&health)?;
                share_medical_resources(&health)?;
                establish_support_networks(&health)?;
                
                Ok(MutualAidResponse::HealthSupport)
            },
        }
    }
}
```

### 5.2 Resource Balancing

```rust
pub struct ResourceBalancing {
    // Democratic Resource Allocation
    pub fn balance_resources(
        federation: &Federation,
        period: TimePeriod
    ) -> Result<ResourceAllocation> {
        
        // 1. Assess needs democratically
        let needs = federation.members.iter()
            .map(|org| assess_needs(org, &period))
            .collect::<Vec<_>>();
        
        // 2. Inventory available resources
        let available = federation.members.iter()
            .map(|org| inventory_surplus(org, &period))
            .collect::<Vec<_>>();
        
        // 3. Match needs with resources
        let allocation = match federation.allocation_method {
            Method::NeedsBased => {
                // Prioritize greatest need
                allocate_by_need(&needs, &available)
            },
            Method::Contributional => {
                // Consider past contributions
                allocate_by_contribution(&needs, &available, &federation.history)
            },
            Method::Rotational => {
                // Rotate priority fairly
                allocate_rotationally(&needs, &available, &federation.rotation_state)
            },
            Method::Consensus => {
                // Require agreement
                reach_consensus_allocation(&needs, &available, &federation.members)
            },
        }?;
        
        // 4. Execute transfers
        for transfer in &allocation.transfers {
            execute_resource_transfer(transfer)?;
            record_in_dag(transfer)?;
        }
        
        Ok(allocation)
    }
}
```

---

## 6. AgoraNet Integration for Deliberation

### 6.1 Deliberative Decision Making

```rust
pub struct DeliberativeGovernance {
    // Create Deliberation Space
    pub fn initiate_deliberation(
        topic: &Topic,
        organization: &Organization
    ) -> Result<AgoraSpace> {
        
        let space = AgoraSpace {
            id: generate_space_id(),
            topic: topic.clone(),
            organization: organization.id,
            
            // Structured phases
            phases: vec![
                Phase::Education {
                    duration: Duration::days(3),
                    resources: gather_educational_resources(topic),
                },
                Phase::Discussion {
                    duration: Duration::days(5),
                    format: DiscussionFormat::ThreadedWithSynthesis,
                },
                Phase::ProposalDevelopment {
                    duration: Duration::days(3),
                    method: ProposalMethod::Collaborative,
                },
                Phase::Refinement {
                    duration: Duration::days(2),
                    process: RefinementProcess::IterativeConsensus,
                },
                Phase::Decision {
                    duration: Duration::days(1),
                    mechanism: organization.decision_mechanism(),
                },
            ],
            
            // Participation settings
            participation: ParticipationRules {
                who_can_read: Anyone,  // Transparency
                who_can_contribute: Members(organization.id),
                who_can_facilitate: ElectedFacilitators,
                anonymous_option: true,  // For sensitive topics
            },
            
            // Quality measures
            quality_metrics: QualityMetrics {
                track_participation_equity: true,
                measure_viewpoint_diversity: true,
                assess_information_quality: true,
                evaluate_process_health: true,
            },
        };
        
        Ok(space)
    }
    
    // Synthesis and Sense-Making
    pub fn synthesize_deliberation(
        space: &AgoraSpace
    ) -> Result<Synthesis> {
        
        let contributions = get_all_contributions(space)?;
        
        let synthesis = Synthesis {
            // Key themes
            themes: extract_themes(&contributions),
            
            // Points of agreement
            consensus_points: find_consensus(&contributions),
            
            // Points of tension
            tension_points: identify_tensions(&contributions),
            
            // Minority views
            minority_perspectives: preserve_minority_views(&contributions),
            
            // Proposed actions
            action_proposals: extract_proposals(&contributions),
            
            // Wisdom gained
            collective_insights: distill_wisdom(&contributions),
        };
        
        Ok(synthesis)
    }
}
```

---

## 7. Emergency Response & Resilience

### 7.1 Crisis Response Protocol

```rust
pub struct CrisisResponse {
    // Rapid Emergency Response
    pub fn respond_to_crisis(
        crisis: Crisis,
        affected_region: Region
    ) -> Result<EmergencyResponse> {
        
        // 1. Immediate triage
        let severity = assess_severity(&crisis);
        let affected_orgs = identify_affected_organizations(&affected_region);
        
        // 2. Activate emergency protocols
        match severity {
            Severity::Critical => {
                // All hands response
                activate_all_emergency_protocols()?;
                suspend_normal_operations()?;
                mobilize_all_resources()?;
            },
            Severity::Serious => {
                // Significant response
                activate_regional_protocols(&affected_region)?;
                mobilize_emergency_resources()?;
            },
            Severity::Moderate => {
                // Measured response
                activate_local_protocols(&affected_orgs)?;
                share_available_resources()?;
            },
        }
        
        // 3. Establish coordination
        let coordinator = elect_emergency_coordinator(&affected_orgs)?;
        let command_structure = create_temporary_coordination(
            coordinator,
            affected_orgs,
            Duration::days(30)
        )?;
        
        // 4. Open all communication channels
        let channels = establish_emergency_channels()?;
        
        // 5. Begin response
        let response = EmergencyResponse {
            crisis_id: generate_crisis_id(),
            coordination: command_structure,
            resources_mobilized: count_mobilized_resources(),
            organizations_involved: affected_orgs,
            communication_channels: channels,
            status: ResponseStatus::Active,
        };
        
        Ok(response)
    }
}
```

---

## 8. Security & Trust

### 8.1 Multi-Layer Security Model

```rust
pub struct SecurityModel {
    // Device Security
    device_security: DeviceSecurity {
        secure_element_required: true,
        biometric_authentication: true,
        secure_enclave_keys: true,
        remote_wipe_capability: true,
    },
    
    // Network Security
    network_security: NetworkSecurity {
        end_to_end_encryption: true,
        perfect_forward_secrecy: true,
        onion_routing_available: true,
        metadata_minimization: true,
    },
    
    // Organizational Security
    organizational_security: OrgSecurity {
        membership_verification: MultiFactorVerification,
        proposal_authentication: ThresholdSignatures,
        resource_transfer_limits: VelocityLimits,
        emergency_freeze_capability: true,
    },
    
    // Social Security
    social_security: SocialSecurity {
        social_recovery: true,
        guardian_networks: true,
        reputation_systems: true,
        ostracism_protocols: true,  // For bad actors
    },
}
```

### 8.2 Trust Building

```rust
pub struct TrustBuilding {
    // Progressive Trust Accumulation
    pub fn build_trust(
        entity: &Entity,
        actions: &[Action]
    ) -> TrustScore {
        
        let trust = TrustScore {
            // Base trust from credentials
            credential_trust: calculate_credential_trust(&entity.credentials),
            
            // Behavioral trust from actions
            behavioral_trust: calculate_behavioral_trust(&actions),
            
            // Social trust from endorsements
            social_trust: calculate_social_trust(&entity.endorsements),
            
            // Economic trust from transactions
            economic_trust: calculate_economic_trust(&entity.transactions),
            
            // Time-based trust
            temporal_trust: calculate_temporal_trust(&entity.membership_duration),
        };
        
        // Weight and combine
        trust.weighted_average()
    }
}
```

---

## 9. Monitoring & Metrics

### 9.1 Organizational Health Metrics

```rust
pub struct HealthMetrics {
    // Democratic Health
    democratic_health: DemocraticMetrics {
        participation_rate: f64,        // % members actively participating
        proposal_diversity: f64,        // Gini coefficient of proposers
        voting_equality: f64,           // Distribution of voting
        deliberation_quality: f64,      // From AgoraNet metrics
    },
    
    // Economic Health
    economic_health: EconomicMetrics {
        wealth_equality: f64,           // Internal Gini coefficient
        resource_utilization: f64,      // % resources in active use
        surplus_generation: f64,        // Sustainable surplus rate
        mutual_aid_strength: f64,       // Aid given/received ratio
    },
    
    // Social Health
    social_health: SocialMetrics {
        member_satisfaction: f64,       // Survey-based
        conflict_resolution: f64,       // Successful mediations
        cultural_vitality: f64,         // Program participation
        diversity_index: f64,           // Multiple dimensions
    },
    
    // Network Health
    network_health: NetworkMetrics {
        compute_contribution: f64,      // To overall network
        federation_participation: f64,  // Active in federations
        bridge_utilization: f64,        // Cross-org cooperation
        resilience_score: f64,          // Crisis response capability
    },
}
```

---

## 10. Implementation Roadmap

### Phase 1: Foundation (Months 1-2)
- [ ] Core organizational types (Cooperative, Community, Federation)
- [ ] Basic membership management with DIDs
- [ ] Simple QR code generation for invitations
- [ ] Initial governance integration

### Phase 2: Authentication & Interaction (Months 3-4)
- [ ] Complete QR/NFC authentication flows
- [ ] Mobile device onboarding and trust building
- [ ] Payment and resource sharing protocols
- [ ] Progressive membership advancement

### Phase 3: Coordination (Months 5-6)
- [ ] Federation formation and governance
- [ ] Inter-organizational resource sharing
- [ ] Mutual aid network activation
- [ ] Cross-org identity verification

### Phase 4: Deliberation & Resilience (Months 7-8)
- [ ] Full AgoraNet integration
- [ ] Emergency response protocols
- [ ] Crisis coordination systems
- [ ] Advanced trust metrics

### Phase 5: Optimization & Scale (Months 9-10)
- [ ] Performance optimization for mobile devices
- [ ] Collective compute coordination
- [ ] Advanced security hardening
- [ ] Comprehensive testing and audits

---

## 11. Revolutionary Implementation Notes

### 11.1 Anti-Patterns to Avoid
- **No Venture Capital Structure**: Organizations cannot be "invested in" or "exit"
- **No Surveillance Capitalism**: Data remains with members, not extracted
- **No Hierarchical Control**: Federations coordinate, not command
- **No Market Fundamentalism**: Needs and contributions, not supply/demand
- **No Digital Feudalism**: Platform owned by users, not shareholders

### 11.2 Patterns to Embrace
- **Mutual Aid First**: Help is automatic, not transactional
- **Democracy by Default**: Every decision is participatory
- **Local Autonomy**: Decisions at the most local level possible
- **Global Solidarity**: Coordinate without centralizing
- **Regenerative Economics**: Wealth circulates, not accumulates

---

## Appendix A: Configuration Reference

```yaml
# Organizational Configuration
organizations:
  # Cooperatives - Economic Engines
  cooperatives:
    mana_multiplier: 1.00
    minimum_members: 3
    probation_period_days: 30
    surplus_redistribution: patron_based
    voting_mechanism: one_member_one_vote
    
  # Communities - Civic Hearts  
  communities:
    mana_multiplier: 0.95
    minimum_members: 5
    provisional_period_days: 60
    governance_models:
      - direct_democracy
      - consensus
      - sociocracy
    cultural_support: true
    
  # Federations - Coordination Bridges
  federations:
    mana_multiplier: 1.25
    minimum_organizations: 2
    formation_stake_mana: 10000
    maximum_hierarchy_depth: 2
    power_balance_threshold: 0.7
    
# Authentication Settings
authentication:
  qr_code:
    expiry_seconds: 300
    challenge_entropy_bits: 256
    accessibility_mode: high_contrast
    
  nfc:
    timeout_seconds: 10
    encryption: aes256_gcm
    range_meters: 0.1
    
# Trust Building
trust:
  initial_device_trust: 0.1
  trust_increment_per_action: 0.01
  trust_decay_per_day: 0.001
  maximum_trust: 1.0
  
  verification_levels:
    basic: 0.3
    verified: 0.6
    trusted: 0.9
    
# Mutual Aid
mutual_aid:
  automatic_activation: true
  crisis_response_time_minutes: 15
  resource_mobilization: immediate
  cost_during_emergency: 0
  
# Democratic Safeguards
democracy:
  quorum_percentage: 25
  consensus_threshold: 80
  veto_enabled: true
  minority_protection: true
  transparency_required: true
```

---

## Appendix B: Security Considerations

### B.1 Threat Model
- **External**: State surveillance, corporate espionage, criminal actors
- **Internal**: Bad faith actors, governance capture attempts, resource hoarding
- **Systemic**: Platform monopolization, wealth concentration, hierarchy formation

### B.2 Mitigations
- **Cryptographic**: End-to-end encryption, zero-knowledge proofs, secure multi-party computation
- **Social**: Reputation systems, democratic oversight, social recovery
- **Economic**: Anti-speculation mechanisms, wealth caps, demurrage
- **Organizational**: Rotation requirements, transparency mandates, federation limits

---

## Appendix C: Human Rights Considerations

This protocol explicitly upholds:
- **Right to Democratic Participation**: Every member has equal voice
- **Right to Economic Security**: Mutual aid and basic needs guaranteed
- **Right to Privacy**: Surveillance resistance built-in
- **Right to Culture**: Indigenous and minority practices protected
- **Right to Leave**: No lock-in, exit always possible
- **Right to Data**: Users control their information completely

---

*This completes the Organizational Role & Association Protocol. It establishes the human and organizational layer of ICN, ensuring that technology serves cooperation, democracy, and mutual aid rather than capital accumulation.*

**Protocol Status**: DEFINITIVE  
**Dependencies**: All other ICN protocols  
**Revolutionary Commitment**: Total  
**Implementation Complexity**: High (human systems are complex)  
**Estimated Development**: 10 months for full implementation with community participation

**Document Control**:
- **Version**: 2.0 (Post-PR Review)
- **Last Updated**: Current
- **Next Review**: After Phase 1 implementation
- **Community Input**: Required for all phases