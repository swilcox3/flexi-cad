import * as ops from '../../operations/operations'
import * as math from '../../utils/math'
const kernel = require("../../../native/index.node")

export class DimensionTool {
    curTemp: any
    lastAttached: BABYLON.Mesh
    offset: number
    undoEventId: string

    constructor(offset = 5)
    {
        this.curTemp = null;
        this.lastAttached = null;
        this.offset = offset;
        this.undoEventId = ''
    }

    canAttach(hovered: BABYLON.Mesh) {
        return hovered && hovered.metadata.type == "Wall";
    }

    createDimension(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Dimension")
        }
        ops.createObj(this.undoEventId, this.curTemp)
        if(this.canAttach(picked)) {
            ops.snapToPoint(this.undoEventId, this.curTemp.get("id"), picked.name, pt)
        }
    }

    onMouseDown(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp == null)
        {
            var first = new math.Point3d(pt.x, pt.y, 0)
            var second = new math.Point3d(pt.x + 1, pt.y, 0)
            this.curTemp = new kernel.Dimension(first, second, this.offset);
            ops.renderTempObject(this.curTemp)
        }
        else {
            this.createDimension(new math.Point3d(pt.x, pt.y, 0), picked);
            this.curTemp = null;
        }
    }

    onMouseMove(pt: math.Point3d, hovered: BABYLON.Mesh)
    {
        const joinable = this.canAttach(hovered);
        if(this.curTemp != null)
        {
            this.curTemp.set("second", new math.Point3d(pt.x, pt.y, 0));
            this.drawDimension()
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

    drawDimension()
    {
        if(this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    finish(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp) {
            this.createDimension(new math.Point3d(pt.x, pt.y, 0), picked);
        }
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
