import type { Awaitable } from '@vitest/utils';
import type { TestFixtures } from './fixture';
import type { Suite, SuiteHooks, Test, TestContext } from './types/tasks';
export declare function setFn(key: Test, fn: () => Awaitable<void>): void;
export declare function getFn<Task = Test>(key: Task): () => Awaitable<void>;
export declare function setTestFixture(key: TestContext, fixture: TestFixtures): void;
export declare function getTestFixtures<Context = TestContext>(key: Context): TestFixtures;
export declare function setHooks(key: Suite, hooks: SuiteHooks): void;
export declare function getHooks(key: Suite): SuiteHooks;
