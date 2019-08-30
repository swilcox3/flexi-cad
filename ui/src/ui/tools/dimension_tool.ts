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

    async canAttach(hovered: BABYLON.Mesh) {
        return hovered && await ops.canReferTo(hovered.name);
    }

    async createDimension(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Dimension")
        }
        var dim = new kernel.Dimension(this.curTemp.get("first"), this.curTemp.get("second"), this.offset)
        ops.deleteTempObject(this.curTemp.get("id"))
        ops.createObj(this.undoEventId, dim)
        if(await this.canAttach(picked)) {
            ops.snapToPoint(this.undoEventId, dim.get("id"), picked.name, dim.get("first"))
            ops.snapToPoint(this.undoEventId, dim.get("id"), picked.name, pt)
        }
    }

    async onMouseDown(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp == null)
        {
            var first = null;
            if(await this.canAttach(picked)) {
                first = await ops.getClosestPoint(picked.name, new math.Point3d(pt.x, pt.y, 0));
            }
            else {
                first = new math.Point3d(pt.x, pt.y, 0)
            }
            var second = new math.Point3d(pt.x + 1, pt.y, 0)
            this.curTemp = new kernel.Dimension(first, second, this.offset);
            ops.renderTempObject(this.curTemp)
        }
        else {
            this.createDimension(new math.Point3d(pt.x, pt.y, 0), picked);
            this.curTemp = null;
        }
    }

    async onMouseMove(pt: math.Point3d, hovered: BABYLON.Mesh)
    {
        const joinable = await this.canAttach(hovered);
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

    async finish(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp) {
            await this.createDimension(new math.Point3d(pt.x, pt.y, 0), picked);
        }
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
