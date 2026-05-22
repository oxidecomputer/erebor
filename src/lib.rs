// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A less ugly pretty-printer for `serde_json`-y ereport data.

use std::fmt;

/// A displayer for `serde_json::Map`s containing ereport data.
///
/// This struct will pretty-print the ereport data in a human-readable format
/// which kind of loosely attempts to ape that of `fmdump -eV` on an illumos
/// system. To print an ereport's data using this type, use [`Displayer::new`]
/// with a reference to the ereport data, and then use [`fmt::Display`] to
/// format the returned `Displayer`.
///
/// The display output from this type is multi-line. By default, it is not
/// indented. To write all output from this displayer indented a certain number
/// of spaces, as one might wish to if the ereport is formatted as part of a
/// longer multi-line output with varying indentation levels, use
/// [`Displayer::with_initial_indent_spaces`] prior to formatting the
/// `Displayer`.
///
/// Each nested object in the ereport data will be indented under its parent
/// object. Each new level of indentation is, by default,
/// [`Displayer::DEFAULT_INDENT_SPACES`] spaces wide (4 spaces). To override
/// this, call [`Displayer::with_indent_spaces`] prior to formatting the
/// `Displayer`.
///
/// ## Example Output
///
/// Consider the following JSON ereport from a service processor:
///
/// ```json
/// ```
///
/// With the default indentation configuration, a `Displayer` for that JSON
/// will produce the following output:
///
/// ```notrust
/// baseboard_part_number = 913-0000023
/// baseboard_rev = 0x3
/// baseboard_serial_number = "2ED5H8WT\0\0\0"
/// ereport_message_version = 0x0
/// hubris_caboose = (object)
///     board = cosmo-b
///     commit = 3b9b40d840984e3cca6d40e73f020f96a40538c5
///     version = 1.0.67
/// (end hubris_caboose)
///
/// hubris_task_gen = 0x0
/// hubris_task_name = cosmo_seq
/// hubris_uptime_ms = 420024730 (420024.73s)
/// class = hw.pwr.pmbus.alert
/// pmbus_status = (PMBus status)
///    STATUS_WORD         = 0x4001 (OutputCurrentFault | NoneOfTheAbove)
///    STATUS_VOUT         = 0x00
///    STATUS_IOUT         = 0x20 (OutputOvercurrentWarning)
///    STATUS_INPUT        = 0x00
///    STATUS_TEMPERATURE  = 0x00
///    STATUS_CML          = 0x00
///    STATUS_MFR_SPECIFIC = 0x00
/// (end PMBus status)
/// pwr_good = true
/// rail = VDDCR_CPU1_A0
/// refdes = U103
/// time = 0x19091194
/// version = 0x0
/// ```
pub struct Displayer<'a> {
    ereport: &'a serde_json::Value,
    indent_spaces: usize,
    initial_indent: usize,
    humanize_durations: bool,
}

type Object = serde_json::Map<String, serde_json::Value>;

impl<'a> Displayer<'a> {
    pub const DEFAULT_INDENT_SPACES: usize = 4;
    pub const DEFAULT_INITIAL_INDENT: usize = 0;

    #[must_use]
    pub fn new(ereport: &'a serde_json::Value) -> Self {
        Self {
            ereport,
            indent_spaces: Self::DEFAULT_INDENT_SPACES,
            initial_indent: 0,
            humanize_durations: true,
        }
    }

    /// Sets the number of spaces in one level of indentation.
    ///
    /// This defaults to [`Self::DEFAULT_INDENT_SIZE`].
    #[must_use]
    pub fn with_indent_spaces(self, indent_size: usize) -> Self {
        Self { indent_spaces: indent_size, ..self }
    }

    #[must_use]
    pub fn with_initial_indent_spaces(self, initial_indent: usize) -> Self {
        Self { initial_indent, ..self }
    }

    /// If `true`, number fields ending in a suffix that looks like a duration
    /// value in a known unit (in this case, `_ns`, `_us`, `_ms`, and `_s`),
    /// will be interpreted as [`std::time::Duration`] values and formatted as
    /// such.
    ///
    /// If `false`, this behavior is disabled.
    ///
    /// By default, this is enabled.
    #[must_use]
    pub fn with_humanized_durations(self, humanize_times: bool) -> Self {
        Self { humanize_durations: humanize_times, ..self }
    }

