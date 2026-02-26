import type { ExpectStatic, MatcherState } from './types';
export declare function getState<State extends MatcherState = MatcherState>(expect: ExpectStatic): State;
export declare function setState<State extends MatcherState = MatcherState>(state: Partial<State>, expect: ExpectStatic): void;
