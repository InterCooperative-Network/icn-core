import type { VisualContract, CCLGenerationResult } from './types';
export declare class CCLGenerator {
    static generateFromContract(contract: VisualContract): CCLGenerationResult;
    private static validateContract;
    private static validateNode;
    private static validateConnection;
    private static buildCCLCode;
}
