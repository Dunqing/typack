import { type BlockCodegenNode, type CacheExpression, type DirectiveNode, type ElementNode, type ExpressionNode, type IfBranchNode, type InterpolationNode, type JSChildNode, type MemoExpression, type Position, type Property, type RenderSlotCall, type RootNode, type SimpleExpressionNode, type SlotOutletNode, type TemplateChildNode, type TemplateNode, type TextNode, type VNodeCall } from './ast';
import type { TransformContext } from './transform';
export declare const isStaticExp: (p: JSChildNode) => p is SimpleExpressionNode;
export declare function isCoreComponent(tag: string): symbol | void;
export declare const isSimpleIdentifier: (name: string) => boolean;
export declare const validFirstIdentCharRE: RegExp;
/**
 * Simple lexer to check if an expression is a member expression. This is
 * lax and only checks validity at the root level (i.e. does not validate exps
 * inside square brackets), but it's ok since these are only used on template
 * expressions and false positives are invalid expressions in the first place.
 */
export declare const isMemberExpressionBrowser: (exp: ExpressionNode) => boolean;
export declare const isMemberExpressionNode: (exp: ExpressionNode, context: TransformContext) => boolean;
export declare const isMemberExpression: (exp: ExpressionNode, context: TransformContext) => boolean;
export declare const isFnExpressionBrowser: (exp: ExpressionNode) => boolean;
export declare const isFnExpressionNode: (exp: ExpressionNode, context: TransformContext) => boolean;
export declare const isFnExpression: (exp: ExpressionNode, context: TransformContext) => boolean;
export declare function advancePositionWithClone(pos: Position, source: string, numberOfCharacters?: number): Position;
export declare function advancePositionWithMutation(pos: Position, source: string, numberOfCharacters?: number): Position;
export declare function assert(condition: boolean, msg?: string): void;
export declare function findDir(node: ElementNode, name: string | RegExp, allowEmpty?: boolean): DirectiveNode | undefined;
export declare function findProp(node: ElementNode, name: string, dynamicOnly?: boolean, allowEmpty?: boolean): ElementNode['props'][0] | undefined;
export declare function isStaticArgOf(arg: DirectiveNode['arg'], name: string): boolean;
export declare function hasDynamicKeyVBind(node: ElementNode): boolean;
export declare function isText(node: TemplateChildNode): node is TextNode | InterpolationNode;
export declare function isVPre(p: ElementNode['props'][0]): p is DirectiveNode;
export declare function isVSlot(p: ElementNode['props'][0]): p is DirectiveNode;
export declare function isTemplateNode(node: RootNode | TemplateChildNode): node is TemplateNode;
export declare function isSlotOutlet(node: RootNode | TemplateChildNode): node is SlotOutletNode;
export declare function injectProp(node: VNodeCall | RenderSlotCall, prop: Property, context: TransformContext): void;
export declare function toValidAssetId(name: string, type: 'component' | 'directive' | 'filter'): string;
export declare function hasScopeRef(node: TemplateChildNode | IfBranchNode | ExpressionNode | CacheExpression | undefined, ids: TransformContext['identifiers']): boolean;
export declare function getMemoedVNodeCall(node: BlockCodegenNode | MemoExpression): VNodeCall | RenderSlotCall;
export declare const forAliasRE: RegExp;
export declare function isAllWhitespace(str: string): boolean;
export declare function isWhitespaceText(node: TemplateChildNode): boolean;
export declare function isCommentOrWhitespace(node: TemplateChildNode): boolean;
