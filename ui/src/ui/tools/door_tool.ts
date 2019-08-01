import * as ops from '../../operations/operations'
import * as math from '../../utils/math'
const kernel = require("../../../native/index.node")

export class DoorTool {
    curTemp: any
    width: number
    height: number
    length: number
    undoEventId: string

    constructor(width = 1, height = 4, length = 6)
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
            var second = new math.Point3d(pt.x + this.length, pt.y, 0)
            this.curTemp = new kernel.Door(first, second, this.width, this.height, this.length);
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
        const joinable = this.canJoinToWall(hovered);
        if(this.curTemp != null)
        {
            if(joinable) {


            }
            else {
                this.curTemp.set("first", new math.Point3d(pt.x, pt.y, 0));
                this.curTemp.set("second", new math.Point3d(pt.x + this.length, pt.y, 0));
            }
            this.drawDoor()
        }
        return joinable;
    }

    cancel()
    {
        if(this.undoEventId) {
            ops.cancelEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.curTemp.get("id"))
    }

    drawDoor()
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
