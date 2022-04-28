use std::io::Write;
use std::iter::Cycle;

use crate::format;
use crate::styles::{Animation, Output};
use crate::term;

#[derive(Debug)]
pub struct BarInternal {
    pub started: bool,
    pub elapsed_time: f64,
    pub its_per: f64,
    pub bar_length: i16,
    pub user_ncols: i16,
    pub charset: String,
    pub charset_len: u64,
    pub timer: std::time::Instant,
    pub force_refresh: bool,
    pub spinner: Cycle<std::slice::Iter<'static, &'static str>>,
}

impl Default for BarInternal {
    fn default() -> BarInternal {
        BarInternal {
            started: false,
            elapsed_time: 0.0,
            its_per: 0.0,
            bar_length: 0,
            user_ncols: -1,
            charset: crate::styles::TQDMCHARSET.join(""),
            charset_len: 8,
            timer: std::time::Instant::now(),
            force_refresh: false,
            spinner: crate::styles::CLASSICSPINNER.iter().cycle(),
        }
    }
}

/// Standard struct implemention of progress bar.
///
/// # Examples
///
/// A clean nice progress bar with a total value.
///
/// ```rust
/// use kdam::Bar;
///
/// fn main() {
///     let mut pb = Bar {
///         total: 100,
///         ..Default::default()
///     };
///
///     for _ in 0..100 {
///         pb.update(1);
///     }
/// }
/// ```
///
/// Another example without a total value. This only shows basic stats.
///
/// ```rust
/// use kdam::Bar;
///
/// fn main() {
///     let mut pb = Bar::default();
///
///     for _ in 0..100 {
///         pb.update(1);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Bar {
    /// Prefix for the progress bar.
    /// (default: `""`)
    pub desc: String,
    /// The number of expected iterations.
    /// If unspecified, iterable.size_hint().0 is used if possible.
    /// If 0, only basic progress statistics are displayed (no ETA, no progressbar).
    /// (default: `0`)
    pub total: u64,
    /// If true, keeps all traces of the progressbar upon termination of iteration.
    /// If false, will leave only if position is 0.
    /// (default: `true`)
    pub leave: bool,
    /// Specifies where to output the progress messages (default: stderr).
    /// Uses file.write_fmt and file.flush methods.
    /// (default: `None`)
    pub file: Option<std::fs::File>,
    /// The width of the entire output message.
    /// If specified, dynamically resizes the progressbar to stay within this bound.
    /// If unspecified, attempts to use environment width.
    /// The fallback is a meter width of 10 and no limit for the counter and statistics.
    /// If 0, will not print any meter (only stats).
    /// (default: `10`)
    pub ncols: i16,
    /// Minimum progress display update interval (in seconds).
    /// (default: `0.1`)
    pub mininterval: f64,
    /// Minimum progress display update interval, in iterations.
    /// If > 0, will skip display of specified number of iterations. Tweak this and mininterval to get very efficient loops.
    /// If your progress is erratic with both fast and slow iterations (network, skipping items, etc) you should set miniters=1.
    /// (default: `1`)
    pub miniters: u64,
    /// Automatically adjusts miniters to correspond to mininterval after long display update lag.
    /// (default: `false`)
    pub dynamic_miniters: bool,
    /// If false, use unicode (smooth blocks) to fill the meter.
    /// If true, use ASCII characters "123456789#" to fill the meter.
    /// You can change ASCII charset using set_charset method.
    /// (default: `false`)
    pub ascii: bool,
    /// Whether to disable the entire progress bar wrapper.
    /// (default: `false`)
    pub disable: bool,
    /// String that will be used to define the unit of each iteration.
    /// (default: `"it"`)
    pub unit: String,
    /// If true, the number of iterations will be reduced/scaled automatically
    /// and a metric prefix following the International System of Units standard will be added (kilo, mega, etc.).
    /// (default: `false`)
    pub unit_scale: bool,
    /// If true, constantly alters ncols to the environment (allowing for window resizes).
    /// (default: `false`)
    pub dynamic_ncols: bool,
    /// The initial counter value. Useful when restarting a progress bar.
    /// (default: `0`)
    pub initial: u64,
    /// Specify the line offset to print this bar (starting from 0).
    /// Useful to manage multiple bars at once (eg, from threads).
    /// (default: `0`)
    pub position: u16,
    /// Specify additional stats to display at the end of the bar.
    /// (default: `""`)
    pub postfix: String,
    /// ignored unless unit_scale is true.
    /// (default: `1024`)
    pub unit_divisor: u64,
    /// Bar colour (e.g. "green", "#00ff00").
    pub colour: String,
    /// Don't display until few seconds have elapsed.
    /// (default: `0`)
    pub delay: f64,
    /// Fill incompleted progress bar with a character.
    /// (default: `" "`)
    pub fill: String,
    /// Defines the animation style to use with progress bar.
    /// For custom type use set_charset method.
    /// (default: `kdam::Animation::TqdmAscii`)
    pub animation: Animation,
    /// Select where to display progress bar output between stdout and stderr.
    /// (default: `kdam::Output::Stderr`)
    pub output: Output,
    /// If true, each update method call will be rendered.
    /// (default: `false`)
    pub max_fps: bool,
    /// Counter of progress bar.
    /// (default: `0`)
    pub n: u64,
    /// Variables for internal use.
    pub internal: BarInternal,
}

