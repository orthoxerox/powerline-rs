use {dirs, Powerline, Segment};
use std::{
    borrow::Cow,
    env,
    ffi::OsStr,
    path::PathBuf
};
use unicode_segmentation::UnicodeSegmentation;

pub fn segment_cwd(p: &mut Powerline, cwd_max_depth: u8, cwd_max_dir_size: u8) {
    let (path, in_home) = get_cwd();

    if in_home {
        p.segments.push(Segment::new(p.theme.home_bg, p.theme.home_fg, "~"));
    }

    let length = path.iter().count();
    let mut dirs = path.iter();

    let cwd_max_depth = cwd_max_depth as usize;

    if cwd_max_depth != 1 {
        if let Some(dir) = dirs.next() {
            // Either there's no cwd_max_depth, or it's bigger than 1
            segment(p, dir, length == 1, cwd_max_dir_size);

            // It would be sane here to subtract 1 from both length and
            // cwd_max_depth, to make it clear that we already tried one and
            // what the below code is doing. HOWEVER, currently that results in
            // the exact same outcome.
        }
    }
    if cwd_max_depth > 0 && length > cwd_max_depth {
        p.segments.push(Segment::new(p.theme.path_bg, p.theme.path_fg, Cow::from("…")));

        for _ in 0..length - cwd_max_depth {
            dirs.next().unwrap();
        }
    }

    let mut next = dirs.next();
    while let Some(cursor) = next {
        next = dirs.next();

        segment(p, cursor, next.is_none(), cwd_max_dir_size);
    }
}

fn get_cwd() -> (PathBuf, bool) {
    let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("error"));
    let mut in_home = false;
    if let Some(home) = dirs::home_dir() {
        let mut new_path = None;
        if let Ok(new) = path.strip_prefix(&home) {
            in_home = true;
            // TODO: NLL: path = new.to_path_buf();
            new_path = Some(new.to_path_buf());
        }
        if let Some(new) = new_path {
            path = new;
        }
    }

    (path, in_home)
}

fn segment(p: &mut Powerline, name: &OsStr, last: bool, cwd_max_dir_size: u8) {
    let name = trim(name, cwd_max_dir_size);

    let fg = if last { p.theme.cwd_fg } else { p.theme.path_fg };
    p.segments.push(Segment::new(p.theme.path_bg, fg, name));
}

fn trim(name: &OsStr, cwd_max_dir_size: u8) -> String {
    let mut name = name.to_string_lossy().into_owned();
    let cwd_max_dir_size = cwd_max_dir_size as usize;

    if cwd_max_dir_size > 0 {
        if let Some((start, _)) = name.grapheme_indices(true).nth(cwd_max_dir_size) {
            if start < name.len() {
                name.drain(start..);
                name.push('…');
            }
        }
    }

    name
}

pub fn segment_brief(p: &mut Powerline, cwd_max_dir_size: u8) {
    let (path, in_home) = get_cwd();
    let mut s = String::new();

    if in_home {
        s.push_str("~");
    }

    let mut dirs = path.iter();

    let mut next = dirs.next();
    while let Some(dir) = next {
        next = dirs.next();
        let last = next.is_none();

        if dir != "/" {
            s.push('/');

            if last {
                s.push_str(&dir.to_string_lossy());
                s.push('/');
            } else {
                s.push_str(&trim(dir, cwd_max_dir_size));
            }
        } else {
            if last {
                s.push('/');
            }
        }
    }

    p.segments.push(Segment::new(p.theme.path_bg, p.theme.cwd_fg, s));
}
