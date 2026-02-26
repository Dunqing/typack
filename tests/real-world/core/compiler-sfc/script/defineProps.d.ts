import type { Expression, LVal, Node } from '@babel/types';
import type { ScriptCompileContext } from './context';
import { type TypeResolveContext } from './resolveType';
export declare const DEFINE_PROPS = "defineProps";
export declare const WITH_DEFAULTS = "withDefaults";
export interface PropTypeData {
    key: string;
    type: string[];
    required: boolean;
    skipCheck: boolean;
}
export type PropsDestructureBindings = Record<string, // public prop key
{
    local: string;
    default?: Expression;
}>;
export declare function processDefineProps(ctx: ScriptCompileContext, node: Node, declId?: LVal, isWithDefaults?: boolean): boolean;
export declare function genRuntimeProps(ctx: ScriptCompileContext): string | undefined;
export declare function extractRuntimeProps(ctx: TypeResolveContext): string | undefined;