impl Default for Bar {
    fn default() -> Bar {
        Bar {
            desc: "".to_string(),
            total: 0,
            leave: true,
            file: None,
            ncols: 10,
            mininterval: 0.1,
            miniters: 1,
            dynamic_miniters: false,
            ascii: false,
            disable: false,
            unit: "it".to_string(),
            unit_scale: false,
            dynamic_ncols: false,
            initial: 0,
            position: 0,
            postfix: "".to_string(),
            unit_divisor: 1000,
            colour: "default".to_string(),
            delay: 0.0,
            fill: " ".to_string(),
            animation: Animation::TqdmAscii,
            output: Output::Stderr,
            max_fps: false,
            n: 0,
            internal: BarInternal::default(),
        }
    }
}

impl Bar {
    /// Create a new instance of `kdam::Bar` with a total value.
    /// You can also set `total=0` if total is unknown.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut pb = kdam::Bar::new(100);
    /// ```
    pub fn new(total: u64) -> Bar {
        Bar {
            total: total,
            ..Default::default()
        }
    }

    /// Initialize struct values.
    fn init(&mut self) {
        self.n = self.initial;

        if self.ncols != 10 {
            self.internal.user_ncols = self.ncols;
        }

        self.set_colour(&self.colour.clone());

        if self.ascii {
            self.set_charset(&crate::styles::TQDMASCIICHARSET);
        } else if matches!(self.animation, Animation::Tqdm) {
            self.set_charset(&crate::styles::TQDMCHARSET);
        } else if matches!(self.animation, Animation::FillUp) {
            self.set_charset(&crate::styles::FILLUPCHARSET);
        } else if matches!(self.animation, Animation::Classic) {
            self.internal.charset = "#".to_string();
            self.fill = ".".to_string();
        } else if matches!(self.animation, Animation::Arrow) {
            self.internal.charset = "=".to_string();
        } else if matches!(self.animation, Animation::FiraCode) {
            self.internal.charset = "\u{EE04}".to_string();
            self.fill = "\u{EE01}".to_string();
            self.internal.spinner = crate::styles::FIRACODESPINNER.iter().cycle();
        }

        if self.max_fps {
            self.internal.force_refresh = true;
        }
    }

