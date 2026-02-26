import { type SFCDescriptor, type SFCScriptBlock } from './parse';
import type { ParserPlugin } from '@babel/parser';
import { type RawSourceMap } from 'source-map-js';
import { type SFCTemplateCompileOptions } from './compileTemplate';
export interface SFCScriptCompileOptions {
    /**
     * Scope ID for prefixing injected CSS variables.
     * This must be consistent with the `id` passed to `compileStyle`.
     */
    id: string;
    /**
     * Production mode. Used to determine whether to generate hashed CSS variables
     */
    isProd?: boolean;
    /**
     * Enable/disable source map. Defaults to true.
     */
    sourceMap?: boolean;
    /**
     * https://babeljs.io/docs/en/babel-parser#plugins
     */
    babelParserPlugins?: ParserPlugin[];
    /**
     * A list of files to parse for global types to be made available for type
     * resolving in SFC macros. The list must be fully resolved file system paths.
     */
    globalTypeFiles?: string[];
    /**
     * Compile the template and inline the resulting render function
     * directly inside setup().
     * - Only affects `<script setup>`
     * - This should only be used in production because it prevents the template
     * from being hot-reloaded separately from component state.
     */
    inlineTemplate?: boolean;
    /**
     * Generate the final component as a variable instead of default export.
     * This is useful in e.g. @vitejs/plugin-vue where the script needs to be
     * placed inside the main module.
     */
    genDefaultAs?: string;
    /**
     * Options for template compilation when inlining. Note these are options that
     * would normally be passed to `compiler-sfc`'s own `compileTemplate()`, not
     * options passed to `compiler-dom`.
     */
    templateOptions?: Partial<SFCTemplateCompileOptions>;
    /**
     * Hoist <script setup> static constants.
     * - Only enables when one `<script setup>` exists.
     * @default true
     */
    hoistStatic?: boolean;
    /**
     * Set to `false` to disable reactive destructure for `defineProps` (pre-3.5
     * behavior), or set to `'error'` to throw hard error on props destructures.
     * @default true
     */
    propsDestructure?: boolean | 'error';
    /**
     * File system access methods to be used when resolving types
     * imported in SFC macros. Defaults to ts.sys in Node.js, can be overwritten
     * to use a virtual file system for use in browsers (e.g. in REPLs)
     */
    fs?: {
        fileExists(file: string): boolean;
        readFile(file: string): string | undefined;
        realpath?(file: string): string;
    };
    /**
     * Transform Vue SFCs into custom elements.
     */
    customElement?: boolean | ((filename: string) => boolean);
}
export interface ImportBinding {
    isType: boolean;
    imported: string;
    local: string;
    source: string;
    isFromSetup: boolean;
    isUsedInTemplate: boolean;
}
/**
 * Compile `<script setup>`
 * It requires the whole SFC descriptor because we need to handle and merge
 * normal `<script>` + `<script setup>` if both are present.
 */
export declare function compileScript(sfc: SFCDescriptor, options: SFCScriptCompileOptions): SFCScriptBlock;
export declare function mergeSourceMaps(scriptMap: RawSourceMap, templateMap: RawSourceMap, templateLineOffset: number): RawSourceMap;
