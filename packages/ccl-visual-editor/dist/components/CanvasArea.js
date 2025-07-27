import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState, useCallback, useRef, useEffect } from 'react';
export function CanvasArea({ nodes = [], connections = [], onNodeCreate, onNodeUpdate, onNodeDelete, onNodeSelect, onConnectionCreate, onConnectionDelete, readOnly = false, className = '' }) {
    const canvasRef = useRef(null);
    const [dragState, setDragState] = useState({
        isDragging: false,
        dragOffset: { x: 0, y: 0 },
        draggedNodeId: null
    });
    const [selectedNodeId, setSelectedNodeId] = useState(null);
    const [connectionPreview, setConnectionPreview] = useState(null);
    // Handle component drop from palette
    const handleDrop = useCallback((e) => {
        e.preventDefault();
        if (readOnly)
            return;
        try {
            const componentData = e.dataTransfer.getData('application/json');
            const component = JSON.parse(componentData);
            const rect = canvasRef.current?.getBoundingClientRect();
            if (rect) {
                const position = {
                    x: e.clientX - rect.left,
                    y: e.clientY - rect.top
                };
                onNodeCreate?.(component, position);
            }
        }
        catch (error) {
            console.error('Failed to handle drop:', error);
        }
    }, [readOnly, onNodeCreate]);
    const handleDragOver = useCallback((e) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'copy';
    }, []);
    // Handle node dragging
    const handleNodeMouseDown = useCallback((e, nodeId) => {
        if (readOnly)
            return;
        e.preventDefault();
        e.stopPropagation();
        const node = nodes.find(n => n.id === nodeId);
        if (!node)
            return;
        const rect = e.currentTarget.getBoundingClientRect();
        const offset = {
            x: e.clientX - rect.left,
            y: e.clientY - rect.top
        };
        setDragState({
            isDragging: true,
            dragOffset: offset,
            draggedNodeId: nodeId
        });
        setSelectedNodeId(nodeId);
        onNodeSelect?.(nodeId);
    }, [readOnly, nodes, onNodeSelect]);
    const handleMouseMove = useCallback((e) => {
        if (!dragState.isDragging || !dragState.draggedNodeId)
            return;
        const rect = canvasRef.current?.getBoundingClientRect();
        if (!rect)
            return;
        const newPosition = {
            x: e.clientX - rect.left - dragState.dragOffset.x,
            y: e.clientY - rect.top - dragState.dragOffset.y
        };
        onNodeUpdate?.(dragState.draggedNodeId, { position: newPosition });
    }, [dragState, onNodeUpdate]);
    const handleMouseUp = useCallback(() => {
        if (dragState.isDragging) {
            setDragState({
                isDragging: false,
                dragOffset: { x: 0, y: 0 },
                draggedNodeId: null
            });
        }
    }, [dragState.isDragging]);
    // Handle connection creation
    const handlePortMouseDown = useCallback((e, nodeId, portId) => {
        if (readOnly)
            return;
        e.preventDefault();
        e.stopPropagation();
        const node = nodes.find(n => n.id === nodeId);
        const port = node?.ports.find(p => p.id === portId);
        if (port?.type === 'output') {
            setConnectionPreview({
                sourceNodeId: nodeId,
                sourcePortId: portId,
                mousePosition: { x: e.clientX, y: e.clientY }
            });
        }
    }, [readOnly, nodes]);
    const handlePortMouseUp = useCallback((e, nodeId, portId) => {
        if (!connectionPreview || readOnly)
            return;
        e.preventDefault();
        e.stopPropagation();
        const sourceNode = nodes.find(n => n.id === connectionPreview.sourceNodeId);
        const targetNode = nodes.find(n => n.id === nodeId);
        const sourcePort = sourceNode?.ports.find(p => p.id === connectionPreview.sourcePortId);
        const targetPort = targetNode?.ports.find(p => p.id === portId);
        if (sourcePort?.type === 'output' && targetPort?.type === 'input' &&
            connectionPreview.sourceNodeId !== nodeId) {
            const newConnection = {
                sourceNodeId: connectionPreview.sourceNodeId,
                targetNodeId: nodeId,
                sourcePortId: connectionPreview.sourcePortId,
                targetPortId: portId
            };
            onConnectionCreate?.(newConnection);
        }
        setConnectionPreview(null);
    }, [connectionPreview, readOnly, nodes, onConnectionCreate]);
    // Handle keyboard shortcuts
    const handleKeyDown = useCallback((e) => {
        if (readOnly)
            return;
        if (e.key === 'Delete' && selectedNodeId) {
            onNodeDelete?.(selectedNodeId);
            setSelectedNodeId(null);
            onNodeSelect?.(null);
        }
    }, [readOnly, selectedNodeId, onNodeDelete, onNodeSelect]);
    // Set up event listeners
    useEffect(() => {
        document.addEventListener('mousemove', handleMouseMove);
        document.addEventListener('mouseup', handleMouseUp);
        document.addEventListener('keydown', handleKeyDown);
        return () => {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [handleMouseMove, handleMouseUp, handleKeyDown]);
    // Handle canvas click (deselect nodes)
    const handleCanvasClick = useCallback((e) => {
        if (e.target === canvasRef.current) {
            setSelectedNodeId(null);
            onNodeSelect?.(null);
        }
    }, [onNodeSelect]);
    return (_jsxs("div", { ref: canvasRef, className: `canvas-area relative w-full h-full bg-gray-100 overflow-hidden ${className}`, onDrop: handleDrop, onDragOver: handleDragOver, onClick: handleCanvasClick, children: [_jsx("div", { className: "absolute inset-0 opacity-20", style: {
                    backgroundImage: `
            radial-gradient(circle, #9ca3af 1px, transparent 1px)
          `,
                    backgroundSize: '20px 20px'
                } }), _jsxs("svg", { className: "absolute inset-0 w-full h-full pointer-events-none", children: [connections.map(connection => {
                        const sourceNode = nodes.find(n => n.id === connection.sourceNodeId);
                        const targetNode = nodes.find(n => n.id === connection.targetNodeId);
                        if (!sourceNode || !targetNode)
                            return null;
                        const sourcePort = sourceNode.ports.find(p => p.id === connection.sourcePortId);
                        const targetPort = targetNode.ports.find(p => p.id === connection.targetPortId);
                        if (!sourcePort || !targetPort)
                            return null;
                        // Calculate port positions
                        const sourceX = sourceNode.position.x + sourceNode.size.width;
                        const sourceY = sourceNode.position.y + 30; // Approximate port position
                        const targetX = targetNode.position.x;
                        const targetY = targetNode.position.y + 30;
                        return (_jsx(ConnectionLine, { sourceX: sourceX, sourceY: sourceY, targetX: targetX, targetY: targetY, selected: false, onClick: () => onConnectionDelete?.(connection.id) }, connection.id));
                    }), connectionPreview && (_jsx(ConnectionLine, { sourceX: 0, sourceY: 0, targetX: connectionPreview.mousePosition.x, targetY: connectionPreview.mousePosition.y, selected: false, preview: true }))] }), nodes.map(node => (_jsx(CanvasNodeComponent, { node: node, selected: selectedNodeId === node.id, onMouseDown: (e) => handleNodeMouseDown(e, node.id), onPortMouseDown: (portId) => (e) => handlePortMouseDown(e, node.id, portId), onPortMouseUp: (portId) => (e) => handlePortMouseUp(e, node.id, portId), readOnly: readOnly }, node.id))), nodes.length === 0 && (_jsx("div", { className: "absolute inset-0 flex items-center justify-center", children: _jsxs("div", { className: "text-center text-gray-500", children: [_jsx("div", { className: "text-4xl mb-4", children: "\uD83C\uDFA8" }), _jsx("h3", { className: "text-lg font-medium mb-2", children: "Start Building Your Contract" }), _jsx("p", { className: "text-sm", children: "Drag components from the palette to begin" })] }) }))] }));
}
function CanvasNodeComponent({ node, selected, onMouseDown, onPortMouseDown, onPortMouseUp, readOnly }) {
    return (_jsxs("div", { className: `absolute bg-white border-2 rounded-lg shadow-lg cursor-move transition-all ${selected ? 'border-blue-500 shadow-xl' : 'border-gray-300'} ${readOnly ? 'cursor-default' : 'cursor-move'}`, style: {
            left: node.position.x,
            top: node.position.y,
            width: node.size.width,
            height: node.size.height,
            minWidth: 200,
            minHeight: 100
        }, onMouseDown: onMouseDown, children: [_jsxs("div", { className: "p-3 border-b border-gray-200 bg-gray-50 rounded-t-lg", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "text-lg", children: node.component.icon }), _jsx("span", { className: "font-medium text-gray-900", children: node.component.name })] }), _jsx("p", { className: "text-xs text-gray-600 mt-1", children: node.component.description })] }), _jsx("div", { className: "absolute -left-2 top-1/2 transform -translate-y-1/2", children: node.ports.filter(p => p.type === 'input').map(port => (_jsx("div", { className: "w-4 h-4 bg-blue-500 border-2 border-white rounded-full cursor-pointer hover:bg-blue-600 mb-2", onMouseUp: onPortMouseUp(port.id), title: port.label }, port.id))) }), _jsx("div", { className: "absolute -right-2 top-1/2 transform -translate-y-1/2", children: node.ports.filter(p => p.type === 'output').map(port => (_jsx("div", { className: "w-4 h-4 bg-green-500 border-2 border-white rounded-full cursor-pointer hover:bg-green-600 mb-2", onMouseDown: onPortMouseDown(port.id), title: port.label }, port.id))) }), _jsx("div", { className: "p-3", children: _jsx("div", { className: "space-y-1", children: Object.entries(node.config).slice(0, 3).map(([key, value]) => (_jsxs("div", { className: "text-xs", children: [_jsxs("span", { className: "text-gray-500", children: [key, ":"] }), ' ', _jsx("span", { className: "text-gray-900", children: String(value) })] }, key))) }) })] }));
}
function ConnectionLine({ sourceX, sourceY, targetX, targetY, selected, preview = false, onClick }) {
    const midX = (sourceX + targetX) / 2;
    const pathData = `M ${sourceX} ${sourceY} C ${midX} ${sourceY}, ${midX} ${targetY}, ${targetX} ${targetY}`;
    return (_jsx("path", { d: pathData, stroke: preview ? '#9ca3af' : selected ? '#3b82f6' : '#6b7280', strokeWidth: preview ? 1 : selected ? 3 : 2, strokeDasharray: preview ? '5,5' : 'none', fill: "none", className: `${onClick ? 'cursor-pointer pointer-events-auto' : ''} transition-all`, onClick: onClick, markerEnd: "url(#arrowhead)" }));
}
