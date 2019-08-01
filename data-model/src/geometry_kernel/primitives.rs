use super::*;
use cgmath::InnerSpace;

pub struct PrismOpening {
    pub interp: Interp,
    pub height: WorldCoord,
    pub length: WorldCoord
}

pub fn prism_and_next_opening(first_pt: &Point3f, second_pt: &Point3f, third_pt: &Point3f, offset: &Vector3f, vert_offset: &Vector3f, hole_offset: &Vector3f, index: u64, results: &mut MeshData) -> u64 {
    let zero = first_pt + offset;
    let first = first_pt - offset;
    let second = second_pt + offset;
    let third = second_pt - offset;
    let fourth = zero + vert_offset;
    let fifth = first + vert_offset;
    let sixth = second + vert_offset;
    let seventh = third + vert_offset;
    let eighth = second + hole_offset;
    let ninth = third + hole_offset;
    let tenth = third_pt + offset + hole_offset;
    let eleventh = third_pt - offset + hole_offset;
    let twelvth = third_pt + offset + vert_offset;
    let thirteenth = third_pt - offset + vert_offset;
    results.push_pt(zero);
    results.push_pt(first);
    results.push_pt(second);
    results.push_pt(third);
    results.push_pt(fourth);
    results.push_pt(fifth);
    results.push_pt(sixth);
    results.push_pt(seventh);
    results.push_pt(eighth);
    results.push_pt(ninth);
    results.push_pt(tenth);
    results.push_pt(eleventh);
    results.push_pt(twelvth);
    results.push_pt(thirteenth);
    results.indices.extend(&[index, index + 1, index + 2]);
    results.indices.extend(&[index + 1, index + 2, index + 3]);
    results.indices.extend(&[index, index + 1, index + 5]);
    results.indices.extend(&[index + 0, index + 5, index + 4]);
    results.indices.extend(&[index + 4, index + 5, index + 7]);
    results.indices.extend(&[index + 4, index + 7, index + 6]);
    results.indices.extend(&[index + 1, index + 3, index + 7]);
    results.indices.extend(&[index + 1, index + 7, index + 5]);
    results.indices.extend(&[index + 2, index + 3, index + 9]);
    results.indices.extend(&[index + 2, index + 9, index + 8]);
    results.indices.extend(&[index + 2, index + 0, index + 4]);
    results.indices.extend(&[index + 2, index + 4, index + 6]);
    results.indices.extend(&[index + 8, index + 9, index + 10]);
    results.indices.extend(&[index + 9, index + 10, index + 11]);
    results.indices.extend(&[index + 8, index + 6, index + 10]);
    results.indices.extend(&[index + 6, index + 10, index + 12]);
    results.indices.extend(&[index + 6, index + 7, index + 12]);
    results.indices.extend(&[index + 7, index + 12, index + 13]);
    results.indices.extend(&[index + 7, index + 9, index + 11]);
    results.indices.extend(&[index + 7, index + 11, index + 13]);
    index + 14
}

fn prism(first_pt: &Point3f, second_pt: &Point3f, offset: &Vector3f, vert_offset: &Vector3f, index: u64, results: &mut MeshData) -> u64 {
    let first = first_pt + offset;
    let second = first_pt - offset;
    let third = second_pt + offset;
    let fourth = second_pt - offset;
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
    results.indices.extend(&[index, index + 1, index + 2]);
    results.indices.extend(&[index + 1, index + 2, index + 3]);
    results.indices.extend(&[index, index + 1, index + 5]);
    results.indices.extend(&[index, index + 5, index + 4]);
    results.indices.extend(&[index + 4, index + 5, index + 7]);
    results.indices.extend(&[index + 4, index + 7, index + 6]);
    results.indices.extend(&[index + 1, index + 3, index + 7]);
    results.indices.extend(&[index + 1, index + 7, index + 5]);
    results.indices.extend(&[index + 2, index + 3, index + 7]);
    results.indices.extend(&[index + 2, index + 7, index + 6]);
    results.indices.extend(&[index + 2, index, index + 4]);
    results.indices.extend(&[index + 2, index + 4, index + 6]);
    index + 8
}

pub fn prism_with_openings(first_pt: &Point3f, second_pt: &Point3f, width: WorldCoord, height: WorldCoord, holes: Vec<PrismOpening>, results: &mut MeshData) {
    let num_pts = 8 + 12 * holes.len();
    results.positions.reserve(num_pts * 3);
    let num_faces = 12 * (holes.len() + 1) + 8 * (holes.len());
    results.indices.reserve(num_faces * 3);
    let dir = second_pt - first_pt;
    let perp = dir.cross(Vector3f::unit_z()).normalize();
    let offset = perp * width;
    let vert_offset = Vector3f::new(0.0, 0.0, height);
    let mut cur_first = *first_pt;
    let mut index = 0;
    for hole in holes {
        let cur_second = first_pt + dir * hole.interp.val;
        let cur_third = cur_second + dir.normalize() * hole.length;
        let hole_offset = Vector3f::new(0.0, 0.0, hole.height);
        index = prism_and_next_opening(&cur_first, &cur_second, &cur_third, &offset, &vert_offset, &hole_offset, index, results);
        cur_first = cur_third;
    }
    prism(&cur_first, &second_pt, &offset, &vert_offset, index, results);
}

pub fn rectangular_prism(first_pt: &Point3f, second_pt: &Point3f, width: WorldCoord, height: WorldCoord, results: &mut MeshData) {
    let dir = second_pt - first_pt;
    let perp = dir.cross(Vector3f::unit_z()).normalize();
    let offset = perp * width;
    let vert_offset = Vector3f::new(0.0, 0.0, height);
    prism(&first_pt, &second_pt, &offset, &vert_offset, 0, results);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_prism() {
        let first = Point3f::new(0.0, 0.0, 0.0);
        let second = Point3f::new(1.0, 0.0, 0.0);
        let width = 1.0;
        let height = 1.0;
        let mut results = MeshData {
            id: RefID::nil(),
            positions: Vec::new(),
            indices: Vec::new(),
            metadata: None
        };
        rectangular_prism(&first, &second, width, height, &mut results);
        assert_eq!(results.positions, vec![0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0]);
        assert_eq!(results.indices, vec![0, 1, 2, 1, 2, 3, 0, 1, 5, 0, 5, 4, 4, 5, 7, 4, 7, 6, 1, 3, 7, 1, 7, 5, 2, 3, 7, 2, 7, 6, 2, 0, 4, 2, 4, 6]);
    }

    #[test]
    fn test_prism_with_openings() {
        let first = Point3f::new(0.0, 0.0, 0.0);
        let second = Point3f::new(6.0, 0.0, 0.0);
        let width = 1.0;
        let height = 1.0;
        let holes = vec![PrismOpening {
            interp: Interp::new(0.5),
            height: 0.75,
            length: 1.0,
        }];
        let mut results = MeshData {
            id: RefID::nil(),
            positions: Vec::new(),
            indices: Vec::new(),
            metadata: None
        };
        prism_with_openings(&first, &second, width, height, holes, &mut results);
        println!("{:?}", results.positions);
        println!("{:?}", results.indices);
    }


}