    fn render_unknown(&mut self, i: u64) -> String {
        let desc_spacing = if self.desc == "" { "" } else { ": " };
        self.internal.elapsed_time = self.internal.timer.elapsed().as_secs_f64();
        self.internal.its_per = i as f64 / self.internal.elapsed_time;
        let elapsed_time_fmt = format::format_interval(self.internal.elapsed_time as u64);

        let count = if self.unit_scale {
            format::format_sizeof(i, self.unit_divisor)
        } else {
            format!("{}", i)
        };

        let rate_fmt = if self.unit_scale {
            format::format_sizeof(self.internal.its_per as u64, self.unit_divisor)
        } else {
            format!("{:.2}", self.internal.its_per).to_string()
        };

        return format!(
            "{} {}{}{} [{}, {}{}/s{}]",
            self.internal.spinner.next().unwrap(),
            self.desc,
            desc_spacing,
            count,
            elapsed_time_fmt,
            rate_fmt,
            self.unit,
            self.postfix
        );
    }

    fn render_lbar(&mut self, i: u64) -> (f64, String) {
        let mut progress = (i as f64) / (self.total as f64);

        if progress >= 1.0 {
            progress = 1.0;
        }

        let desc_spacing = if self.desc == "" { "" } else { ": " };
        let percentage = (progress * 100.0) as u64;
        let mut spacing = if percentage >= 10 { " " } else { "  " };

        if progress >= 1.0 {
            spacing = "";
        }

        return (
            progress,
            format!("{}{}{}{}%", self.desc, desc_spacing, spacing, percentage),
        );
    }

    fn render_rbar(&mut self, i: u64) -> String {
        let count = if self.unit_scale {
            format::format_sizeof(i, self.unit_divisor)
        } else {
            format!("{}", i)
        };

        let total = if self.unit_scale {
            format::format_sizeof(self.total, self.unit_divisor)
        } else {
            format!("{}", self.total)
        };

        self.internal.elapsed_time = self.internal.timer.elapsed().as_secs_f64();
        self.internal.its_per = i as f64 / self.internal.elapsed_time;

        let remaning_time = (self.total - i) as f64 / self.internal.its_per;

        let elapsed_time_fmt = format::format_interval(self.internal.elapsed_time as u64);
        let mut remaning_time_fmt = format::format_interval(remaning_time as u64);
        let mut rate_fmt = if self.unit_scale {
            format::format_sizeof(self.internal.its_per as u64, self.unit_divisor)
        } else {
            format!("{:.2}", self.internal.its_per).to_string()
        };

        if i == 0 {
            remaning_time_fmt = "00:00".to_string();
            rate_fmt = "?".to_string();
        }

        return format!(
            " {}/{} [{}<{}, {}{}/s{}]",
            count, total, elapsed_time_fmt, remaning_time_fmt, rate_fmt, self.unit, self.postfix,
        );
    }

    fn set_ncols(&mut self, lbar_rbar_len: i16) {
        if self.dynamic_ncols || (lbar_rbar_len + self.ncols + 2 - self.internal.bar_length) > 0 {
            if self.internal.user_ncols != -1 {
                self.ncols = self.internal.user_ncols;
            } else {
                let columns = term::get_columns();

                if columns != 0 {
                    let new_ncols = columns as i16 - lbar_rbar_len - 3;
                    if new_ncols >= 0 {
                        self.ncols = new_ncols;
                    }
                } else {
                    self.ncols = 10;

                    if !self.dynamic_ncols {
                        self.internal.user_ncols = 10;
                    }
                }
            }
        }
    }

