//! Venue TOML parse and emit helpers.

use spyder_core::{Anchor, PlatformAttachment, Robot, Vec3};

use crate::state::CableModelParams;

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
    let mut cur_pulley_radius: Option<f64> = None;
    let mut cur_pulley_axis: Option<(Option<f64>, Option<f64>, Option<f64>)> = None;
    let mut cur_winch: Option<(Option<f64>, Option<f64>, Option<f64>)> = None;
    let mut cur_runout: Option<f64> = None;
    let mut home = Vec3::new(0.0, 0.0, 2.0);
    let mut in_home = false;
    let mut cable_model = String::from("ideal");
    let mut pulley_radius = 0.05f64;
    let mut sag_mu = 1.0f64;
    let mut sag_ea = 1.0e6f64;

    let flush = |cur: &mut Option<(Option<f64>, Option<f64>, Option<f64>)>,
                 kind: &str,
                 pulley_radius: &mut Option<f64>,
                 pulley_axis: &mut Option<(Option<f64>, Option<f64>, Option<f64>)>,
                 winch: &mut Option<(Option<f64>, Option<f64>, Option<f64>)>,
                 runout: &mut Option<f64>,
                 anchors: &mut Vec<Anchor>,
                 attachments: &mut Vec<PlatformAttachment>| {
        if let Some((x, y, z)) = cur.take() {
            let v = Vec3::new(
                x.ok_or("missing x")?,
                y.ok_or("missing y")?,
                z.ok_or("missing z")?,
            );
            if kind == "anchor" {
                let mut a = Anchor::point(v);
                if let Some(r) = pulley_radius.take() {
                    a.pulley_radius = r;
                    a.pulley_axis = Some(Vec3::z());
                }
                if let Some((ax, ay, az)) = pulley_axis.take() {
                    a.pulley_axis = Some(Vec3::new(
                        ax.ok_or("missing pulley_axis x")?,
                        ay.ok_or("missing pulley_axis y")?,
                        az.ok_or("missing pulley_axis z")?,
                    ));
                }
                if let Some((wx, wy, wz)) = winch.take() {
                    a.pulley_winch_exit = Some(Vec3::new(
                        wx.ok_or("missing winch x")?,
                        wy.ok_or("missing winch y")?,
                        wz.ok_or("missing winch z")?,
                    ));
                }
                if let Some(ro) = runout.take() {
                    a.pulley_runout_m = ro;
                }
                anchors.push(a);
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
            flush(
                &mut cur_xyz,
                cur_kind,
                &mut cur_pulley_radius,
                &mut cur_pulley_axis,
                &mut cur_winch,
                &mut cur_runout,
                &mut anchors,
                &mut attachments,
            )?;
            cur_kind = "anchor";
            in_home = false;
            cur_xyz = Some((None, None, None));
            cur_pulley_radius = None;
            cur_pulley_axis = None;
            cur_winch = None;
            cur_runout = None;
            continue;
        }
        if line == "[[attachments]]" {
            flush(
                &mut cur_xyz,
                cur_kind,
                &mut cur_pulley_radius,
                &mut cur_pulley_axis,
                &mut cur_winch,
                &mut cur_runout,
                &mut anchors,
                &mut attachments,
            )?;
            cur_kind = "attachment";
            in_home = false;
            cur_xyz = Some((None, None, None));
            continue;
        }
        if line == "[home]" {
            flush(
                &mut cur_xyz,
                cur_kind,
                &mut cur_pulley_radius,
                &mut cur_pulley_axis,
                &mut cur_winch,
                &mut cur_runout,
                &mut anchors,
                &mut attachments,
            )?;
            cur_kind = "";
            in_home = true;
            continue;
        }
        if line.starts_with('[') {
            flush(
                &mut cur_xyz,
                cur_kind,
                &mut cur_pulley_radius,
                &mut cur_pulley_axis,
                &mut cur_winch,
                &mut cur_runout,
                &mut anchors,
                &mut attachments,
            )?;
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
                    "pulley_radius" if cur_kind == "anchor" => {
                        cur_pulley_radius = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?)
                    }
                    "pulley_runout_m" if cur_kind == "anchor" => {
                        cur_runout = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?)
                    }
                    _ => {}
                }
                continue;
            }
            if cur_kind == "anchor" {
                if k == "pulley_axis" {
                    // skip — handled via [pulley_axis] section not supported; use axis_x keys below
                }
            }
            if cur_kind == "anchor" {
                if cur_pulley_axis.is_none() {
                    cur_pulley_axis = Some((None, None, None));
                }
                if let Some(axis) = &mut cur_pulley_axis {
                    match k {
                        "axis_x" | "pulley_axis_x" => axis.0 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                        "axis_y" | "pulley_axis_y" => axis.1 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                        "axis_z" | "pulley_axis_z" => axis.2 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                        "winch_x" => {
                            let w = cur_winch.get_or_insert((None, None, None));
                            w.0 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?);
                        }
                        "winch_y" => {
                            let w = cur_winch.get_or_insert((None, None, None));
                            w.1 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?);
                        }
                        "winch_z" => {
                            let w = cur_winch.get_or_insert((None, None, None));
                            w.2 = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?);
                        }
                        _ => {}
                    }
                }
            }
            if in_home {
                match k {
                    "x" => home.x = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                    "y" => home.y = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                    "z" => home.z = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                    _ => in_home = false,
                }
                if in_home {
                    continue;
                }
            }
            match k {
                "preset" => preset = v.to_string(),
                "width" => width = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "depth" => depth = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "height" => height = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "n" => n = Some(v.parse::<usize>().map_err(|e: std::num::ParseIntError| e.to_string())?),
                "radius" => radius = Some(v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?),
                "point_mass" => point_mass = v.parse::<bool>().map_err(|e: std::str::ParseBoolError| e.to_string())?,
                "cable_model" => cable_model = v.to_string(),
                "pulley_radius" => pulley_radius = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                "sag_mu" => sag_mu = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                "sag_ea" => sag_ea = v.parse::<f64>().map_err(|e: std::num::ParseFloatError| e.to_string())?,
                _ => {}
            }
        }
    }
    flush(
        &mut cur_xyz,
        cur_kind,
        &mut cur_pulley_radius,
        &mut cur_pulley_axis,
        &mut cur_winch,
        &mut cur_runout,
        &mut anchors,
        &mut attachments,
    )?;

    if !anchors.is_empty() {
        let atts = if attachments.is_empty() {
            None
        } else {
            Some(attachments)
        };
        let mut r = Robot::from_anchors(anchors, atts, point_mass).map_err(|e| e.to_string())?;
        r.point_mass = point_mass;
        crate::state::apply_cable_model(
            &mut r,
            &CableModelParams {
                model: cable_model,
                pulley_radius,
                sag_mu,
                sag_ea,
            },
        )?;
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
    crate::state::apply_cable_model(
        &mut r,
        &CableModelParams {
            model: cable_model,
            pulley_radius,
            sag_mu,
            sag_ea,
        },
    )?;
    Ok((r, home))
}

