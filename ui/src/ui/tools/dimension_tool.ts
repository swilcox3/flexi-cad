import * as ops from '../../operations/operations'
import { Point3d } from "../../utils/math"
import * as BABYLON from 'babylonjs'

export class DimensionTool {
    curTemp: JsDimension
    lastAttached: BABYLON.Mesh
    offset: number
    undoEventId: string

    constructor(offset = 5) {
        this.curTemp = null;
        this.lastAttached = null;
        this.offset = offset;
        this.undoEventId = ''
    }

    canAttach(hovered: BABYLON.Mesh) {
        return hovered && ops.canReferTo(hovered.name);
    }

    createDimension(pt: Point3d, picked: BABYLON.Mesh) {
        if (!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Dimension")
        }
        var dim = new ops.dataModel.JsDimension(this.curTemp.first_pt(), this.curTemp.second_pt(), this.offset)
        ops.deleteTempObject(this.curTemp.id())
        ops.createObj(this.undoEventId, dim)
        if (this.canAttach(picked)) {
            ops.snapToPoint(this.undoEventId, dim.id(), picked.name, dim.first_pt())
            ops.snapToPoint(this.undoEventId, dim.id(), picked.name, pt)
        }
    }

    onMouseDown(pt: Point3d, picked: BABYLON.Mesh) {
        if (this.curTemp == null) {
            var first: Point3d = null;
            if (this.canAttach(picked)) {
                ops.getClosestPoint(picked.name, new Point3d(pt.x, pt.y, 0)).then(pt => {
                    first = pt;
                })
            }
            else {
                first = new Point3d(pt.x, pt.y, 0)
            }
            var second = new Point3d(pt.x + 1, pt.y, 0)
            this.curTemp = new ops.dataModel.JsDimension(first, second, this.offset);
            ops.renderTempObject(this.curTemp)
        }
        else {
            this.createDimension(new Point3d(pt.x, pt.y, 0), picked);
            this.curTemp = null;
        }
    }

    onMouseMove(pt: Point3d, hovered: BABYLON.Mesh) {
        if (this.curTemp != null) {
            //@ts-ignore
            this.curTemp.set_second_pt(new Point3d(pt.x, pt.y, 0));
            this.drawDimension()
        }
        var joinable = false;
        if (this.canAttach(hovered)) {
            joinable = true;
        }
        return joinable;
    }

    cancel() {
        if (this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.curTemp.id())
    }

    drawDimension() {
        if (this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    finish(pt: Point3d, picked: BABYLON.Mesh) {
        if (this.curTemp) {
            this.createDimension(new Point3d(pt.x, pt.y, 0), picked);
            if (this.undoEventId) {
                ops.endUndoEvent(this.undoEventId)
            }
        }
    }
}
