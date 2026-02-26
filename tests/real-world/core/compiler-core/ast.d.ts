import { type PatchFlags } from '@vue/shared';
import { CREATE_BLOCK, CREATE_ELEMENT_BLOCK, CREATE_ELEMENT_VNODE, type CREATE_SLOTS, CREATE_VNODE, type FRAGMENT, type RENDER_LIST, type RENDER_SLOT, type WITH_MEMO } from './runtimeHelpers';
import type { PropsExpression } from './transforms/transformElement';
import type { ImportItem, TransformContext } from './transform';
import type { Node as BabelNode } from '@babel/types';
export type Namespace = number;
export declare enum Namespaces {
    HTML = 0,
    SVG = 1,
    MATH_ML = 2
}
export declare enum NodeTypes {
    ROOT = 0,
    ELEMENT = 1,
    TEXT = 2,
    COMMENT = 3,
    SIMPLE_EXPRESSION = 4,
    INTERPOLATION = 5,
    ATTRIBUTE = 6,
    DIRECTIVE = 7,
    COMPOUND_EXPRESSION = 8,
    IF = 9,
    IF_BRANCH = 10,
    FOR = 11,
    TEXT_CALL = 12,
    VNODE_CALL = 13,
    JS_CALL_EXPRESSION = 14,
    JS_OBJECT_EXPRESSION = 15,
    JS_PROPERTY = 16,
    JS_ARRAY_EXPRESSION = 17,
    JS_FUNCTION_EXPRESSION = 18,
    JS_CONDITIONAL_EXPRESSION = 19,
    JS_CACHE_EXPRESSION = 20,
    JS_BLOCK_STATEMENT = 21,
    JS_TEMPLATE_LITERAL = 22,
    JS_IF_STATEMENT = 23,
    JS_ASSIGNMENT_EXPRESSION = 24,
    JS_SEQUENCE_EXPRESSION = 25,
    JS_RETURN_STATEMENT = 26
}
export declare enum ElementTypes {
    ELEMENT = 0,
    COMPONENT = 1,
    SLOT = 2,
    TEMPLATE = 3
}
export interface Node {
    type: NodeTypes;
    loc: SourceLocation;
}
export interface SourceLocation {
    start: Position;
    end: Position;
    source: string;
}
export interface Position {
    offset: number;
    line: number;
    column: number;
}
export type ParentNode = RootNode | ElementNode | IfBranchNode | ForNode;
export type ExpressionNode = SimpleExpressionNode | CompoundExpressionNode;
export type TemplateChildNode = ElementNode | InterpolationNode | CompoundExpressionNode | TextNode | CommentNode | IfNode | IfBranchNode | ForNode | TextCallNode;
export interface RootNode extends Node {
    type: NodeTypes.ROOT;
    source: string;
    children: TemplateChildNode[];
    helpers: Set<symbol>;
    components: string[];
    directives: string[];
    hoists: (JSChildNode | null)[];
    imports: ImportItem[];
    cached: (CacheExpression | null)[];
    temps: number;
    ssrHelpers?: symbol[];
    codegenNode?: TemplateChildNode | JSChildNode | BlockStatement;
    transformed?: boolean;
    filters?: string[];
}
export type ElementNode = PlainElementNode | ComponentNode | SlotOutletNode | TemplateNode;
export interface BaseElementNode extends Node {
    type: NodeTypes.ELEMENT;
    ns: Namespace;
    tag: string;
    tagType: ElementTypes;
    props: Array<AttributeNode | DirectiveNode>;
    children: TemplateChildNode[];
    isSelfClosing?: boolean;
    innerLoc?: SourceLocation;
}
export interface PlainElementNode extends BaseElementNode {
    tagType: ElementTypes.ELEMENT;
    codegenNode: VNodeCall | SimpleExpressionNode | CacheExpression | MemoExpression | undefined;
    ssrCodegenNode?: TemplateLiteral;
}
export interface ComponentNode extends BaseElementNode {
    tagType: ElementTypes.COMPONENT;
    codegenNode: VNodeCall | CacheExpression | MemoExpression | undefined;
    ssrCodegenNode?: CallExpression;
}
export interface SlotOutletNode extends BaseElementNode {
    tagType: ElementTypes.SLOT;
    codegenNode: RenderSlotCall | CacheExpression | undefined;
    ssrCodegenNode?: CallExpression;
}
export interface TemplateNode extends BaseElementNode {
    tagType: ElementTypes.TEMPLATE;
    codegenNode: undefined;
}
export interface TextNode extends Node {
    type: NodeTypes.TEXT;
    content: string;
}
export interface CommentNode extends Node {
    type: NodeTypes.COMMENT;
    content: string;
}
export interface AttributeNode extends Node {
    type: NodeTypes.ATTRIBUTE;
    name: string;
    nameLoc: SourceLocation;
    value: TextNode | undefined;
}
export interface DirectiveNode extends Node {
    type: NodeTypes.DIRECTIVE;
    /**
     * the normalized name without prefix or shorthands, e.g. "bind", "on"
     */
    name: string;
    /**
     * the raw attribute name, preserving shorthand, and including arg & modifiers
     * this is only used during parse.
     */
    rawName?: string;
    exp: ExpressionNode | undefined;
    arg: ExpressionNode | undefined;
    modifiers: SimpleExpressionNode[];
    /**
     * optional property to cache the expression parse result for v-for
     */
    forParseResult?: ForParseResult;
}
/**
 * Static types have several levels.
 * Higher levels implies lower levels. e.g. a node that can be stringified
 * can always be hoisted and skipped for patch.
 */
