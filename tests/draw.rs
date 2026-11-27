//! The outlines of curved faces in a hidden-line drawing.

use springrep::{
    draw::hidden_lines,
    math::{Frame, Point3, Vec3},
    primitive::cylinder,
    tol,
};

#[test]
fn a_side_view_shows_both_generators_of_a_cylinder() {
    let (radius, height) = (5.0, 2.0);
    let b = cylinder(&Frame::WORLD, radius, height, "post").unwrap();
    let drawing = hidden_lines(&b, Vec3::Y, 0.1).unwrap();
    let generators: Vec<_> = drawing
        .visible
        .iter()
        .filter(|line| {
            let Some((a, z)) = line.first().zip(line.last()) else {
                return false;
            };
            ((a.z - z.z).abs() - height).abs() <= tol::LINEAR
        })
        .collect();
    assert_eq!(generators.len(), 2);
    // Looking along y, the grazing points lie at (±r, 0) all the way up the cylinder.
    for x in [-radius, radius] {
        let foot = Point3::new(x, 0.0, 0.0);
        let tip = Point3::new(x, 0.0, height);
        assert!(generators.iter().any(|line| {
            let (a, z) = (line[0], line[line.len() - 1]);
            (a.dist(foot) <= tol::LINEAR && z.dist(tip) <= tol::LINEAR)
                || (z.dist(foot) <= tol::LINEAR && a.dist(tip) <= tol::LINEAR)
        }));
    }
}

#[test]
fn a_thin_foreground_block_hides_a_short_arc_of_a_rim() {
    use std::f64::consts::PI;

    let radius = 100.0;
    let half_width = 2.0;
    let mut scene = cylinder(&Frame::WORLD, radius, 1.0, "rim").unwrap();
    let radial = Frame::WORLD.radial(PI / 64.0);
    let tangent = Vec3::Z.cross(radial);
    let origin = radial * (radius - 0.06) - tangent * half_width + Vec3::Z * 4.0;
    let frame = Frame::new(origin, Vec3::Z, radial).unwrap();
    let foreground =
        springrep::primitive::block(&frame, Vec3::new(0.16, 2.0 * half_width, 1.0), "bar").unwrap();
    scene.append(&foreground);

    let drawing = hidden_lines(&scene, -Vec3::Z, 0.5).unwrap();
    let hidden: Vec<_> = drawing
        .hidden
        .iter()
        .filter(|line| (line[0].z - 1.0).abs() <= tol::LINEAR)
        .collect();
    assert_eq!(hidden.len(), 1, "{hidden:?}");
    let line = hidden[0];
    let along_radial = (radius * radius - half_width * half_width).sqrt();
    for side in [-1.0, 1.0] {
        let crossing = radial * along_radial + tangent * (side * half_width) + Vec3::Z;
        assert!(
            [line[0], *line.last().unwrap()]
                .into_iter()
                .any(|p| p.dist(crossing) <= tol::LINEAR),
            "{line:?} should end at {crossing:?}"
        );
    }
}

#[test]
fn a_foreground_pin_hides_an_arc_between_two_rims_seen_obliquely() {
    use std::f64::consts::PI;

    let radius = 100.0;
    let pin_radius = 0.8;
    let centre_distance = radius + pin_radius - 0.08;
    let radial = Frame::WORLD.radial(PI / 64.0);
    let tangent = Vec3::Z.cross(radial);
    let viewer = Vec3::Z - radial * 0.25;
    let mut scene = cylinder(&Frame::WORLD, radius, 1.0, "rim").unwrap();
    let frame = Frame::new(
        radial * centre_distance + Vec3::Z + viewer * 3.0,
        Vec3::Z,
        Vec3::X,
    )
    .unwrap();
    scene.append(&cylinder(&frame, pin_radius, 1.0, "pin").unwrap());

    let drawing = hidden_lines(&scene, -viewer, 0.5).unwrap();
    let hidden: Vec<_> = drawing
        .hidden
        .iter()
        .filter(|line| line.iter().all(|p| (p.z - 1.0).abs() <= tol::LINEAR))
        .collect();
    assert_eq!(hidden.len(), 1, "{hidden:?}");
    let x = (radius * radius + centre_distance * centre_distance - pin_radius * pin_radius)
        / (2.0 * centre_distance);
    let y = (radius * radius - x * x).sqrt();
    let line = hidden[0];
    for side in [-1.0, 1.0] {
        let crossing = radial * x + tangent * (side * y) + Vec3::Z;
        assert!(
            [line[0], *line.last().unwrap()]
                .into_iter()
                .any(|p| p.dist(crossing) <= tol::LINEAR),
            "{line:?} should end at {crossing:?}"
        );
    }
}

#[test]
fn a_cylinder_rim_hides_the_inner_end_of_a_straight_edge() {
    let radius = 100.0;
    let half_width = 2.0;
    let inner_radius = radius - 0.06;
    let radial = Frame::WORLD.radial(std::f64::consts::PI / 64.0);
    let tangent = Vec3::Z.cross(radial);
    let origin = radial * inner_radius - tangent * half_width - Vec3::Z * 4.0;
    let frame = Frame::new(origin, Vec3::Z, radial).unwrap();
    let strip =
        springrep::primitive::block(&frame, Vec3::new(0.16, 2.0 * half_width, 1.0), "strip")
            .unwrap();
    let mut scene = cylinder(&Frame::WORLD, radius, 1.0, "rim").unwrap();
    scene.append(&strip);

    let drawing = hidden_lines(&scene, -Vec3::Z, 0.5).unwrap();
    let inner = origin + Vec3::Z;
    let rim_radius = (radius * radius - half_width * half_width).sqrt();
    let crossing = inner + radial * (rim_radius - inner_radius);
    assert!(
        drawing.hidden.iter().any(|line| {
            let (a, z) = (line[0], *line.last().unwrap());
            (a.dist(inner) <= tol::LINEAR && z.dist(crossing) <= tol::LINEAR)
                || (z.dist(inner) <= tol::LINEAR && a.dist(crossing) <= tol::LINEAR)
        }),
        "expected a hidden edge from {inner:?} to {crossing:?}, got {:?}",
        drawing.hidden
    );
}
