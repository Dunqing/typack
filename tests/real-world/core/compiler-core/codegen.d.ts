import type { CodegenOptions } from './options';
import { type JSChildNode, type RootNode, type SSRCodegenNode, type TemplateChildNode } from './ast';
/**
 * The `SourceMapGenerator` type from `source-map-js` is a bit incomplete as it
 * misses `toJSON()`. We also need to add types for internal properties which we
 * need to access for better performance.
 *
 * Since TS 5.3, dts generation starts to strangely include broken triple slash
 * references for source-map-js, so we are inlining all source map related types
 * here to to workaround that.
 */
export interface CodegenSourceMapGenerator {
    setSourceContent(sourceFile: string, sourceContent: string): void;
    toJSON(): RawSourceMap;
    _sources: Set<string>;
    _names: Set<string>;
    _mappings: {
        add(mapping: MappingItem): void;
    };
}
export interface RawSourceMap {
    file?: string;
    sourceRoot?: string;
    version: string;
    sources: string[];
    names: string[];
    sourcesContent?: string[];
    mappings: string;
}
interface MappingItem {
    source: string;
    generatedLine: number;
    generatedColumn: number;
    originalLine: number;
    originalColumn: number;
    name: string | null;
}
type CodegenNode = TemplateChildNode | JSChildNode | SSRCodegenNode;
export interface CodegenResult {
    code: string;
    preamble: string;
    ast: RootNode;
    map?: RawSourceMap;
}
export interface CodegenContext extends Omit<Required<CodegenOptions>, 'bindingMetadata' | 'inline'> {
    source: string;
    code: string;
    line: number;
    column: number;
    offset: number;
    indentLevel: number;
    pure: boolean;
    map?: CodegenSourceMapGenerator;
    helper(key: symbol): string;
    push(code: string, newlineIndex?: number, node?: CodegenNode): void;
    indent(): void;
    deindent(withoutNewLine?: boolean): void;
    newline(): void;
}
export declare function generate(ast: RootNode, options?: CodegenOptions & {
    onContextCreated?: (context: CodegenContext) => void;
}): CodegenResult;
export {};
