import type { VisualContract, CanvasNode, Connection, CCLGenerationResult, ValidationError } from './types'
import { CCLUtils } from '@icn/ts-sdk'

export class CCLGenerator {
  static generateFromContract(contract: VisualContract): CCLGenerationResult {
    const errors: ValidationError[] = []
    const warnings: ValidationError[] = []

    try {
      // Validate the contract structure
      const validationResult = this.validateContract(contract)
      errors.push(...validationResult.errors)
      warnings.push(...validationResult.warnings)

      if (errors.length > 0) {
        return {
          code: '',
          valid: false,
          errors,
          warnings
        }
      }

      // Generate CCL code
      const code = this.buildCCLCode(contract)

      return {
        code,
        valid: true,
        errors: [],
        warnings
      }
    } catch (error) {
      errors.push({
        message: `Code generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        severity: 'error'
      })

      return {
        code: '',
        valid: false,
        errors,
        warnings
      }
    }
  }

  private static validateContract(contract: VisualContract): { errors: ValidationError[], warnings: ValidationError[] } {
    const errors: ValidationError[] = []
    const warnings: ValidationError[] = []

    // Check for nodes
    if (contract.nodes.length === 0) {
      warnings.push({
        message: 'Contract has no components. Add some governance components to get started.',
        severity: 'warning'
      })
      return { errors, warnings }
    }

    // Validate each node
    for (const node of contract.nodes) {
      const nodeErrors = this.validateNode(node)
      errors.push(...nodeErrors)
    }

    // Validate connections
    for (const connection of contract.connections) {
      const connectionErrors = this.validateConnection(connection, contract.nodes)
      errors.push(...connectionErrors)
    }

    // Check for disconnected nodes
    const connectedNodeIds = new Set<string>()
    contract.connections.forEach(conn => {
      connectedNodeIds.add(conn.sourceNodeId)
      connectedNodeIds.add(conn.targetNodeId)
    })

    contract.nodes.forEach(node => {
      if (!connectedNodeIds.has(node.id) && contract.nodes.length > 1) {
        warnings.push({
          nodeId: node.id,
          message: `Component "${node.component.name}" is not connected to other components`,
          severity: 'warning'
        })
      }
    })

    return { errors, warnings }
  }

  private static validateNode(node: CanvasNode): ValidationError[] {
    const errors: ValidationError[] = []

    // Validate component parameters
    if (node.component.parameters) {
      const validation = CCLUtils.validateTemplateParameters(node.component, node.config)
      if (!validation.valid) {
        validation.errors.forEach(error => {
          errors.push({
            nodeId: node.id,
            message: `${node.component.name}: ${error}`,
            severity: 'error'
          })
        })
      }
    }

    return errors
  }

  private static validateConnection(connection: Connection, nodes: CanvasNode[]): ValidationError[] {
    const errors: ValidationError[] = []

    const sourceNode = nodes.find(n => n.id === connection.sourceNodeId)
    const targetNode = nodes.find(n => n.id === connection.targetNodeId)

    if (!sourceNode) {
      errors.push({
        connectionId: connection.id,
        message: 'Connection source node not found',
        severity: 'error'
      })
    }

    if (!targetNode) {
      errors.push({
        connectionId: connection.id,
        message: 'Connection target node not found',
        severity: 'error'
      })
    }

    if (sourceNode && targetNode) {
      const sourcePort = sourceNode.ports.find(p => p.id === connection.sourcePortId)
      const targetPort = targetNode.ports.find(p => p.id === connection.targetPortId)

      if (!sourcePort) {
        errors.push({
          connectionId: connection.id,
          message: 'Source port not found',
          severity: 'error'
        })
      }

      if (!targetPort) {
        errors.push({
          connectionId: connection.id,
          message: 'Target port not found',
          severity: 'error'
        })
      }

      if (sourcePort && targetPort) {
        if (sourcePort.type !== 'output') {
          errors.push({
            connectionId: connection.id,
            message: 'Source port must be an output port',
            severity: 'error'
          })
        }

        if (targetPort.type !== 'input') {
          errors.push({
            connectionId: connection.id,
            message: 'Target port must be an input port',
            severity: 'error'
          })
        }

        // Check data type compatibility
        if (sourcePort.dataType !== targetPort.dataType && 
            targetPort.dataType !== 'data' && 
            sourcePort.dataType !== 'data') {
          errors.push({
            connectionId: connection.id,
            message: `Incompatible data types: ${sourcePort.dataType} → ${targetPort.dataType}`,
            severity: 'error'
          })
        }
      }
    }

    return errors
  }

  private static buildCCLCode(contract: VisualContract): string {
    const ccl = new CCLBuilder()

    // Add contract header
    ccl.addHeader(contract.name, contract.description)

    // Process nodes by category
    const governanceNodes = contract.nodes.filter(n => n.component.category === 'governance')
    const economicsNodes = contract.nodes.filter(n => n.component.category === 'economics')
    const identityNodes = contract.nodes.filter(n => n.component.category === 'identity')
    const logicNodes = contract.nodes.filter(n => n.component.category === 'logic')

    // Add role definitions first
    const roleNodes = governanceNodes.filter(n => n.component.id === 'member_role')
    roleNodes.forEach(node => ccl.addRole(node))

    // Add governance functions
    const votingNodes = governanceNodes.filter(n => n.component.id === 'voting_mechanism')
    votingNodes.forEach(node => ccl.addVotingMechanism(node))

    const proposalNodes = governanceNodes.filter(n => n.component.id === 'proposal_creation')
    proposalNodes.forEach(node => ccl.addProposalCreation(node))

    // Add economic functions
    const budgetNodes = economicsNodes.filter(n => n.component.id === 'budget_allocation')
    budgetNodes.forEach(node => ccl.addBudgetAllocation(node))

    // Add identity functions
    const reputationNodes = identityNodes.filter(n => n.component.id === 'reputation_check')
    reputationNodes.forEach(node => ccl.addReputationCheck(node))

    // Add logic functions
    const conditionalNodes = logicNodes.filter(n => n.component.id === 'if_condition')
    conditionalNodes.forEach(node => ccl.addConditional(node))

    // Add main execution flow based on connections
    ccl.addMainFunction(contract.nodes, contract.connections)

    return ccl.build()
  }
}

class CCLBuilder {
  private sections: string[] = []
  private structs: Set<string> = new Set()
  private functions: Set<string> = new Set()

  addHeader(name: string, description: string): void {
    this.sections.push(`// Generated CCL Contract: ${name}`)
    this.sections.push(`// Description: ${description}`)
    this.sections.push(`// Generated on: ${new Date().toISOString()}`)
    this.sections.push('')
  }

  addRole(node: CanvasNode): void {
    const config = node.config
    const roleName = config.role_name || 'member'
    
    if (!this.structs.has('Role')) {
      this.sections.push('struct Role {')
      this.sections.push('    name: String,')
      this.sections.push('    can_vote: Boolean,')
      this.sections.push('    can_propose: Boolean,')
      this.sections.push('    voting_weight: Integer')
      this.sections.push('}')
      this.sections.push('')
      this.structs.add('Role')
    }

    this.sections.push(`// Role: ${roleName}`)
    this.sections.push(`fn create_${roleName}_role() -> Role {`)
    this.sections.push('    return Role {')
    this.sections.push(`        name: "${roleName}",`)
    this.sections.push(`        can_vote: ${config.can_vote ? 'true' : 'false'},`)
    this.sections.push(`        can_propose: ${config.can_propose ? 'true' : 'false'},`)
    this.sections.push(`        voting_weight: ${config.voting_weight || 1}`)
    this.sections.push('    };')
    this.sections.push('}')
    this.sections.push('')
  }

  addVotingMechanism(node: CanvasNode): void {
    const config = node.config

    if (!this.structs.has('Proposal')) {
      this.sections.push('struct Proposal {')
      this.sections.push('    id: String,')
      this.sections.push('    proposer: Did,')
      this.sections.push('    title: String,')
      this.sections.push('    description: String,')
      this.sections.push('    votes_yes: Integer,')
      this.sections.push('    votes_no: Integer,')
      this.sections.push('    status: String')
      this.sections.push('}')
      this.sections.push('')
      this.structs.add('Proposal')
    }

    this.sections.push('// Voting mechanism')
    this.sections.push('fn conduct_vote(proposal: Proposal, voter: Did, vote: String) -> Proposal {')
    this.sections.push('    // Verify voter role and reputation')
    this.sections.push('    require_role(voter, "member");')
    this.sections.push('')
    this.sections.push('    // Update vote counts')
    this.sections.push('    let updated_proposal = proposal;')
    this.sections.push('    if (vote == "yes") {')
    this.sections.push('        updated_proposal.votes_yes = updated_proposal.votes_yes + 1;')
    this.sections.push('    } else if (vote == "no") {')
    this.sections.push('        updated_proposal.votes_no = updated_proposal.votes_no + 1;')
    this.sections.push('    }')
    this.sections.push('')
    this.sections.push('    // Check if voting is complete')
    this.sections.push(`    let total_votes = updated_proposal.votes_yes + updated_proposal.votes_no;`)
    this.sections.push(`    let quorum = ${config.quorum || 50};`)
    this.sections.push(`    let threshold = ${config.threshold || 0.6};`)
    this.sections.push('')
    this.sections.push('    if (total_votes >= quorum) {')
    this.sections.push('        let approval_rate = updated_proposal.votes_yes / total_votes;')
    this.sections.push('        if (approval_rate >= threshold) {')
    this.sections.push('            updated_proposal.status = "approved";')
    this.sections.push('        } else {')
    this.sections.push('            updated_proposal.status = "rejected";')
    this.sections.push('        }')
    this.sections.push('    }')
    this.sections.push('')
    this.sections.push('    return updated_proposal;')
    this.sections.push('}')
    this.sections.push('')
  }

  addProposalCreation(node: CanvasNode): void {
    const config = node.config

    this.sections.push('// Proposal creation')
    this.sections.push('fn create_proposal(proposer: Did, title: String, description: String) -> Proposal {')
    this.sections.push(`    require_role(proposer, "${config.require_role || 'member'}");`)
    this.sections.push('')
    this.sections.push(`    // Validate description length`)
    this.sections.push(`    require(description.length() >= ${config.min_description_length || 10}, "Description too short");`)
    this.sections.push(`    require(description.length() <= ${config.max_description_length || 1000}, "Description too long");`)
    this.sections.push('')
    this.sections.push('    let proposal = Proposal {')
    this.sections.push('        id: generate_proposal_id(),')
    this.sections.push('        proposer: proposer,')
    this.sections.push('        title: title,')
    this.sections.push('        description: description,')
    this.sections.push('        votes_yes: 0,')
    this.sections.push('        votes_no: 0,')
    this.sections.push('        status: "active"')
    this.sections.push('    };')
    this.sections.push('')
    this.sections.push('    return proposal;')
    this.sections.push('}')
    this.sections.push('')
  }

  addBudgetAllocation(node: CanvasNode): void {
    const config = node.config

    this.sections.push('// Budget allocation')
    this.sections.push('fn allocate_budget(proposal: Proposal, amount: Integer) -> Boolean {')
    this.sections.push(`    require(amount <= ${config.max_budget || 10000}, "Amount exceeds maximum budget");`)
    this.sections.push('')
    
    if (config.require_approval) {
      this.sections.push('    // Require governance approval')
      this.sections.push('    require(proposal.status == "approved", "Proposal must be approved");')
      this.sections.push('')
      this.sections.push('    let total_votes = proposal.votes_yes + proposal.votes_no;')
      this.sections.push('    let approval_rate = proposal.votes_yes / total_votes;')
      this.sections.push(`    require(approval_rate >= ${config.approval_threshold || 0.7}, "Insufficient approval for budget allocation");`)
    }
    
    this.sections.push('')
    this.sections.push('    // Allocate budget')
    this.sections.push('    transfer_budget(proposal.proposer, amount);')
    this.sections.push('    return true;')
    this.sections.push('}')
    this.sections.push('')
  }

  addReputationCheck(node: CanvasNode): void {
    const config = node.config

    this.sections.push('// Reputation verification')
    this.sections.push('fn verify_reputation(member: Did) -> Boolean {')
    this.sections.push('    let reputation = get_reputation(member);')
    this.sections.push(`    let min_reputation = ${config.min_reputation || 0.5};`)
    this.sections.push('')
    this.sections.push('    if (reputation >= min_reputation) {')
    this.sections.push('        return true;')
    this.sections.push('    } else {')
    this.sections.push('        return false;')
    this.sections.push('    }')
    this.sections.push('}')
    this.sections.push('')
  }

  addConditional(node: CanvasNode): void {
    const config = node.config
    const operator = config.operator || 'equals'
    const value = config.comparison_value || ''

    this.sections.push(`// Conditional logic: ${operator}`)
    this.sections.push('fn check_condition(input_value: String) -> Boolean {')
    
    switch (operator) {
      case 'equals':
        this.sections.push(`    return input_value == "${value}";`)
        break
      case 'not_equals':
        this.sections.push(`    return input_value != "${value}";`)
        break
      case 'contains':
        this.sections.push(`    return input_value.contains("${value}");`)
        break
      default:
        this.sections.push(`    return input_value == "${value}";`)
    }
    
    this.sections.push('}')
    this.sections.push('')
  }

  addMainFunction(nodes: CanvasNode[], connections: Connection[]): void {
    this.sections.push('// Main execution function')
    this.sections.push('fn main() -> String {')
    this.sections.push('    // Initialize governance system')
    
    // Find entry points (nodes with no incoming connections)
    const targetNodeIds = new Set(connections.map(c => c.targetNodeId))
    const entryNodes = nodes.filter(node => !targetNodeIds.has(node.id))
    
    if (entryNodes.length > 0) {
      this.sections.push('    // Execute entry point functions')
      entryNodes.forEach(node => {
        const functionName = this.getFunctionNameForNode(node)
        if (functionName) {
          this.sections.push(`    ${functionName}();`)
        }
      })
    } else {
      this.sections.push('    // No clear entry point found, initializing basic governance')
      if (nodes.some(n => n.component.id === 'member_role')) {
        this.sections.push('    create_member_role();')
      }
    }
    
    this.sections.push('')
    this.sections.push('    return "Governance system initialized";')
    this.sections.push('}')
  }

  private getFunctionNameForNode(node: CanvasNode): string | null {
    switch (node.component.id) {
      case 'member_role':
        return `create_${node.config.role_name || 'member'}_role`
      case 'proposal_creation':
        return 'create_proposal'
      case 'voting_mechanism':
        return 'conduct_vote'
      case 'budget_allocation':
        return 'allocate_budget'
      case 'reputation_check':
        return 'verify_reputation'
      case 'if_condition':
        return 'check_condition'
      default:
        return null
    }
  }

  build(): string {
    return this.sections.join('\n')
  }
}