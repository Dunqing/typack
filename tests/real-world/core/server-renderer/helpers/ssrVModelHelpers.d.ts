export declare const ssrLooseEqual: (a: unknown, b: unknown) => boolean;
export declare function ssrLooseContain(arr: unknown[], value: unknown): boolean;
export declare function ssrRenderDynamicModel(type: unknown, model: unknown, value: unknown): string;
export declare function ssrGetDynamicModelProps(existingProps: any, model: unknown): {
    checked: true;
} | {
    value: any;
} | null;
