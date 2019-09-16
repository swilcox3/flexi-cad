import {JsDimension, Point3d, dataModelWasm} from '../../operations/operations'

export class DimensionTool {
    curTemp: JsDimension
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

    async createDimension(pt: Point3d, picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Dimension")
        }
        var dim = new dataModelWasm.JsDimension(this.curTemp.first_pt, this.curTemp.second_pt, this.offset)
        ops.deleteTempObject(this.curTemp.id)
        ops.createObj(this.undoEventId, dim)
        if(await this.canAttach(picked)) {
            ops.snapToPoint(this.undoEventId, dim.id, picked.name, dim.first_pt)
            ops.snapToPoint(this.undoEventId, dim.id, picked.name, pt)
        }
    }

    async onMouseDown(pt: Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp == null)
        {
            var first = null;
            if(await this.canAttach(picked)) {
                first = await ops.getClosestPoint(picked.name, new dataModelWasm.Point3d(pt.x, pt.y, 0));
            }
            else {
                first = new dataModelWasm.Point3d(pt.x, pt.y, 0)
            }
            var second = new dataModelWasm.Point3d(pt.x + 1, pt.y, 0)
            this.curTemp = new dataModelWasm.JsDimension(first, second, this.offset);
            ops.renderTempObject(this.curTemp)
        }
        else {
            this.createDimension(new dataModelWasm.Point3d(pt.x, pt.y, 0), picked);
            this.curTemp = null;
        }
    }

    async onMouseMove(pt: Point3d, hovered: BABYLON.Mesh)
    {
        const joinable = await this.canAttach(hovered);
        if(this.curTemp != null)
        {
            this.curTemp.second_pt = new dataModelWasm.Point3d(pt.x, pt.y, 0);
            this.drawDimension()
        }
        return joinable;
    }

    cancel()
    {
        if(this.undoEventId) {
            ops.cancelEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.curTemp.id)
    }

    drawDimension()
    {
        if(this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    async finish(pt: Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp) {
            await this.createDimension(new dataModelWasm.Point3d(pt.x, pt.y, 0), picked);
        }
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
