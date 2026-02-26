import type { RendererOptions } from '@vue/runtime-core';
import type { TrustedHTML } from 'trusted-types/lib';
export declare const unsafeToTrustedHTML: (value: string) => TrustedHTML | string;
export declare const svgNS = "http://www.w3.org/2000/svg";
export declare const mathmlNS = "http://www.w3.org/1998/Math/MathML";
export declare const nodeOps: Omit<RendererOptions<Node, Element>, 'patchProp'>;
