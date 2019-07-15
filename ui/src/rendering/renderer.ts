import * as BABYLON from "babylonjs";
var gui = require('../ui/gui')
var mouse = require('../ui/mouse_events')
import * as ops from '../operations/operations'
import * as math from '../utils/math'

function getHoveredMesh(scene: BABYLON.Scene, ground: BABYLON.Mesh)
{
    var pickinfo = scene.pick(scene.pointerX, scene.pointerY, mesh => { return mesh != ground});
    if (pickinfo.hit) {
        return pickinfo.pickedMesh;
    }
    return null;
}

export class Renderer {
    private _canvas: HTMLCanvasElement
    private _engine: BABYLON.Engine
    private _scene: BABYLON.Scene
    private _highlight: BABYLON.HighlightLayer

    createScene(canvas: HTMLCanvasElement, engine: BABYLON.Engine) {
        this._canvas = canvas;
        this._engine = engine;
        // This creates a basic Babylon Scene object (non-mesh)
        const scene = new BABYLON.Scene(engine);
        var _highlight = new BABYLON.HighlightLayer("highlight1", scene);
        this._scene = scene;
        // This creates and positions a free camera (non-mesh)
        const camera = new BABYLON.ArcRotateCamera("camera1", -Math.PI / 2, 1.0, 110, BABYLON.Vector3.Zero(), scene);
        // This attaches the camera to the canvas
        camera.attachControl(canvas, true);
        // This creates a light, aiming 0,1,0 - to the sky (non-mesh)
        const light = new BABYLON.HemisphericLight("light1", new BABYLON.Vector3(0, 1, 0), scene);
        // Default intensity is 1. Let's dim the light a small amount
        light.intensity = 0.7;
        light.parent = camera;

        var ground = BABYLON.Mesh.CreateGround("ground", 1000, 1000, 0, scene, false);
        var groundMaterial = new BABYLON.StandardMaterial("ground", scene);
        groundMaterial.specularColor = BABYLON.Color3.Black();
        ground.material = groundMaterial;

        gui.guiInstance.init();

        var onPointerDown = (evt: MouseEvent) => {
            mouse.onPointerDown(this._scene, this._canvas, evt, ground, camera)
        }

        var current_hover: BABYLON.Mesh = null;
        var onPointerMove = (evt: MouseEvent) => {
            var hovered = getHoveredMesh(scene, ground)
            var layer = scene.getHighlightLayerByName("highlight1");
            if (current_hover && hovered != current_hover) {
                layer.removeMesh(current_hover)
            }
            if (hovered) {
                layer.addMesh(hovered as BABYLON.Mesh, BABYLON.Color3.Green());
                current_hover = hovered as BABYLON.Mesh;
            }

            mouse.onPointerMove(this._scene, ground)
        }

        canvas.addEventListener("pointerdown", onPointerDown, false);
        canvas.addEventListener("pointermove", onPointerMove, false);

        scene.onDispose = function () {
            canvas.removeEventListener("pointerdown", onPointerDown);
            canvas.removeEventListener("pointermove", onPointerMove);
        }
    }
    initialize(canvas: HTMLCanvasElement) {
        const engine = new BABYLON.Engine(canvas, true, {stencil: true});
        this.createScene(canvas, engine);
        engine.runRenderLoop(() => {
            this._scene.render();
        });
        window.addEventListener('resize', function () {
            engine.resize();
        });
    }

    renderMesh(triangles: any, id: string) {
        var mesh = this._scene.getMeshByName(id) as BABYLON.Mesh
        if(!mesh) {
            mesh = new BABYLON.Mesh(id, this._scene);
            var objMaterial = new BABYLON.StandardMaterial("obj", this._scene);
            objMaterial.diffuseColor = BABYLON.Color3.Gray();
            objMaterial.backFaceCulling = false;
            mesh.material = objMaterial;
            var pointerDragBehavior = new BABYLON.PointerDragBehavior({dragPlaneNormal: new BABYLON.Vector3(0,1,0)});
            pointerDragBehavior.useObjectOrienationForDragging = false;
            var event = '';
            pointerDragBehavior.onDragObservable.add((ev)=>{
                if(!event) {
                    event = ops.beginUndoEvent("Move object")
                    ops.takeUndoSnapshot(event, mesh.name)
                    ops.suspendEvent(event)
                }
                var modelDelta = math.transformGraphicToModelCoords(ev.delta) 
                ops.moveObj(event, mesh.name, modelDelta)
            })
            pointerDragBehavior.onDragEndObservable.add((ev)=>{
                if(event) {
                    ops.resumeEvent(event)
                    ops.endUndoEvent(event)
                }
                event = ''
            })
            pointerDragBehavior.moveAttached = false
            mesh.addBehavior(pointerDragBehavior)
        }
        var vertexData = new BABYLON.VertexData();
        vertexData.positions = triangles.positions;
        vertexData.indices = triangles.indices;
        vertexData.applyToMesh(mesh);
    }

    deleteMesh(id:string) {
        var mesh = this._scene.getMeshByName(id)
        mesh.dispose()
    }
}