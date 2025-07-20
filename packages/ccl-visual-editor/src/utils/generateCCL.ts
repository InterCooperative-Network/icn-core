import { CanvasNode, Connection, CCLGenerationResult } from '../types'

export default function generateCCLFromNodes(
  nodes: CanvasNode[], 
  connections: Connection[]
): CCLGenerationResult {
  const errors: string[] = []
  const warnings: string[] = []
  
  if (nodes.length === 0) {
    return {
      code: '// Add components to generate CCL code\n\nfn run() -> Integer {\n    return 0;\n}',
      metadata: {
        version: '0.1.0',
        generator: 'ICN Visual Editor',
        timestamp: new Date().toISOString(),
        nodeCount: 0,
        connectionCount: 0,
      },
      errors,
      warnings
    }
  }
  
  const lines: string[] = []
  
  // Generate contract header
  lines.push('// Generated CCL Contract')
  lines.push('// Created with ICN Visual Editor')
  lines.push(`// Generated at: ${new Date().toISOString()}`)
  lines.push('')
  
  // Import statements (based on components used)
  const usedComponents = new Set(nodes.map(n => n.component.category))
  if (usedComponents.has('governance')) {
    lines.push('// Governance imports')
    lines.push('use icn::governance::*;')
  }
  if (usedComponents.has('economics')) {
    lines.push('// Economics imports')
    lines.push('use icn::economics::*;')
  }
  lines.push('')
  
  // Generate functions for each component
  nodes.forEach((node, index) => {
    const component = node.component
    const config = node.config
    
    try {
      switch (component.id) {
        case 'voting_mechanism':
          lines.push(`fn create_voting_${index}() -> Result<Proposal, RuntimeError> {`)
          lines.push(`    let quorum = ${config.quorum};`)
          lines.push(`    let threshold = ${config.threshold};`)
          lines.push(`    let duration = ${config.votingDuration};`)
          lines.push(`    let allow_delegation = ${config.allowDelegation};`)
          lines.push('')
          lines.push(`    // Create voting mechanism with specified parameters`)
          lines.push(`    let proposal = create_proposal(quorum, threshold, duration)?;`)
          lines.push(`    if allow_delegation {`)
          lines.push(`        enable_delegation(&proposal)?;`)
          lines.push(`    }`)
          lines.push(`    Ok(proposal)`)
          lines.push(`}`)
          lines.push('')
          break
          
        case 'member_role':
          lines.push(`fn assign_role_${index}(member: Did) -> Result<Bool, RuntimeError> {`)
          lines.push(`    let role_name = "${config.roleName}";`)
          lines.push(`    let permissions = "${config.permissions}";`)
          lines.push(`    let inheritance = "${config.inheritance}";`)
          lines.push('')
          lines.push(`    // Assign role to member`)
          lines.push(`    assign_member_role(member, role_name, permissions)?;`)
          lines.push(`    set_role_inheritance(member, role_name, inheritance)?;`)
          lines.push(`    Ok(true)`)
          lines.push(`}`)
          lines.push('')
          break
          
        case 'budget_request':
          lines.push(`fn create_budget_request_${index}(requester: Did) -> Result<RequestId, RuntimeError> {`)
          lines.push(`    let amount = ${config.amount};`)
          lines.push(`    let category = "${config.category}";`)
          lines.push(`    let approval_tier = "${config.approvalTier}";`)
          lines.push(`    let deadline = ${config.deadline || 30};`)
          lines.push('')
          lines.push(`    // Validate requester permissions`)
          lines.push(`    require_permission(requester, "budget_request")?;`)
          lines.push('')
          lines.push(`    // Create budget request`)
          lines.push(`    let request_id = create_budget_request(`)
          lines.push(`        requester,`)
          lines.push(`        amount,`)
          lines.push(`        category,`)
          lines.push(`        approval_tier,`)
          lines.push(`        deadline`)
          lines.push(`    )?;`)
          lines.push('')
          lines.push(`    Ok(request_id)`)
          lines.push(`}`)
          lines.push('')
          break
          
        case 'proposal_creation':
          lines.push(`fn create_proposal_${index}(proposer: Did, title: String, description: String) -> Result<ProposalId, RuntimeError> {`)
          lines.push(`    let proposal_type = "${config.proposalType}";`)
          lines.push(`    let requires_deposit = ${config.requiresDeposit};`)
          lines.push(`    let deposit_amount = ${config.depositAmount || 0};`)
          lines.push(`    let minimum_discussion = ${config.minimumDiscussion || 0};`)
          lines.push('')
          lines.push(`    // Validate proposer`)
          lines.push(`    require_permission(proposer, "propose")?;`)
          lines.push('')
          if (config.requiresDeposit && config.depositAmount > 0) {
            lines.push(`    // Check mana deposit`)
            lines.push(`    let proposer_mana = host_account_get_balance(proposer)?;`)
            lines.push(`    if proposer_mana < deposit_amount {`)
            lines.push(`        return Err(RuntimeError::InsufficientMana);`)
            lines.push(`    }`)
            lines.push(`    host_account_spend_mana(proposer, deposit_amount)?;`)
            lines.push('')
          }
          lines.push(`    // Create proposal`)
          lines.push(`    let proposal_id = create_governance_proposal(`)
          lines.push(`        proposer,`)
          lines.push(`        title,`)
          lines.push(`        description,`)
          lines.push(`        proposal_type,`)
          lines.push(`        minimum_discussion`)
          lines.push(`    )?;`)
          lines.push('')
          lines.push(`    Ok(proposal_id)`)
          lines.push(`}`)
          lines.push('')
          break
          
        case 'reputation_weighting':
          lines.push(`fn apply_reputation_weight_${index}(member: Did, vote_power: u64) -> Result<u64, RuntimeError> {`)
          lines.push(`    let weighting_type = "${config.weightingType}";`)
          lines.push(`    let minimum_reputation = ${config.minimumReputation};`)
          lines.push(`    let maximum_weight = ${config.maximumWeight};`)
          lines.push(`    let decay_factor = ${config.decayFactor};`)
          lines.push('')
          lines.push(`    // Get member reputation`)
          lines.push(`    let reputation = get_member_reputation(member)?;`)
          lines.push('')
          lines.push(`    // Check minimum reputation`)
          lines.push(`    if reputation < minimum_reputation {`)
          lines.push(`        return Err(RuntimeError::InsufficientReputation);`)
          lines.push(`    }`)
          lines.push('')
          lines.push(`    // Calculate weight based on type`)
          lines.push(`    let weight_multiplier = match weighting_type {`)
          lines.push(`        "linear" => min(reputation as f64 / 100.0, maximum_weight),`)
          lines.push(`        "quadratic" => min((reputation as f64 / 100.0).sqrt(), maximum_weight),`)
          lines.push(`        "logarithmic" => min((reputation as f64).ln() / 5.0, maximum_weight),`)
          lines.push(`        _ => 1.0,`)
          lines.push(`    };`)
          lines.push('')
          lines.push(`    let weighted_power = (vote_power as f64 * weight_multiplier) as u64;`)
          lines.push(`    Ok(weighted_power)`)
          lines.push(`}`)
          lines.push('')
          break
          
        case 'assembly_governance':
          lines.push(`fn conduct_assembly_${index}(members: Array<Did>, agenda: Object) -> Result<AssemblyResult, RuntimeError> {`)
          lines.push(`    let assembly_type = "${config.assemblyType}";`)
          lines.push(`    let allow_delegation = ${config.allowDelegation};`)
          lines.push(`    let max_delegation_depth = ${config.maxDelegationDepth};`)
          lines.push(`    let quorum_percentage = ${config.quorumPercentage};`)
          lines.push('')
          lines.push(`    // Calculate required quorum`)
          lines.push(`    let total_members = array_len(members);`)
          lines.push(`    let required_quorum = (total_members * quorum_percentage) / 100;`)
          lines.push('')
          lines.push(`    // Initialize assembly`)
          lines.push(`    let assembly_id = create_assembly(assembly_type, members, agenda)?;`)
          lines.push('')
          lines.push(`    if allow_delegation {`)
          lines.push(`        enable_assembly_delegation(assembly_id, max_delegation_depth)?;`)
          lines.push(`    }`)
          lines.push('')
          lines.push(`    // Set quorum requirement`)
          lines.push(`    set_assembly_quorum(assembly_id, required_quorum)?;`)
          lines.push('')
          lines.push(`    Ok(AssemblyResult {`)
          lines.push(`        assembly_id,`)
          lines.push(`        required_quorum,`)
          lines.push(`        total_members,`)
          lines.push(`    })`)
          lines.push(`}`)
          lines.push('')
          break
          
        default:
          warnings.push(`Unknown component type: ${component.id}`)
          lines.push(`// Unknown component: ${component.name}`)
          lines.push(`// Configuration: ${JSON.stringify(config, null, 2)}`)
          lines.push('')
      }
    } catch (error) {
      errors.push(`Error generating code for ${component.name}: ${error}`)
    }
  })
  
  // Generate main run function
  lines.push('fn run() -> Result<Integer, RuntimeError> {')
  lines.push('    // Main contract execution')
  lines.push('    let caller = host_get_caller()?;')
  lines.push('')
  
  // Call component functions
  nodes.forEach((node, index) => {
    const component = node.component
    switch (component.id) {
      case 'voting_mechanism':
        lines.push(`    let proposal_${index} = create_voting_${index}()?;`)
        break
      case 'budget_request':
        lines.push(`    let request_${index} = create_budget_request_${index}(caller)?;`)
        break
      case 'proposal_creation':
        lines.push(`    // Proposal creation requires title and description parameters`)
        lines.push(`    // let proposal_${index} = create_proposal_${index}(caller, "Title", "Description")?;`)
        break
      case 'member_role':
        lines.push(`    let role_assigned_${index} = assign_role_${index}(caller)?;`)
        break
      case 'assembly_governance':
        lines.push(`    // Assembly governance requires members array and agenda`)
        lines.push(`    // let assembly_${index} = conduct_assembly_${index}(members, agenda)?;`)
        break
    }
  })
  
  lines.push('')
  lines.push('    // Return success')
  lines.push('    Ok(0)')
  lines.push('}')
  
  // Add warnings for missing connections
  if (connections.length === 0 && nodes.length > 1) {
    warnings.push('Components are not connected. Consider adding data flow connections.')
  }
  
  return {
    code: lines.join('\n'),
    metadata: {
      version: '0.1.0',
      generator: 'ICN Visual Editor',
      timestamp: new Date().toISOString(),
      nodeCount: nodes.length,
      connectionCount: connections.length,
    },
    errors,
    warnings
  }
} 