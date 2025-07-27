import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState, useEffect } from 'react';
import { CCLUtils } from '@icn/ts-sdk';
export function PropertyInspector({ selectedNode, onPropertyChange, readOnly = false, className = '' }) {
    const [localConfig, setLocalConfig] = useState({});
    const [validationErrors, setValidationErrors] = useState({});
    // Update local config when selected node changes
    useEffect(() => {
        if (selectedNode) {
            setLocalConfig({ ...selectedNode.config });
            validateConfiguration(selectedNode, selectedNode.config);
        }
        else {
            setLocalConfig({});
            setValidationErrors({});
        }
    }, [selectedNode]);
    const validateConfiguration = (node, config) => {
        if (!node.component.parameters) {
            setValidationErrors({});
            return;
        }
        const validation = CCLUtils.validateTemplateParameters(node.component, config);
        const errors = {};
        if (!validation.valid) {
            validation.errors.forEach(error => {
                // Extract parameter name from error message
                const paramMatch = error.match(/^(\w+)/);
                if (paramMatch) {
                    errors[paramMatch[1]] = error;
                }
            });
        }
        setValidationErrors(errors);
    };
    const handleParameterChange = (parameterName, value) => {
        if (readOnly || !selectedNode)
            return;
        const newConfig = { ...localConfig, [parameterName]: value };
        setLocalConfig(newConfig);
        validateConfiguration(selectedNode, newConfig);
        onPropertyChange?.(selectedNode.id, parameterName, value);
    };
    if (!selectedNode) {
        return (_jsxs("div", { className: `property-inspector bg-gray-50 border-l border-gray-200 flex flex-col ${className}`, children: [_jsx("div", { className: "p-4 border-b border-gray-200", children: _jsx("h3", { className: "text-lg font-semibold text-gray-900 m-0", children: "Properties" }) }), _jsx("div", { className: "flex-1 flex items-center justify-center", children: _jsxs("div", { className: "text-center text-gray-500", children: [_jsx("div", { className: "text-4xl mb-4", children: "\u2699\uFE0F" }), _jsx("p", { className: "text-sm", children: "Select a component to view its properties" })] }) })] }));
    }
    return (_jsxs("div", { className: `property-inspector bg-gray-50 border-l border-gray-200 flex flex-col ${className}`, children: [_jsx("div", { className: "p-4 border-b border-gray-200", children: _jsx("h3", { className: "text-lg font-semibold text-gray-900 m-0", children: "Properties" }) }), _jsxs("div", { className: "p-4 border-b border-gray-200 bg-white", children: [_jsxs("div", { className: "flex items-center gap-3 mb-3", children: [_jsx("span", { className: "text-2xl", children: selectedNode.component.icon }), _jsxs("div", { children: [_jsx("h4", { className: "font-semibold text-gray-900", children: selectedNode.component.name }), _jsx("p", { className: "text-sm text-gray-600", children: selectedNode.component.description })] })] }), _jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "px-2 py-1 text-xs font-medium bg-gray-100 text-gray-700 rounded-full", children: selectedNode.component.category }), _jsxs("span", { className: "text-xs text-gray-500", children: ["ID: ", selectedNode.id.slice(0, 8)] })] })] }), _jsxs("div", { className: "flex-1 overflow-y-auto", children: [selectedNode.component.parameters && selectedNode.component.parameters.length > 0 ? (_jsxs("div", { className: "p-4 space-y-4", children: [_jsx("h5", { className: "font-medium text-gray-900 mb-3", children: "Configuration" }), selectedNode.component.parameters.map(parameter => (_jsx(ParameterInput, { parameter: parameter, value: localConfig[parameter.name], error: validationErrors[parameter.name], onChange: (value) => handleParameterChange(parameter.name, value), readOnly: readOnly }, parameter.name)))] })) : (_jsx("div", { className: "p-4 text-center text-gray-500", children: _jsx("p", { className: "text-sm", children: "This component has no configurable parameters." }) })), _jsxs("div", { className: "p-4 border-t border-gray-200 bg-gray-50", children: [_jsx("h5", { className: "font-medium text-gray-900 mb-2", children: "Position" }), _jsxs("div", { className: "grid grid-cols-2 gap-2 text-sm", children: [_jsxs("div", { children: [_jsx("label", { className: "text-gray-600", children: "X:" }), _jsx("span", { className: "ml-2 text-gray-900", children: Math.round(selectedNode.position.x) })] }), _jsxs("div", { children: [_jsx("label", { className: "text-gray-600", children: "Y:" }), _jsx("span", { className: "ml-2 text-gray-900", children: Math.round(selectedNode.position.y) })] })] })] }), selectedNode.ports.length > 0 && (_jsxs("div", { className: "p-4 border-t border-gray-200", children: [_jsx("h5", { className: "font-medium text-gray-900 mb-2", children: "Ports" }), _jsx("div", { className: "space-y-2", children: selectedNode.ports.map(port => (_jsxs("div", { className: "flex items-center gap-2 text-sm", children: [_jsx("div", { className: `w-3 h-3 rounded-full ${port.type === 'input' ? 'bg-blue-500' : 'bg-green-500'}` }), _jsx("span", { className: "text-gray-900", children: port.label }), _jsxs("span", { className: "text-gray-500 text-xs", children: ["(", port.dataType, ")"] })] }, port.id))) })] }))] })] }));
}
function ParameterInput({ parameter, value, error, onChange, readOnly }) {
    const handleChange = (newValue) => {
        if (readOnly)
            return;
        onChange(newValue);
    };
    const renderInput = () => {
        switch (parameter.type) {
            case 'boolean':
                return (_jsxs("label", { className: "flex items-center gap-2 cursor-pointer", children: [_jsx("input", { type: "checkbox", checked: value ?? parameter.default ?? false, onChange: (e) => handleChange(e.target.checked), disabled: readOnly, className: "rounded border-gray-300 text-blue-600 focus:ring-blue-500" }), _jsx("span", { className: "text-sm text-gray-700", children: value ? 'Enabled' : 'Disabled' })] }));
            case 'number':
                return (_jsx("input", { type: "number", value: value ?? parameter.default ?? '', onChange: (e) => handleChange(Number(e.target.value)), min: parameter.validation?.min, max: parameter.validation?.max, disabled: readOnly, className: "w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100" }));
            case 'string':
                if (parameter.validation?.options) {
                    // Select dropdown for options
                    return (_jsxs("select", { value: value ?? parameter.default ?? '', onChange: (e) => handleChange(e.target.value), disabled: readOnly, className: "w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100", children: [_jsx("option", { value: "", children: "Select an option..." }), parameter.validation.options.map(option => (_jsx("option", { value: option, children: option }, option)))] }));
                }
                else {
                    // Text input
                    return (_jsx("input", { type: "text", value: value ?? parameter.default ?? '', onChange: (e) => handleChange(e.target.value), disabled: readOnly, className: "w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100" }));
                }
            case 'did':
                return (_jsx("input", { type: "text", value: value ?? parameter.default ?? '', onChange: (e) => handleChange(e.target.value), placeholder: "did:method:identifier", disabled: readOnly, className: "w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100" }));
            case 'duration':
                return (_jsxs("div", { className: "flex gap-2", children: [_jsx("input", { type: "number", value: value ?? parameter.default ?? '', onChange: (e) => handleChange(Number(e.target.value)), min: parameter.validation?.min, max: parameter.validation?.max, disabled: readOnly, className: "flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100" }), _jsx("span", { className: "px-3 py-2 text-sm text-gray-600 bg-gray-100 rounded-md", children: "days" })] }));
            default:
                return (_jsx("input", { type: "text", value: value ?? parameter.default ?? '', onChange: (e) => handleChange(e.target.value), disabled: readOnly, className: "w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100" }));
        }
    };
    return (_jsxs("div", { className: "space-y-2", children: [_jsxs("label", { className: "block", children: [_jsxs("div", { className: "flex items-center gap-2 mb-1", children: [_jsx("span", { className: "text-sm font-medium text-gray-900", children: parameter.name }), parameter.required && (_jsx("span", { className: "text-red-500 text-xs", children: "*" }))] }), parameter.description && (_jsx("p", { className: "text-xs text-gray-600 mb-2", children: parameter.description })), renderInput()] }), error && (_jsx("p", { className: "text-xs text-red-600 mt-1", children: error })), parameter.validation && !error && (_jsxs("div", { className: "text-xs text-gray-500", children: [parameter.validation.min !== undefined && parameter.validation.max !== undefined && (_jsxs("span", { children: ["Range: ", parameter.validation.min, " - ", parameter.validation.max] })), parameter.validation.pattern && (_jsxs("span", { children: ["Pattern: ", parameter.validation.pattern] }))] }))] }));
}
