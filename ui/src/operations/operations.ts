var kernel = require('../../native/index.node')
import {Renderer} from '../rendering/renderer'
import * as math from '../utils/math'

var renderers: Map<String, Renderer> = new Map()
var filename: string

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
    var existing = renderers.get(filename)
    if(existing) {
        existing.stop()
        renderers.delete(filename)
    }
    console.log(in_file)
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
    console.log(in_file)
    kernel.save_as_file(filename, in_file)
    var renderer = renderers.get(filename)
    renderers.delete(filename)
    filename = in_file
    renderers.set(filename, renderer)
}

export function getCurrentFile()
{
    return filename;
}

export function setCurrentFile(in_file:string)
{
    filename = in_file;
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

export function renderTempWall(firstPt: math.Point3d, secondPt: math.Point3d, width: number, height: number, id?: string) 
{
    var msg = kernel.get_temp_wall(firstPt, secondPt, width, height, id)
    if(msg.Mesh) {
        renderers.get(filename).renderMesh(msg.Mesh.data, msg.Mesh.data.id)
        return msg.Mesh.data.id
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
                if(msg.Mesh) {
                    renderers.get(filename).renderMesh(msg.Mesh.data, msg.Mesh.data.id)
                }
                if(msg.Delete) {
                    renderers.get(filename).deleteMesh(msg.Delete.key)
                }
            })
        }
        renderNext(filename)
    })
}

export function createWall(event: string, firstPt: math.Point3d, secondPt: math.Point3d, width: number, height: number, id?: string)
{
    kernel.create_wall(firstPt, secondPt, width, height, filename, event, id)
}

export function joinAtPoint(event: string, id_1: string, id_2: string, pt: math.Point3d) 
{
    kernel.join_at_point(filename, event, id_1, id_2, pt)
}

export function moveObj(event: string, id: string, delta: math.Point3d)
{
    kernel.move_object(filename, event, id, delta)
}

export function moveObjs(event: string, ids: Array<string>, delta: math.Point3d)
{
    kernel.move_objects(filename, event, ids, delta)
}

export function setObjectData(event: string, id: string, data:any) 
{
    kernel.set_object_data(filename, event, id, JSON.stringify(data))
}

export function setObjectsDatas(event: string, data: Array<[string, any]>)
{
    kernel.set_objects_datas(filename, event, data)
}

export function copyObjs(event: string, ids:Array<string>, delta: math.Point3d)
{
    console.log("Going into rust")
    kernel.copy_objects(filename, event, ids, delta)
}