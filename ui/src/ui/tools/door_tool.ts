import * as ops from '../../operations/operations'
import * as math from '../../utils/math'
const kernel = require("../../../native/index.node")

export class DoorTool {
    curTemp: any
    width: number
    height: number
    length: number
    undoEventId: string

    constructor(width = 1, height = 4, length = 2)
    {
        this.curTemp = null;
        this.width = width;
        this.height = height;
        this.length = length;
        this.undoEventId = ''
    }

    canJoinToWall(hovered: BABYLON.Mesh) {
        return hovered && hovered.metadata.type == "Wall";
    }

    createDoor(picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Door")
        }
        ops.createObj(this.undoEventId, this.curTemp)
    }

    onMouseDown(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp == null)
        {
            var first = new math.Point3d(pt.x, pt.y, 0)
            var second = new math.Point3d(pt.x + 1, pt.y + 1, 0)
            this.curTemp = new kernel.Door(first, second, this.width, this.height, this.length);
            this.curTemp.set_dir(new math.Point3d(1, 0, 0))
            ops.renderTempObject(this.curTemp)
        }
        else
        {
            this.createDoor(picked);
            this.curTemp = null;
        }
    }

    onMouseMove(pt: math.Point3d, hovered: BABYLON.Mesh)
    {
        if(this.curTemp != null)
        {
            var cur_first = this.curTemp.get("first");
            var cur_dir = new math.Point3d(pt.x - cur_first.x, pt.y - cur_first.y, 0);
            this.curTemp.set_dir(cur_dir)
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
        this.createDoor(picked);
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
