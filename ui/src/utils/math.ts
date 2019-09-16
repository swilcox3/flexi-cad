var BABYLON = require("babylonjs");
import {dataModelWasm} from "../operations/operations"

export interface CoordTriple
{
    x: number,
    y: number,
    z: number
}

export function transformModelToGraphicCoords(point: CoordTriple)
{
    return new dataModelWasm.Point3d(point.x, point.z, -point.y)
}

export function transformGraphicToModelCoords(point: CoordTriple)
{
    return new dataModelWasm.Point3d(point.x, -point.z, point.y)
}

export function toBabylonVector3(point: CoordTriple)
{
    return new BABYLON.Vector3(point.x, -point.z, point.y)
}