import { type BlockStatement, type CallExpression, type CompilerError, type CompilerOptions, type IfStatement, type JSChildNode, type RootNode, type TemplateChildNode, type TemplateLiteral } from '@vue/compiler-dom';
export declare function ssrCodegenTransform(ast: RootNode, options: CompilerOptions): void;
export interface SSRTransformContext {
    root: RootNode;
    options: CompilerOptions;
    body: (JSChildNode | IfStatement)[];
    helpers: Set<symbol>;
    withSlotScopeId: boolean;
    onError: (error: CompilerError) => void;
    helper<T extends symbol>(name: T): T;
    pushStringPart(part: TemplateLiteral['elements'][0]): void;
    pushStatement(statement: IfStatement | CallExpression): void;
}
interface Container {
    children: TemplateChildNode[];
}
export declare function processChildren(parent: Container, context: SSRTransformContext, asFragment?: boolean, disableNestedFragments?: boolean, disableComment?: boolean): void;
export declare function processChildrenAsStatement(parent: Container, parentContext: SSRTransformContext, asFragment?: boolean, withSlotScopeId?: boolean): BlockStatement;
export {};
