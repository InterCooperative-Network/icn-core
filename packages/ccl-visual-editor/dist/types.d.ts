import type { CCLTemplate, CCLParameter } from '@icn/ts-sdk';
export interface Position {
    x: number;
    y: number;
}
export interface Size {
    width: number;
    height: number;
}
export interface ComponentPort {
    id: string;
    label: string;
    type: 'input' | 'output';
    dataType: 'data' | 'control' | 'vote' | 'proposal';
}
export interface PaletteComponent {
    id: string;
    category: 'governance' | 'economics' | 'identity' | 'logic';
    name: string;
    description: string;
    icon: string;
    template?: CCLTemplate;
    ports: ComponentPort[];
    defaultConfig: Record<string, any>;
    parameters: CCLParameter[];
}
export interface CanvasNode {
    id: string;
    type: 'component' | 'logic' | 'data';
    position: Position;
    size: Size;
    component: PaletteComponent;
    config: Record<string, any>;
    ports: ComponentPort[];
    selected?: boolean;
    dragging?: boolean;
}
export interface Connection {
    id: string;
    sourceNodeId: string;
    targetNodeId: string;
    sourcePortId: string;
    targetPortId: string;
}
export interface VisualContract {
    id: string;
    name: string;
    description: string;
    nodes: CanvasNode[];
    connections: Connection[];
    metadata: {
        created: Date;
        modified: Date;
        version: string;
    };
}
export interface ValidationError {
    nodeId?: string;
    connectionId?: string;
    message: string;
    severity: 'error' | 'warning' | 'info';
}
export interface CCLGenerationResult {
    code: string;
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationError[];
}
export interface VisualEditorProps {
    initialContract?: VisualContract;
    readOnly?: boolean;
    onContractChange?: (contract: VisualContract) => void;
    onCodeGenerated?: (result: CCLGenerationResult) => void;
    onContractDeploy?: (code: string) => Promise<void>;
    className?: string;
}
export interface ComponentPaletteProps {
    components: PaletteComponent[];
    onComponentSelect?: (component: PaletteComponent) => void;
    searchTerm?: string;
    selectedCategory?: string;
    className?: string;
}
export interface CanvasAreaProps {
    nodes: CanvasNode[];
    connections: Connection[];
    onNodeCreate?: (component: PaletteComponent, position: Position) => void;
    onNodeUpdate?: (nodeId: string, updates: Partial<CanvasNode>) => void;
    onNodeDelete?: (nodeId: string) => void;
    onNodeSelect?: (nodeId: string | null) => void;
    onConnectionCreate?: (connection: Omit<Connection, 'id'>) => void;
    onConnectionDelete?: (connectionId: string) => void;
    readOnly?: boolean;
    className?: string;
}
export interface PropertyInspectorProps {
    selectedNode: CanvasNode | null;
    onPropertyChange?: (nodeId: string, property: string, value: any) => void;
    readOnly?: boolean;
    className?: string;
}
export interface CodePreviewProps {
    contract: VisualContract;
    generationResult?: CCLGenerationResult;
    onRefresh?: () => void;
    onCopy?: () => void;
    className?: string;
}
export interface DeploymentPanelProps {
    cclCode: string;
    onDeploy?: (code: string) => Promise<void>;
    deploymentStatus?: 'idle' | 'deploying' | 'success' | 'error';
    deploymentResult?: any;
    className?: string;
}
