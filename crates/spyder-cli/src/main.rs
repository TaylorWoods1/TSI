//! Spyder CLI
//!
//!   spyder ik|fk|workspace|scene|calibrate|play|axis-map-example ...

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use spyder_core::{Anchor, Pose, Preset, Robot, Vec3};

fn usage() -> ! {
    eprintln!(
        "Usage:
  spyder ik <config.toml> <x,y,z>
  spyder fk <config.toml> <l1,l2,...> [seed]
  spyder workspace <config.toml> [out_prefix]
  spyder scene <config.toml> <x,y,z> [out.html]
  spyder calibrate <config.toml> <x,y,z> [out.json]
  spyder play <config.toml> <x0,y0,z0> <x1,y1,z1> [segments]
           [--backend mock|stepper|odrive] [--device PATH|host:port] [--baud N]
           [--closed-loop] [--cal cal.json] [--axis-map map.json]
  spyder axis-map-example [out.json]"
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

fn take_flag(args: &mut Vec<String>, name: &str) -> Option<String> {
    if let Some(i) = args.iter().position(|a| a == name) {
        args.remove(i);
        if i < args.len() {
            return Some(args.remove(i));
        }
    }
    None
}

fn open_transport(
    device: &str,
    baud: u32,
) -> spyder_runtime::Result<Box<dyn spyder_runtime::Transport>> {
    use spyder_runtime::{SerialTransport, TcpTransport};
    if device.contains(':') && !device.starts_with('/') && !device.starts_with("COM") {
        // host:port
        Ok(Box::new(TcpTransport::connect(device)?))
    } else {
        Ok(Box::new(SerialTransport::open(device, baud)?))
    }
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let cmd = args.remove(0);
    match cmd.as_str() {
        "ik" => {
            if args.len() < 2 {
                usage();
            }
            let cfg_path = args.remove(0);
            let xyz = args.remove(0);
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
            if args.len() < 2 {
                usage();
            }
            let cfg_path = args.remove(0);
            let lengths = args.remove(0);
            let seed = if args.is_empty() {
                "0,0,1".into()
            } else {
                args.remove(0)
            };
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
            if args.is_empty() {
                usage();
            }
            let cfg_path = args.remove(0);
            let out_prefix = if args.is_empty() {
                "artifacts/workspace".into()
            } else {
                args.remove(0)
            };
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
        "scene" => {
            if args.len() < 2 {
                usage();
            }
            let cfg_path = args.remove(0);
            let xyz = parse_xyz(&args.remove(0));
            let out = if args.is_empty() {
                "artifacts/scene.html".into()
            } else {
                args.remove(0)
            };
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let robot = robot_from_toml(&text);
            spyder_sim::write_scene_at(&robot, xyz, PathBuf::from(&out).as_path(), "spyder scene")
                .expect("scene");
            println!("wrote {out}");
        }
        "calibrate" => {
            use spyder_runtime::Calibration;
            if args.len() < 2 {
                usage();
            }
            let cfg_path = args.remove(0);
            let home = parse_xyz(&args.remove(0));
            let out = if args.is_empty() {
                "artifacts/calibration.json".into()
            } else {
                args.remove(0)
            };
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let robot = robot_from_toml(&text);
            let cal = Calibration::capture(&robot, home, 0.05, 200.0).expect("cal");
            let path = PathBuf::from(&out);
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            cal.save_json(&path).expect("save");
            println!(
                "calibration saved {out} home={:?} lengths={:?}",
                cal.home, cal.home_lengths_m
            );
        }
        "axis-map-example" => {
            use spyder_runtime::AxisMap;
            let out = if args.is_empty() {
                "configs/axis_map_dual_odrive.json".into()
            } else {
                args.remove(0)
            };
            let map = AxisMap::example_dual_odrive();
            let path = PathBuf::from(&out);
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            map.save_json(&path).expect("save");
            println!("wrote example axis map {out} devices={:?}", map.devices());
        }
        "play" => {
            use spyder_runtime::{
                Axis, AxisMap, Calibration, MockBackend, MotorBackend, ODriveAxis, ODriveBackend,
                Player, SafetyLimits, StepperBackend,
            };
            let backend_name = take_flag(&mut args, "--backend").unwrap_or_else(|| "mock".into());
            let device = take_flag(&mut args, "--device");
            let baud: u32 = take_flag(&mut args, "--baud")
                .unwrap_or_else(|| "115200".into())
                .parse()
                .expect("baud");
            let cal_path = take_flag(&mut args, "--cal");
            let axis_map_path = take_flag(&mut args, "--axis-map");
            let closed_loop = args.iter().any(|a| a == "--closed-loop");
            args.retain(|a| a != "--closed-loop");
            if args.len() < 3 {
                usage();
            }
            let cfg_path = args.remove(0);
            let start = parse_xyz(&args.remove(0));
            let end = parse_xyz(&args.remove(0));
            let segments: usize = if args.is_empty() {
                10
            } else {
                args.remove(0).parse().expect("segments")
            };
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let mut robot = robot_from_toml(&text);
            if let Some(path) = &cal_path {
                let cal = Calibration::load_json(PathBuf::from(path).as_path()).expect("cal");
                if let Some(anchors) = &cal.anchors_m {
                    spyder_runtime::apply_anchor_override(&mut robot, anchors).expect("anchors");
                }
            }
            if let Some(path) = &axis_map_path {
                let map = AxisMap::load_json(PathBuf::from(path).as_path()).expect("axis-map");
                println!(
                    "axis-map cables={} devices={:?}",
                    map.cables.len(),
                    map.devices()
                );
            }
            let n = robot.anchors.len();
            let axes: Vec<_> = (0..n)
                .map(|_| Axis::new(0.05, 200.0, 1.0).expect("axis"))
                .collect();
            let safety = SafetyLimits {
                min: Vec3::new(-3.0, -3.0, 0.2),
                max: Vec3::new(3.0, 3.0, 6.0),
                max_speed_mps: 1.5,
                ..SafetyLimits::default()
            };

            match backend_name.as_str() {
                "mock" => {
                    let mut player = Player::new(&robot, axes, MockBackend::new(n), start)
                        .expect("player")
                        .with_safety(safety)
                        .with_closed_loop(closed_loop);
                    if let Some(path) = &cal_path {
                        let cal =
                            Calibration::load_json(PathBuf::from(path).as_path()).expect("cal");
                        player.apply_calibration(&cal).expect("apply cal");
                        player.home().expect("home");
                    }
                    player.move_line(start, end, segments, 2.0).expect("play");
                    let fb = player.feedback_pose().unwrap_or(end);
                    println!(
                        "play backend=mock closed_loop={closed_loop} segments={segments} final_steps={:?} feedback_pose=[{:.3},{:.3},{:.3}]",
                        player.backend.steps, fb.x, fb.y, fb.z
                    );
                }
                "stepper" => {
                    let device = device.expect("--device required for stepper");
                    let mut transport = open_transport(&device, baud).expect("transport");
                    let _ = transport.read_line();
                    let backend = StepperBackend::new(transport, n);
                    let mut player = Player::new(&robot, axes, backend, start)
                        .expect("player")
                        .with_safety(safety)
                        .with_closed_loop(closed_loop);
                    if let Some(path) = &cal_path {
                        let cal =
                            Calibration::load_json(PathBuf::from(path).as_path()).expect("cal");
                        player.apply_calibration(&cal).expect("apply cal");
                        player.home().expect("home");
                    }
                    player.move_line(start, end, segments, 2.0).expect("play");
                    println!(
                        "play backend=stepper device={device} closed_loop={closed_loop} final_steps={:?}",
                        player.backend.positions()
                    );
                }
                "odrive" => {
                    let device = device.expect("--device required for odrive");
                    let transport = open_transport(&device, baud).expect("transport");
                    let oaxes: Vec<_> = (0..n)
                        .map(|i| ODriveAxis::new((i % 2) as u8, 200.0))
                        .collect();
                    let mut backend = ODriveBackend::new(transport, oaxes);
                    backend.enter_closed_loop().expect("closed loop");
                    let mut player = Player::new(&robot, axes, backend, start)
                        .expect("player")
                        .with_safety(safety)
                        .with_closed_loop(closed_loop);
                    if let Some(path) = &cal_path {
                        let cal =
                            Calibration::load_json(PathBuf::from(path).as_path()).expect("cal");
                        player.apply_calibration(&cal).expect("apply cal");
                        player.home().expect("home");
                    }
                    player.move_line(start, end, segments, 2.0).expect("play");
                    println!(
                        "play backend=odrive device={device} closed_loop={closed_loop} final_steps={:?}",
                        player.backend.positions()
                    );
                }
                other => {
                    eprintln!("unknown backend {other}");
                    usage();
                }
            }
        }
        _ => usage(),
    }
}