    fn render_mbar(&mut self, progress: f64) -> String {
        let mut bar_animation: String;

        match self.animation {
            Animation::TqdmAscii | Animation::Tqdm | Animation::FillUp => {
                let nsyms = self.internal.charset_len - 1;
                let (bar_length, frac_bar_length) = format::divmod(
                    (progress * self.ncols as f64 * nsyms as f64) as u64,
                    nsyms as u64,
                );
                bar_animation = self
                    .internal
                    .charset
                    .chars()
                    .nth_back(0)
                    .unwrap()
                    .to_string()
                    .repeat(bar_length as usize);

                if bar_length < self.ncols as u64 {
                    bar_animation += &self
                        .internal
                        .charset
                        .chars()
                        .nth((frac_bar_length as usize) + 1)
                        .unwrap()
                        .to_string();
                    bar_animation += &self
                        .fill
                        .repeat((self.ncols - (bar_length as i16) - 1) as usize);
                }
            }

            Animation::Classic | Animation::FiraCode => {
                let block = (self.ncols as f64 * progress) as i16;
                bar_animation = self.internal.charset.repeat(block as usize);
                bar_animation += &self.fill.repeat((self.ncols - block) as usize);
            }

            Animation::Arrow => {
                let block = (self.ncols as f64 * progress) as i16;
                bar_animation = self.internal.charset.repeat(block as usize);
                let x = self.ncols - block - 1;
                if x > 0 {
                    bar_animation += ">";
                    bar_animation += &self.fill.repeat(x as usize);
                }
            }
        }

        match self.animation {
            Animation::TqdmAscii | Animation::Tqdm | Animation::FillUp => {
                if self.colour == "default" {
                    return format!("|{}|", bar_animation);
                } else {
                    return format!("|{}{}{}|", self.colour, bar_animation, term::COLOUR_RESET);
                }
            }

            Animation::Classic | Animation::Arrow => {
                if self.colour == "default" {
                    return format!("[{}]", bar_animation);
                } else {
                    return format!("[{}{}{}]", self.colour, bar_animation, term::COLOUR_RESET);
                }
            }

            Animation::FiraCode => {
                let bar_end = if progress >= 1.0 {
                    "\u{EE05}"
                } else {
                    "\u{EE02}"
                };

                if self.colour == "default" {
                    return format!(
                        "{}\u{EE03}{}{}{}",
                        self.colour,
                        bar_animation,
                        bar_end,
                        term::COLOUR_RESET
                    );
                } else {
                    return format!("\u{EE03}{}{}", bar_animation, bar_end);
                }
            }
        }
    }

    /// Render progress bar text using given value.
    fn render(&mut self, mut i: u64) -> (String, String, String) {
        let (progress, lbar) = self.render_lbar(i);

        if progress == 1.0 {
            i = self.total;

            if !self.leave {
                return (
                    " ".repeat(self.internal.bar_length as usize).to_string(),
                    "".to_string(),
                    "\r".to_string(),
                );
            }
        }

        let rbar = self.render_rbar(i);

        self.set_ncols(format!("\r{}{}", lbar, rbar).len() as i16);

        if self.ncols <= 0 {
            return (lbar, "".to_string(), rbar);
        }

        let mbar = self.render_mbar(progress);

        return (lbar, mbar, rbar);
    }

    /// Manually update the progress bar, useful for streams such as reading files.
    pub fn update(&mut self, n: u64) {
        if !self.internal.started {
            term::init();
            self.init();
            self.internal.timer = std::time::Instant::now();
            self.internal.started = true;
        }

        self.n += n;

        if !self.disable {
            let elapsed_time_now = self.internal.timer.elapsed().as_secs_f64();
            let mininterval_constraint =
                self.mininterval <= (elapsed_time_now - self.internal.elapsed_time);

            if self.dynamic_miniters && !mininterval_constraint {
                self.miniters += n;
            }

            let miniters_constraint;

            if self.miniters <= 1 {
                miniters_constraint = true;
            } else {
                miniters_constraint = self.n % self.miniters == 0;
            }

            if (mininterval_constraint && miniters_constraint && (self.delay <= elapsed_time_now))
                || self.n == self.total
                || self.internal.force_refresh
            {
                if self.dynamic_miniters {
                    self.miniters = 0;
                }

                if self.total != 0 {
                    let (lbar, mbar, rbar) = self.render(self.n);
                    self.internal.bar_length = ((lbar.len() + rbar.len()) as i16) + self.ncols + 2;
                    self.write_at(format!("{}{}{}", lbar, mbar, rbar));
                } else {
                    let text = self.render_unknown(self.n);
                    self.internal.bar_length = text.len() as i16;
                    self.write_at(format!("{}", text));
                }
            }
        }
    }

