import { type CompilerError, type SourceLocation } from '@vue/compiler-dom';
export interface SSRCompilerError extends CompilerError {
    code: SSRErrorCodes;
}
export declare function createSSRCompilerError(code: SSRErrorCodes, loc?: SourceLocation): SSRCompilerError;
export declare enum SSRErrorCodes {
    X_SSR_UNSAFE_ATTR_NAME = 65,
    X_SSR_NO_TELEPORT_TARGET = 66,
    X_SSR_INVALID_AST_NODE = 67
}
export declare const SSRErrorMessages: {
    [code: number]: string;
};