export declare enum ConstantTypes {
    NOT_CONSTANT = 0,
    CAN_SKIP_PATCH = 1,
    CAN_CACHE = 2,
    CAN_STRINGIFY = 3
}
export interface SimpleExpressionNode extends Node {
    type: NodeTypes.SIMPLE_EXPRESSION;
    content: string;
    isStatic: boolean;
    constType: ConstantTypes;
    /**
     * - `null` means the expression is a simple identifier that doesn't need
     *    parsing
     * - `false` means there was a parsing error
     */
    ast?: BabelNode | null | false;
    /**
     * Indicates this is an identifier for a hoist vnode call and points to the
     * hoisted node.
     */
    hoisted?: JSChildNode;
    /**
     * an expression parsed as the params of a function will track
     * the identifiers declared inside the function body.
     */
    identifiers?: string[];
    isHandlerKey?: boolean;
}
export interface InterpolationNode extends Node {
    type: NodeTypes.INTERPOLATION;
    content: ExpressionNode;
}
export interface CompoundExpressionNode extends Node {
    type: NodeTypes.COMPOUND_EXPRESSION;
    /**
     * - `null` means the expression is a simple identifier that doesn't need
     *    parsing
     * - `false` means there was a parsing error
     */
    ast?: BabelNode | null | false;
    children: (SimpleExpressionNode | CompoundExpressionNode | InterpolationNode | TextNode | string | symbol)[];
    /**
     * an expression parsed as the params of a function will track
     * the identifiers declared inside the function body.
     */
    identifiers?: string[];
    isHandlerKey?: boolean;
}
export interface IfNode extends Node {
    type: NodeTypes.IF;
    branches: IfBranchNode[];
    codegenNode?: IfConditionalExpression | CacheExpression;
}
export interface IfBranchNode extends Node {
    type: NodeTypes.IF_BRANCH;
    condition: ExpressionNode | undefined;
    children: TemplateChildNode[];
    userKey?: AttributeNode | DirectiveNode;
    isTemplateIf?: boolean;
}
export interface ForNode extends Node {
    type: NodeTypes.FOR;
    source: ExpressionNode;
    valueAlias: ExpressionNode | undefined;
    keyAlias: ExpressionNode | undefined;
    objectIndexAlias: ExpressionNode | undefined;
    parseResult: ForParseResult;
    children: TemplateChildNode[];
    codegenNode?: ForCodegenNode;
}
export interface ForParseResult {
    source: ExpressionNode;
    value: ExpressionNode | undefined;
    key: ExpressionNode | undefined;
    index: ExpressionNode | undefined;
    finalized: boolean;
}
export interface TextCallNode extends Node {
    type: NodeTypes.TEXT_CALL;
    content: TextNode | InterpolationNode | CompoundExpressionNode;
    codegenNode: CallExpression | SimpleExpressionNode;
}
export type TemplateTextChildNode = TextNode | InterpolationNode | CompoundExpressionNode;
export interface VNodeCall extends Node {
    type: NodeTypes.VNODE_CALL;
    tag: string | symbol | CallExpression;
    props: PropsExpression | undefined;
    children: TemplateChildNode[] | TemplateTextChildNode | SlotsExpression | ForRenderListExpression | SimpleExpressionNode | CacheExpression | undefined;
    patchFlag: PatchFlags | undefined;
    dynamicProps: string | SimpleExpressionNode | undefined;
    directives: DirectiveArguments | undefined;
    isBlock: boolean;
    disableTracking: boolean;
    isComponent: boolean;
}
export type JSChildNode = VNodeCall | CallExpression | ObjectExpression | ArrayExpression | ExpressionNode | FunctionExpression | ConditionalExpression | CacheExpression | AssignmentExpression | SequenceExpression;
export interface CallExpression extends Node {
    type: NodeTypes.JS_CALL_EXPRESSION;
    callee: string | symbol;
    arguments: (string | symbol | JSChildNode | SSRCodegenNode | TemplateChildNode | TemplateChildNode[])[];
}
export interface ObjectExpression extends Node {
    type: NodeTypes.JS_OBJECT_EXPRESSION;
    properties: Array<Property>;
}
export interface Property extends Node {
    type: NodeTypes.JS_PROPERTY;
    key: ExpressionNode;
    value: JSChildNode;
}
export interface ArrayExpression extends Node {
    type: NodeTypes.JS_ARRAY_EXPRESSION;
    elements: Array<string | Node>;
}
export interface FunctionExpression extends Node {
    type: NodeTypes.JS_FUNCTION_EXPRESSION;
    params: ExpressionNode | string | (ExpressionNode | string)[] | undefined;
    returns?: TemplateChildNode | TemplateChildNode[] | JSChildNode;
    body?: BlockStatement | IfStatement;
    newline: boolean;
    /**
     * This flag is for codegen to determine whether it needs to generate the
     * withScopeId() wrapper
     */
    isSlot: boolean;
    /**
     * __COMPAT__ only, indicates a slot function that should be excluded from
     * the legacy $scopedSlots instance property.
     */
    isNonScopedSlot?: boolean;
}
export interface ConditionalExpression extends Node {
    type: NodeTypes.JS_CONDITIONAL_EXPRESSION;
    test: JSChildNode;
    consequent: JSChildNode;
    alternate: JSChildNode;
    newline: boolean;
}
export interface CacheExpression extends Node {
    type: NodeTypes.JS_CACHE_EXPRESSION;
    index: number;
    value: JSChildNode;
    needPauseTracking: boolean;
    inVOnce: boolean;
    needArraySpread: boolean;
}
export interface MemoExpression extends CallExpression {
    callee: typeof WITH_MEMO;
    arguments: [ExpressionNode, MemoFactory, string, string];
}
interface MemoFactory extends FunctionExpression {
    returns: BlockCodegenNode;
}
export type SSRCodegenNode = BlockStatement | TemplateLiteral | IfStatement | AssignmentExpression | ReturnStatement | SequenceExpression;
export interface BlockStatement extends Node {
    type: NodeTypes.JS_BLOCK_STATEMENT;
    body: (JSChildNode | IfStatement)[];
}
export interface TemplateLiteral extends Node {
    type: NodeTypes.JS_TEMPLATE_LITERAL;
    elements: (string | JSChildNode)[];
}
export interface IfStatement extends Node {
    type: NodeTypes.JS_IF_STATEMENT;
    test: ExpressionNode;
    consequent: BlockStatement;
    alternate: IfStatement | BlockStatement | ReturnStatement | undefined;
}
export interface AssignmentExpression extends Node {
    type: NodeTypes.JS_ASSIGNMENT_EXPRESSION;
    left: SimpleExpressionNode;
    right: JSChildNode;
}
export interface SequenceExpression extends Node {
    type: NodeTypes.JS_SEQUENCE_EXPRESSION;
    expressions: JSChildNode[];
}
export interface ReturnStatement extends Node {
    type: NodeTypes.JS_RETURN_STATEMENT;
    returns: TemplateChildNode | TemplateChildNode[] | JSChildNode;
}
export interface DirectiveArguments extends ArrayExpression {
    elements: DirectiveArgumentNode[];
}
export interface DirectiveArgumentNode extends ArrayExpression {
    elements: [string] | [string, ExpressionNode] | [string, ExpressionNode, ExpressionNode] | [string, ExpressionNode, ExpressionNode, ObjectExpression];
}
export interface RenderSlotCall extends CallExpression {
    callee: typeof RENDER_SLOT;
    arguments: [string, string | ExpressionNode] | [string, string | ExpressionNode, PropsExpression] | [
        string,
        string | ExpressionNode,
        PropsExpression | '{}',
        TemplateChildNode[]
    ];
}
export type SlotsExpression = SlotsObjectExpression | DynamicSlotsExpression;
export interface SlotsObjectExpression extends ObjectExpression {
    properties: SlotsObjectProperty[];
}
export interface SlotsObjectProperty extends Property {
    value: SlotFunctionExpression;
}
export interface SlotFunctionExpression extends FunctionExpression {
    returns: TemplateChildNode[] | CacheExpression;
}
export interface DynamicSlotsExpression extends CallExpression {
    callee: typeof CREATE_SLOTS;
    arguments: [SlotsObjectExpression, DynamicSlotEntries];
}
export interface DynamicSlotEntries extends ArrayExpression {
    elements: (ConditionalDynamicSlotNode | ListDynamicSlotNode)[];
}
export interface ConditionalDynamicSlotNode extends ConditionalExpression {
    consequent: DynamicSlotNode;
    alternate: DynamicSlotNode | SimpleExpressionNode;
}
export interface ListDynamicSlotNode extends CallExpression {
    callee: typeof RENDER_LIST;
    arguments: [ExpressionNode, ListDynamicSlotIterator];
}
export interface ListDynamicSlotIterator extends FunctionExpression {
    returns: DynamicSlotNode;
}
export interface DynamicSlotNode extends ObjectExpression {
    properties: [Property, DynamicSlotFnProperty];
}
export interface DynamicSlotFnProperty extends Property {
    value: SlotFunctionExpression;
}
export type BlockCodegenNode = VNodeCall | RenderSlotCall;
export interface IfConditionalExpression extends ConditionalExpression {
    consequent: BlockCodegenNode | MemoExpression;
    alternate: BlockCodegenNode | IfConditionalExpression | MemoExpression;
}
export interface ForCodegenNode extends VNodeCall {
    isBlock: true;
    tag: typeof FRAGMENT;
    props: undefined;
    children: ForRenderListExpression;
    patchFlag: PatchFlags;
    disableTracking: boolean;
}
export interface ForRenderListExpression extends CallExpression {
    callee: typeof RENDER_LIST;
    arguments: [ExpressionNode, ForIteratorExpression];
}
export interface ForIteratorExpression extends FunctionExpression {
    returns?: BlockCodegenNode;
}
export declare const locStub: SourceLocation;
export declare function createRoot(children: TemplateChildNode[], source?: string): RootNode;
export declare function createVNodeCall(context: TransformContext | null, tag: VNodeCall['tag'], props?: VNodeCall['props'], children?: VNodeCall['children'], patchFlag?: VNodeCall['patchFlag'], dynamicProps?: VNodeCall['dynamicProps'], directives?: VNodeCall['directives'], isBlock?: VNodeCall['isBlock'], disableTracking?: VNodeCall['disableTracking'], isComponent?: VNodeCall['isComponent'], loc?: SourceLocation): VNodeCall;
export declare function createArrayExpression(elements: ArrayExpression['elements'], loc?: SourceLocation): ArrayExpression;
export declare function createObjectExpression(properties: ObjectExpression['properties'], loc?: SourceLocation): ObjectExpression;
export declare function createObjectProperty(key: Property['key'] | string, value: Property['value']): Property;
export declare function createSimpleExpression(content: SimpleExpressionNode['content'], isStatic?: SimpleExpressionNode['isStatic'], loc?: SourceLocation, constType?: ConstantTypes): SimpleExpressionNode;
export declare function createInterpolation(content: InterpolationNode['content'] | string, loc: SourceLocation): InterpolationNode;
export declare function createCompoundExpression(children: CompoundExpressionNode['children'], loc?: SourceLocation): CompoundExpressionNode;
type InferCodegenNodeType<T> = T extends typeof RENDER_SLOT ? RenderSlotCall : CallExpression;
export declare function createCallExpression<T extends CallExpression['callee']>(callee: T, args?: CallExpression['arguments'], loc?: SourceLocation): InferCodegenNodeType<T>;
export declare function createFunctionExpression(params: FunctionExpression['params'], returns?: FunctionExpression['returns'], newline?: boolean, isSlot?: boolean, loc?: SourceLocation): FunctionExpression;
export declare function createConditionalExpression(test: ConditionalExpression['test'], consequent: ConditionalExpression['consequent'], alternate: ConditionalExpression['alternate'], newline?: boolean): ConditionalExpression;
export declare function createCacheExpression(index: number, value: JSChildNode, needPauseTracking?: boolean, inVOnce?: boolean): CacheExpression;
export declare function createBlockStatement(body: BlockStatement['body']): BlockStatement;
export declare function createTemplateLiteral(elements: TemplateLiteral['elements']): TemplateLiteral;
export declare function createIfStatement(test: IfStatement['test'], consequent: IfStatement['consequent'], alternate?: IfStatement['alternate']): IfStatement;
export declare function createAssignmentExpression(left: AssignmentExpression['left'], right: AssignmentExpression['right']): AssignmentExpression;
export declare function createSequenceExpression(expressions: SequenceExpression['expressions']): SequenceExpression;
export declare function createReturnStatement(returns: ReturnStatement['returns']): ReturnStatement;
export declare function getVNodeHelper(ssr: boolean, isComponent: boolean): typeof CREATE_VNODE | typeof CREATE_ELEMENT_VNODE;
export declare function getVNodeBlockHelper(ssr: boolean, isComponent: boolean): typeof CREATE_BLOCK | typeof CREATE_ELEMENT_BLOCK;
export declare function convertToBlock(node: VNodeCall, { helper, removeHelper, inSSR }: TransformContext): void;
export {};
