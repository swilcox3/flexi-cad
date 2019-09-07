var kernel = require('../../native/index.node')
var BABYLON = require("babylonjs");

export class Point2d
{
    x: number
    y: number
    constructor( inX: number, inY: number) {
        this.x = inX
        this.y = inY
    }
}

export class Point3d
{
    x: number
    y: number
    z: number
    constructor( inX: number, inY: number, inZ: number) {
        this.x = inX
        this.y = inY
        this.z = inZ
    }

    subtract(pt: Point3d) {
        return new Point3d(this.x - pt.x, this.y - pt.y, this.z - pt.z);
    }
}

export class Vec2d
{
    x: number
    y: number
    constructor( inX: number, inY: number) {
        this.x = inX
        this.y = inY
    }
}

export function doubleEquals(first: number, second: number)
{
    return Math.abs(first - second) < 1e-8
}

export function point2dEquals(first: Point2d, second: Point2d)
{
    return doubleEquals(first.x, second.x) && doubleEquals(first.y, second.y)
}

export function vector2d(point0: Point2d, point1: Point2d)
{
    return new Vec2d(point1.x - point0.x, point1.y - point0.y)
}

export function dot2d(vec0: Vec2d, vec1: Vec2d)
{
    return vec0.x*vec1.x + vec0.y*vec1.y
}

export function cross2d(point0: Vec2d, point1: Vec2d)
{
    return point0.x*point1.y - point0.y*point1.y
}

export function transformModelToGraphicCoords(point: Point3d)
{
    return new Point3d(point.x, point.z, -point.y)
}

export function transformGraphicToModelCoords(point: Point3d)
{
    return new Point3d(point.x, -point.z, point.y)
}

export function toBabylonVector3(point: Point3d)
{
    return new BABYLON.Vector3(point.x, -point.z, point.y)
}

export function projectOnLine(first: Point3d, second: Point3d, project: Point3d)
{
    return kernel.project_on_line(first, second, project)
}