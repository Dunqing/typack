import type { TransformOptions } from './options';
import { type ArrayExpression, type CacheExpression, ConstantTypes, type DirectiveNode, type ElementNode, type ExpressionNode, type JSChildNode, type ParentNode, type Property, type RootNode, type SimpleExpressionNode, type TemplateChildNode, type TemplateLiteral } from './ast';
import type { CompilerCompatOptions } from './compat/compatConfig';
export type NodeTransform = (node: RootNode | TemplateChildNode, context: TransformContext) => void | (() => void) | (() => void)[];
export type DirectiveTransform = (dir: DirectiveNode, node: ElementNode, context: TransformContext, augmentor?: (ret: DirectiveTransformResult) => DirectiveTransformResult) => DirectiveTransformResult;
export interface DirectiveTransformResult {
    props: Property[];
    needRuntime?: boolean | symbol;
    ssrTagParts?: TemplateLiteral['elements'];
}
export type StructuralDirectiveTransform = (node: ElementNode, dir: DirectiveNode, context: TransformContext) => void | (() => void);
export interface ImportItem {
    exp: string | ExpressionNode;
    path: string;
}
export interface TransformContext extends Required<Omit<TransformOptions, keyof CompilerCompatOptions>>, CompilerCompatOptions {
    selfName: string | null;
    root: RootNode;
    helpers: Map<symbol, number>;
    components: Set<string>;
    directives: Set<string>;
    hoists: (JSChildNode | null)[];
    imports: ImportItem[];
    temps: number;
    cached: (CacheExpression | null)[];
    identifiers: {
        [name: string]: number | undefined;
    };
    scopes: {
        vFor: number;
        vSlot: number;
        vPre: number;
        vOnce: number;
    };
    parent: ParentNode | null;
    grandParent: ParentNode | null;
    childIndex: number;
    currentNode: RootNode | TemplateChildNode | null;
    inVOnce: boolean;
    helper<T extends symbol>(name: T): T;
    removeHelper<T extends symbol>(name: T): void;
    helperString(name: symbol): string;
    replaceNode(node: TemplateChildNode): void;
    removeNode(node?: TemplateChildNode): void;
    onNodeRemoved(): void;
    addIdentifiers(exp: ExpressionNode | string): void;
    removeIdentifiers(exp: ExpressionNode | string): void;
    hoist(exp: string | JSChildNode | ArrayExpression): SimpleExpressionNode;
    cache(exp: JSChildNode, isVNode?: boolean, inVOnce?: boolean): CacheExpression;
    constantCache: WeakMap<TemplateChildNode, ConstantTypes>;
    filters?: Set<string>;
}
export declare function createTransformContext(root: RootNode, { filename, prefixIdentifiers, hoistStatic, hmr, cacheHandlers, nodeTransforms, directiveTransforms, transformHoist, isBuiltInComponent, isCustomElement, expressionPlugins, scopeId, slotted, ssr, inSSR, ssrCssVars, bindingMetadata, inline, isTS, onError, onWarn, compatConfig, }: TransformOptions): TransformContext;
export declare function transform(root: RootNode, options: TransformOptions): void;
export declare function traverseChildren(parent: ParentNode, context: TransformContext): void;
export declare function traverseNode(node: RootNode | TemplateChildNode, context: TransformContext): void;
export declare function createStructuralDirectiveTransform(name: string | RegExp, fn: StructuralDirectiveTransform): NodeTransform;
