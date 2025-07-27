import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState, useCallback, useEffect } from 'react';
import { ComponentPalette } from './ComponentPalette';
import { CanvasArea } from './CanvasArea';
import { PropertyInspector } from './PropertyInspector';
import { CodePreview } from './CodePreview';
import { CCLGenerator } from '../ccl-generator';
// Generate unique IDs
function generateId() {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}
export function VisualEditor({ initialContract, readOnly = false, onContractChange, onCodeGenerated, onContractDeploy, className = '' }) {
    // Contract state
    const [contract, setContract] = useState(() => {
        return initialContract || {
            id: generateId(),
            name: 'New Governance Contract',
            description: 'Generated visual governance contract',
            nodes: [],
            connections: [],
            metadata: {
                created: new Date(),
                modified: new Date(),
                version: '1.0.0'
            }
        };
    });
    // UI state
    const [selectedNodeId, setSelectedNodeId] = useState(null);
    const [searchTerm, setSearchTerm] = useState('');
    const [selectedCategory, setSelectedCategory] = useState('');
    const [codeGenerationResult, setCodeGenerationResult] = useState(null);
    const [isDeploying, setIsDeploying] = useState(false);
    // Update contract when it changes
    const updateContract = useCallback((updates) => {
        const updatedContract = {
            ...contract,
            ...updates,
            metadata: {
                ...contract.metadata,
                modified: new Date()
            }
        };
        setContract(updatedContract);
        onContractChange?.(updatedContract);
    }, [contract, onContractChange]);
    // Generate CCL code when contract changes
    useEffect(() => {
        const result = CCLGenerator.generateFromContract(contract);
        setCodeGenerationResult(result);
        onCodeGenerated?.(result);
    }, [contract, onCodeGenerated]);
    // Node management
    const handleNodeCreate = useCallback((component, position) => {
        if (readOnly)
            return;
        const newNode = {
            id: generateId(),
            type: 'component',
            position,
            size: { width: 200, height: 120 },
            component,
            config: { ...component.defaultConfig },
            ports: [...component.ports],
            selected: false,
            dragging: false
        };
        updateContract({
            nodes: [...contract.nodes, newNode]
        });
        // Auto-select the new node
        setSelectedNodeId(newNode.id);
    }, [readOnly, contract.nodes, updateContract]);
    const handleNodeUpdate = useCallback((nodeId, updates) => {
        if (readOnly)
            return;
        const updatedNodes = contract.nodes.map(node => node.id === nodeId ? { ...node, ...updates } : node);
        updateContract({ nodes: updatedNodes });
    }, [readOnly, contract.nodes, updateContract]);
    const handleNodeDelete = useCallback((nodeId) => {
        if (readOnly)
            return;
        // Remove the node
        const updatedNodes = contract.nodes.filter(node => node.id !== nodeId);
        // Remove connections involving this node
        const updatedConnections = contract.connections.filter(conn => conn.sourceNodeId !== nodeId && conn.targetNodeId !== nodeId);
        updateContract({
            nodes: updatedNodes,
            connections: updatedConnections
        });
        // Clear selection if deleted node was selected
        if (selectedNodeId === nodeId) {
            setSelectedNodeId(null);
        }
    }, [readOnly, contract.nodes, contract.connections, selectedNodeId, updateContract]);
    const handleNodeSelect = useCallback((nodeId) => {
        setSelectedNodeId(nodeId);
    }, []);
    // Property management
    const handlePropertyChange = useCallback((nodeId, property, value) => {
        if (readOnly)
            return;
        const updatedNodes = contract.nodes.map(node => node.id === nodeId
            ? { ...node, config: { ...node.config, [property]: value } }
            : node);
        updateContract({ nodes: updatedNodes });
    }, [readOnly, contract.nodes, updateContract]);
    // Connection management
    const handleConnectionCreate = useCallback((connection) => {
        if (readOnly)
            return;
        // Check if connection already exists
        const existingConnection = contract.connections.find(conn => conn.sourceNodeId === connection.sourceNodeId &&
            conn.targetNodeId === connection.targetNodeId &&
            conn.sourcePortId === connection.sourcePortId &&
            conn.targetPortId === connection.targetPortId);
        if (existingConnection)
            return;
        const newConnection = {
            ...connection,
            id: generateId()
        };
        updateContract({
            connections: [...contract.connections, newConnection]
        });
    }, [readOnly, contract.connections, updateContract]);
    const handleConnectionDelete = useCallback((connectionId) => {
        if (readOnly)
            return;
        const updatedConnections = contract.connections.filter(conn => conn.id !== connectionId);
        updateContract({ connections: updatedConnections });
    }, [readOnly, contract.connections, updateContract]);
    // Code deployment
    const handleDeploy = useCallback(async () => {
        if (!codeGenerationResult?.valid || !onContractDeploy)
            return;
        setIsDeploying(true);
        try {
            await onContractDeploy(codeGenerationResult.code);
        }
        catch (error) {
            console.error('Deployment failed:', error);
        }
        finally {
            setIsDeploying(false);
        }
    }, [codeGenerationResult, onContractDeploy]);
    // Get selected node
    const selectedNode = selectedNodeId
        ? contract.nodes.find(node => node.id === selectedNodeId) || null
        : null;
    return (_jsxs("div", { className: `visual-editor h-full flex flex-col bg-gray-100 ${className}`, children: [_jsxs("div", { className: "flex items-center justify-between p-4 bg-white border-b border-gray-200", children: [_jsxs("div", { className: "flex items-center gap-4", children: [_jsx("h1", { className: "text-xl font-semibold text-gray-900", children: "CCL Visual Editor" }), _jsx("div", { className: "text-sm text-gray-600", children: contract.name })] }), _jsxs("div", { className: "flex items-center gap-3", children: [_jsxs("div", { className: "flex items-center gap-4 text-sm text-gray-600", children: [_jsxs("span", { children: [contract.nodes.length, " components"] }), _jsxs("span", { children: [contract.connections.length, " connections"] })] }), codeGenerationResult?.valid && onContractDeploy && (_jsx("button", { onClick: handleDeploy, disabled: isDeploying || readOnly, className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors", children: isDeploying ? 'Deploying...' : 'Deploy Contract' }))] })] }), _jsxs("div", { className: "flex-1 flex overflow-hidden", children: [_jsx("div", { className: "w-80 flex-shrink-0", children: _jsx(ComponentPalette, { onComponentSelect: handleNodeCreate, searchTerm: searchTerm, selectedCategory: selectedCategory }) }), _jsxs("div", { className: "flex-1 flex flex-col", children: [_jsx("div", { className: "flex-1", children: _jsx(CanvasArea, { nodes: contract.nodes, connections: contract.connections, onNodeCreate: handleNodeCreate, onNodeUpdate: handleNodeUpdate, onNodeDelete: handleNodeDelete, onNodeSelect: handleNodeSelect, onConnectionCreate: handleConnectionCreate, onConnectionDelete: handleConnectionDelete, readOnly: readOnly }) }), _jsx("div", { className: "h-80 flex-shrink-0", children: _jsx(CodePreview, { contract: contract, generationResult: codeGenerationResult, onCopy: () => {
                                        // Could add toast notification here
                                        console.log('Code copied to clipboard!');
                                    } }) })] }), _jsx("div", { className: "w-80 flex-shrink-0", children: _jsx(PropertyInspector, { selectedNode: selectedNode, onPropertyChange: handlePropertyChange, readOnly: readOnly }) })] }), _jsx("div", { className: "px-4 py-2 bg-gray-50 border-t border-gray-200", children: _jsxs("div", { className: "flex items-center justify-between text-sm text-gray-600", children: [_jsxs("div", { className: "flex items-center gap-4", children: [_jsxs("span", { children: ["Contract: ", contract.name] }), _jsxs("span", { children: ["Version: ", contract.metadata.version] })] }), _jsxs("div", { className: "flex items-center gap-4", children: [codeGenerationResult && (_jsx("span", { className: codeGenerationResult.errors.length > 0
                                        ? 'text-red-600'
                                        : codeGenerationResult.warnings.length > 0
                                            ? 'text-yellow-600'
                                            : 'text-green-600', children: codeGenerationResult.errors.length > 0
                                        ? `${codeGenerationResult.errors.length} errors`
                                        : codeGenerationResult.warnings.length > 0
                                            ? `${codeGenerationResult.warnings.length} warnings`
                                            : 'Valid' })), _jsxs("span", { children: ["Last modified: ", contract.metadata.modified.toLocaleTimeString()] })] })] }) })] }));
}
