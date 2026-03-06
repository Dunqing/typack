export interface Keep {
  kept: true;
}
export interface Missing {
  found: true;
}
export interface Unused {
  shouldNotAppear: true;
}
