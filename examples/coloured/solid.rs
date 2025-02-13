use kdam::{tqdm, BarExt};

fn main() {
    let mut pb = tqdm!(total = 100, colour = "#a485ca");

    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_secs_f32(0.01));
        pb.update(1);
    }

    pb.set_colour("#da70d6");
    pb.refresh();

    eprintln!();
}
