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
/// hubris_uptime_ms = 0x1909119a
/// class = hw.pwr.pmbus.alert
/// pmbus_status = (PMBus status)
///     STATUS_WORD = 0b100000000000001
///     STATUS_VOUT = 0b000000
///     STATUS_IOUT = 0b100000
///     STATUS_INPUT = 0b000000
///     STATUS_TEMPERATURE = 0b000000
///     STATUS_CML = 0b000000
///     STATUS_MFR_SPECIFIC = 0b000000
/// (end PMBus status)
///
/// pwr_good = true
/// rail = VDDCR_CPU1_A0
/// refdes = U103
/// time = 0x19091194
/// version = 0x0
/// ```
pub struct Displayer<'a> {
    ereport: &'a serde_json::Map<String, serde_json::Value>,
    indent_spaces: usize,
    initial_indent: usize,
}

impl<'a> Displayer<'a> {
    pub const DEFAULT_INDENT_SPACES: usize = 4;
    pub const DEFAULT_INITIAL_INDENT: usize = 0;

    #[must_use]
    pub fn new(
        ereport: &'a serde_json::Map<String, serde_json::Value>,
    ) -> Self {
        Self {
            ereport,
            indent_spaces: Self::DEFAULT_INDENT_SPACES,
            initial_indent: 0,
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
                    if name.ends_with("ms") =>
                {
                    writeln!(f, "{:>indent$}{name} = {n} ms", "")?;
                    continue;
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
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> fmt::Result {
        fn print_status_reg(
            obj: &serde_json::Map<String, serde_json::Value>,
            f: &mut fmt::Formatter<'_>,
            indent: usize,
            nbits: usize,
            val_name: &str,
            reg_name: &str,
        ) -> fmt::Result {
            if let Some(val) = obj.get(val_name) {
                if let Some(val) = val.as_u64() {
                    writeln!(f, "{:>indent$}{reg_name} = {val:#0nbits$b}", "")?;
                } else {
                    writeln!(f, "{:>indent$}{reg_name} = <wrong type>", "")?;
                }
            } else {
                writeln!(f, "{:>indent$}{reg_name} = <missing>", "")?;
            }

            Ok(())
        }

        print_status_reg(obj, f, indent, 16, "word", "STATUS_WORD")?;
        print_status_reg(obj, f, indent, 8, "vout", "STATUS_VOUT")?;
        print_status_reg(obj, f, indent, 8, "iout", "STATUS_IOUT")?;
        print_status_reg(obj, f, indent, 8, "input", "STATUS_INPUT")?;
        print_status_reg(obj, f, indent, 8, "temp", "STATUS_TEMPERATURE")?;
        print_status_reg(obj, f, indent, 8, "cml", "STATUS_CML")?;
        print_status_reg(obj, f, indent, 8, "mfr", "STATUS_MFR_SPECIFIC")?;

        Ok(())
    }
}

impl fmt::Display for Displayer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.prettyprint_json_obj(f, self.initial_indent, self.ereport)
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
