export declare let activeEffectScope: EffectScope | undefined;
export declare class EffectScope {
    detached: boolean;
    private _isPaused;
    readonly __v_skip = true;
    constructor(detached?: boolean);
    get active(): boolean;
    pause(): void;
    /**
     * Resumes the effect scope, including all child scopes and effects.
     */
    resume(): void;
    run<T>(fn: () => T): T | undefined;
    prevScope: EffectScope | undefined;
    stop(fromParent?: boolean): void;
}
/**
 * Creates an effect scope object which can capture the reactive effects (i.e.
 * computed and watchers) created within it so that these effects can be
 * disposed together. For detailed use cases of this API, please consult its
 * corresponding {@link https://github.com/vuejs/rfcs/blob/master/active-rfcs/0041-reactivity-effect-scope.md | RFC}.
 *
 * @param detached - Can be used to create a "detached" effect scope.
 * @see {@link https://vuejs.org/api/reactivity-advanced.html#effectscope}
 */
export declare function effectScope(detached?: boolean): EffectScope;
/**
 * Returns the current active effect scope if there is one.
 *
 * @see {@link https://vuejs.org/api/reactivity-advanced.html#getcurrentscope}
 */
export declare function getCurrentScope(): EffectScope | undefined;
/**
 * Registers a dispose callback on the current active effect scope. The
 * callback will be invoked when the associated effect scope is stopped.
 *
 * @param fn - The callback function to attach to the scope's cleanup.
 * @see {@link https://vuejs.org/api/reactivity-advanced.html#onscopedispose}
 */
export declare function onScopeDispose(fn: () => void, failSilently?: boolean): void;
