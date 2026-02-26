export declare function initSyntaxLexers(): Promise<void>;
export declare function transformCode(code: string, filename: string): string;
export declare function collectModuleExports(filename: string, code: string, format: 'module' | 'commonjs', exports?: string[]): string[];
export declare function resolveModuleFormat(url: string, code: string): 'module' | 'commonjs' | undefined;
