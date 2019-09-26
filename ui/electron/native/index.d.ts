interface Point3d {
  x: number
  y: number
  z: number
}

interface Vector3d {
  x: number
  y: number
  z: number
}

/**
* @param {Point3d} first 
* @param {Point3d} second 
* @param {Point3d} project 
* @returns {Point3d} 
*/
export function projectOnLine(first: Point3d, second: Point3d, project: Point3d): Point3d;
/**
* @returns {string} 
*/
export function getUserId(): string;
/**
*/
export class JsDimension {
  free(): void;
/**
* @param {Point3d} first 
* @param {Point3d} second 
* @param {number} offset 
* @returns {JsDimension} 
*/
  constructor(first: Point3d, second: Point3d, offset: number);
/**
* @returns {any} 
*/
  getTempRepr(): any;
/**
* @param {Vector3d} delta 
*/
  moveObj(delta: Vector3d): void;
/**
* @returns {any} 
*/
  getObj(): any;
/**
* @returns {Point3d} 
*/
  first_pt(): Point3d;
/**
* @param {Point3d} val 
*/
  set_first_pt(val: Point3d): void;
/**
* @returns {Point3d} 
*/
  second_pt(): Point3d;
/**
* @param {Point3d} val 
*/
  set_second_pt(val: Point3d): void;
/**
* @returns {number} 
*/
  offset(): number;
/**
* @param {number} val 
*/
  set_offset(val: number): void;
/**
* @returns {string} 
*/
  id(): string;
}
/**
*/
export class JsDoor {
  free(): void;
/**
* @param {Point3d} first 
* @param {Point3d} second 
* @param {number} width 
* @param {number} height 
* @returns {JsDoor} 
*/
  constructor(first: Point3d, second: Point3d, width: number, height: number);
/**
* @returns {any} 
*/
  getTempRepr(): any;
/**
* @param {Vector3d} delta 
*/
  moveObj(delta: Vector3d): void;
/**
* @returns {any} 
*/
  getObj(): any;
/**
* @param {Vector3d} delta 
*/
  setDir(delta: Vector3d): void;
/**
* @returns {Point3d} 
*/
  first_pt(): Point3d;
/**
* @param {Point3d} val 
*/
  set_first_pt(val: Point3d): void;
/**
* @returns {Point3d} 
*/
  second_pt(): Point3d;
/**
* @param {Point3d} val 
*/
  set_second_pt(val: Point3d): void;
/**
* @returns {number} 
*/
  height(): number;
/**
* @param {number} val 
*/
  set_height(val: number): void;
/**
* @returns {number} 
*/
  width(): number;
/**
* @param {number} val 
*/
  set_width(val: number): void;
/**
* @returns {string} 
*/
  id(): string;
}
/**
*/
export class JsWall {
  free(): void;
/**
* @param {Point3d} first 
* @param {Point3d} second 
* @param {number} width 
* @param {number} height 
* @returns {JsWall} 
*/
  constructor(first: Point3d, second: Point3d, width: number, height: number);
/**
* @returns {any} 
*/
  getTempRepr(): any;
/**
* @param {Vector3d} delta 
*/
  moveObj(delta: Vector3d): void;
/**
* @returns {any} 
*/
  getObj(): any;
/**
* @returns {Point3d} 
*/
  first_pt(): Point3d;
/**
* @param {Point3d} val 
*/
  set_first_pt(val: Point3d): void;
/**
* @returns {Point3d} 
*/
  second_pt(): Point3d;
/**
* @param {Point3d} val 
*/
  set_second_pt(val: Point3d): void;
/**
* @returns {number} 
*/
  height(): number;
/**
* @param {number} val 
*/
  set_height(val: number): void;
/**
* @returns {number} 
*/
  width(): number;
/**
* @param {number} val 
*/
  set_width(val: number): void;
/**
* @returns {string} 
*/
  id(): string;
}
/**
*/