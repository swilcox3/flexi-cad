var BABYLON = require('babylonjs')
console.log("Made it renderer 1");
var mouse = require('../ui/mouse_events')
console.log("Made it renderer 2");
var gui = require('../ui/gui')
console.log("Made it renderer 3");
var uiController = require('../ui/controller')
console.log("Made it renderer 4");

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
        const camera = new BABYLON.ArcRotateCamera("camera1", -Math.PI / 2, 1.0, 500, new BABYLON.Vector3(500, 0, -500), scene);
        camera.panningSensibility = 50;
        camera.panningInertia = .7;
        // This attaches the camera to the canvas
        camera.attachControl(canvas, true);
        // This creates a light, aiming 0,1,0 - to the sky (non-mesh)
        const light = new BABYLON.HemisphericLight("light1", new BABYLON.Vector3(0, 1, 0), scene);
        // Default intensity is 1. Let's dim the light a small amount
        light.intensity = 0.7;
        light.parent = camera;

        var ground = BABYLON.Mesh.CreateGround("ground", 10000, 10000, 0, scene, false);
        var groundMaterial = new BABYLON.StandardMaterial("ground", scene);
        groundMaterial.specularColor = BABYLON.Color3.Black();
        ground.material = groundMaterial;

        gui.guiInstance.init();

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

    applyNewMeshProps(mesh: BABYLON.Mesh, temp?:boolean) {
        if(!temp) {
            var objMaterial = new BABYLON.StandardMaterial("obj", this._scene);
            objMaterial.diffuseColor = BABYLON.Color3.Gray();
            objMaterial.backFaceCulling = false;
            mesh.material = objMaterial;
            var pointerDragBehavior = new BABYLON.PointerDragBehavior({dragPlaneNormal: new BABYLON.Vector3(0,1,0)});
            pointerDragBehavior.useObjectOrienationForDragging = false;
            var uiSingleton = new uiController().getInstance();
            pointerDragBehavior.onDragObservable.add((ev:any)=>{
                uiSingleton.objDrag(ev, mesh)
            })
            pointerDragBehavior.onDragEndObservable.add((ev:any)=>{
                uiSingleton.objDragEnd(ev)
            })
            pointerDragBehavior.moveAttached = false;
            mesh.addBehavior(pointerDragBehavior)
        }
        else {
            var objMaterial = new BABYLON.StandardMaterial("temp", this._scene);
            objMaterial.backFaceCulling = false;
            objMaterial.wireframe = true;
            mesh.material = objMaterial;
        }
    }

    renderMesh(triangles: any, id: string, temp?:boolean) {
        var mesh = this._scene.getMeshByName(id) as BABYLON.Mesh
        if(!mesh) {
            mesh = new BABYLON.Mesh(id, this._scene);
            this.applyNewMeshProps(mesh, temp);
        }
        mesh.metadata = triangles.metadata;
        var vertexData = new BABYLON.VertexData();
        vertexData.positions = triangles.positions;
        vertexData.indices = triangles.indices;
        vertexData.applyToMesh(mesh);
    }

    renderObject(json: any, id: string, temp?:boolean) {
        var mesh = this._scene.getMeshByName(id) as BABYLON.Mesh
        switch (json.metadata.type) {
            case "Dimension":
                var first = new BABYLON.Vector3(json.first.x, 1, json.first.z);
                var first_off = new BABYLON.Vector3(json.first_off.x, 1, json.first_off.z);
                var second = new BABYLON.Vector3(json.second.x, 1, json.second.z);
                var second_off = new BABYLON.Vector3(json.second_off.x, 1, json.second_off.z);
                var text_pos = new BABYLON.Vector3(json.text_pos.x, 1, json.text_pos.z);

                if(!mesh) {
                    mesh = new BABYLON.MeshBuilder.CreateSphere(id, {diameter: 2}, this._scene);
                }
                this.applyNewMeshProps(mesh, temp);
                mesh.metadata = json.metadata;
                mesh.position = text_pos;

                var line_1 = this._scene.getMeshByName(id + "_line1");
                if(!line_1) {
                    var line_1_pts = [first, first_off];
                    line_1 = BABYLON.MeshBuilder.CreateLines(id + "_line1", {points: line_1_pts, updatable: true}, this._scene)
                }
                else {
                    var positions = line_1.getVerticesData(BABYLON.VertexBuffer.PositionKind);
                    positions = [first.x, first.y, first.z, first_off.x, first_off.y, first_off.z]
                    line_1.updateVerticesData(BABYLON.VertexBuffer.PositionKind, positions);
                }
                
                var line_2 = this._scene.getMeshByName(id + "_line2");
                if(!line_2) {
                    var line_2_pts = [second, second_off];
                    line_2 = BABYLON.MeshBuilder.CreateLines(id + "_line2", {points: line_2_pts, updatable: true}, this._scene)
                }
                else {
                    var positions = line_2.getVerticesData(BABYLON.VertexBuffer.PositionKind);
                    positions = [second.x, second.y, second.z, second_off.x, second_off.y, second_off.z]
                    line_2.updateVerticesData(BABYLON.VertexBuffer.PositionKind, positions);
                }
                var line_3 = this._scene.getMeshByName(id + "_line3");
                if(!line_3) {
                    var line_3_pts = [first_off, second_off];
                    line_3 = BABYLON.MeshBuilder.CreateLines(id + "_line3", {points: line_3_pts, updatable: true}, this._scene)
                }
                else {
                    var positions = line_3.getVerticesData(BABYLON.VertexBuffer.PositionKind);
                    positions = [first_off.x, first_off.y, first_off.z, second_off.x, second_off.y, second_off.z]
                    line_3.updateVerticesData(BABYLON.VertexBuffer.PositionKind, positions);
                }
                break;
        }
    }

    deleteMesh(id:string) {
        var mesh = this._scene.getMeshByName(id)
        if(mesh) {
            mesh.dispose()
        }
        var mesh = this._scene.getMeshByName(id + "_line1")
        if(mesh) {
            mesh.dispose()
        }
        var mesh = this._scene.getMeshByName(id + "_line2")
        if(mesh) {
            mesh.dispose()
        }
        var mesh = this._scene.getMeshByName(id + "_line3")
        if(mesh) {
            mesh.dispose()
        }
    }

    getMesh(id:string) {
        return this._scene.getMeshByName(id) as BABYLON.Mesh
    }
}