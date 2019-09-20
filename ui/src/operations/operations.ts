//@ts-ignore
import WebSocketAsPromised from "websocket-as-promised"

/*export type DataModelMod = typeof import("../../data-model-wasm/pkg/index");
export var dataModel: DataModelMod = null;

var loaded = import("../../data-model-wasm/pkg/index").then( mod => {
    dataModel = mod;
    user = dataModel.getUserId();
});
export type Point3d = import("../../data-model-wasm/pkg/index").Point3d;
export type Vector3d = import("../../data-model-wasm/pkg/index").Vector3d;
export type JsDimension = import("../../data-model-wasm/pkg/index").JsDimension;
export type JsWall = import("../../data-model-wasm/pkg/index").JsWall;
export type JsDoor = import("../../data-model-wasm/pkg/index").JsDoor;*/

var kernel = require("../../native/index.node")
export var dataModel = kernel;
export class Point3d {
    public x: number
    public y: number
    public z: number
    constructor(x: number, y: number, z: number) {
        this.x = x;
        this.y = y;
        this.z = z;
    }
}

export class Vector3d {
    public x: number
    public y: number
    public z: number
    constructor(x: number, y: number, z: number) {
        this.x = x;
        this.y = y;
        this.z = z;
    }
}
dataModel.Point3d = Point3d;
dataModel.Vector3d = Vector3d;
export type JsDimension = import("../../native/index").JsDimension;
export type JsWall = import("../../native/index").JsWall;
export type JsDoor = import("../../native/index").JsDoor;

var user:string = kernel.getUserId();

import {Renderer} from '../rendering/renderer'

var renderer: Renderer = null;
var filename: string = "";
var pendingChanges: Map<String, Array<(obj: BABYLON.Mesh) => void>> = new Map();
var pendingReads: Map<String, (val: any) => void> = new Map();

var connection: WebSocketAsPromised = null;

export interface DataObject {
    getTempRepr(): any
    moveObj(delta: Vector3d): void
    getObj(): any
    id(): string
}

function initRenderer(canvas: HTMLCanvasElement)
{
    renderer = new Renderer()
    renderer.initialize(canvas)
}

export async function setConnection(conn: string) {
    if(conn) {
        conn = conn + "?user_id=" + user;
        console.log(conn)
        connection = new WebSocketAsPromised(conn, {
            packMessage: (data:any) => JSON.stringify(data),
            unpackMessage: (data:any) => {
                JSON.parse(data);
            }
        });
        connection.onUnpackedMessage.addListener((data:any) => {
            handleUpdate(data)
        })
        await connection.open();
    }
}

function send(func: string, params: Array<any>) {
    var msg = {
        "func_name": func,
        "params": params
    }
    connection.sendPacked(msg)
}

export function initFile(canvas: HTMLCanvasElement)
{
    filename = "defaultNew.flx"
    if(connection) {
        send("init_file", [filename])
    }
    else {
        kernel.init_file(filename, user)
    }
    initRenderer(canvas)
    renderNext(filename)  //This will readd itself, so it's an infinite loop in the background
}

export function openFile(in_file:string, canvas:HTMLCanvasElement)
{
    filename = in_file;
    if(connection) {
        send("open_file", [filename])
    }
    else {
        kernel.open_file(filename, user)
    }
    initRenderer(canvas)
    renderNext(filename)
}

export function saveFile()
{
    if(connection) {
        send("save_file", [filename])
    }
    else {
        kernel.save_file(filename)
    }
}

export function saveAsFile(in_file:string)
{
    if(connection) {
        send("save_as_file", [filename, in_file])
    }
    else {
        kernel.save_as_file(filename, in_file)
    }
}

export function beginUndoEvent(desc: string)
{
    var event = kernel.getUndoEventId();
    if(connection) {
        send("begin_undo_event", [filename, event, desc])
    }
    else {
        kernel.begin_undo_event(filename, user, event, desc)
    }
    return event;
}

export function endUndoEvent(event: string)
{
    if(connection) {
        send("end_undo_event", [filename, event])
    }
    else {
        kernel.end_undo_event(filename, event)
    }
}

export function undoLatest()
{
    if(connection) {
        send("undo_latest", [filename])
    }
    else {
        kernel.undo_latest(filename, user)
    }
    renderNext(filename)
}

export function suspendEvent(event: string)
{
    if(connection) {
        send("suspend_event", [filename, event])
    }
    else {
        kernel.suspend_event(filename, event)
    }
}

export function resumeEvent(event: string)
{
    if(connection) {
        send("resume_event", [filename, event])
    }
    else {
        kernel.resume_event(filename, event)
    }
}

export function cancelEvent(event: string)
{
    if(connection) {
        send("cancel_event", [filename, event])
    }
    else {
        kernel.cancel_event(filename, event)
    }
}

export function redoLatest()
{
    if(connection) {
        send("redo_latest", [filename])
    }
    else {
        kernel.redo_latest(filename, user)
    }
}

