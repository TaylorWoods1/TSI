//! Venue TOML parsing for the CLI (shared with tests).

use spyder_core::{Anchor, PlatformAttachment, Preset, Robot, Vec3};

/// Parse a venue TOML file into a [`Robot`].
pub fn robot_from_toml(text: &str) -> Robot {
    let mut preset = String::from("rect");
    let mut width = None;
    let mut depth = None;
    let mut height = None;
    let mut n = None;
    let mut radius = None;
    let mut point_mass = true;
    let mut anchors: Vec<Anchor> = Vec::new();
    let mut attachments: Vec<PlatformAttachment> = Vec::new();
    let mut cur_xyz: Option<(Option<f64>, Option<f64>, Option<f64>)> = None;
    let mut cur_kind = "";

    let flush = |cur: &mut Option<(Option<f64>, Option<f64>, Option<f64>)>,
                 kind: &str,
                 anchors: &mut Vec<Anchor>,
                 attachments: &mut Vec<PlatformAttachment>| {
        if let Some((x, y, z)) = cur.take() {
            let v = Vec3::new(
                x.expect("x"),
                y.expect("y"),
                z.expect("z"),
            );
            if kind == "anchor" {
                anchors.push(Anchor::point(v));
            } else if kind == "attachment" {
                attachments.push(PlatformAttachment::at(v));
            }
        }
    };

    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[anchors]]" {
            flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments);
            cur_kind = "anchor";
            cur_xyz = Some((None, None, None));
            continue;
        }
        if line == "[[attachments]]" {
            flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments);
            cur_kind = "attachment";
            cur_xyz = Some((None, None, None));
            continue;
        }
        if line.starts_with('[') {
            flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments);
            cur_kind = "";
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if let Some(tuple) = cur_xyz.as_mut() {
                match k {
                    "x" => tuple.0 = Some(v.parse().unwrap()),
                    "y" => tuple.1 = Some(v.parse().unwrap()),
                    "z" => tuple.2 = Some(v.parse().unwrap()),
                    _ => {}
                }
                continue;
            }
            match k {
                "preset" => preset = v.to_string(),
                "width" => width = Some(v.parse().unwrap()),
                "depth" => depth = Some(v.parse().unwrap()),
                "height" => height = Some(v.parse().unwrap()),
                "n" => n = Some(v.parse().unwrap()),
                "radius" => radius = Some(v.parse().unwrap()),
                "point_mass" => point_mass = v.parse().unwrap(),
                _ => {}
            }
        }
    }
    flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments);

    if !anchors.is_empty() {
        let atts = if attachments.is_empty() {
            None
        } else {
            Some(attachments)
        };
        let mut r = Robot::from_anchors(anchors, atts, point_mass).expect("robot");
        r.point_mass = point_mass;
        return r;
    }

    let p = match preset.as_str() {
        "rect" => Preset::Rect {
            width: width.expect("width"),
            depth: depth.expect("depth"),
            height: height.expect("height"),
        },
        "polygon" => Preset::RegularPolygon {
            n: n.expect("n"),
            radius: radius.expect("radius"),
            height: height.expect("height"),
        },
        other => panic!("unknown preset {other}"),
    };
    let mut r = Robot::from_preset(p).expect("robot");
    if !attachments.is_empty() {
        r.attachments = attachments;
        r.point_mass = point_mass;
    } else {
        r.point_mass = point_mass;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn parses_rect_preset() {
        let text = r#"
preset = "rect"
width = 10.0
depth = 6.0
height = 8.0
point_mass = true
"#;
        let robot = robot_from_toml(text);
        assert_eq!(robot.anchors.len(), 4);
        assert!(robot.point_mass);
    }

    #[test]
    fn parses_explicit_anchors() {
        let text = r#"
point_mass = true
[[anchors]]
x = 5.0
y = 3.0
z = 8.0
[[anchors]]
x = -5.0
y = 3.0
z = 8.0
[[anchors]]
x = -5.0
y = -3.0
z = 8.0
"#;
        let robot = robot_from_toml(text);
        assert_eq!(robot.anchors.len(), 3);
        assert_relative_eq!(robot.anchors[0].exit.x, 5.0);
    }

    #[test]
    fn ignores_comments() {
        let text = r#"
# comment line
preset = "rect"
width = 4.0 # inline
depth = 4.0
height = 3.0
point_mass = true
"#;
        let robot = robot_from_toml(text);
        assert_eq!(robot.anchors.len(), 4);
    }

    #[test]
    fn parses_polygon_preset() {
        let text = r#"
preset = "polygon"
n = 5
radius = 4.0
height = 7.0
point_mass = true
"#;
        let robot = robot_from_toml(text);
        assert_eq!(robot.anchors.len(), 5);
    }
}
