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
        return hovered && hovered.metadata && hovered.metadata.type == "Wall" && hovered.name != this.lastId && hovered.name != this.curTemp.get("id");
    }

    createWall(picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Wall")
        }
        var wall = new kernel.Wall(this.curTemp.get("first"), this.curTemp.get("second"), this.width, this.height);
        ops.deleteTempObject(this.curTemp.get("id"));
        ops.createObj(this.undoEventId, wall)
        if(this.lastId) {
            ops.joinAtPoints(this.undoEventId, this.lastId, wall.get("id"), wall.get("first"))
        }
        if(this.canJoinToWall(picked)) {
            ops.joinAtPoints(this.undoEventId, picked.name, wall.get("id"), wall.get("second"));
        }
        this.lastId = wall.get("id");
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
