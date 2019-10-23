import { Point3d, Vector3d } from "../utils/math"
import * as BABYLON from 'babylonjs'

export var dataModel: any = null;
var user: string = null;
var connection: any = null;
export function initialize(mod: any) {
    dataModel = mod;
    user = dataModel.getUserId();
    return user;
}

export async function setConnection(conn?: any) {
    if (conn) {
        connection = conn;
        connection.onUnpackedMessage.addListener((data: any) => {
            handleUpdate(data);
        });
        await connection.open();
    }
}

import { Renderer } from '../rendering/renderer'

var renderer: Renderer = null;
var filename: string = "";
var pendingChanges: Map<String, Array<(obj: BABYLON.Mesh) => void>> = new Map();
var pendingReads: Map<String, (val: any) => void> = new Map();


export interface DataObject {
    getTempRepr(): any
    moveObj(delta: Vector3d): void
    getObj(): any
    id(): string
}

function initRenderer(canvas: HTMLCanvasElement) {
    renderer = new Renderer()
    renderer.initialize(canvas)
}

function send(func: string, params: Array<any>) {
    var msg = {
        "func_name": func,
        "params": params
    }
    connection.sendPacked(msg)
}

export function initFile(canvas: HTMLCanvasElement) {
    filename = "defaultNew.flx"
    if (connection) {
        send("init_file", [filename])
    }
    else {
        dataModel.init_file(filename, user)
    }
    initRenderer(canvas)
    renderNext(filename)  //This will readd itself, so it's an infinite loop in the background
}

export function openFile(in_file: string, canvas: HTMLCanvasElement) {
    filename = in_file;
    if (connection) {
        send("open_file", [filename])
    }
    else {
        dataModel.open_file(filename, user)
    }
    initRenderer(canvas)
    renderNext(filename)
}

export function saveFile() {
    if (connection) {
        send("save_file", [filename])
    }
    else {
        dataModel.save_file(filename)
    }
}

export function saveAsFile(in_file: string) {
    if (connection) {
        send("save_as_file", [filename, in_file])
    }
    else {
        dataModel.save_as_file(filename, in_file)
    }
}

export function beginUndoEvent(desc: string) {
    var event = dataModel.getUndoEventId();
    if (connection) {
        send("begin_undo_event", [filename, event, desc])
    }
    else {
        dataModel.begin_undo_event(filename, user, event, desc)
    }
    return event;
}

export function endUndoEvent(event: string) {
    if (connection) {
        send("end_undo_event", [filename, event])
    }
    else {
        dataModel.end_undo_event(filename, event)
    }
}

export function undoLatest() {
    if (connection) {
        send("undo_latest", [filename])
    }
    else {
        dataModel.undo_latest(filename, user)
    }
    renderNext(filename)
}

export function suspendEvent(event: string) {
    if (connection) {
        send("suspend_event", [filename, event])
    }
    else {
        dataModel.suspend_event(filename, event)
    }
}

export function resumeEvent(event: string) {
    if (connection) {
        send("resume_event", [filename, event])
    }
    else {
        dataModel.resume_event(filename, event)
    }
}

export function cancelEvent(event: string) {
    if (connection) {
        send("cancel_event", [filename, event])
    }
    else {
        dataModel.cancel_event(filename, event)
    }
}

export function redoLatest() {
    if (connection) {
        send("redo_latest", [filename])
    }
    else {
        dataModel.redo_latest(filename, user)
    }
}

export function takeUndoSnapshot(event: string, id: string) {
    if (connection) {
        send("take_undo_snapshot", [filename, event, id])
    }
    else {
        dataModel.take_undo_snapshot(filename, event, id)
    }
}

export function renderTempObject(obj: DataObject) {
    var msg = obj.getTempRepr();
    if (msg.Mesh) {
        renderer.renderMesh(msg.Mesh.data, msg.Mesh.data.id, true)
    }
    if (msg.Other) {
        renderer.renderObject(msg.Other.data, msg.Other.data.id, true)
    }
}

export function deleteTempObject(id: string) {
    renderer.deleteMesh(id)
}

export function deleteObject(event: string, id: string) {
    if (connection) {
        send("delete_object", [filename, event, id])
    }
    else {
        dataModel.delete_object(filename, event, id)
    }
}