    fn write_at(&self, text: String) {
        if self.file.is_none() {
            crate::lock::acquire();

            if self.position == 0 {
                if matches!(self.output, Output::Stderr) {
                    term::write_to_stderr(format_args!("\r{}", text));
                } else if matches!(self.output, Output::Stdout) {
                    term::write_to_stdout(format_args!("\r{}", text));
                }
            } else {
                if matches!(self.output, Output::Stderr) {
                    term::write_to_stderr(format_args!(
                        "{}{}{}",
                        "\n".repeat(self.position as usize),
                        text,
                        format!("\x1b[{}A", self.position)
                    ));
                } else if matches!(self.output, Output::Stdout) {
                    term::write_to_stdout(format_args!(
                        "{}{}{}",
                        "\n".repeat(self.position as usize),
                        text,
                        format!("\x1b[{}A", self.position)
                    ));
                }
            }

            crate::lock::release();
        } else {
            let mut file = self.file.as_ref().unwrap();
            file.write_fmt(format_args!("{}\n", text.as_str())).unwrap();
            file.flush().unwrap();
        }
    }

    /// Clear current bar display.
    pub fn clear(&mut self) {
        if self.file.is_none() {
            let mut columns = term::get_columns() as usize;

            if columns == 0 {
                columns = self.internal.bar_length as usize;
            }

            if matches!(self.output, Output::Stderr) {
                term::write_to_stderr(format_args!("\r{}\r", " ".repeat(columns)));
            } else if matches!(self.output, Output::Stdout) {
                term::write_to_stdout(format_args!("\r{}\r", " ".repeat(columns)));
            }
        }
    }

    /// Force refresh the display of this bar.
    pub fn refresh(&mut self) {
        self.internal.force_refresh = true;
        self.update(0);
        self.internal.force_refresh = false;
    }

    /// Resets to intial iterations for repeated use.
    /// Consider combining with `leave=true`.
    pub fn reset(&mut self, total: Option<u64>) {
        self.internal.started = false;

        if total.is_some() {
            self.total = total.unwrap();
        }
    }

    /// Print a message via bar (without overlap with bars).
    /// This message is printed to stdout.
    pub fn write(&mut self, text: String) {
        self.clear();

        term::write_to_stdout(format_args!("{}\n", text));

        if self.leave {
            self.refresh();
        }
    }

    /// Take input via bar (without overlap with bars).
    /// The input message is printed to stdout.
    pub fn input(&mut self, text: &str) -> Result<String, std::io::Error> {
        self.clear();

        term::write_to_stdout(format_args!("{}", text));
        
        let mut input_string = String::new();
        std::io::stdin().read_line(&mut input_string)?;

        if self.leave {
            self.refresh();
        }

        Ok(input_string)
    }

    /// Set/Modify position of the progress bar.
    pub fn set_position(&mut self, position: u64) {
        self.n = position;
        self.update(0);
    }

    /// Set/Modify description of the progress bar.
    pub fn set_description(&mut self, desc: String) {
        self.desc = desc;
    }

    /// Set/Modify postfix (additional stats) with automatic formatting based on datatype.
    pub fn set_postfix(&mut self, postfix: String) {
        self.postfix = format!(", {}", postfix);
    }

    /// Set/Modify colour of the progress bar.
    pub fn set_colour(&mut self, colour: &str) {
        if colour != "default" {
            self.colour = term::colour(colour);
        } else {
            self.colour = "default".to_string();
        }
    }

    /// Set/Modify charset of the progress bar.
    pub fn set_charset(&mut self, charset: &[&str]) {
        self.internal.charset = charset.join("");
        self.internal.charset_len = charset.len() as u64;
        self.animation = Animation::TqdmAscii;
    }

    /// EXPERIMENTAL - monitor mode support.
    pub fn monitor(&mut self, maxinterval: f32) {
        let mut n = self.n;

        while self.n != self.total {
            std::thread::sleep(std::time::Duration::from_secs_f32(maxinterval));
            if self.n == n {
                self.refresh();
            } else {
                n = self.n
            }
        }
    }
}
