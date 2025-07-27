import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState, useEffect } from 'react';
import { CCLGenerator } from '../ccl-generator';
export function CodePreview({ contract, generationResult, onRefresh, onCopy, className = '' }) {
    const [result, setResult] = useState(null);
    const [isGenerating, setIsGenerating] = useState(false);
    const [showLineNumbers, setShowLineNumbers] = useState(true);
    const [activeTab, setActiveTab] = useState('code');
    // Generate CCL code when contract changes
    useEffect(() => {
        if (contract) {
            generateCode();
        }
    }, [contract]);
    // Use provided result if available
    useEffect(() => {
        if (generationResult) {
            setResult(generationResult);
        }
    }, [generationResult]);
    const generateCode = async () => {
        setIsGenerating(true);
        try {
            // Small delay to show loading state
            await new Promise(resolve => setTimeout(resolve, 100));
            const generatedResult = CCLGenerator.generateFromContract(contract);
            setResult(generatedResult);
        }
        catch (error) {
            setResult({
                code: '',
                valid: false,
                errors: [{
                        message: `Generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
                        severity: 'error'
                    }],
                warnings: []
            });
        }
        finally {
            setIsGenerating(false);
        }
    };
    const handleCopyCode = async () => {
        if (result?.code) {
            try {
                await navigator.clipboard.writeText(result.code);
                onCopy?.();
                // Could add toast notification here
            }
            catch (error) {
                console.error('Failed to copy code:', error);
            }
        }
    };
    const handleRefresh = () => {
        generateCode();
        onRefresh?.();
    };
    if (!result) {
        return (_jsx("div", { className: `code-preview bg-white border-t border-gray-200 flex flex-col ${className}`, children: _jsx("div", { className: "p-4 flex items-center justify-center text-gray-500", children: _jsx("p", { children: "No contract to preview" }) }) }));
    }
    const hasErrors = result.errors.length > 0;
    const hasWarnings = result.warnings.length > 0;
    const codeLines = result.code.split('\n');
    return (_jsxs("div", { className: `code-preview bg-white border-t border-gray-200 flex flex-col ${className}`, children: [_jsxs("div", { className: "p-3 border-b border-gray-200 bg-gray-50", children: [_jsxs("div", { className: "flex items-center justify-between", children: [_jsxs("div", { className: "flex items-center gap-3", children: [_jsx("h3", { className: "text-sm font-semibold text-gray-900", children: "Generated CCL Code" }), _jsx("div", { className: "flex items-center gap-2", children: isGenerating ? (_jsxs("div", { className: "flex items-center gap-2 text-blue-600", children: [_jsx("div", { className: "w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" }), _jsx("span", { className: "text-xs", children: "Generating..." })] })) : (_jsxs("div", { className: `flex items-center gap-1 text-xs px-2 py-1 rounded-full ${hasErrors
                                                ? 'bg-red-100 text-red-700'
                                                : hasWarnings
                                                    ? 'bg-yellow-100 text-yellow-700'
                                                    : 'bg-green-100 text-green-700'}`, children: [hasErrors ? '❌' : hasWarnings ? '⚠️' : '✅', hasErrors ? 'Errors' : hasWarnings ? 'Warnings' : 'Valid'] })) })] }), _jsxs("div", { className: "flex items-center gap-2", children: [_jsx("button", { onClick: () => setShowLineNumbers(!showLineNumbers), className: "px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded", title: "Toggle line numbers", children: "#" }), _jsx("button", { onClick: handleRefresh, disabled: isGenerating, className: "px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded disabled:opacity-50", title: "Refresh code", children: "\uD83D\uDD04" }), _jsx("button", { onClick: handleCopyCode, disabled: !result.code, className: "px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50", title: "Copy code", children: "Copy" })] })] }), _jsxs("div", { className: "flex gap-1 mt-3", children: [_jsx("button", { onClick: () => setActiveTab('code'), className: `px-3 py-1 text-xs rounded ${activeTab === 'code'
                                    ? 'bg-white text-gray-900 border border-gray-200'
                                    : 'text-gray-600 hover:text-gray-900'}`, children: "Code" }), (hasErrors || hasWarnings) && (_jsxs("button", { onClick: () => setActiveTab('errors'), className: `px-3 py-1 text-xs rounded ${activeTab === 'errors'
                                    ? 'bg-white text-gray-900 border border-gray-200'
                                    : 'text-gray-600 hover:text-gray-900'}`, children: ["Issues (", result.errors.length + result.warnings.length, ")"] }))] })] }), _jsx("div", { className: "flex-1 overflow-hidden", children: activeTab === 'code' ? (_jsx("div", { className: "h-full overflow-auto", children: result.code ? (_jsx("div", { className: "relative", children: _jsxs("pre", { className: "p-4 text-sm font-mono leading-relaxed", children: [showLineNumbers && (_jsx("div", { className: "absolute left-0 top-0 bottom-0 w-12 bg-gray-50 border-r border-gray-200 flex flex-col", children: codeLines.map((_, index) => (_jsx("div", { className: "px-2 py-0 text-xs text-gray-500 text-right", children: index + 1 }, index))) })), _jsx("code", { className: `block ${showLineNumbers ? 'ml-12' : ''}`, children: result.code })] }) })) : (_jsxs("div", { className: "p-8 text-center text-gray-500", children: [_jsx("p", { className: "text-sm", children: "No code generated yet" }), _jsx("p", { className: "text-xs mt-1", children: "Add some components to your contract" })] })) })) : (_jsx("div", { className: "h-full overflow-auto p-4", children: _jsxs("div", { className: "space-y-3", children: [result.errors.map((error, index) => (_jsx("div", { className: "p-3 bg-red-50 border border-red-200 rounded-lg", children: _jsxs("div", { className: "flex items-start gap-2", children: [_jsx("span", { className: "text-red-500 mt-0.5", children: "\u274C" }), _jsxs("div", { className: "flex-1", children: [_jsx("p", { className: "text-sm text-red-800 font-medium", children: "Error" }), _jsx("p", { className: "text-sm text-red-700 mt-1", children: error.message }), error.nodeId && (_jsxs("p", { className: "text-xs text-red-600 mt-1", children: ["Component: ", error.nodeId] }))] })] }) }, `error-${index}`))), result.warnings.map((warning, index) => (_jsx("div", { className: "p-3 bg-yellow-50 border border-yellow-200 rounded-lg", children: _jsxs("div", { className: "flex items-start gap-2", children: [_jsx("span", { className: "text-yellow-500 mt-0.5", children: "\u26A0\uFE0F" }), _jsxs("div", { className: "flex-1", children: [_jsx("p", { className: "text-sm text-yellow-800 font-medium", children: "Warning" }), _jsx("p", { className: "text-sm text-yellow-700 mt-1", children: warning.message }), warning.nodeId && (_jsxs("p", { className: "text-xs text-yellow-600 mt-1", children: ["Component: ", warning.nodeId] }))] })] }) }, `warning-${index}`))), !hasErrors && !hasWarnings && (_jsxs("div", { className: "p-3 bg-green-50 border border-green-200 rounded-lg", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "text-green-500", children: "\u2705" }), _jsx("p", { className: "text-sm text-green-800 font-medium", children: "No issues found" })] }), _jsx("p", { className: "text-sm text-green-700 mt-1", children: "Your contract code is valid and ready to deploy!" })] }))] }) })) }), _jsx("div", { className: "px-4 py-2 border-t border-gray-200 bg-gray-50", children: _jsxs("div", { className: "flex items-center justify-between text-xs text-gray-600", children: [_jsxs("div", { className: "flex items-center gap-4", children: [_jsxs("span", { children: ["Lines: ", codeLines.length] }), _jsxs("span", { children: ["Characters: ", result.code.length] })] }), _jsxs("div", { className: "flex items-center gap-4", children: [hasErrors && _jsxs("span", { className: "text-red-600", children: ["Errors: ", result.errors.length] }), hasWarnings && _jsxs("span", { className: "text-yellow-600", children: ["Warnings: ", result.warnings.length] }), _jsxs("span", { children: ["Last updated: ", new Date().toLocaleTimeString()] })] })] }) })] }));
}
