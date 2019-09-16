var user:string = null;

export type WasmMod = typeof import("../../data-model-wasm/pkg/index");
export var dataModelWasm: WasmMod = null;

var loaded = import("../../data-model-wasm/pkg/index").then( mod => {
    dataModelWasm = mod;
    user = dataModelWasm.getUserId();
});
export type Point3d = import("../../data-model-wasm/pkg/index").Point3d;
export type Vector3d = import("../../data-model-wasm/pkg/index").Vector3d;
export type JsDimension = import("../../data-model-wasm/pkg/index").JsDimension;
export type JsWall = import("../../data-model-wasm/pkg/index").JsWall;
export type JsDoor = import("../../data-model-wasm/pkg/index").JsDoor;

var kernel = require('../../native/index.node')
import {Renderer} from '../rendering/renderer'

var renderer: Renderer = null;
var filename: string = "";
var connection: string = null;
var pendingChanges: Map<String, Array<(obj: BABYLON.Mesh) => void>> = new Map();
var pendingReads: Map<String, (val: any) => void> = new Map();

export interface DataObject {
    getTempRepr(): any
    moveObj(delta: Vector3d): void
    getObj(): any
    readonly id: string
}

function initRenderer(canvas: HTMLCanvasElement)
{
    renderer = new Renderer()
    renderer.initialize(canvas)
}

export function setConnection(conn: string) {
    connection = conn;
}

export function initFile(canvas: HTMLCanvasElement)
{
    loaded.then( () => {
        console.log("Compiled!")
        filename = "defaultNew.flx"
        kernel.init_file(filename, user)
        initRenderer(canvas)
        renderNext(filename)  //This will readd itself, so it's an infinite loop in the background
    })
}

export function openFile(in_file:string, canvas:HTMLCanvasElement)
{
    filename = in_file;
    kernel.open_file(filename, user)
    initRenderer(canvas)
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
    return kernel.begin_undo_event(filename, desc, user)
}

export function endUndoEvent(event: string)
{
    kernel.end_undo_event(filename, event)
}

export function undoLatest()
{
    kernel.undo_latest(filename, user)
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
    kernel.redo_latest(filename, user)
}

export function takeUndoSnapshot(event: string, id: string)
{
    kernel.take_undo_snapshot(filename, event, id)
}

export function renderTempObject(obj: DataObject) 
{
    var msg = obj.getTempRepr();
    if(msg.Mesh) {
        renderer.renderMesh(msg.Mesh.data, msg.Mesh.data.id, true)
    }
    if(msg.Other) {
        renderer.renderObject(msg.Other.data, msg.Other.data.id, true)
    }
}

export function deleteTempObject(id: string)
{
    renderer.deleteMesh(id)
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
                    renderer.deleteMesh(msg.Delete.key)
                }
                else {
                    if(msg.Read) {
                        var cb = pendingReads.get(msg.Read.query_id) 
                        if(cb) {
                            cb(msg.Read.data)
                            pendingReads.delete(msg.Read.query_id)
                        }
                    }
                    else {
                        var id = null;
                        if(msg.Mesh) {
                            id = msg.Mesh.data.id;
                            renderer.renderMesh(msg.Mesh.data, id)
                        }
                        if(msg.Other) {
                            id = msg.Other.data.id;
                            renderer.renderObject(msg.Other.data, id)
                        }
                        if(id) {
                            let callbacks = pendingChanges.get(id)
                            if(callbacks) {
                                let mesh = renderer.getMesh(id)
                                callbacks.forEach((callback) => {
                                    callback(mesh)
                                })
                            }
                            pendingChanges.delete(id)
                        }
                    }
                }
            })
        }
        renderNext(filename)
    })
}

function addPendingChange(id: string, callback: (obj: BABYLON.Mesh) => void) 
{
    var arr = pendingChanges.get(id)
    if(!arr) {
        arr = []
    }
    arr.push(callback)
    pendingChanges.set(id, arr)
}

function addPendingRead(id: string, callback: (val: any) => void)
{
    pendingReads.set(id, callback)
}

