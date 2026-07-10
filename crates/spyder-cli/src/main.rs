//! Spyder CLI
//!
//!   spyder ik|fk|workspace|scene|calibrate|play|axis-map-example ...

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use spyder_core::{Pose, Robot, Vec3};

mod toml;
use toml::robot_from_toml;

fn usage() -> ! {
    eprintln!(
        "Usage:
  spyder ik <config.toml> <x,y,z>
  spyder fk <config.toml> <l1,l2,...> [seed]
  spyder workspace <config.toml> [out_prefix]
  spyder scene <config.toml> <x,y,z> [out.html]
           [--to x,y,z] [--segments N] [--workspace]
  spyder calibrate <config.toml> <x,y,z> [out.json]
  spyder field-cal <x,y,z;x,y,z;...> <home_x,y,z> [out.toml]
           [--drum R] [--steps N] [--platform]
  spyder venue-from-cal <cal.json> [out.toml]
  spyder play <config.toml> <x0,y0,z0> <x1,y1,z1> [segments]
           [--backend mock|stepper|odrive|multiboard]
           [--device PATH|host:port] [--baud N]
           [--closed-loop] [--realtime] [--cal cal.json] [--axis-map map.json]
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

fn parse_anchor_list(s: &str) -> Vec<[f64; 3]> {
    s.split(';')
        .map(|part| {
            let p = parse_xyz(part.trim());
            [p.x, p.y, p.z]
        })
        .collect()
}

fn parse_list(s: &str) -> Vec<f64> {
    s.split(',')
        .map(|t| t.trim().parse().expect("float"))
        .collect()
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
            let rv = fk.orientation.scaled_axis();
            println!(
                "position = [{:.6}, {:.6}, {:.6}] orientation_rv = [{:.6}, {:.6}, {:.6}] method={:?} residual={:.3e}",
                fk.position.x, fk.position.y, fk.position.z, rv.x, rv.y, rv.z, fk.method, fk.residual
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
            let to = take_flag(&mut args, "--to");
            let segments: usize = take_flag(&mut args, "--segments")
                .unwrap_or_else(|| "12".into())
                .parse()
                .expect("segments");
            let with_ws = args.iter().any(|a| a == "--workspace");
            args.retain(|a| a != "--workspace");
            if args.len() < 2 {
                usage();
            }
            let cfg_path = args.remove(0);
            let xyz = parse_xyz(&args.remove(0));
            let out = if args.is_empty() {
                if to.is_some() {
                    "artifacts/scene_anim.html".into()
                } else {
                    "artifacts/scene.html".into()
                }
            } else {
                args.remove(0)
            };
            let text = fs::read_to_string(&cfg_path).expect("read config");
            let robot = robot_from_toml(&text);
            let ws = if with_ws {
                Some(default_workspace(&robot))
            } else {
                None
            };
            if let Some(end) = to {
                let end = parse_xyz(&end);
                spyder_sim::write_scene_line(
                    &robot,
                    xyz,
                    end,
                    segments,
                    PathBuf::from(&out).as_path(),
                    "spyder scene animation",
                    ws.as_ref(),
                )
                .expect("scene anim");
            } else {
                spyder_sim::write_scene_at(&robot, xyz, PathBuf::from(&out).as_path(), "spyder scene")
                    .expect("scene");
            }
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
        "field-cal" => {
            use spyder_runtime::venue_toml_from_anchors;
            let drum: f64 = take_flag(&mut args, "--drum")
                .unwrap_or_else(|| "0.05".into())
                .parse()
                .expect("drum");
            let steps: f64 = take_flag(&mut args, "--steps")
                .unwrap_or_else(|| "200".into())
                .parse()
                .expect("steps");
            let platform = args.iter().any(|a| a == "--platform");
            args.retain(|a| a != "--platform");
            if args.len() < 2 {
                usage();
            }
            let anchors = parse_anchor_list(&args.remove(0));
            let home = parse_xyz(&args.remove(0));
            let out = if args.is_empty() {
                "artifacts/venue.toml".into()
            } else {
                args.remove(0)
            };
            let text = venue_toml_from_anchors(
                &anchors,
                home,
                !platform,
                None,
                drum,
                steps,
            )
            .expect("venue");
            let path = PathBuf::from(&out);
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(&path, &text).expect("write venue");
            // Smoke: load back and run IK at home
            let robot = robot_from_toml(&text);
            let ik = robot
                .ik(&Pose::from_position(home))
                .expect("ik at home");
            println!(
                "wrote {out} anchors={} point_mass={} home_lengths={:?}",
                anchors.len(),
                !platform,
                ik.lengths
            );
        }
        "venue-from-cal" => {
            use spyder_runtime::Calibration;
            if args.is_empty() {
                usage();
            }
            let cal_path = args.remove(0);
            let out = if args.is_empty() {
                "artifacts/venue.toml".into()
            } else {
                args.remove(0)
            };
            let cal = Calibration::load_json(PathBuf::from(&cal_path).as_path()).expect("cal");
            cal.save_venue_toml(PathBuf::from(&out).as_path(), true)
                .expect("venue");
            println!("wrote {out} from {cal_path}");
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
                Axis, AxisMap, Calibration, MockBackend, MotorBackend, MultiBoardBackend,
                ODriveAxis, ODriveBackend, Player, SafetyLimits, StepperBackend,
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
            let realtime = args.iter().any(|a| a == "--realtime");
            args.retain(|a| a != "--closed-loop" && a != "--realtime");
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

            fn finish_play<'a, B: MotorBackend>(
                mut player: Player<'a, B>,
                cal_path: &Option<String>,
                start: Vec3,
                end: Vec3,
                segments: usize,
            ) -> Player<'a, B> {
                if let Some(path) = cal_path {
                    let cal = Calibration::load_json(PathBuf::from(path).as_path()).expect("cal");
                    player.apply_calibration(&cal).expect("apply cal");
                    player.home().expect("home");
                }
                player.move_line(start, end, segments, 2.0).expect("play");
                player
            }

            match backend_name.as_str() {
                "mock" => {
                    if let Some(path) = &axis_map_path {
                        let map = AxisMap::load_json(PathBuf::from(path).as_path()).expect("axis-map");
                        println!(
                            "axis-map cables={} devices={:?} (mock multi-board)",
                            map.cables.len(),
                            map.devices()
                        );
                        let backend = MultiBoardBackend::mock_from_map(map).expect("multiboard");
                        let player = Player::new(&robot, axes, backend, start)
                            .expect("player")
                            .with_safety(safety)
                            .with_closed_loop(closed_loop)
                            .with_realtime(realtime);
                        let player = finish_play(player, &cal_path, start, end, segments);
                        println!(
                            "play backend=multiboard-mock closed_loop={closed_loop} realtime={realtime} boards={} final_steps={:?}",
                            player.backend.board_count(),
                            player.backend.positions()
                        );
                    } else {
                        let player = Player::new(&robot, axes, MockBackend::new(n), start)
                            .expect("player")
                            .with_safety(safety)
                            .with_closed_loop(closed_loop)
                            .with_realtime(realtime);
                        let mut player = finish_play(player, &cal_path, start, end, segments);
                        let fb = player.feedback_pose().unwrap_or(end);
                        println!(
                            "play backend=mock closed_loop={closed_loop} realtime={realtime} segments={segments} final_steps={:?} feedback_pose=[{:.3},{:.3},{:.3}]",
                            player.backend.steps, fb.x, fb.y, fb.z
                        );
                    }
                }
                "multiboard" => {
                    let path = axis_map_path.expect("--axis-map required for multiboard");
                    let map = AxisMap::load_json(PathBuf::from(path).as_path()).expect("axis-map");
                    let devices = map.devices();
                    println!(
                        "axis-map cables={} devices={:?}",
                        map.cables.len(),
                        devices
                    );
                    let mut boards: Vec<Box<dyn MotorBackend>> = Vec::new();
                    for device in &devices {
                        let baud = map
                            .cables
                            .iter()
                            .find(|c| &c.device == device)
                            .map(|c| c.baud)
                            .unwrap_or(baud);
                        let n_local = map.cables.iter().filter(|c| &c.device == device).count();
                        let mut transport = open_transport(device, baud).expect("transport");
                        let _ = transport.read_line();
                        boards.push(Box::new(StepperBackend::new(transport, n_local)));
                    }
                    let backend = MultiBoardBackend::new(map, boards).expect("multiboard");
                    let player = Player::new(&robot, axes, backend, start)
                        .expect("player")
                        .with_safety(safety)
                        .with_closed_loop(closed_loop)
                        .with_realtime(realtime);
                    let player = finish_play(player, &cal_path, start, end, segments);
                    println!(
                        "play backend=multiboard closed_loop={closed_loop} realtime={realtime} boards={} final_steps={:?}",
                        player.backend.board_count(),
                        player.backend.positions()
                    );
                }
                "stepper" => {
                    let device = device.expect("--device required for stepper");
                    let mut transport = open_transport(&device, baud).expect("transport");
                    let _ = transport.read_line();
                    let backend = StepperBackend::new(transport, n);
                    let player = Player::new(&robot, axes, backend, start)
                        .expect("player")
                        .with_safety(safety)
                        .with_closed_loop(closed_loop)
                        .with_realtime(realtime);
                    let player = finish_play(player, &cal_path, start, end, segments);
                    println!(
                        "play backend=stepper device={device} closed_loop={closed_loop} realtime={realtime} final_steps={:?}",
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
                    let player = Player::new(&robot, axes, backend, start)
                        .expect("player")
                        .with_safety(safety)
                        .with_closed_loop(closed_loop)
                        .with_realtime(realtime);
                    let player = finish_play(player, &cal_path, start, end, segments);
                    println!(
                        "play backend=odrive device={device} closed_loop={closed_loop} realtime={realtime} final_steps={:?}",
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
