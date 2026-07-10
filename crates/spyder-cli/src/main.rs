//! Spyder CLI (minimal, no clap/toml deps for older Rust toolchains).
//!
//! Usage:
//!   spyder ik <config.toml> <x,y,z>
//!   spyder fk <config.toml> <l1,l2,...> [seed_x,y,z]
//!   spyder workspace <config.toml> [out_prefix]
//!   spyder play <config.toml> <x0,y0,z0> <x1,y1,z1> [segments]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use spyder_core::{Anchor, Pose, Preset, Robot, Vec3};

fn usage() -> ! {
    eprintln!(
        "Usage:\n  spyder ik <config.toml> <x,y,z>\n  spyder fk <config.toml> <l1,l2,...> [seed_x,y,z]\n  spyder workspace <config.toml> [out_prefix]\n  spyder play <config.toml> <x0,y0,z0> <x1,y1,z1> [segments]"
    );
    process::exit(2);
}

fn parse_xyz(s: &str) -> Vec3 {
    let p: Vec<f64> = s
        .split(',')
        .map(|t| t.trim().parse().expect("float"))
        .collect();
    assert_eq!(p.len(), 3, "expected x,y,z");
    Vec3::new(p[0], p[1], p[2])
}

fn parse_list(s: &str) -> Vec<f64> {
    s.split(',')
        .map(|t| t.trim().parse().expect("float"))
        .collect()
}

/// Extremely small TOML subset reader for our configs.
fn robot_from_toml(text: &str) -> Robot {
    let mut preset = String::from("rect");
    let mut width = None;
    let mut depth = None;
    let mut height = None;
    let mut n = None;
    let mut radius = None;
    let mut point_mass = true;
    let mut anchors: Vec<Anchor> = Vec::new();
    let mut cur_anchor: Option<(Option<f64>, Option<f64>, Option<f64>)> = None;

    let flush_anchor = |cur: &mut Option<(Option<f64>, Option<f64>, Option<f64>)>,
                        anchors: &mut Vec<Anchor>| {
        if let Some((x, y, z)) = cur.take() {
            anchors.push(Anchor::point(Vec3::new(
                x.expect("anchor.x"),
                y.expect("anchor.y"),
                z.expect("anchor.z"),
            )));
        }
    };

    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[anchors]]" {
            flush_anchor(&mut cur_anchor, &mut anchors);
            cur_anchor = Some((None, None, None));
            continue;
        }
        if line.starts_with('[') {
            flush_anchor(&mut cur_anchor, &mut anchors);
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if let Some(tuple) = cur_anchor.as_mut() {
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
    flush_anchor(&mut cur_anchor, &mut anchors);

    if !anchors.is_empty() {
        let mut r = Robot::from_anchors(anchors, None, point_mass).expect("robot");
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
    r.point_mass = point_mass;
    r
}

fn default_workspace(robot: &Robot) -> spyder_sim::WorkspaceReport {
    use spyder_sim::{sample_wrench_feasible, SampleBox};
    let box_ = SampleBox {
        min: Vec3::new(-2.0, -2.0, 0.5),
        max: Vec3::new(2.0, 2.0, 4.0),
        nx: 9,
        ny: 9,
        nz: 7,
    };
    let w = nalgebra::DVector::from_vec(vec![0.0, 0.0, -9.81]);
    sample_wrench_feasible(robot, &box_, w, 0.5, 500.0)
}

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| usage());
    match cmd.as_str() {
        "ik" => {
            let cfg_path = args.next().unwrap_or_else(|| usage());
            let xyz = args.next().unwrap_or_else(|| usage());
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let robot = robot_from_toml(&text);
            let pose = Pose::from_position(parse_xyz(&xyz));
            let ik = robot.ik(&pose).expect("ik");
            print!("lengths_m = [");
            for (i, l) in ik.lengths.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{l:.8}");
            }
            println!("]");
        }
        "fk" => {
            let cfg_path = args.next().unwrap_or_else(|| usage());
            let lengths = args.next().unwrap_or_else(|| usage());
            let seed = args.next().unwrap_or_else(|| "0,0,1".to_string());
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let robot = robot_from_toml(&text);
            let lens = parse_list(&lengths);
            let fk = robot.fk(&lens, parse_xyz(&seed)).expect("fk");
            println!(
                "position = [{:.6}, {:.6}, {:.6}] method={:?} residual={:.3e}",
                fk.position.x, fk.position.y, fk.position.z, fk.method, fk.residual
            );
        }
        "workspace" => {
            let cfg_path = args.next().unwrap_or_else(|| usage());
            let out_prefix = args
                .next()
                .unwrap_or_else(|| "artifacts/workspace".to_string());
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let robot = robot_from_toml(&text);
            let report = default_workspace(&robot);
            println!(
                "workspace feasible={}/{} fraction={:.3}",
                report.feasible, report.total, report.fraction
            );
            let prefix = PathBuf::from(&out_prefix);
            if let Some(parent) = prefix.parent() {
                let _ = fs::create_dir_all(parent);
            }
            spyder_sim::write_csv(&report, &PathBuf::from(format!("{out_prefix}.csv")))
                .expect("csv");
            spyder_sim::write_json(&report, &PathBuf::from(format!("{out_prefix}.json")))
                .expect("json");
            spyder_sim::write_html(
                &report,
                &PathBuf::from(format!("{out_prefix}.html")),
                &format!("spyder workspace ({cfg_path})"),
            )
            .expect("html");
            println!("wrote {out_prefix}.{{csv,json,html}}");
        }
        "play" => {
            use spyder_runtime::{Axis, MockBackend, Player};
            let cfg_path = args.next().unwrap_or_else(|| usage());
            let start = parse_xyz(&args.next().unwrap_or_else(|| usage()));
            let end = parse_xyz(&args.next().unwrap_or_else(|| usage()));
            let segments: usize = args
                .next()
                .unwrap_or_else(|| "10".into())
                .parse()
                .expect("segments");
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let robot = robot_from_toml(&text);
            let n = robot.anchors.len();
            let axes: Vec<_> = (0..n)
                .map(|_| Axis::new(0.05, 200.0, 1.0).expect("axis"))
                .collect();
            let mut player =
                Player::new(&robot, axes, MockBackend::new(n), start).expect("player");
            player.move_line(start, end, segments, 2.0).expect("play");
            println!(
                "play done segments={segments} final_steps={:?} moves={}",
                player.backend.steps,
                player.backend.log.len()
            );
        }
        _ => usage(),
    }
}