/// Emit venue TOML from anchors, attachments, home, and cable model.
pub fn emit_venue_toml(
    anchors: &[Anchor],
    attachments: &[Vec3],
    point_mass: bool,
    home: Vec3,
    model: &CableModelParams,
) -> Result<String, String> {
    let mut toml = String::new();
    toml.push_str(&format!("point_mass = {}\n\n", point_mass));

    for (i, a) in anchors.iter().enumerate() {
        toml.push_str("[[anchors]]\n");
        toml.push_str(&format!("# cable {i}\n"));
        toml.push_str(&format!("x = {:.6}\n", a.exit.x));
        toml.push_str(&format!("y = {:.6}\n", a.exit.y));
        toml.push_str(&format!("z = {:.6}\n", a.exit.z));
        if a.pulley_radius > 0.0 {
            toml.push_str(&format!("pulley_radius = {:.6}\n", a.pulley_radius));
        }
        if let Some(axis) = a.pulley_axis {
            toml.push_str(&format!(
                "pulley_axis_x = {:.6}\npulley_axis_y = {:.6}\npulley_axis_z = {:.6}\n",
                axis.x, axis.y, axis.z
            ));
        }
        if let Some(w) = a.pulley_winch_exit {
            toml.push_str(&format!(
                "winch_x = {:.6}\nwinch_y = {:.6}\nwinch_z = {:.6}\n",
                w.x, w.y, w.z
            ));
        }
        if a.pulley_runout_m > 0.0 {
            toml.push_str(&format!("pulley_runout_m = {:.6}\n", a.pulley_runout_m));
        }
        toml.push('\n');
    }

    for (i, b) in attachments.iter().enumerate() {
        toml.push_str("[[attachments]]\n");
        toml.push_str(&format!("# body point {i}\n"));
        toml.push_str(&format!("x = {:.6}\n", b.x));
        toml.push_str(&format!("y = {:.6}\n", b.y));
        toml.push_str(&format!("z = {:.6}\n\n", b.z));
    }

    toml.push_str("[home]\n");
    toml.push_str(&format!("x = {:.6}\n", home.x));
    toml.push_str(&format!("y = {:.6}\n", home.y));
    toml.push_str(&format!("z = {:.6}\n\n", home.z));

    toml.push_str(&format!("cable_model = \"{}\"\n", model.model));
    if model.model == "pulley" {
        toml.push_str(&format!("pulley_radius = {:.6}\n", model.pulley_radius));
    }
    if model.model == "sag" {
        toml.push_str(&format!("sag_mu = {:.6}\n", model.sag_mu));
        toml.push_str(&format!("sag_ea = {:.1}\n", model.sag_ea));
    }
    Ok(toml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CableModelParams;
    use spyder_core::Vec3;

    #[test]
    fn emit_and_parse_four_anchors() {
        let exits = vec![
            Vec3::new(5.0, 3.0, 8.0),
            Vec3::new(-5.0, 3.0, 8.0),
            Vec3::new(-5.0, -3.0, 8.0),
            Vec3::new(5.0, -3.0, 8.0),
        ];
        let anchors: Vec<Anchor> = exits.iter().map(|v| Anchor::point(*v)).collect();
        let params = CableModelParams {
            model: "pulley".into(),
            pulley_radius: 0.06,
            ..Default::default()
        };
        let toml = emit_venue_toml(&anchors, &[], true, Vec3::new(0.0, 0.0, 1.5), &params).unwrap();
        let (robot, home) = parse_venue_toml(&toml).unwrap();
        assert_eq!(robot.anchors.len(), 4);
        assert!((home.z - 1.5).abs() < 1e-9);
        assert_eq!(crate::state::cable_model_str(&robot.cable_model), "pulley");
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
