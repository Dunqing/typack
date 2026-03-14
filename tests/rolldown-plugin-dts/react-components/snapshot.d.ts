import React from "react";

//#region tests/rolldown-plugin-dts/react-components/index.d.ts
export interface MyComponentProps extends React.HtmlHTMLAttributes<HTMLDivElement> {
  foo: string;
}
export declare class MyComponent extends React.Component<MyComponentProps> {}
//#endregion
