import { type ForNode, type NodeTransform } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare const ssrTransformFor: NodeTransform;
export declare function ssrProcessFor(node: ForNode, context: SSRTransformContext, disableNestedFragments?: boolean): void;
