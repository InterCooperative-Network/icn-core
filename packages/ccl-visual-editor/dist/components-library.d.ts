import type { PaletteComponent } from './types';
export declare const GOVERNANCE_COMPONENTS: PaletteComponent[];
export declare const COMPONENT_CATEGORIES: {
    id: string;
    name: string;
    icon: string;
    color: string;
}[];
export declare function getComponentsByCategory(category: string): PaletteComponent[];
export declare function getComponentById(id: string): PaletteComponent | undefined;
export declare function searchComponents(searchTerm: string): PaletteComponent[];
