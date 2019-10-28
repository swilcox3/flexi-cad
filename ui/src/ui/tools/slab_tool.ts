import * as ops from '../../operations/operations'
import { Point3d } from "../../utils/math"
import * as BABYLON from 'babylonjs'

export class SlabTool {
    curTemp: JsSlab;
    undoEventId: string

    constructor() {
        this.curTemp = null;
        this.undoEventId = ''
    }

    createSlab(picked: BABYLON.Mesh) {
        if (!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Slab")
        }
        var slab = new ops.dataModel.JsSlab();
        ops.deleteTempObject(this.curTemp.id());
        ops.createObj(this.undoEventId, slab)
    }

    onMouseDown(pt: Point3d, picked: BABYLON.Mesh) {
        if (this.curTemp == null) {
            this.curTemp = new ops.dataModel.JsSlab();
            ops.renderTempObject(this.curTemp)
        }
        else {
            this.createSlab(picked);
            this.curTemp = new ops.dataModel.JsSlab();
            ops.renderTempObject(this.curTemp)
        }
    }

    onMouseMove(pt: Point3d, hovered: BABYLON.Mesh) {
        if (this.curTemp != null) {
            //@ts-ignore
            this.drawSlab()
        }
        return false;
    }

    cancel() {
        if (this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
        if (this.curTemp) {
            ops.deleteTempObject(this.curTemp.id())
        }
    }

    drawSlab() {
        if (this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    finish(pt: Point3d, picked: BABYLON.Mesh) {
        this.createSlab(picked);
        if (this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
