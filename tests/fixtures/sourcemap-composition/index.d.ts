import { foo } from "./mod";
export interface Foo {
  bar: typeof foo;
}
