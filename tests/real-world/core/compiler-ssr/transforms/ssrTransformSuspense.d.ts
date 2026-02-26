import { type ComponentNode, type TransformContext } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare function ssrTransformSuspense(node: ComponentNode, context: TransformContext): () => void;
export declare function ssrProcessSuspense(node: ComponentNode, context: SSRTransformContext): void;
