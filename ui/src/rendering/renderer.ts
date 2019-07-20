import * as BABYLON from "babylonjs";
var gui = require('../ui/gui')
var mouse = require('../ui/mouse_events')
var uiController = require('../ui/controller')

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
            mouse.onPointerDown(this._scene, evt, ground)
        }

        var onPointerClick = (evt: MouseEvent) => {
            mouse.onPointerClick(this._scene, evt, ground)
        }

        var current_hover: BABYLON.Mesh = null;
        var onPointerMove = (evt: MouseEvent) => {
            var hovered = getHoveredMesh(scene, ground)
            var layer = scene.getHighlightLayerByName("highlight1");
            if (current_hover && hovered != current_hover) {
                layer.removeMesh(current_hover)
            }
            if(mouse.onPointerMove(this._scene, ground, hovered)) {
                if(hovered) {
                    layer.addMesh(hovered as BABYLON.Mesh, BABYLON.Color3.Green());
                    current_hover = hovered as BABYLON.Mesh;
                }
            }
        }

        this._scene.onPointerObservable.add((pointerInfo) => {
            switch(pointerInfo.type) {
                case BABYLON.PointerEventTypes.POINTERDOWN:
                    onPointerDown(pointerInfo.event)
                    break;
                case BABYLON.PointerEventTypes.POINTERUP:
                    break;
                case BABYLON.PointerEventTypes.POINTERMOVE:
                    onPointerMove(pointerInfo.event)
                    break;
                case BABYLON.PointerEventTypes.POINTERWHEEL:
                    break;
                case BABYLON.PointerEventTypes.POINTERPICK:
                    break;
                case BABYLON.PointerEventTypes.POINTERTAP:
                    onPointerClick(pointerInfo.event)
                    break;
                case BABYLON.PointerEventTypes.POINTERDOUBLETAP:
                    break;
            }
        });
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

    stop() {
        this._engine.stopRenderLoop();
    }

    renderMesh(triangles: any, id: string, temp?:boolean) {
        var mesh = this._scene.getMeshByName(id) as BABYLON.Mesh
        if(!mesh) {
            mesh = new BABYLON.Mesh(id, this._scene);
            var objMaterial = new BABYLON.StandardMaterial("obj", this._scene);
            objMaterial.diffuseColor = BABYLON.Color3.Gray();
            objMaterial.backFaceCulling = false;
            mesh.material = objMaterial;
            if(!temp) {
                var pointerDragBehavior = new BABYLON.PointerDragBehavior({dragPlaneNormal: new BABYLON.Vector3(0,1,0)});
                pointerDragBehavior.useObjectOrienationForDragging = false;
                var uiSingleton = new uiController().getInstance();
                pointerDragBehavior.onDragStartObservable.add((ev)=>{
                    uiSingleton.selectObj(mesh)
                })
                pointerDragBehavior.onDragObservable.add((ev)=>{
                    uiSingleton.moveSelected(ev);
                })
                pointerDragBehavior.onDragEndObservable.add((ev)=>{
                    uiSingleton.endMoveSelected(ev);
                })
                pointerDragBehavior.moveAttached = false
                mesh.addBehavior(pointerDragBehavior)
            }
        }
        mesh.metadata = triangles.metadata;
        var vertexData = new BABYLON.VertexData();
        vertexData.positions = triangles.positions;
        vertexData.indices = triangles.indices;
        vertexData.applyToMesh(mesh);
    }

    deleteMesh(id:string) {
        var mesh = this._scene.getMeshByName(id)
        if(mesh) {
            mesh.dispose()
        }
    }
}