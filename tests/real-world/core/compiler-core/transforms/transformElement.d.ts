import type { NodeTransform, TransformContext } from '../transform';
import { type ArrayExpression, type CallExpression, type ComponentNode, type DirectiveNode, type ElementNode, type ExpressionNode, type ObjectExpression } from '../ast';
export declare const transformElement: NodeTransform;
export declare function resolveComponentType(node: ComponentNode, context: TransformContext, ssr?: boolean): string | symbol | CallExpression;
export type PropsExpression = ObjectExpression | CallExpression | ExpressionNode;
export declare function buildProps(node: ElementNode, context: TransformContext, props: ElementNode['props'] | undefined, isComponent: boolean, isDynamicComponent: boolean, ssr?: boolean): {
    props: PropsExpression | undefined;
    directives: DirectiveNode[];
    patchFlag: number;
    dynamicPropNames: string[];
    shouldUseBlock: boolean;
};
export declare function buildDirectiveArgs(dir: DirectiveNode, context: TransformContext): ArrayExpression;
