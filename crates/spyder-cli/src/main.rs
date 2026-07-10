//! Spyder CLI
//!
//! Usage:
//!   spyder ik <config.toml> <x,y,z>
//!   spyder fk <config.toml> <l1,l2,...> [seed_x,y,z]
//!   spyder workspace <config.toml> [out_prefix]
//!   spyder play <config.toml> <x0,y0,z0> <x1,y1,z1> [segments] [--backend mock|stepper|odrive] [--device PATH|host:port] [--baud N]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use spyder_core::{Anchor, Pose, Preset, Robot, Vec3};

fn usage() -> ! {
    eprintln!(
        "Usage:\n  spyder ik <config.toml> <x,y,z>\n  spyder fk <config.toml> <l1,l2,...> [seed_x,y,z]\n  spyder workspace <config.toml> [out_prefix]\n  spyder play <config.toml> <x0,y0,z0> <x1,y1,z1> [segments] [--backend mock|stepper|odrive] [--device PATH|host:port] [--baud N]"
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
        "play" => {
            use spyder_runtime::{
                Axis, MockBackend, MotorBackend, ODriveAxis, ODriveBackend, Player,
                StepperBackend,
            };
            let backend_name = take_flag(&mut args, "--backend").unwrap_or_else(|| "mock".into());
            let device = take_flag(&mut args, "--device");
            let baud: u32 = take_flag(&mut args, "--baud")
                .unwrap_or_else(|| "115200".into())
                .parse()
                .expect("baud");
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
            let robot = robot_from_toml(&text);
            let n = robot.anchors.len();
            let axes: Vec<_> = (0..n)
                .map(|_| Axis::new(0.05, 200.0, 1.0).expect("axis"))
                .collect();

            match backend_name.as_str() {
                "mock" => {
                    let mut player =
                        Player::new(&robot, axes, MockBackend::new(n), start).expect("player");
                    player.move_line(start, end, segments, 2.0).expect("play");
                    println!(
                        "play backend=mock segments={segments} final_steps={:?} moves={}",
                        player.backend.steps,
                        player.backend.log.len()
                    );
                }
                "stepper" => {
                    let device = device.expect("--device required for stepper (e.g. /dev/ttyUSB0 or 127.0.0.1:9002)");
                    // Discard banner if any
                    let mut transport = open_transport(&device, baud).expect("transport");
                    // Read optional greeting
                    let _ = transport.read_line();
                    let backend = StepperBackend::new(transport, n);
                    let mut player = Player::new(&robot, axes, backend, start).expect("player");
                    player.move_line(start, end, segments, 2.0).expect("play");
                    println!(
                        "play backend=stepper device={device} segments={segments} final_steps={:?}",
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
                    let mut player = Player::new(&robot, axes, backend, start).expect("player");
                    player.move_line(start, end, segments, 2.0).expect("play");
                    println!(
                        "play backend=odrive device={device} segments={segments} final_steps={:?}",
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
