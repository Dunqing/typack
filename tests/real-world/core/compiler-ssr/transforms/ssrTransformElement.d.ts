import { type DirectiveNode, type JSChildNode, type NodeTransform, type PlainElementNode, type PropsExpression, type TransformContext } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare const ssrTransformElement: NodeTransform;
export declare function buildSSRProps(props: PropsExpression | undefined, directives: DirectiveNode[], context: TransformContext): JSChildNode;
export declare function ssrProcessElement(node: PlainElementNode, context: SSRTransformContext): void;
