//! Venue TOML parse and emit helpers.

use spyder_core::{Anchor, PlatformAttachment, Robot, Vec3};
use spyder_runtime::venue_toml_from_anchors;

/// Parse venue TOML into a robot and home pose.
pub fn parse_venue_toml(text: &str) -> Result<(Robot, Vec3), String> {
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
    let mut home = Vec3::new(0.0, 0.0, 2.0);
    let mut in_home = false;

    let flush = |cur: &mut Option<(Option<f64>, Option<f64>, Option<f64>)>,
                 kind: &str,
                 anchors: &mut Vec<Anchor>,
                 attachments: &mut Vec<PlatformAttachment>| {
        if let Some((x, y, z)) = cur.take() {
            let v = Vec3::new(
                x.ok_or("missing x")?,
                y.ok_or("missing y")?,
                z.ok_or("missing z")?,
            );
            if kind == "anchor" {
                anchors.push(Anchor::point(v));
            } else if kind == "attachment" {
                attachments.push(PlatformAttachment::at(v));
            }
            Ok::<(), String>(())
        } else {
            Ok(())
        }
    };

    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[anchors]]" {
            flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments)?;
            cur_kind = "anchor";
            in_home = false;
            cur_xyz = Some((None, None, None));
            continue;
        }
        if line == "[[attachments]]" {
            flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments)?;
            cur_kind = "attachment";
            in_home = false;
            cur_xyz = Some((None, None, None));
            continue;
        }
        if line == "[home]" {
            flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments)?;
            cur_kind = "";
            in_home = true;
            continue;
        }
        if line.starts_with('[') {
            flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments)?;
            cur_kind = "";
            in_home = false;
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if let Some(tuple) = cur_xyz.as_mut() {
                match k {
                    "x" => tuple.0 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                    "y" => tuple.1 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                    "z" => tuple.2 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                    _ => {}
                }
                continue;
            }
            if in_home {
                match k {
                    "x" => home.x = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                    "y" => home.y = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                    "z" => home.z = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                    _ => {}
                }
                continue;
            }
            match k {
                "preset" => preset = v.to_string(),
                "width" => width = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "depth" => depth = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "height" => height = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "n" => n = Some(v.parse::<usize>().map_err(|e: std::num::ParseIntError| e.to_string())?),
                "radius" => radius = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "point_mass" => point_mass = v.parse::<bool>().map_err(|e: std::str::ParseBoolError| e.to_string())?,
                _ => {}
            }
        }
    }
    flush(&mut cur_xyz, cur_kind, &mut anchors, &mut attachments)?;

    if !anchors.is_empty() {
        let atts = if attachments.is_empty() {
            None
        } else {
            Some(attachments)
        };
        let mut r = Robot::from_anchors(anchors, atts, point_mass).map_err(|e| e.to_string())?;
        r.point_mass = point_mass;
        return Ok((r, home));
    }

    use spyder_core::Preset;
    let p = match preset.as_str() {
        "rect" => Preset::Rect {
            width: width.ok_or("missing width")?,
            depth: depth.ok_or("missing depth")?,
            height: height.ok_or("missing height")?,
        },
        "polygon" => Preset::RegularPolygon {
            n: n.ok_or("missing n")?,
            radius: radius.ok_or("missing radius")?,
            height: height.ok_or("missing height")?,
        },
        other => return Err(format!("unknown preset {other}")),
    };
    let mut r = Robot::from_preset(p).map_err(|e| e.to_string())?;
    if !attachments.is_empty() {
        r.attachments = attachments;
    }
    r.point_mass = point_mass;
    Ok((r, home))
}

/// Emit venue TOML from anchors, attachments, and home.
pub fn emit_venue_toml(
    anchors: &[Vec3],
    attachments: &[Vec3],
    point_mass: bool,
    home: Vec3,
) -> Result<String, String> {
    let anchors_m: Vec<[f64; 3]> = anchors
        .iter()
        .map(|a| [a.x, a.y, a.z])
        .collect();
    let atts: Option<Vec<[f64; 3]>> = if attachments.is_empty() {
        None
    } else {
        Some(
            attachments
                .iter()
                .map(|a| [a.x, a.y, a.z])
                .collect(),
        )
    };
    venue_toml_from_anchors(
        &anchors_m,
        home,
        point_mass,
        atts.as_deref(),
        0.05,
        200.0,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spyder_core::Vec3;

    #[test]
    fn emit_and_parse_four_anchors() {
        let anchors = vec![
            Vec3::new(5.0, 3.0, 8.0),
            Vec3::new(-5.0, 3.0, 8.0),
            Vec3::new(-5.0, -3.0, 8.0),
            Vec3::new(5.0, -3.0, 8.0),
        ];
        let toml = emit_venue_toml(&anchors, &[], true, Vec3::new(0.0, 0.0, 1.5)).unwrap();
        let (robot, home) = parse_venue_toml(&toml).unwrap();
        assert_eq!(robot.anchors.len(), 4);
        assert!((home.z - 1.5).abs() < 1e-9);
    }

    #[test]
    fn parse_preset_toml() {
        let text = r#"
preset = "rect"
width = 10.0
depth = 6.0
height = 8.0
point_mass = true

[home]
x = 0.0
y = 0.0
z = 2.0
"#;
        let (robot, home) = parse_venue_toml(text).unwrap();
        assert_eq!(robot.anchors.len(), 4);
        assert!((home.z - 2.0).abs() < 1e-9);
    }

    #[test]
    fn parse_empty_config_errors() {
        assert!(parse_venue_toml("point_mass = true\n").is_err());
    }
}
