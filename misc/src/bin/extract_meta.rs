use std::path::Path;

use media_info::VideoInfo;

fn main() {
    let pathstr = "./test/video.MP4";

    let path = Path::new(pathstr);
    let video_info = VideoInfo::new(path).unwrap();

    let creation_time = video_info.creation_date;

    let parsable = to_naive_parseable(&creation_time).unwrap();

    println!("{}", parsable);
    println!("{}", creation_time);
}

fn to_naive_parseable(datetime: &str) -> Option<String> {
    // Expected input: 2025-07-30T12:20:10.000000Z

    let without_z = datetime.strip_suffix('Z')?;

    let (date, time) = without_z.split_once('T')?;

    let time_no_frac = time.split('.').next()?;

    Some(format!("{date} {time_no_frac}"))
}
