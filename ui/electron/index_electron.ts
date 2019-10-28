import * as ops from '../src/operations/operations';

declare global {
    export type JsDimension = import("./native/index").JsDimension;
    export type JsWall = import("./native/index").JsWall;
    export type JsDoor = import("./native/index").JsDoor;
    export type JsSlab = import("./native/index").JsSlab;
}

export = {};
