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
  first_pt: Point3d;
  readonly id: string;
  offset: number;
  second_pt: Point3d;
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
  first_pt: Point3d;
  height: number;
  readonly id: string;
  second_pt: Point3d;
  width: number;
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
  first_pt: Point3d;
  height: number;
  readonly id: string;
  second_pt: Point3d;
  width: number;
}
/**
*/
export class Point3d {
  free(): void;
/**
* @param {number} x 
* @param {number} y 
* @param {number} z 
* @returns {Point3d} 
*/
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
/**
*/
export class Vector3d {
  free(): void;
/**
* @param {number} x 
* @param {number} y 
* @param {number} z 
* @returns {Vector3d} 
*/
  constructor(x: number, y: number, z: number);
  x: number;
  y: number;
  z: number;
}
