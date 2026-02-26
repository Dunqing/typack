import type { SFCDescriptor } from '../parse';
/**
 * Check if an import is used in the SFC's template. This is used to determine
 * the properties that should be included in the object returned from setup()
 * when not using inline mode.
 */
export declare function isImportUsed(local: string, sfc: SFCDescriptor): boolean;
export declare function resolveTemplateVModelIdentifiers(sfc: SFCDescriptor): Set<string>;
