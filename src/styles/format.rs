//! Functions for formatting values.

/// Returns floor division and modulus of two values.
pub fn divmod(x: usize, y: usize) -> (usize, usize) {
    (x / y as usize, x % y)
}

/// Formats a number (greater than unity) with SI order of magnitude prefixes.
pub fn format_sizeof(num: f64, divisor: f64) -> String {
    let mut value = num;

    for i in ["", "k", "M", "G", "T", "P", "E", "Z"] {
        if value.abs() < 999.5 {
            if value.abs() < 99.95 {
                if value.abs() < 9.995 {
                    return format!("{:1.2}{}", value, i);
                }
                return format!("{:2.1}{}", value, i);
            }
            return format!("{:3.0}{}", value, i);
        }
        value /= divisor;
    }
    format!("{:3.1}Y", value)
}

pub fn format_time(num: f64) -> String {
    let mut value = num;
    let units = [(60., "s"), (60., "min"), (24., "hr")];
    for (d, i) in units {
        if value.abs() < d - 0.005 {
            return format!("{value:1.2}{i}");
        }
        value /= d;
    }
    format!("{value:1.2}days")
}

/// Formats a number of seconds as a clock time, \[H:\]MM:SS and SSs.
pub fn format_interval(seconds: usize, human: bool) -> String {
    if human && seconds < 60 {
        return seconds.to_string() + "s";
    }

    let (minutes, seconds) = divmod(seconds, 60);
    let (hours, minutes) = divmod(minutes, 60);

    if hours == 0 {
        format!("{:02}:{:02}", minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

// Intelligent scientific notation (.3g).
// pub fn format_num(n: usize) -> String {
//     let f = format!("{:.3g}", n)
//         .replace("+0", "+")
//         .replace("-0", "-");
//     let n = format!("{}", n).to_string();
//     return if f.len() < n.len() { f } else { n };
// }