function handleUpdate(msg: any) {
    console.log(msg);
    if (msg.Error) {
        console.log(msg.Error.msg)
    }
    else if (msg.Delete) {
        renderer.deleteMesh(msg.Delete.key)
    }
    else {
        if (msg.Read) {
            var cb = pendingReads.get(msg.Read.query_id)
            if (cb) {
                cb(msg.Read.data)
                pendingReads.delete(msg.Read.query_id)
            }
        }
        else {
            var id = null;
            if (msg.Mesh) {
                id = msg.Mesh.data.id;
                renderer.renderMesh(msg.Mesh.data, id)
            }
            if (msg.Other) {
                id = msg.Other.data.id;
                renderer.renderObject(msg.Other.data, id)
            }
            if (id) {
                let callbacks = pendingChanges.get(id)
                if (callbacks) {
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

function handleUpdates(err: any, updates: any) {
    if (!err) {
        updates.forEach((msg: any) => {
            handleUpdate(msg)
        })
    }
    renderNext(filename)
}

function renderNext(filename: string) {
    if (!connection) {
        dataModel.get_updates(filename, handleUpdates);
    }
}

function addPendingChange(id: string, callback: (obj: BABYLON.Mesh) => void) {
    var arr = pendingChanges.get(id)
    if (!arr) {
        arr = []
    }
    arr.push(callback)
    pendingChanges.set(id, arr)
}

function addPendingRead(id: string, callback: (val: any) => void) {
    pendingReads.set(id, callback)
}

function waitForChange(id: string) {
    return new Promise((resolve: (value: BABYLON.Mesh) => void, reject) => {
        addPendingChange(id, (mesh: BABYLON.Mesh) => {
            resolve(mesh)
        })
    })
}

function waitForRead(id: string, mapper?: (val: any) => any) {
    return new Promise((resolve: (val: any) => void, reject) => {
        addPendingRead(id, (val: any) => {
            if (mapper) {
                val = mapper(val)
            }
            resolve(val)
        })
    })
}

function waitForAllChanges(ids: Array<string>) {
    var promises: Array<Promise<BABYLON.Mesh>> = [];
    ids.forEach((id) => {
        promises.push(waitForChange(id))
    })
    return Promise.all(promises)
}

function waitForAllReads(ids: Array<string>) {
    var promises: Array<Promise<BABYLON.Mesh>> = [];
    ids.forEach((id) => {
        promises.push(waitForRead(id))
    })
    return Promise.all(promises)
}

function renderFromMsg(msg: any) {
    if (msg.Mesh) {
        renderer.renderMesh(msg.Mesh.data, msg.Mesh.data.id, false)
    }
    if (msg.Other) {
        renderer.renderObject(msg.Other.data, msg.Other.data.id, false)
    }
}

export function createObj(event: string, obj: DataObject) {
    renderFromMsg(obj.getTempRepr());
    let json_obj = obj.getObj();
    if (connection) {
        send("add_object", [filename, event, json_obj.type, json_obj.obj])
    }
    else {
        dataModel.add_object(filename, event, json_obj.type, json_obj.obj)
    }
    return waitForChange(obj.id());
}

export function joinAtPoints(event: string, id_1: string, id_2: string, pt: Point3d) {
    if (connection) {
        send("join_at_points", [filename, event, id_1, id_2, pt])
    }
    else {
        dataModel.join_at_points(filename, event, id_1, id_2, pt)
    }
    return waitForAllChanges([id_1, id_2])
}

export function canReferTo(id: string) {
    var mesh = renderer.getMesh(id);
    if (mesh) {
        if (mesh.metadata) {
            if (mesh.metadata.traits) {
                if (mesh.metadata.traits.includes("ReferTo")) {
                    return true;
                }
            }
        }
    }
    return false;
}

export function getClosestPoint(id: string, pt: Point3d) {
    const query = dataModel.getQueryId();
    if (connection) {
        send("get_closest_point", [filename, id, pt, query])
    }
    else {
        dataModel.get_closest_point(filename, id, pt, user, query)
    }
    return waitForRead(query, (val: any) => {
        if (val) {
            return val[1].Point.pt;
        }
        else {
            return val
        }
    })
}

export function snapToPoint(event: string, id: string, snap_to_id: string, pt: Point3d) {
    if (connection) {
        send("snap_to_point", [filename, event, id, snap_to_id, pt])
    }
    else {
        dataModel.snap_to_point(filename, event, id, snap_to_id, pt)
    }
    return waitForChange(id)
}

export function snapToLine(event: string, id: string, snap_to_id: string, pt: Point3d) {
    if (connection) {
        send("snap_to_line", [filename, event, id, snap_to_id, pt])
    }
    else {
        dataModel.snap_to_line(filename, event, id, snap_to_id, pt)
    }
    return waitForChange(id)
}

export function moveObj(event: string, id: string, delta: Point3d) {
    if (connection) {
        send("move_object", [filename, event, id, delta])
    }
    else {
        dataModel.move_object(filename, event, id, delta)
    }
    return waitForChange(id)
}

export function moveObjs(event: string, ids: Array<string>, delta: Point3d) {
    if (connection) {
        send("move_objects", [filename, event, ids, delta])
    }
    else {
        dataModel.move_objects(filename, event, ids, delta)
    }
    return waitForAllChanges(ids)
}

export function getObjectData(id: string, prop_name: string) {
    const query = dataModel.getQueryId();
    if (connection) {
        send("get_object_data", [filename, id, prop_name, query])
    }
    else {
        dataModel.get_object_data(filename, id, prop_name, user, query)
    }
    return waitForRead(query)
}

export function setObjectData(event: string, id: string, data: any) {
    if (connection) {
        send("set_object_data", [filename, event, id, data])
    }
    else {
        dataModel.set_object_data(filename, event, id, JSON.stringify(data))
    }
    return waitForChange(id)
}

export function setObjectsDatas(event: string, data: Array<[string, any]>) {
    if (connection) {
        send("set_object_datas", [filename, event, data])
    }
    else {
        dataModel.set_objects_datas(filename, event, data)
    }
    return waitForAllChanges(data.map(val => val[0]))
}

export function getMeshByID(id: string) {
    return renderer.getMesh(id)
}

export function copyObjs(event: string, ids: Array<string>, delta: Point3d) {
    const query = dataModel.getQueryId();
    if (connection) {
        send("copy_objects", [filename, event, ids, delta, user, query])
    }
    else {
        dataModel.copy_objects(filename, event, ids, delta, user, query);
    }
    return waitForRead(query)
}

export function demo(position: Point3d) {
    if (connection) {
        send("demo", [filename, position])
    }
    else {
        dataModel.demo(filename, position, user);
    }
}

export function demo_100(position: Point3d) {
    if (connection) {
        send("demo_100", [filename, position])
    }
    else {
        dataModel.demo_100(filename, position, user);
    }
}

export function createDataObjectFromJSON(data: any) {
    console.log(data)
    switch (data.type) {
        case "Wall":
            return new dataModel.JsWall(data.obj.First, data.obj.Second, data.obj.Width, data.obj.Height)
        case "Door":
            return new dataModel.JsDoor(data.obj.First, data.obj.Second, data.obj.Width, data.obj.Height)
    }
}