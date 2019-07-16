var kernel = require('../../native/index.node')
import * as uuid from 'uuid/v1'
import {Renderer} from '../rendering/renderer'
import * as math from '../utils/math'

var renderers: Map<String, Renderer> = new Map()

function getFilename()
{
    return document.getElementById('render-canvas').getAttribute('data-filename')
}

export function initFile(renderer: Renderer)
{
    let filename = getFilename()
    kernel.init_file(filename)
    renderers.set(filename, renderer)
    renderNext(filename)  //This will readd itself, so it's an infinite loop in the background
}

export function beginUndoEvent(desc: string)
{
    let filename = getFilename()
    return kernel.begin_undo_event(filename, desc)
}

export function endUndoEvent(event: string)
{
    let filename = getFilename()
    kernel.end_undo_event(filename, event)
}

export function undoLatest()
{
    let filename = getFilename()
    kernel.undo_latest(filename)
    renderNext(filename)
}

export function suspendEvent(event: string)
{
    let filename = getFilename()
    kernel.suspend_event(filename, event)
}

export function resumeEvent(event: string)
{
    let filename = getFilename()
    kernel.resume_event(filename, event)
}

export function cancelEvent(event: string)
{
    let filename = getFilename()
    kernel.cancel_event(filename, event)
}

export function redoLatest()
{
    let filename = getFilename()
    kernel.redo_latest(filename)
}

export function takeUndoSnapshot(event: string, id: string)
{
    let filename = getFilename()
    kernel.take_undo_snapshot(filename, event, id)
}

export function renderTempWall(firstPt: math.Point3d, secondPt: math.Point3d, width: number, height: number, id?: string) 
{
    let filename = getFilename()
    var msg = kernel.get_temp_wall(firstPt, secondPt, width, height, id)
    if(msg.Mesh) {
        renderers.get(filename).renderMesh(msg.Mesh.data, msg.Mesh.data.id)
        return msg.Mesh.data.id
    }
}

export function deleteTempObject(id: string)
{
    let filename = getFilename()
    renderers.get(filename).deleteMesh(id)
}

export function deleteObject(event: string, id: string)
{
    let filename = getFilename()
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
    let filename = getFilename()
    kernel.create_wall(firstPt, secondPt, width, height, filename, event, id)
}

export function joinWalls(event: string, id_1: string, id_2: string, pt: math.Point3d) 
{
    let filename = getFilename()
    kernel.join_walls(filename, event, id_1, id_2, pt)
}

export function moveObj(event: string, id: string, delta: math.Point3d)
{
    let filename = getFilename()
    kernel.move_object(filename, event, id, delta)
}

export function setObjectData(event: string, id: string, data:any) 
{
    let filename = getFilename()
    kernel.set_object_data(filename, event, id, JSON.stringify(data))
}