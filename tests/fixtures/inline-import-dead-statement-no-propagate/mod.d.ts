export interface Keep {
  kept: true;
}

export type Dead = import("./dep.js").Unused;
