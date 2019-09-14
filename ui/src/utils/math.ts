var BABYLON = require("babylonjs");
console.log("Made it math 1");
import {Point3d} from "../../data-model-wasm/dist/index"
console.log("Made it math 2");

export interface CoordTriple
{
    x: number,
    y: number,
    z: number
}

export function transformModelToGraphicCoords(point: CoordTriple)
{
    return new Point3d(point.x, point.z, -point.y)
}

export function transformGraphicToModelCoords(point: CoordTriple)
{
    return new Point3d(point.x, -point.z, point.y)
}

export function toBabylonVector3(point: CoordTriple)
{
    return new BABYLON.Vector3(point.x, -point.z, point.y)
}