    fn prettyprint_json(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
        name: Name<'_>,
        value: &serde_json::Value,
    ) -> fmt::Result {
        match value {
            serde_json::Value::Object(obj) => {
                f.write_str("(object)\n")?;
                self.prettyprint_json_obj(f, indent + self.indent_spaces, obj)?;
                if !obj.is_empty() {
                    writeln!(f, "{:>indent$}(end {name})", "")?;
                }
            }
            serde_json::Value::Array(arr) => {
                writeln!(f, "(array of {} elements)", arr.len())
                    .expect("writing to a string should always work");
                if !arr.is_empty() {
                    for (idx, value) in arr.iter().enumerate() {
                        let indent = indent + self.indent_spaces;
                        write!(f, "{:>indent$}", "")
                            .expect("writing to a string should always work");
                        self.prettyprint_json(
                            f,
                            indent,
                            Name::ArrayIndex(&name, idx),
                            value,
                        )?;
                        f.write_str("\n")?;
                    }
                    writeln!(f, "{:>indent$}(end {name})", "")?;
                }
            }
            serde_json::Value::String(s) => {
                if s.contains('\0') {
                    write!(f, "{s:?}")?;
                } else if !s.contains('\n') {
                    f.write_str(s)?;
                } else {
                    let indent = indent + self.indent_spaces;
                    let mut lines = s.lines();
                    if let Some(line) = lines.next() {
                        writeln!(f, "\n{:>indent$}{line}", "")?;
                        for line in lines {
                            writeln!(f, "{:>indent$}{line}", "")?;
                        }
                    }
                }
            }
            serde_json::Value::Number(n) => {
                if let Some(n) = n.as_u64() {
                    write!(f, "{n:#x}")?;
                } else {
                    fmt::Display::fmt(n, f)?;
                }
            }
            serde_json::Value::Bool(b) => fmt::Display::fmt(b, f)?,
            serde_json::Value::Null => {
                f.write_str(NULL)?;
            }
        }

        Ok(())
    }

