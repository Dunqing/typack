import type { ScriptCompileContext } from './context';
import type { SFCScriptBlock } from '../parse';
export declare const normalScriptDefaultVar = "__default__";
export declare function processNormalScript(ctx: ScriptCompileContext, scopeId: string): SFCScriptBlock;
