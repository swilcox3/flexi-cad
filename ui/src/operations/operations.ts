var kernel = require('../../native/index.node')
import {Renderer} from '../rendering/renderer'
import * as math from '../utils/math'
import * as BABYLON from "babylonjs";

var renderers: Map<String, Renderer> = new Map()
var filename: string
var pendingCallbacks: Map<String, Array<(obj: BABYLON.Mesh) => void>> = new Map()

interface DataObject {
    get(prop: string): string,
    set(prop: string, val: any): string,
    getUpdateMsg(): any,
    addObject(filename: string, event: string):undefined
}

function initRenderer(canvas: HTMLCanvasElement)
{
    var renderer = new Renderer()
    renderer.initialize(canvas)
    return renderer
}

export function initFile(canvas: HTMLCanvasElement)
{
    renderers.delete(filename)
    filename = "defaultNew.flx"
    kernel.init_file(filename)
    renderers.set(filename, initRenderer(canvas))
    renderNext(filename)  //This will readd itself, so it's an infinite loop in the background
}

export function openFile(in_file:string, canvas:HTMLCanvasElement)
{
    filename = in_file;
    kernel.open_file(filename)
    renderers.set(filename, initRenderer(canvas))
    renderNext(filename)
}

export function saveFile()
{
    kernel.save_file(filename)
}

export function saveAsFile(in_file:string)
{
    kernel.save_as_file(filename, in_file)
}

export function beginUndoEvent(desc: string)
{
    return kernel.begin_undo_event(filename, desc)
}

export function endUndoEvent(event: string)
{
    kernel.end_undo_event(filename, event)
}

export function undoLatest()
{
    kernel.undo_latest(filename)
    renderNext(filename)
}

export function suspendEvent(event: string)
{
    kernel.suspend_event(filename, event)
}

export function resumeEvent(event: string)
{
    kernel.resume_event(filename, event)
}

export function cancelEvent(event: string)
{
    kernel.cancel_event(filename, event)
}

export function redoLatest()
{
    kernel.redo_latest(filename)
}

export function takeUndoSnapshot(event: string, id: string)
{
    kernel.take_undo_snapshot(filename, event, id)
}

export function renderTempObject(obj: DataObject) 
{
    var msg = obj.getUpdateMsg();
    if(msg.Mesh) {
        renderers.get(filename).renderMesh(msg.Mesh.data, msg.Mesh.data.id)
        return msg.Mesh.data.id
    }
    if(msg.Other) {
        renderers.get(filename).renderObject(msg.Other.data, msg.Other.data.id)
        return msg.Other.data.id
    }
}

export function deleteTempObject(id: string)
{
    renderers.get(filename).deleteMesh(id)
}

export function deleteObject(event: string, id: string)
{
    kernel.delete_object(filename, event, id)
}

function renderNext(filename: string) 
{
    kernel.get_updates(filename, (err: any, updates: any) => {
        if(!err) {
            updates.forEach((msg: any) => {
                //console.log(msg);
                if(msg.Delete) {
                    renderers.get(filename).deleteMesh(msg.Delete.key)
                }
                else {
                    var id = null;
                    if(msg.Mesh) {
                        id = msg.Mesh.data.id;
                        renderers.get(filename).renderMesh(msg.Mesh.data, id)
                    }
                    if(msg.Other) {
                        id = msg.Other.data.id;
                        renderers.get(filename).renderObject(msg.Other.data, id)
                    }
                    if(id) {
                        let callbacks = pendingCallbacks.get(id)
                        if(callbacks) {
                            let mesh = renderers.get(filename).getMesh(id)
                            callbacks.forEach((callback) => {
                                callback(mesh)
                            })
                        }
                        pendingCallbacks.delete(id)
                    }
                }
            })
        }
        renderNext(filename)
    })
}

function addPendingCallback(id: string, callback: (obj: BABYLON.Mesh) => void) 
{
    var arr = pendingCallbacks.get(id)
    if(!arr) {
        arr = []
    }
    arr.push(callback)
    pendingCallbacks.set(id, arr)
}

function waitForUpdate(id: string)
{
    return new Promise((resolve: (value: BABYLON.Mesh)=>void, reject) => {
        addPendingCallback(id, (mesh: BABYLON.Mesh) => {
            resolve(mesh)
        })
    })
}

function waitForAllUpdates(ids: Array<string>)
{
    var promises: Array<Promise<BABYLON.Mesh>> = [];
    ids.forEach((id) => {
        promises.push(waitForUpdate(id))
    })
    return Promise.all(promises)
}

export function createObj(event: string, obj: DataObject)
{
    obj.addObject(filename, event)
    return waitForUpdate(obj.get("id"));
}

export function joinAtPoints(event: string, id_1: string, id_2: string, pt: math.Point3d) 
{
    kernel.join_at_points(filename, event, id_1, id_2, pt)
    return waitForAllUpdates([id_1, id_2])
}

export function canReferTo(id:string)
{
    return kernel.can_refer_to(filename, id)
}

export function getClosestPoint(id:string, pt: math.Point3d)
{
    return kernel.get_closest_point(filename, id, pt)
}

export function snapToPoint(event: string, id: string, snap_to_id: string, pt: math.Point3d)
{
    kernel.snap_to_point(filename, event, id, snap_to_id, pt)
    return waitForUpdate(id)
}

export function snapToLine(event: string, id: string, snap_to_id: string, pt: math.Point3d) 
{
    kernel.snap_to_line(filename, event, id, snap_to_id, pt)
    return waitForUpdate(id)
}

export function moveObj(event: string, id: string, delta: math.Point3d)
{
    kernel.move_object(filename, event, id, delta)
    return waitForUpdate(id)
}

export function moveObjs(event: string, ids: Array<string>, delta: math.Point3d)
{
    kernel.move_objects(filename, event, ids, delta)
    return waitForAllUpdates(ids)
}

export async function getObjectData(id: string, prop_name: string)
{
    return await kernel.get_object_data(filename, id, prop_name)
}

export function setObjectData(event: string, id: string, data:any) 
{
    kernel.set_object_data(filename, event, id, JSON.stringify(data))
    return waitForUpdate(id)
}

export function setObjectsDatas(event: string, data: Array<[string, any]>)
{
    kernel.set_objects_datas(filename, event, data)
    return waitForAllUpdates(data.map(val => val[0]))
}

export function getMeshByID(id: string)
{
    return renderers.get(filename).getMesh(id)
}

export function copyObjs(event: string, ids:Array<string>, delta: math.Point3d)
{
    var copyIds: Array<[string, string]> = kernel.copy_objects(filename, event, ids, delta);
    return waitForAllUpdates(copyIds.map(val => val[1]))
}

export function debugState()
{
    kernel.debug_state();
}