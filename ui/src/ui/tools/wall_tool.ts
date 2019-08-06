import * as ops from '../../operations/operations'
import * as math from '../../utils/math'
const kernel = require("../../../native/index.node")

export class WallTool {
    curTemp: any
    width: number
    height: number
    lastId: string
    undoEventId: string

    constructor(width = 1, height = 5)
    {
        this.curTemp = null;
        this.width = width;
        this.height = height;
        this.lastId = ''
        this.undoEventId = ''
    }

    canJoinToWall(hovered: BABYLON.Mesh) {
        return hovered && hovered.metadata.type == "Wall" && hovered.name != this.lastId && hovered.name != this.curTemp.get("id");
    }

    createWall(picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Wall")
        }
        ops.createObj(this.undoEventId, this.curTemp)
        if(this.lastId) {
            ops.joinAtPoints(this.undoEventId, this.lastId, this.curTemp.get("id"), this.curTemp.get("first"))
        }
        if(this.canJoinToWall(picked)) {
            ops.joinAtPoints(this.undoEventId, picked.name, this.curTemp.get("id"), this.curTemp.get("second"));
        }
    }

    onMouseDown(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp == null)
        {
            var first = new math.Point3d(pt.x, pt.y, 0)
            var second = new math.Point3d(pt.x + 1, pt.y + 1, 0)
            this.curTemp = new kernel.Wall(first, second, this.width, this.height);
            ops.renderTempObject(this.curTemp)
        }
        else
        {
            this.createWall(picked);
            this.lastId = this.curTemp.get("id");
            var first = new math.Point3d(pt.x, pt.y, 0)
            var second = new math.Point3d(pt.x + .1, pt.y + .1, 0)
            this.curTemp = new kernel.Wall(first, second, this.width, this.height);
            ops.renderTempObject(this.curTemp)
        }
    }

    onMouseMove(pt: math.Point3d, hovered: BABYLON.Mesh)
    {
        if(this.curTemp != null)
        {
            this.curTemp.set("second", new math.Point3d(pt.x, pt.y, 0));
            this.drawWall()
        }
        return this.canJoinToWall(hovered);
    }

    cancel()
    {
        if(this.undoEventId) {
            ops.cancelEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.curTemp.get("id"))
    }

    drawWall()
    {
        if(this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    finish(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        this.createWall(picked);
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
