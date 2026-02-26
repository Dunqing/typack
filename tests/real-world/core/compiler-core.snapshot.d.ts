import { PatchFlags, generateCodeFrame } from "@vue/shared";
import { BlockStatement, Function, Identifier, Node, Node as BabelNode, ObjectProperty, Program, SwitchCase } from "@babel/types";
import { ParserPlugin } from "@babel/parser";

//#region tests/real-world/core/compiler-core/runtimeHelpers.d.ts
declare const FRAGMENT: unique symbol;
declare const TELEPORT: unique symbol;
declare const SUSPENSE: unique symbol;
declare const KEEP_ALIVE: unique symbol;
declare const BASE_TRANSITION: unique symbol;
declare const OPEN_BLOCK: unique symbol;
declare const CREATE_BLOCK: unique symbol;
declare const CREATE_ELEMENT_BLOCK: unique symbol;
declare const CREATE_VNODE: unique symbol;
declare const CREATE_ELEMENT_VNODE: unique symbol;
declare const CREATE_COMMENT: unique symbol;
declare const CREATE_TEXT: unique symbol;
declare const CREATE_STATIC: unique symbol;
declare const RESOLVE_COMPONENT: unique symbol;
declare const RESOLVE_DYNAMIC_COMPONENT: unique symbol;
declare const RESOLVE_DIRECTIVE: unique symbol;
declare const RESOLVE_FILTER: unique symbol;
declare const WITH_DIRECTIVES: unique symbol;
declare const RENDER_LIST: unique symbol;
declare const RENDER_SLOT: unique symbol;
declare const CREATE_SLOTS: unique symbol;
declare const TO_DISPLAY_STRING: unique symbol;
declare const MERGE_PROPS: unique symbol;
declare const NORMALIZE_CLASS: unique symbol;
declare const NORMALIZE_STYLE: unique symbol;
declare const NORMALIZE_PROPS: unique symbol;
declare const GUARD_REACTIVE_PROPS: unique symbol;
declare const TO_HANDLERS: unique symbol;
declare const CAMELIZE: unique symbol;
declare const CAPITALIZE: unique symbol;
declare const TO_HANDLER_KEY: unique symbol;
declare const SET_BLOCK_TRACKING: unique symbol;
/**
* @deprecated no longer needed in 3.5+ because we no longer hoist element nodes
* but kept for backwards compat
*/
declare const PUSH_SCOPE_ID: unique symbol;
/**
* @deprecated kept for backwards compat
*/
declare const POP_SCOPE_ID: unique symbol;
declare const WITH_CTX: unique symbol;
declare const UNREF: unique symbol;
declare const IS_REF: unique symbol;
declare const WITH_MEMO: unique symbol;
declare const IS_MEMO_SAME: unique symbol;
declare const helperNameMap: Record<symbol, string>;
declare function registerRuntimeHelpers(helpers: Record<symbol, string>): void;
//#endregion
//#region tests/real-world/core/compiler-core/parser.d.ts
type OptionalOptions = "decodeEntities" | "whitespace" | "isNativeTag" | "isBuiltInComponent" | "expressionPlugins" | keyof CompilerCompatOptions;
type MergedParserOptions = Omit<Required<ParserOptions>, OptionalOptions> & Pick<ParserOptions, OptionalOptions>;
declare function baseParse(input: string, options?: ParserOptions): RootNode;
//#endregion
//#region tests/real-world/core/compiler-core/compat/compatConfig.d.ts
type CompilerCompatConfig = Partial<Record<CompilerDeprecationTypes, boolean | "suppress-warning">> & {
  MODE?: 2 | 3;
};
interface CompilerCompatOptions {
  compatConfig?: CompilerCompatConfig;
}
declare enum CompilerDeprecationTypes {
  COMPILER_IS_ON_ELEMENT = "COMPILER_IS_ON_ELEMENT",
  COMPILER_V_BIND_SYNC = "COMPILER_V_BIND_SYNC",
  COMPILER_V_BIND_OBJECT_ORDER = "COMPILER_V_BIND_OBJECT_ORDER",
  COMPILER_V_ON_NATIVE = "COMPILER_V_ON_NATIVE",
  COMPILER_V_IF_V_FOR_PRECEDENCE = "COMPILER_V_IF_V_FOR_PRECEDENCE",
  COMPILER_NATIVE_TEMPLATE = "COMPILER_NATIVE_TEMPLATE",
  COMPILER_INLINE_TEMPLATE = "COMPILER_INLINE_TEMPLATE",
  COMPILER_FILTERS = "COMPILER_FILTERS"
}
declare function checkCompatEnabled(key: CompilerDeprecationTypes, context: MergedParserOptions | TransformContext, loc: SourceLocation | null, ...args: any[]): boolean;
declare function warnDeprecation(key: CompilerDeprecationTypes, context: MergedParserOptions | TransformContext, loc: SourceLocation | null, ...args: any[]): void;
//#endregion
//#region tests/real-world/core/compiler-core/transform.d.ts
type NodeTransform = (node: RootNode | TemplateChildNode, context: TransformContext) => void | (() => void) | (() => void)[];
type DirectiveTransform = (dir: DirectiveNode, node: ElementNode, context: TransformContext, augmentor?: (ret: DirectiveTransformResult) => DirectiveTransformResult) => DirectiveTransformResult;
interface DirectiveTransformResult {
  props: Property[];
  needRuntime?: boolean | symbol;
  ssrTagParts?: TemplateLiteral["elements"];
}
type StructuralDirectiveTransform = (node: ElementNode, dir: DirectiveNode, context: TransformContext) => void | (() => void);
interface ImportItem {
  exp: string | ExpressionNode;
  path: string;
}
interface TransformContext extends Required<Omit<TransformOptions, keyof CompilerCompatOptions>>, CompilerCompatOptions {
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
declare function createTransformContext(root: RootNode, { filename, prefixIdentifiers, hoistStatic, hmr, cacheHandlers, nodeTransforms, directiveTransforms, transformHoist, isBuiltInComponent, isCustomElement, expressionPlugins, scopeId, slotted, ssr, inSSR, ssrCssVars, bindingMetadata, inline, isTS, onError, onWarn, compatConfig }: TransformOptions): TransformContext;
declare function transform(root: RootNode, options: TransformOptions): void;
declare function traverseNode(node: RootNode | TemplateChildNode, context: TransformContext): void;
declare function createStructuralDirectiveTransform(name: string | RegExp, fn: StructuralDirectiveTransform): NodeTransform;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/transformElement.d.ts
declare const transformElement: NodeTransform;
declare function resolveComponentType(node: ComponentNode, context: TransformContext, ssr?: boolean): string | symbol | CallExpression;
type PropsExpression = ObjectExpression | CallExpression | ExpressionNode;
declare function buildProps(node: ElementNode, context: TransformContext, props: ElementNode["props"] | undefined, isComponent: boolean, isDynamicComponent: boolean, ssr?: boolean): {
  props: PropsExpression | undefined;
  directives: DirectiveNode[];
  patchFlag: number;
  dynamicPropNames: string[];
  shouldUseBlock: boolean;
};
declare function buildDirectiveArgs(dir: DirectiveNode, context: TransformContext): ArrayExpression;
//#endregion
//#region tests/real-world/core/compiler-core/ast.d.ts
type Namespace = number;
declare enum Namespaces {
  HTML = 0,
  SVG = 1,
  MATH_ML = 2
}
declare enum NodeTypes {
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
declare enum ElementTypes {
  ELEMENT = 0,
  COMPONENT = 1,
  SLOT = 2,
  TEMPLATE = 3
}
interface Node {
  type: NodeTypes;
  loc: SourceLocation;
}
interface SourceLocation {
  start: Position;
  end: Position;
  source: string;
}
interface Position {
  offset: number;
  line: number;
  column: number;
}
type ParentNode = RootNode | ElementNode | IfBranchNode | ForNode;
type ExpressionNode = SimpleExpressionNode | CompoundExpressionNode;
type TemplateChildNode = ElementNode | InterpolationNode | CompoundExpressionNode | TextNode | CommentNode | IfNode | IfBranchNode | ForNode | TextCallNode;
interface RootNode extends Node {
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
type ElementNode = PlainElementNode | ComponentNode | SlotOutletNode | TemplateNode;
interface BaseElementNode extends Node {
  type: NodeTypes.ELEMENT;
  ns: Namespace;
  tag: string;
  tagType: ElementTypes;
  props: Array<AttributeNode | DirectiveNode>;
  children: TemplateChildNode[];
  isSelfClosing?: boolean;
  innerLoc?: SourceLocation;
}
interface PlainElementNode extends BaseElementNode {
  tagType: ElementTypes.ELEMENT;
  codegenNode: VNodeCall | SimpleExpressionNode | CacheExpression | MemoExpression | undefined;
  ssrCodegenNode?: TemplateLiteral;
}
interface ComponentNode extends BaseElementNode {
  tagType: ElementTypes.COMPONENT;
  codegenNode: VNodeCall | CacheExpression | MemoExpression | undefined;
  ssrCodegenNode?: CallExpression;
}
interface SlotOutletNode extends BaseElementNode {
  tagType: ElementTypes.SLOT;
  codegenNode: RenderSlotCall | CacheExpression | undefined;
  ssrCodegenNode?: CallExpression;
}
interface TemplateNode extends BaseElementNode {
  tagType: ElementTypes.TEMPLATE;
  codegenNode: undefined;
}
interface TextNode extends Node {
  type: NodeTypes.TEXT;
  content: string;
}
interface CommentNode extends Node {
  type: NodeTypes.COMMENT;
  content: string;
}
interface AttributeNode extends Node {
  type: NodeTypes.ATTRIBUTE;
  name: string;
  nameLoc: SourceLocation;
  value: TextNode | undefined;
}
interface DirectiveNode extends Node {
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
declare enum ConstantTypes {
  NOT_CONSTANT = 0,
  CAN_SKIP_PATCH = 1,
  CAN_CACHE = 2,
  CAN_STRINGIFY = 3
}
interface SimpleExpressionNode extends Node {
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
interface InterpolationNode extends Node {
  type: NodeTypes.INTERPOLATION;
  content: ExpressionNode;
}
interface CompoundExpressionNode extends Node {
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
interface IfNode extends Node {
  type: NodeTypes.IF;
  branches: IfBranchNode[];
  codegenNode?: IfConditionalExpression | CacheExpression;
}
interface IfBranchNode extends Node {
  type: NodeTypes.IF_BRANCH;
  condition: ExpressionNode | undefined;
  children: TemplateChildNode[];
  userKey?: AttributeNode | DirectiveNode;
  isTemplateIf?: boolean;
}
interface ForNode extends Node {
  type: NodeTypes.FOR;
  source: ExpressionNode;
  valueAlias: ExpressionNode | undefined;
  keyAlias: ExpressionNode | undefined;
  objectIndexAlias: ExpressionNode | undefined;
  parseResult: ForParseResult;
  children: TemplateChildNode[];
  codegenNode?: ForCodegenNode;
}
interface ForParseResult {
  source: ExpressionNode;
  value: ExpressionNode | undefined;
  key: ExpressionNode | undefined;
  index: ExpressionNode | undefined;
  finalized: boolean;
}
interface TextCallNode extends Node {
  type: NodeTypes.TEXT_CALL;
  content: TextNode | InterpolationNode | CompoundExpressionNode;
  codegenNode: CallExpression | SimpleExpressionNode;
}
type TemplateTextChildNode = TextNode | InterpolationNode | CompoundExpressionNode;
interface VNodeCall extends Node {
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
type JSChildNode = VNodeCall | CallExpression | ObjectExpression | ArrayExpression | ExpressionNode | FunctionExpression | ConditionalExpression | CacheExpression | AssignmentExpression | SequenceExpression;
interface CallExpression extends Node {
  type: NodeTypes.JS_CALL_EXPRESSION;
  callee: string | symbol;
  arguments: (string | symbol | JSChildNode | SSRCodegenNode | TemplateChildNode | TemplateChildNode[])[];
}
interface ObjectExpression extends Node {
  type: NodeTypes.JS_OBJECT_EXPRESSION;
  properties: Array<Property>;
}
interface Property extends Node {
  type: NodeTypes.JS_PROPERTY;
  key: ExpressionNode;
  value: JSChildNode;
}
interface ArrayExpression extends Node {
  type: NodeTypes.JS_ARRAY_EXPRESSION;
  elements: Array<string | Node>;
}
interface FunctionExpression extends Node {
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
interface ConditionalExpression extends Node {
  type: NodeTypes.JS_CONDITIONAL_EXPRESSION;
  test: JSChildNode;
  consequent: JSChildNode;
  alternate: JSChildNode;
  newline: boolean;
}
interface CacheExpression extends Node {
  type: NodeTypes.JS_CACHE_EXPRESSION;
  index: number;
  value: JSChildNode;
  needPauseTracking: boolean;
  inVOnce: boolean;
  needArraySpread: boolean;
}
interface MemoExpression extends CallExpression {
  callee: typeof WITH_MEMO;
  arguments: [ExpressionNode, MemoFactory, string, string];
}
interface MemoFactory extends FunctionExpression {
  returns: BlockCodegenNode;
}
type SSRCodegenNode = BlockStatement | TemplateLiteral | IfStatement | AssignmentExpression | ReturnStatement | SequenceExpression;
interface BlockStatement extends Node {
  type: NodeTypes.JS_BLOCK_STATEMENT;
  body: (JSChildNode | IfStatement)[];
}
interface TemplateLiteral extends Node {
  type: NodeTypes.JS_TEMPLATE_LITERAL;
  elements: (string | JSChildNode)[];
}
interface IfStatement extends Node {
  type: NodeTypes.JS_IF_STATEMENT;
  test: ExpressionNode;
  consequent: BlockStatement;
  alternate: IfStatement | BlockStatement | ReturnStatement | undefined;
}
interface AssignmentExpression extends Node {
  type: NodeTypes.JS_ASSIGNMENT_EXPRESSION;
  left: SimpleExpressionNode;
  right: JSChildNode;
}
interface SequenceExpression extends Node {
  type: NodeTypes.JS_SEQUENCE_EXPRESSION;
  expressions: JSChildNode[];
}
interface ReturnStatement extends Node {
  type: NodeTypes.JS_RETURN_STATEMENT;
  returns: TemplateChildNode | TemplateChildNode[] | JSChildNode;
}
interface DirectiveArguments extends ArrayExpression {
  elements: DirectiveArgumentNode[];
}
interface DirectiveArgumentNode extends ArrayExpression {
  elements: [string] | [string, ExpressionNode] | [string, ExpressionNode, ExpressionNode] | [string, ExpressionNode, ExpressionNode, ObjectExpression];
}
interface RenderSlotCall extends CallExpression {
  callee: typeof RENDER_SLOT;
  arguments: [string, string | ExpressionNode] | [string, string | ExpressionNode, PropsExpression] | [string, string | ExpressionNode, PropsExpression | "{}", TemplateChildNode[]];
}
type SlotsExpression = SlotsObjectExpression | DynamicSlotsExpression;
interface SlotsObjectExpression extends ObjectExpression {
  properties: SlotsObjectProperty[];
}
interface SlotsObjectProperty extends Property {
  value: SlotFunctionExpression;
}
interface SlotFunctionExpression extends FunctionExpression {
  returns: TemplateChildNode[] | CacheExpression;
}
interface DynamicSlotsExpression extends CallExpression {
  callee: typeof CREATE_SLOTS;
  arguments: [SlotsObjectExpression, DynamicSlotEntries];
}
interface DynamicSlotEntries extends ArrayExpression {
  elements: (ConditionalDynamicSlotNode | ListDynamicSlotNode)[];
}
interface ConditionalDynamicSlotNode extends ConditionalExpression {
  consequent: DynamicSlotNode;
  alternate: DynamicSlotNode | SimpleExpressionNode;
}
interface ListDynamicSlotNode extends CallExpression {
  callee: typeof RENDER_LIST;
  arguments: [ExpressionNode, ListDynamicSlotIterator];
}
interface ListDynamicSlotIterator extends FunctionExpression {
  returns: DynamicSlotNode;
}
interface DynamicSlotNode extends ObjectExpression {
  properties: [Property, DynamicSlotFnProperty];
}
interface DynamicSlotFnProperty extends Property {
  value: SlotFunctionExpression;
}
type BlockCodegenNode = VNodeCall | RenderSlotCall;
interface IfConditionalExpression extends ConditionalExpression {
  consequent: BlockCodegenNode | MemoExpression;
  alternate: BlockCodegenNode | IfConditionalExpression | MemoExpression;
}
interface ForCodegenNode extends VNodeCall {
  isBlock: true;
  tag: typeof FRAGMENT;
  props: undefined;
  children: ForRenderListExpression;
  patchFlag: PatchFlags;
  disableTracking: boolean;
}
interface ForRenderListExpression extends CallExpression {
  callee: typeof RENDER_LIST;
  arguments: [ExpressionNode, ForIteratorExpression];
}
interface ForIteratorExpression extends FunctionExpression {
  returns?: BlockCodegenNode;
}
declare const locStub: SourceLocation;
declare function createRoot(children: TemplateChildNode[], source?: string): RootNode;
declare function createVNodeCall(context: TransformContext | null, tag: VNodeCall["tag"], props?: VNodeCall["props"], children?: VNodeCall["children"], patchFlag?: VNodeCall["patchFlag"], dynamicProps?: VNodeCall["dynamicProps"], directives?: VNodeCall["directives"], isBlock?: VNodeCall["isBlock"], disableTracking?: VNodeCall["disableTracking"], isComponent?: VNodeCall["isComponent"], loc?: SourceLocation): VNodeCall;
declare function createArrayExpression(elements: ArrayExpression["elements"], loc?: SourceLocation): ArrayExpression;
declare function createObjectExpression(properties: ObjectExpression["properties"], loc?: SourceLocation): ObjectExpression;
declare function createObjectProperty(key: Property["key"] | string, value: Property["value"]): Property;
declare function createSimpleExpression(content: SimpleExpressionNode["content"], isStatic?: SimpleExpressionNode["isStatic"], loc?: SourceLocation, constType?: ConstantTypes): SimpleExpressionNode;
declare function createInterpolation(content: InterpolationNode["content"] | string, loc: SourceLocation): InterpolationNode;
declare function createCompoundExpression(children: CompoundExpressionNode["children"], loc?: SourceLocation): CompoundExpressionNode;
type InferCodegenNodeType<T> = T extends typeof RENDER_SLOT ? RenderSlotCall : CallExpression;
declare function createCallExpression<T extends CallExpression["callee"]>(callee: T, args?: CallExpression["arguments"], loc?: SourceLocation): InferCodegenNodeType<T>;
declare function createFunctionExpression(params: FunctionExpression["params"], returns?: FunctionExpression["returns"], newline?: boolean, isSlot?: boolean, loc?: SourceLocation): FunctionExpression;
declare function createConditionalExpression(test: ConditionalExpression["test"], consequent: ConditionalExpression["consequent"], alternate: ConditionalExpression["alternate"], newline?: boolean): ConditionalExpression;
declare function createCacheExpression(index: number, value: JSChildNode, needPauseTracking?: boolean, inVOnce?: boolean): CacheExpression;
declare function createBlockStatement(body: BlockStatement["body"]): BlockStatement;
declare function createTemplateLiteral(elements: TemplateLiteral["elements"]): TemplateLiteral;
declare function createIfStatement(test: IfStatement["test"], consequent: IfStatement["consequent"], alternate?: IfStatement["alternate"]): IfStatement;
declare function createAssignmentExpression(left: AssignmentExpression["left"], right: AssignmentExpression["right"]): AssignmentExpression;
declare function createSequenceExpression(expressions: SequenceExpression["expressions"]): SequenceExpression;
declare function createReturnStatement(returns: ReturnStatement["returns"]): ReturnStatement;
declare function getVNodeHelper(ssr: boolean, isComponent: boolean): typeof CREATE_VNODE | typeof CREATE_ELEMENT_VNODE;
declare function getVNodeBlockHelper(ssr: boolean, isComponent: boolean): typeof CREATE_BLOCK | typeof CREATE_ELEMENT_BLOCK;
declare function convertToBlock(node: VNodeCall, { helper, removeHelper, inSSR }: TransformContext): void;
//#endregion
//#region tests/real-world/core/compiler-core/errors.d.ts
interface CompilerError extends SyntaxError {
  code: number | string;
  loc?: SourceLocation;
}
interface CoreCompilerError extends CompilerError {
  code: ErrorCodes;
}
type InferCompilerError<T> = T extends ErrorCodes ? CoreCompilerError : CompilerError;
declare function createCompilerError<T extends number>(code: T, loc?: SourceLocation, messages?: {
  [code: number]: string;
}, additionalMessage?: string): InferCompilerError<T>;
declare enum ErrorCodes {
  ABRUPT_CLOSING_OF_EMPTY_COMMENT = 0,
  CDATA_IN_HTML_CONTENT = 1,
  DUPLICATE_ATTRIBUTE = 2,
  END_TAG_WITH_ATTRIBUTES = 3,
  END_TAG_WITH_TRAILING_SOLIDUS = 4,
  EOF_BEFORE_TAG_NAME = 5,
  EOF_IN_CDATA = 6,
  EOF_IN_COMMENT = 7,
  EOF_IN_SCRIPT_HTML_COMMENT_LIKE_TEXT = 8,
  EOF_IN_TAG = 9,
  INCORRECTLY_CLOSED_COMMENT = 10,
  INCORRECTLY_OPENED_COMMENT = 11,
  INVALID_FIRST_CHARACTER_OF_TAG_NAME = 12,
  MISSING_ATTRIBUTE_VALUE = 13,
  MISSING_END_TAG_NAME = 14,
  MISSING_WHITESPACE_BETWEEN_ATTRIBUTES = 15,
  NESTED_COMMENT = 16,
  UNEXPECTED_CHARACTER_IN_ATTRIBUTE_NAME = 17,
  UNEXPECTED_CHARACTER_IN_UNQUOTED_ATTRIBUTE_VALUE = 18,
  UNEXPECTED_EQUALS_SIGN_BEFORE_ATTRIBUTE_NAME = 19,
  UNEXPECTED_NULL_CHARACTER = 20,
  UNEXPECTED_QUESTION_MARK_INSTEAD_OF_TAG_NAME = 21,
  UNEXPECTED_SOLIDUS_IN_TAG = 22,
  X_INVALID_END_TAG = 23,
  X_MISSING_END_TAG = 24,
  X_MISSING_INTERPOLATION_END = 25,
  X_MISSING_DIRECTIVE_NAME = 26,
  X_MISSING_DYNAMIC_DIRECTIVE_ARGUMENT_END = 27,
  X_V_IF_NO_EXPRESSION = 28,
  X_V_IF_SAME_KEY = 29,
  X_V_ELSE_NO_ADJACENT_IF = 30,
  X_V_FOR_NO_EXPRESSION = 31,
  X_V_FOR_MALFORMED_EXPRESSION = 32,
  X_V_FOR_TEMPLATE_KEY_PLACEMENT = 33,
  X_V_BIND_NO_EXPRESSION = 34,
  X_V_ON_NO_EXPRESSION = 35,
  X_V_SLOT_UNEXPECTED_DIRECTIVE_ON_SLOT_OUTLET = 36,
  X_V_SLOT_MIXED_SLOT_USAGE = 37,
  X_V_SLOT_DUPLICATE_SLOT_NAMES = 38,
  X_V_SLOT_EXTRANEOUS_DEFAULT_SLOT_CHILDREN = 39,
  X_V_SLOT_MISPLACED = 40,
  X_V_MODEL_NO_EXPRESSION = 41,
  X_V_MODEL_MALFORMED_EXPRESSION = 42,
  X_V_MODEL_ON_SCOPE_VARIABLE = 43,
  X_V_MODEL_ON_PROPS = 44,
  X_V_MODEL_ON_CONST = 45,
  X_INVALID_EXPRESSION = 46,
  X_KEEP_ALIVE_INVALID_CHILDREN = 47,
  X_PREFIX_ID_NOT_SUPPORTED = 48,
  X_MODULE_MODE_NOT_SUPPORTED = 49,
  X_CACHE_HANDLER_NOT_SUPPORTED = 50,
  X_SCOPE_ID_NOT_SUPPORTED = 51,
  X_VNODE_HOOKS = 52,
  X_V_BIND_INVALID_SAME_NAME_ARGUMENT = 53,
  __EXTEND_POINT__ = 54
}
declare const errorMessages: Record<ErrorCodes, string>;
//#endregion
//#region tests/real-world/core/compiler-core/options.d.ts
interface ErrorHandlingOptions {
  onWarn?: (warning: CompilerError) => void;
  onError?: (error: CompilerError) => void;
}
interface ParserOptions extends ErrorHandlingOptions, CompilerCompatOptions {
  /**
  * Base mode is platform agnostic and only parses HTML-like template syntax,
  * treating all tags the same way. Specific tag parsing behavior can be
  * configured by higher-level compilers.
  *
  * HTML mode adds additional logic for handling special parsing behavior in
  * `<script>`, `<style>`,`<title>` and `<textarea>`.
  * The logic is handled inside compiler-core for efficiency.
  *
  * SFC mode treats content of all root-level tags except `<template>` as plain
  * text.
  */
  parseMode?: "base" | "html" | "sfc";
  /**
  * Specify the root namespace to use when parsing a template.
  * Defaults to `Namespaces.HTML` (0).
  */
  ns?: Namespaces;
  /**
  * e.g. platform native elements, e.g. `<div>` for browsers
  */
  isNativeTag?: (tag: string) => boolean;
  /**
  * e.g. native elements that can self-close, e.g. `<img>`, `<br>`, `<hr>`
  */
  isVoidTag?: (tag: string) => boolean;
  /**
  * e.g. elements that should preserve whitespace inside, e.g. `<pre>`
  */
  isPreTag?: (tag: string) => boolean;
  /**
  * Elements that should ignore the first newline token per parinsg spec
  * e.g. `<textarea>` and `<pre>`
  */
  isIgnoreNewlineTag?: (tag: string) => boolean;
  /**
  * Platform-specific built-in components e.g. `<Transition>`
  */
  isBuiltInComponent?: (tag: string) => symbol | void;
  /**
  * Separate option for end users to extend the native elements list
  */
  isCustomElement?: (tag: string) => boolean | void;
  /**
  * Get tag namespace
  */
  getNamespace?: (tag: string, parent: ElementNode | undefined, rootNamespace: Namespace) => Namespace;
  /**
  * @default ['{{', '}}']
  */
  delimiters?: [string, string];
  /**
  * Whitespace handling strategy
  * @default 'condense'
  */
  whitespace?: "preserve" | "condense";
  /**
  * Only used for DOM compilers that runs in the browser.
  * In non-browser builds, this option is ignored.
  */
  decodeEntities?: (rawText: string, asAttr: boolean) => string;
  /**
  * Whether to keep comments in the templates AST.
  * This defaults to `true` in development and `false` in production builds.
  */
  comments?: boolean;
  /**
  * Parse JavaScript expressions with Babel.
  * @default false
  */
  prefixIdentifiers?: boolean;
  /**
  * A list of parser plugins to enable for `@babel/parser`, which is used to
  * parse expressions in bindings and interpolations.
  * https://babeljs.io/docs/en/next/babel-parser#plugins
  */
  expressionPlugins?: ParserPlugin[];
}
type HoistTransform = (children: TemplateChildNode[], context: TransformContext, parent: ParentNode) => void;
declare enum BindingTypes {
  /**
  * returned from data()
  */
  DATA = "data",
  /**
  * declared as a prop
  */
  PROPS = "props",
  /**
  * a local alias of a `<script setup>` destructured prop.
  * the original is stored in __propsAliases of the bindingMetadata object.
  */
  PROPS_ALIASED = "props-aliased",
  /**
  * a let binding (may or may not be a ref)
  */
  SETUP_LET = "setup-let",
  /**
  * a const binding that can never be a ref.
  * these bindings don't need `unref()` calls when processed in inlined
  * template expressions.
  */
  SETUP_CONST = "setup-const",
  /**
  * a const binding that does not need `unref()`, but may be mutated.
  */
  SETUP_REACTIVE_CONST = "setup-reactive-const",
  /**
  * a const binding that may be a ref.
  */
  SETUP_MAYBE_REF = "setup-maybe-ref",
  /**
  * bindings that are guaranteed to be refs
  */
  SETUP_REF = "setup-ref",
  /**
  * declared by other options, e.g. computed, inject
  */
  OPTIONS = "options",
  /**
  * a literal constant, e.g. 'foo', 1, true
  */
  LITERAL_CONST = "literal-const"
}
type BindingMetadata = {
  [key: string]: BindingTypes | undefined;
} & {
  __isScriptSetup?: boolean;
  __propsAliases?: Record<string, string>;
};
interface SharedTransformCodegenOptions {
  /**
  * Transform expressions like {{ foo }} to `_ctx.foo`.
  * If this option is false, the generated code will be wrapped in a
  * `with (this) { ... }` block.
  * - This is force-enabled in module mode, since modules are by default strict
  * and cannot use `with`
  * @default mode === 'module'
  */
  prefixIdentifiers?: boolean;
  /**
  * Control whether generate SSR-optimized render functions instead.
  * The resulting function must be attached to the component via the
  * `ssrRender` option instead of `render`.
  *
  * When compiler generates code for SSR's fallback branch, we need to set it to false:
  *  - context.ssr = false
  *
  * see `subTransform` in `ssrTransformComponent.ts`
  */
  ssr?: boolean;
  /**
  * Indicates whether the compiler generates code for SSR,
  * it is always true when generating code for SSR,
  * regardless of whether we are generating code for SSR's fallback branch,
  * this means that when the compiler generates code for SSR's fallback branch:
  *  - context.ssr = false
  *  - context.inSSR = true
  */
  inSSR?: boolean;
  /**
  * Optional binding metadata analyzed from script - used to optimize
  * binding access when `prefixIdentifiers` is enabled.
  */
  bindingMetadata?: BindingMetadata;
  /**
  * Compile the function for inlining inside setup().
  * This allows the function to directly access setup() local bindings.
  */
  inline?: boolean;
  /**
  * Indicates that transforms and codegen should try to output valid TS code
  */
  isTS?: boolean;
  /**
  * Filename for source map generation.
  * Also used for self-recursive reference in templates
  * @default 'template.vue.html'
  */
  filename?: string;
}
interface TransformOptions extends SharedTransformCodegenOptions, ErrorHandlingOptions, CompilerCompatOptions {
  /**
  * An array of node transforms to be applied to every AST node.
  */
  nodeTransforms?: NodeTransform[];
  /**
  * An object of { name: transform } to be applied to every directive attribute
  * node found on element nodes.
  */
  directiveTransforms?: Record<string, DirectiveTransform | undefined>;
  /**
  * An optional hook to transform a node being hoisted.
  * used by compiler-dom to turn hoisted nodes into stringified HTML vnodes.
  * @default null
  */
  transformHoist?: HoistTransform | null;
  /**
  * If the pairing runtime provides additional built-in elements, use this to
  * mark them as built-in so the compiler will generate component vnodes
  * for them.
  */
  isBuiltInComponent?: (tag: string) => symbol | void;
  /**
  * Used by some transforms that expects only native elements
  */
  isCustomElement?: (tag: string) => boolean | void;
  /**
  * Transform expressions like {{ foo }} to `_ctx.foo`.
  * If this option is false, the generated code will be wrapped in a
  * `with (this) { ... }` block.
  * - This is force-enabled in module mode, since modules are by default strict
  * and cannot use `with`
  * @default mode === 'module'
  */
  prefixIdentifiers?: boolean;
  /**
  * Cache static VNodes and props objects to `_hoisted_x` constants
  * @default false
  */
  hoistStatic?: boolean;
  /**
  * Cache v-on handlers to avoid creating new inline functions on each render,
  * also avoids the need for dynamically patching the handlers by wrapping it.
  * e.g `@click="foo"` by default is compiled to `{ onClick: foo }`. With this
  * option it's compiled to:
  * ```js
  * { onClick: _cache[0] || (_cache[0] = e => _ctx.foo(e)) }
  * ```
  * - Requires "prefixIdentifiers" to be enabled because it relies on scope
  * analysis to determine if a handler is safe to cache.
  * @default false
  */
  cacheHandlers?: boolean;
  /**
  * A list of parser plugins to enable for `@babel/parser`, which is used to
  * parse expressions in bindings and interpolations.
  * https://babeljs.io/docs/en/next/babel-parser#plugins
  */
  expressionPlugins?: ParserPlugin[];
  /**
  * SFC scoped styles ID
  */
  scopeId?: string | null;
  /**
  * Indicates this SFC template has used :slotted in its styles
  * Defaults to `true` for backwards compatibility - SFC tooling should set it
  * to `false` if no `:slotted` usage is detected in `<style>`
  */
  slotted?: boolean;
  /**
  * SFC `<style vars>` injection string
  * Should already be an object expression, e.g. `{ 'xxxx-color': color }`
  * needed to render inline CSS variables on component root
  */
  ssrCssVars?: string;
  /**
  * Whether to compile the template assuming it needs to handle HMR.
  * Some edge cases may need to generate different code for HMR to work
  * correctly, e.g. #6938, #7138
  */
  hmr?: boolean;
}
interface CodegenOptions extends SharedTransformCodegenOptions {
  /**
  * - `module` mode will generate ES module import statements for helpers
  * and export the render function as the default export.
  * - `function` mode will generate a single `const { helpers... } = Vue`
  * statement and return the render function. It expects `Vue` to be globally
  * available (or passed by wrapping the code with an IIFE). It is meant to be
  * used with `new Function(code)()` to generate a render function at runtime.
  * @default 'function'
  */
  mode?: "module" | "function";
  /**
  * Generate source map?
  * @default false
  */
  sourceMap?: boolean;
  /**
  * SFC scoped styles ID
  */
  scopeId?: string | null;
  /**
  * Option to optimize helper import bindings via variable assignment
  * (only used for webpack code-split)
  * @default false
  */
  optimizeImports?: boolean;
  /**
  * Customize where to import runtime helpers from.
  * @default 'vue'
  */
  runtimeModuleName?: string;
  /**
  * Customize where to import ssr runtime helpers from/**
  * @default 'vue/server-renderer'
  */
  ssrRuntimeModuleName?: string;
  /**
  * Customize the global variable name of `Vue` to get helpers from
  * in function mode
  * @default 'Vue'
  */
  runtimeGlobalName?: string;
}
type CompilerOptions = ParserOptions & TransformOptions & CodegenOptions;
//#endregion
//#region tests/real-world/core/compiler-core/codegen.d.ts
/**
* The `SourceMapGenerator` type from `source-map-js` is a bit incomplete as it
* misses `toJSON()`. We also need to add types for internal properties which we
* need to access for better performance.
*
* Since TS 5.3, dts generation starts to strangely include broken triple slash
* references for source-map-js, so we are inlining all source map related types
* here to to workaround that.
*/
interface CodegenSourceMapGenerator {
  setSourceContent(sourceFile: string, sourceContent: string): void;
  toJSON(): RawSourceMap;
  _sources: Set<string>;
  _names: Set<string>;
  _mappings: {
    add(mapping: MappingItem): void;
  };
}
interface RawSourceMap {
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
interface CodegenResult {
  code: string;
  preamble: string;
  ast: RootNode;
  map?: RawSourceMap;
}
interface CodegenContext extends Omit<Required<CodegenOptions>, "bindingMetadata" | "inline"> {
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
declare function generate(ast: RootNode, options?: CodegenOptions & {
  onContextCreated?: (context: CodegenContext) => void;
}): CodegenResult;
//#endregion
//#region tests/real-world/core/compiler-core/compile.d.ts
type TransformPreset = [NodeTransform[], Record<string, DirectiveTransform>];
declare function getBaseTransformPreset(prefixIdentifiers?: boolean): TransformPreset;
declare function baseCompile(source: string | RootNode, options?: CompilerOptions): CodegenResult;
//#endregion
//#region tests/real-world/core/compiler-core/utils.d.ts
declare const isStaticExp: (p: JSChildNode) => p is SimpleExpressionNode;
declare function isCoreComponent(tag: string): symbol | void;
declare const isSimpleIdentifier: (name: string) => boolean;
declare const validFirstIdentCharRE: RegExp;
/**
* Simple lexer to check if an expression is a member expression. This is
* lax and only checks validity at the root level (i.e. does not validate exps
* inside square brackets), but it's ok since these are only used on template
* expressions and false positives are invalid expressions in the first place.
*/
declare const isMemberExpressionBrowser: (exp: ExpressionNode) => boolean;
declare const isMemberExpressionNode: (exp: ExpressionNode, context: TransformContext) => boolean;
declare const isMemberExpression: (exp: ExpressionNode, context: TransformContext) => boolean;
declare const isFnExpressionBrowser: (exp: ExpressionNode) => boolean;
declare const isFnExpressionNode: (exp: ExpressionNode, context: TransformContext) => boolean;
declare const isFnExpression: (exp: ExpressionNode, context: TransformContext) => boolean;
declare function advancePositionWithClone(pos: Position, source: string, numberOfCharacters?: number): Position;
declare function advancePositionWithMutation(pos: Position, source: string, numberOfCharacters?: number): Position;
declare function assert(condition: boolean, msg?: string): void;
declare function findDir(node: ElementNode, name: string | RegExp, allowEmpty?: boolean): DirectiveNode | undefined;
declare function findProp(node: ElementNode, name: string, dynamicOnly?: boolean, allowEmpty?: boolean): ElementNode["props"][0] | undefined;
declare function isStaticArgOf(arg: DirectiveNode["arg"], name: string): boolean;
declare function hasDynamicKeyVBind(node: ElementNode): boolean;
declare function isText(node: TemplateChildNode): node is TextNode | InterpolationNode;
declare function isVPre(p: ElementNode["props"][0]): p is DirectiveNode;
declare function isVSlot(p: ElementNode["props"][0]): p is DirectiveNode;
declare function isTemplateNode(node: RootNode | TemplateChildNode): node is TemplateNode;
declare function isSlotOutlet(node: RootNode | TemplateChildNode): node is SlotOutletNode;
declare function injectProp(node: VNodeCall | RenderSlotCall, prop: Property, context: TransformContext): void;
declare function toValidAssetId(name: string, type: "component" | "directive" | "filter"): string;
declare function hasScopeRef(node: TemplateChildNode | IfBranchNode | ExpressionNode | CacheExpression | undefined, ids: TransformContext["identifiers"]): boolean;
declare function getMemoedVNodeCall(node: BlockCodegenNode | MemoExpression): VNodeCall | RenderSlotCall;
declare const forAliasRE: RegExp;
declare function isAllWhitespace(str: string): boolean;
declare function isWhitespaceText(node: TemplateChildNode): boolean;
declare function isCommentOrWhitespace(node: TemplateChildNode): boolean;
//#endregion
//#region tests/real-world/core/compiler-core/babelUtils.d.ts
/**
* Return value indicates whether the AST walked can be a constant
*/
declare function walkIdentifiers(root: Node, onIdentifier: (node: Identifier, parent: Node | null, parentStack: Node[], isReference: boolean, isLocal: boolean) => void, includeAll?: boolean, parentStack?: Node[], knownIds?: Record<string, number>): void;
declare function isReferencedIdentifier(id: Identifier, parent: Node | null, parentStack: Node[]): boolean;
declare function isInDestructureAssignment(parent: Node, parentStack: Node[]): boolean;
declare function isInNewExpression(parentStack: Node[]): boolean;
declare function walkFunctionParams(node: Function, onIdent: (id: Identifier) => void): void;
declare function walkBlockDeclarations(block: BlockStatement | SwitchCase | Program, onIdent: (node: Identifier) => void): void;
declare function extractIdentifiers(param: Node, nodes?: Identifier[]): Identifier[];
declare const isFunctionType: (node: Node) => node is Function;
declare const isStaticProperty: (node: Node) => node is ObjectProperty;
declare const isStaticPropertyKey: (node: Node, parent: Node) => boolean;
declare const TS_NODE_TYPES: string[];
declare function unwrapTSNode(node: Node): Node;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/vModel.d.ts
declare const transformModel: DirectiveTransform;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/vOn.d.ts
declare const transformOn: DirectiveTransform;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/vBind.d.ts
declare const transformBind: DirectiveTransform;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/noopDirectiveTransform.d.ts
declare const noopDirectiveTransform: DirectiveTransform;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/vIf.d.ts
declare function processIf(node: ElementNode, dir: DirectiveNode, context: TransformContext, processCodegen?: (node: IfNode, branch: IfBranchNode, isRoot: boolean) => (() => void) | undefined): (() => void) | undefined;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/vFor.d.ts
declare function processFor(node: ElementNode, dir: DirectiveNode, context: TransformContext, processCodegen?: (forNode: ForNode) => (() => void) | undefined): (() => void) | undefined;
declare function createForLoopParams({ value, key, index }: ForParseResult, memoArgs?: ExpressionNode[]): ExpressionNode[];
//#endregion
//#region tests/real-world/core/compiler-core/transforms/transformExpression.d.ts
declare const transformExpression: NodeTransform;
declare function processExpression(node: SimpleExpressionNode, context: TransformContext, asParams?: boolean, asRawStatements?: boolean, localVars?: Record<string, number>): ExpressionNode;
declare function stringifyExpression(exp: ExpressionNode | string): string;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/vSlot.d.ts
declare const trackSlotScopes: NodeTransform;
declare const trackVForSlotScopes: NodeTransform;
type SlotFnBuilder = (slotProps: ExpressionNode | undefined, vFor: DirectiveNode | undefined, slotChildren: TemplateChildNode[], loc: SourceLocation) => FunctionExpression;
declare function buildSlots(node: ElementNode, context: TransformContext, buildSlotFn?: SlotFnBuilder): {
  slots: SlotsExpression;
  hasDynamicSlots: boolean;
};
//#endregion
//#region tests/real-world/core/compiler-core/transforms/transformVBindShorthand.d.ts
declare const transformVBindShorthand: NodeTransform;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/transformSlotOutlet.d.ts
interface SlotOutletProcessResult {
  slotName: string | ExpressionNode;
  slotProps: PropsExpression | undefined;
}
declare function processSlotOutlet(node: SlotOutletNode, context: TransformContext): SlotOutletProcessResult;
//#endregion
//#region tests/real-world/core/compiler-core/transforms/cacheStatic.d.ts
declare function getConstantType(node: TemplateChildNode | SimpleExpressionNode | CacheExpression, context: TransformContext): ConstantTypes;
//#endregion
export { ArrayExpression, AssignmentExpression, AttributeNode, BASE_TRANSITION, BaseElementNode, type BindingMetadata, BindingTypes, BlockCodegenNode, BlockStatement, CAMELIZE, CAPITALIZE, CREATE_BLOCK, CREATE_COMMENT, CREATE_ELEMENT_BLOCK, CREATE_ELEMENT_VNODE, CREATE_SLOTS, CREATE_STATIC, CREATE_TEXT, CREATE_VNODE, CacheExpression, CallExpression, type CodegenContext, type CodegenOptions, type CodegenResult, type CodegenSourceMapGenerator, CommentNode, CompilerDeprecationTypes, type CompilerError, type CompilerOptions, ComponentNode, CompoundExpressionNode, ConditionalDynamicSlotNode, ConditionalExpression, ConstantTypes, type CoreCompilerError, DirectiveArgumentNode, DirectiveArguments, DirectiveNode, type DirectiveTransform, DynamicSlotEntries, DynamicSlotFnProperty, DynamicSlotNode, DynamicSlotsExpression, ElementNode, ElementTypes, ErrorCodes, ExpressionNode, FRAGMENT, ForCodegenNode, ForIteratorExpression, ForNode, ForParseResult, ForRenderListExpression, FunctionExpression, GUARD_REACTIVE_PROPS, type HoistTransform, IS_MEMO_SAME, IS_REF, IfBranchNode, IfConditionalExpression, IfNode, IfStatement, InterpolationNode, JSChildNode, KEEP_ALIVE, ListDynamicSlotIterator, ListDynamicSlotNode, MERGE_PROPS, MemoExpression, NORMALIZE_CLASS, NORMALIZE_PROPS, NORMALIZE_STYLE, Namespace, Namespaces, Node, type NodeTransform, NodeTypes, OPEN_BLOCK, ObjectExpression, POP_SCOPE_ID, PUSH_SCOPE_ID, ParentNode, type ParserOptions, PlainElementNode, Position, Property, type PropsExpression, RENDER_LIST, RENDER_SLOT, RESOLVE_COMPONENT, RESOLVE_DIRECTIVE, RESOLVE_DYNAMIC_COMPONENT, RESOLVE_FILTER, type RawSourceMap, RenderSlotCall, ReturnStatement, RootNode, SET_BLOCK_TRACKING, SSRCodegenNode, SUSPENSE, SequenceExpression, SimpleExpressionNode, type SlotFnBuilder, SlotFunctionExpression, SlotOutletNode, SlotsExpression, SlotsObjectExpression, SlotsObjectProperty, SourceLocation, type StructuralDirectiveTransform, TELEPORT, TO_DISPLAY_STRING, TO_HANDLERS, TO_HANDLER_KEY, TS_NODE_TYPES, TemplateChildNode, TemplateLiteral, TemplateNode, TemplateTextChildNode, TextCallNode, TextNode, type TransformContext, type TransformOptions, type TransformPreset, UNREF, VNodeCall, WITH_CTX, WITH_DIRECTIVES, WITH_MEMO, advancePositionWithClone, advancePositionWithMutation, assert, baseCompile, baseParse, buildDirectiveArgs, buildProps, buildSlots, checkCompatEnabled, convertToBlock, createArrayExpression, createAssignmentExpression, createBlockStatement, createCacheExpression, createCallExpression, createCompilerError, createCompoundExpression, createConditionalExpression, createForLoopParams, createFunctionExpression, createIfStatement, createInterpolation, createObjectExpression, createObjectProperty, createReturnStatement, createRoot, createSequenceExpression, createSimpleExpression, createStructuralDirectiveTransform, createTemplateLiteral, createTransformContext, createVNodeCall, errorMessages, extractIdentifiers, findDir, findProp, forAliasRE, generate, generateCodeFrame, getBaseTransformPreset, getConstantType, getMemoedVNodeCall, getVNodeBlockHelper, getVNodeHelper, hasDynamicKeyVBind, hasScopeRef, helperNameMap, injectProp, isAllWhitespace, isCommentOrWhitespace, isCoreComponent, isFnExpression, isFnExpressionBrowser, isFnExpressionNode, isFunctionType, isInDestructureAssignment, isInNewExpression, isMemberExpression, isMemberExpressionBrowser, isMemberExpressionNode, isReferencedIdentifier, isSimpleIdentifier, isSlotOutlet, isStaticArgOf, isStaticExp, isStaticProperty, isStaticPropertyKey, isTemplateNode, isText, isVPre, isVSlot, isWhitespaceText, locStub, noopDirectiveTransform, processExpression, processFor, processIf, processSlotOutlet, registerRuntimeHelpers, resolveComponentType, stringifyExpression, toValidAssetId, trackSlotScopes, trackVForSlotScopes, transform, transformBind, transformElement, transformExpression, transformModel, transformOn, transformVBindShorthand, traverseNode, unwrapTSNode, validFirstIdentCharRE, walkBlockDeclarations, walkFunctionParams, walkIdentifiers, warnDeprecation };