export function takeUndoSnapshot(event: string, id: string)
{
    if(connection) {
        send("take_undo_snapshot", [filename, event, id])
    }
    else {
        kernel.take_undo_snapshot(filename, event, id)
    }
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
    if(connection) {
        send("delete_object", [filename, event, id])
    }
    else {
        kernel.delete_object(filename, event, id)
    }
}

function handleUpdate(msg: any) {
    //console.log(msg);
    if(msg.Error) {
        console.log(msg.Error.msg)
    }
    else if(msg.Delete) {
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
                console.log("Made it mesh")
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
}

function handleUpdates(err: any, updates:any) {
    if(!err) {
        updates.forEach((msg: any) => {
            handleUpdate(msg)
        })
    }
    renderNext(filename)
}

function renderNext(filename: string) 
{
    if(!connection) {
        kernel.get_updates(filename, handleUpdates);
    }
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
    if(connection) {
        send("add_object", [filename, event, json_obj.type, json_obj.obj])
    }
    else {
        kernel.add_object(filename, event, json_obj.type, json_obj.obj)
    }
    return waitForChange(obj.id());
}

export function joinAtPoints(event: string, id_1: string, id_2: string, pt: Point3d) 
{
    if(connection) {
        send("join_at_points", [filename, event, id_1, id_2, pt])
    }
    else {
        kernel.join_at_points(filename, event, id_1, id_2, pt)
    }
    return waitForAllChanges([id_1, id_2])
}

export function canReferTo(id:string)
{
    const query = kernel.getQueryId();
    if(connection) {
        send("can_refer_to", [filename, id, query])
    } 
    else {
        kernel.can_refer_to(filename, id, user, query)
    }
    return waitForRead(query)
}

export function getClosestPoint(id:string, pt: Point3d)
{
    const query = kernel.getQueryId();
    if(connection) {
        send("get_closest_point", [filename, id, pt, query])
    }
    else {
        kernel.get_closest_point(filename, id, pt, user, query)
    }
    return waitForRead(query)
}

export function snapToPoint(event: string, id: string, snap_to_id: string, pt: Point3d)
{
    if(connection) {
        send("snap_to_point", [filename, event, id, snap_to_id, pt])
    }
    else {
        kernel.snap_to_point(filename, event, id, snap_to_id, pt)
    }
    return waitForChange(id)
}

export function snapToLine(event: string, id: string, snap_to_id: string, pt: Point3d) 
{
    if(connection) {
        send("snap_to_line", [filename, event, id, snap_to_id, pt])
    }
    else {
        kernel.snap_to_line(filename, event, id, snap_to_id, pt)
    }
    return waitForChange(id)
}

export function moveObj(event: string, id: string, delta: Point3d)
{
    if(connection) {
        send("move_object", [filename, event, id, delta])
    }
    else {
        kernel.move_object(filename, event, id, delta)
    }
    return waitForChange(id)
}

export function moveObjs(event: string, ids: Array<string>, delta: Point3d)
{
    if(connection) {
        send("move_objects", [filename, event, ids, delta])
    }
    else {
        kernel.move_objects(filename, event, ids, delta)
    }
    return waitForAllChanges(ids)
}

export function getObjectData(id: string, prop_name: string)
{
    const query = kernel.getQueryId();
    if(connection) {
        send("get_object_data", [filename, id, prop_name, query])
    }
    else {
        kernel.get_object_data(filename, id, prop_name, user, query)
    }
    return waitForRead(query)
}

export function setObjectData(event: string, id: string, data:any) 
{
    if(connection) {
        send("set_object_data", [filename, event, id, data])
    }
    else {
        kernel.set_object_data(filename, event, id, JSON.stringify(data))
    }
    return waitForChange(id)
}

export function setObjectsDatas(event: string, data: Array<[string, any]>)
{
    if(connection) {
        send("set_object_datas", [filename, event, data])
    }
    else {
        kernel.set_objects_datas(filename, event, data)
    }
    return waitForAllChanges(data.map(val => val[0]))
}

export function getMeshByID(id: string)
{
    return renderer.getMesh(id)
}

export function copyObjs(event: string, ids:Array<string>, delta: Point3d)
{
    const query = kernel.getQueryId();
    if(connection) {
        send("copy_objects", [filename, event, ids, delta, user, query])
    }
    else {
        kernel.copy_objects(filename, event, ids, delta, user, query);
    }
    return waitForRead(query)
}

export function demo(position: Point3d)
{
    if(connection) {
        send("demo", [filename, position])
    }
    else {
        kernel.demo(filename, position, user);
    }
}

export function demo_100(position: Point3d)
{
    if(connection) {
        send("demo_100", [filename, position])
    }
    else {
        kernel.demo_100(filename, position, user);
    }
}

export function createDataObjectFromJSON(data: any) 
{
    switch (data.type)
    {
        case "Wall":
            return new kernel.JsWall(data.obj.first_pt.geom.pt, data.obj.second_pt.geom.pt, data.obj.width, data.obj.height)
        case "Door":
            return new kernel.JsDoor(data.obj.dir.geom.pt_1, data.obj.dir.geom.pt_2, data.obj.width, data.obj.height)
    }
}