function waitForChange(id: string)
{
    return new Promise((resolve: (value: BABYLON.Mesh)=>void, reject) => {
        addPendingChange(id, (mesh: BABYLON.Mesh) => {
            resolve(mesh)
        })
    })
}

function waitForRead(id: string)
{
    return new Promise((resolve: (val: any)=>void, reject) => {
        addPendingRead(id, (val: any) => {
            resolve(val)
        })
    })
}

function waitForAllChanges(ids: Array<string>)
{
    var promises: Array<Promise<BABYLON.Mesh>> = [];
    ids.forEach((id) => {
        promises.push(waitForChange(id))
    })
    return Promise.all(promises)
}

function waitForAllReads(ids: Array<string>)
{
    var promises: Array<Promise<BABYLON.Mesh>> = [];
    ids.forEach((id) => {
        promises.push(waitForRead(id))
    })
    return Promise.all(promises)
}

function renderFromMsg(msg: any)
{
    if(msg.Mesh) {
        renderer.renderMesh(msg.Mesh.data, msg.Mesh.data.id, false)
    }
    if(msg.Other) {
        renderer.renderObject(msg.Other.data, msg.Mesh.data.id, false)
    }
}

export function createObj(event: string, obj: DataObject)
{
    renderFromMsg(obj.getTempRepr());
    let json_obj = obj.getObj();
    kernel.add_object(filename, event, json_obj.type, json_obj.obj)
    return waitForChange(obj.id);
}

export function joinAtPoints(event: string, id_1: string, id_2: string, pt: Point3d) 
{
    kernel.join_at_points(filename, event, id_1, id_2, pt)
    return waitForAllChanges([id_1, id_2])
}

export function canReferTo(id:string)
{
    const query = kernel.can_refer_to(filename, id, user)
    return waitForRead(query)
}

export function getClosestPoint(id:string, pt: Point3d)
{
    const query = kernel.get_closest_point(filename, id, pt, user)
    return waitForRead(query)
}

export function snapToPoint(event: string, id: string, snap_to_id: string, pt: Point3d)
{
    kernel.snap_to_point(filename, event, id, snap_to_id, pt)
    return waitForChange(id)
}

export function snapToLine(event: string, id: string, snap_to_id: string, pt: Point3d) 
{
    kernel.snap_to_line(filename, event, id, snap_to_id, pt)
    return waitForChange(id)
}

export function moveObj(event: string, id: string, delta: Point3d)
{
    kernel.move_object(filename, event, id, delta)
    return waitForChange(id)
}

export function moveObjs(event: string, ids: Array<string>, delta: Point3d)
{
    kernel.move_objects(filename, event, ids, delta)
    return waitForAllChanges(ids)
}

export function getObjectData(id: string, prop_name: string)
{
    const query = kernel.get_object_data(filename, id, prop_name, user)
    return waitForRead(query)
}

export function setObjectData(event: string, id: string, data:any) 
{
    kernel.set_object_data(filename, event, id, JSON.stringify(data))
    return waitForChange(id)
}

export function setObjectsDatas(event: string, data: Array<[string, any]>)
{
    kernel.set_objects_datas(filename, event, data)
    return waitForAllChanges(data.map(val => val[0]))
}

export function getMeshByID(id: string)
{
    return renderer.getMesh(id)
}

export function copyObjs(event: string, ids:Array<string>, delta: Point3d)
{
    var copyIds: Array<[string, string]> = kernel.copy_objects(filename, event, ids, delta, user);
    return waitForAllChanges(copyIds.map(val => val[1]))
}

export async function demo(position: Point3d)
{
    await kernel.demo(filename, position, user);
}

export async function demo_100(position: Point3d)
{
    await kernel.demo_100(filename, position, user);
}

export function createDataObjectFromJSON(data: any) 
{
    switch (data.type)
    {
        case "Wall":
            return new kernel.Wall(data.obj.first_pt.geom.pt, data.obj.second_pt.geom.pt, data.obj.width, data.obj.height)
        case "Door":
            return new kernel.Door(data.obj.dir.geom.pt_1, data.obj.dir.geom.pt_2, data.obj.width, data.obj.height)
    }
}