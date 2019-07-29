use super::*;
use cgmath::InnerSpace;

pub fn rectangular_prism(first_pt: &Point3f, second_pt: &Point3f, width: WorldCoord, height: WorldCoord, results: &mut MeshData) {
    let dir = second_pt - first_pt;
    let perp = dir.cross(Vector3f::unit_z()).normalize();
    let offset = perp * width;
    let first = first_pt + offset;
    let second = first_pt - offset;
    let third = second_pt + offset;
    let fourth = second_pt - offset;
    let vert_offset = Vector3f::new(0.0, 0.0, height);
    let fifth = first + vert_offset;
    let sixth = second + vert_offset;
    let seventh = third + vert_offset;
    let eighth = fourth + vert_offset;
    results.push_pt(first);
    results.push_pt(second);
    results.push_pt(third);
    results.push_pt(fourth);
    results.push_pt(fifth);
    results.push_pt(sixth);
    results.push_pt(seventh);
    results.push_pt(eighth);
    results.indices.extend(&[0, 1, 2]);
    results.indices.extend(&[1, 2, 3]);
    results.indices.extend(&[0, 1, 5]);
    results.indices.extend(&[0, 5, 4]);
    results.indices.extend(&[4, 5, 7]);
    results.indices.extend(&[4, 7, 6]);
    results.indices.extend(&[1, 3, 7]);
    results.indices.extend(&[1, 7, 5]);
    results.indices.extend(&[2, 3, 7]);
    results.indices.extend(&[2, 7, 6]);
    results.indices.extend(&[2, 0, 4]);
    results.indices.extend(&[2, 4, 6]);
}