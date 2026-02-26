import { type BindingMetadata } from '@vue/compiler-dom';
import type { SFCDescriptor } from '../parse';
import type { PluginCreator } from 'postcss';
export declare const CSS_VARS_HELPER = "useCssVars";
export declare function genCssVarsFromList(vars: string[], id: string, isProd: boolean, isSSR?: boolean): string;
export declare function parseCssVars(sfc: SFCDescriptor): string[];
export interface CssVarsPluginOptions {
    id: string;
    isProd: boolean;
}
export declare const cssVarsPlugin: PluginCreator<CssVarsPluginOptions>;
export declare function genCssVarsCode(vars: string[], bindings: BindingMetadata, id: string, isProd: boolean): string;
export declare function genNormalScriptCssVarsCode(cssVars: string[], bindings: BindingMetadata, id: string, isProd: boolean, defaultVar: string): string;
