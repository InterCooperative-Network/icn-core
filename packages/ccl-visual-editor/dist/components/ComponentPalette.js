import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState, useCallback } from 'react';
import { GOVERNANCE_COMPONENTS, COMPONENT_CATEGORIES, searchComponents } from '../components-library';
export function ComponentPalette({ components = GOVERNANCE_COMPONENTS, onComponentSelect, searchTerm = '', selectedCategory = '', className = '' }) {
    const [draggedComponent, setDraggedComponent] = useState(null);
    // Filter components based on search and category
    const filteredComponents = React.useMemo(() => {
        let filtered = components;
        if (searchTerm) {
            filtered = searchComponents(searchTerm);
        }
        if (selectedCategory) {
            filtered = filtered.filter(comp => comp.category === selectedCategory);
        }
        return filtered;
    }, [components, searchTerm, selectedCategory]);
    const handleDragStart = useCallback((e, component) => {
        setDraggedComponent(component);
        e.dataTransfer.setData('application/json', JSON.stringify(component));
        e.dataTransfer.effectAllowed = 'copy';
    }, []);
    const handleDragEnd = useCallback(() => {
        setDraggedComponent(null);
    }, []);
    const handleComponentClick = useCallback((component) => {
        onComponentSelect?.(component);
    }, [onComponentSelect]);
    return (_jsxs("div", { className: `component-palette bg-gray-50 border-r border-gray-200 flex flex-col ${className}`, children: [_jsx("div", { className: "p-4 border-b border-gray-200", children: _jsx("h3", { className: "text-lg font-semibold text-gray-900 m-0", children: "Components" }) }), _jsxs("div", { className: "flex flex-wrap gap-2 p-3 border-b border-gray-200", children: [_jsx("button", { className: "flex items-center gap-1 px-3 py-1.5 text-sm border border-gray-300 rounded-md bg-white text-gray-600 hover:bg-gray-50 hover:border-gray-400 transition-all", children: "All" }), COMPONENT_CATEGORIES.map(category => (_jsxs("button", { className: "flex items-center gap-1 px-3 py-1.5 text-sm border border-gray-300 rounded-md bg-white text-gray-600 hover:bg-gray-50 hover:border-gray-400 transition-all", style: { borderColor: category.color }, children: [_jsx("span", { className: "text-base", children: category.icon }), category.name] }, category.id)))] }), _jsx("div", { className: "p-3 border-b border-gray-200", children: _jsx("input", { type: "text", placeholder: "Search components...", className: "w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" }) }), _jsx("div", { className: "flex-1 overflow-y-auto p-2", children: filteredComponents.map(component => (_jsxs("div", { className: `flex items-center gap-3 p-3 mb-2 bg-white border border-gray-200 rounded-lg cursor-grab transition-all hover:border-blue-500 hover:shadow-md ${draggedComponent?.id === component.id ? 'opacity-50 transform rotate-1' : ''}`, draggable: true, onDragStart: (e) => handleDragStart(e, component), onDragEnd: handleDragEnd, onClick: () => handleComponentClick(component), children: [_jsx("div", { className: "text-2xl w-8 h-8 flex items-center justify-center", children: component.icon }), _jsxs("div", { className: "flex-1", children: [_jsx("div", { className: "font-semibold text-gray-900 mb-1", children: component.name }), _jsx("div", { className: "text-xs text-gray-600 leading-tight", children: component.description })] }), _jsx("div", { className: "text-base", style: { color: COMPONENT_CATEGORIES.find(c => c.id === component.category)?.color }, children: COMPONENT_CATEGORIES.find(c => c.id === component.category)?.icon })] }, component.id))) }), filteredComponents.length === 0 && (_jsxs("div", { className: "p-8 text-center text-gray-500", children: [_jsx("p", { className: "mb-2", children: "No components found." }), searchTerm && (_jsx("p", { children: "Try adjusting your search term." }))] }))] }));
}