    fn prettyprint_json_obj(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> fmt::Result {
        for (key, value) in obj {
            let key = match (key.as_str(), value) {
                ("k", _) => "class",
                // special-case versions so that they aren't in hex
                (name, serde_json::Value::Number(n))
                    if name == "v" || name == "version" =>
                {
                    writeln!(f, "{:>indent$}version = {n}", "")?;
                    continue;
                }
                // special-case uptime ms so they aren't in hex
                (name, serde_json::Value::Number(n))
                    if self.humanize_durations
                        && name_looks_duration_y(name) =>
                {
                    if let Some(dur) = self.number_to_duration(name, n) {
                        writeln!(f, "{:>indent$}{name} = {n} ({dur:?})", "")?;
                        continue;
                    } else {
                        name
                    }
                }
                ("pmbus_status", serde_json::Value::Object(status)) => {
                    writeln!(f, "{:>indent$}{key} = (PMBus status)", "")?;
                    self.prettyprint_pmbus_status(
                        f,
                        indent + self.indent_spaces,
                        status,
                    )?;
                    writeln!(f, "{:>indent$}(end PMBus status)\n", "")?;
                    continue;
                }
                (k, _) => k,
            };
            write!(f, "{:>indent$}{key} = ", "")?;
            self.prettyprint_json(f, indent, Name::Key(key), value)?;
            writeln!(f)?;
        }
        Ok(())
    }

    fn prettyprint_pmbus_status(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        obj: &Object,
    ) -> fmt::Result {
        macro_rules! status_regs {
            ($($key:literal, $REG:ident $(as $Width:ty)?);+;) => {
                // First calculate the maximum register name width.
                const REGWIDTH: usize = const_max_len(&[$(
                    stringify!($REG),
                )+]);
                // Next, generate code to print each register, optionally using
                // the `pmbus` crate to decode its bits, if our `pmbus`
                // dependency is enabled.
                $(
                    let name = stringify!($REG);
                    if let Some(val) = obj.get($key) {
                        if let Some(val) = val.as_u64() {
                            $(
                                #[cfg(feature = "pmbus")]
                                print_cmddata(
                                    f,
                                    indent,
                                    &pmbus::commands::$REG::CommandData(val as $Width),
                                )?;
                                #[cfg(not(feature = "pmbus"))]
                            )?
                            writeln!(
                                f,
                                "{:>indent$}{name:<REGWIDTH$} = {val:#04x}",
                                "",
                            )?;

                        } else {
                            let err = if val.is_null() {
                                NULL
                            } else {
                                WRONG_TYPE
                            };
                            writeln!(
                                f,
                                "{:>indent$}{name:<REGWIDTH$} = {err}",
                                ""
                            )?;
                        }
                    } else {
                        writeln!(
                            f,
                            "{:>indent$}{name:<REGWIDTH$} = {MISSING}",
                            ""
                        )?;
                    }
                )+

            };
        }
        #[cfg(feature = "pmbus")]
        fn print_cmddata(
            f: &mut fmt::Formatter<'_>,
            indent: usize,
            data: &impl pmbus::CommandData,
        ) -> fmt::Result {
            let mut cmdres = Ok(());
            data.command(|cmd| {
                let (bits, pmbus::Bitwidth(bitwidth)) = data.raw();
                let hexchars = (bitwidth / 4) as usize + 2;
                let name = cmd.name();
                cmdres = write!(
                    f,
                    "{:>indent$}{name:<REGWIDTH$} = {bits:#0hexchars$x}",
                    "",
                );
            });
            cmdres?;
            let mut seenany = false;
            let mut res = Ok(());
            let interpret_ok = data.interpret(
                || unreachable!("not VoutMode"),
                |field, value| {
                    if value.raw() != 0 {
                        res = write!(
                            f,
                            "{}{}",
                            if seenany { " | " } else { " (" },
                            field.name()
                        );
                        seenany = true;
                    }
                },
            );
            res?;
            if let Err(e) = interpret_ok {
                // For a status register this should never happen, since it's
                // just bitflags, and every bit is defined...but handle it
                // gracefully anyhow!
                writeln!(f, " (uninterpretable: {e:?})")?;
            } else {
                writeln!(f, "{}", if seenany { ")" } else { "" })?;
            }
            Ok(())
        }

        status_regs! {
            "word", STATUS_WORD as u16;
            "vout", STATUS_VOUT as u8;
            "iout", STATUS_IOUT as u8;
            "input", STATUS_INPUT as u8;
            "temp", STATUS_TEMPERATURE as u8;
            "cml", STATUS_CML as u8;
            "mfr", STATUS_MFR_SPECIFIC;
        }

        Ok(())
    }

    fn number_to_duration(
        &self,
        name: &str,
        value: &serde_json::Number,
    ) -> Option<std::time::Duration> {
        if !self.humanize_durations {
            return None;
        }

        let value = value.as_u64()?;
        if name.ends_with(NANOSECOND_SUFFIX) {
            Some(std::time::Duration::from_nanos(value))
        } else if name.ends_with(MICROSECOND_SUFFIX) {
            Some(std::time::Duration::from_micros(value))
        } else if name.ends_with(MILLISECOND_SUFFIX) {
            Some(std::time::Duration::from_millis(value))
        } else if name.ends_with(SECOND_SUFFIX) {
            Some(std::time::Duration::from_secs(value))
        } else {
            None
        }
    }
}

impl fmt::Display for Displayer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ereport {
            // If the top-level value is an object, go straight to the object
            // prettyprinter to avoid trying to wrap it in a named object.
            serde_json::Value::Object(obj) => {
                self.prettyprint_json_obj(f, self.initial_indent, obj)
            }
            val => self.prettyprint_json(
                f,
                self.initial_indent,
                // If it's *not* an object and we need a name for printing "(end
                // {name})", it must be an array, so give it the generic name
                // "array", as it doesn't have a field name.
                Name::Key("array"),
                val,
            ),
        }
    }
}

enum Name<'a> {
    Key(&'a str),
    ArrayIndex(&'a Name<'a>, usize),
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Name::Key(key) => write!(f, "{key}"),
            Name::ArrayIndex(key, index) => {
                write!(f, "{key}[{index}]")
            }
        }
    }
}

const NULL: &str = "<null>";
const WRONG_TYPE: &str = "<wrong type>";
const MISSING: &str = "<missing>";

const NANOSECOND_SUFFIX: &str = "_ns";
const MICROSECOND_SUFFIX: &str = "_us";
const MILLISECOND_SUFFIX: &str = "_ms";
const SECOND_SUFFIX: &str = "_s";

fn name_looks_duration_y(name: &str) -> bool {
    name.ends_with(NANOSECOND_SUFFIX)
        || name.ends_with(MICROSECOND_SUFFIX)
        || name.ends_with(MILLISECOND_SUFFIX)
        || name.ends_with(SECOND_SUFFIX)
}

pub(crate) const fn const_max_len(strs: &[&str]) -> usize {
    let mut max = 0;
    let mut i = 0;
    while i < strs.len() {
        let len = strs[i].len();
        if len > max {
            max = len;
        }
        i += 1;
    }
    max
}
