import { type IfNode, type NodeTransform } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare const ssrTransformIf: NodeTransform;
export declare function ssrProcessIf(node: IfNode, context: SSRTransformContext, disableNestedFragments?: boolean, disableComment?: boolean): void;
