use erebor::Displayer;
use expectorate::assert_contents;

type Object = serde_json::Map<String, serde_json::Value>;

const PMBUS_ALERT_EREPORT: &str = r#"{
  "baseboard_part_number": "913-0000023",
  "baseboard_rev": 3,
  "baseboard_serial_number": "2ED5H8WT\u0000\u0000\u0000",
  "ereport_message_version": 0,
  "hubris_caboose": {
    "board": "cosmo-b",
    "commit": "3b9b40d840984e3cca6d40e73f020f96a40538c5",
    "version": "1.0.67"
  },
  "hubris_task_gen": 0,
  "hubris_task_name": "cosmo_seq",
  "hubris_uptime_ms": 420024730,
  "k": "hw.pwr.pmbus.alert",
  "pmbus_status": {
    "cml": 0,
    "input": 0,
    "iout": 32,
    "mfr": 0,
    "temp": 0,
    "vout": 0,
    "word": 16385
  },
  "pwr_good": true,
  "rail": "VDDCR_CPU1_A0",
  "refdes": "U103",
  "time": 420024724,
  "v": 0
}
"#;

/// This is like `PMBUS_ALERT_EREPORT`, but with some PMBus status fields not
/// present, null, and ill-typed.
const PMBUS_ALERT_MISSING_FIELDS_EREPORT: &str = r#"{
  "baseboard_part_number": "913-0000023",
  "baseboard_rev": 3,
  "baseboard_serial_number": "2ED5H8WT\u0000\u0000\u0000",
  "ereport_message_version": 0,
  "hubris_caboose": {
    "board": "cosmo-b",
    "commit": "3b9b40d840984e3cca6d40e73f020f96a40538c5",
    "version": "1.0.67"
  },
  "hubris_task_gen": 0,
  "hubris_task_name": "cosmo_seq",
  "hubris_uptime_ms": 420024730,
  "k": "hw.pwr.pmbus.alert",
  "pmbus_status": {
    "cml": 1000.6962,
    "input": true,
    "iout": 32,
    "temp": null,
    "vout": 0,
    "word": 16385
  },
  "pwr_good": true,
  "rail": "VDDCR_CPU1_A0",
  "refdes": "U103",
  "time": 420024724,
  "v": 0
}
"#;

const APOLLO_13_EREPORT: &str = r#"{
  "baseboard_part_number": "913-0000023",
  "baseboard_rev": 3,
  "baseboard_serial_number": "BRM4200069",
  "ereport_message_version": 0,
  "hubris_caboose": {
    "board": "cosmo-b",
    "commit": "3b9b40d840984e3cca6d40e73f020f96a40538c5",
    "version": "1.0.67"
  },
  "hubris_task_gen": 0,
  "hubris_task_name": "cosmo_seq",
  "hubris_uptime_ms": 420024730,
  "k": "hw.pwr.pmbus.alert",
  "crew": [
     "Swigert",
     "Haise",
     "Lovell"
  ],
  "message": "Houston, we have a problem!",
  "mission": {
     "program": "Apollo",
     "flight": 13
  },
  "fault": {
     "kind": "MAIN_BUS_B_UNDERVOLT",
     "main_bus_b_voltage": 0.0
  },
  "O2_tanks_stirred": true,
  "time": 420024724,
  "v": 0
}
"#;

const BAD_ASHIFT_EREPORT: &str = r#"
{
    "class": "ereport.fs.zfs.vdev.bad_ashift",
    "ena": 26298341695423489,
    "detector": {
        "version": 0,
        "scheme": "zfs",
        "pool": 667943453307405115,
        "vdev": 13092845122578407208
    },
    "pool": "rpool",
    "pool_guid": 667943453307405115,
    "pool_context": 0,
    "pool_failmode": "wait",
    "vdev_guid": 13092845122578407208,
    "vdev_type": "disk",
    "vdev_path": "/dev/dsk/c1t0d0s1",
    "vdev_devid": "id1,kdev@ABHYVE-9F86-8195-80CF/b",
    "vdev_ashift": 9,
    "parent_guid": 667943453307405115,
    "parent_type": "root",
    "prev_state": 0,
    "__ttl": 1,
    "__tod": [
        1774613124,
        398912144
    ]
}
"#;

// We only run tests for formatting ereports with PMBus status objects when the
// `pmbus` dependency is enabled. They will be formatted differently when the
// `pmbus` dependency is not enabled, since we require the `pmbus` crate to
// determine the names of bitfields.
#[cfg_attr(not(feature = "pmbus"), ignore)]
#[test]
fn pmbus_alert_default_indent() {
    let json = serde_json::from_str::<Object>(PMBUS_ALERT_EREPORT)
        .expect("JSON must parse");
    let displayed = Displayer::new(&json).to_string();
    assert_contents(
        "tests/output/pmbus_alert_default_indent.txt",
        displayed.as_ref(),
    );
}

// We only run tests for formatting ereports with PMBus status objects when the
// `pmbus` dependency is enabled. They will be formatted differently when the
// `pmbus` dependency is not enabled, since we require the `pmbus` crate to
// determine the names of bitfields.
#[cfg_attr(not(feature = "pmbus"), ignore)]
#[test]
fn pmbus_alert_8_space_tabs() {
    let json = serde_json::from_str::<Object>(PMBUS_ALERT_EREPORT)
        .expect("JSON must parse");
    let displayed = Displayer::new(&json).with_indent_spaces(8).to_string();
    assert_contents(
        "tests/output/pmbus_alert_8_space_tabs.txt",
        displayed.as_ref(),
    );
}

// We only run tests for formatting ereports with PMBus status objects when the
// `pmbus` dependency is enabled. They will be formatted differently when the
// `pmbus` dependency is not enabled, since we require the `pmbus` crate to
// determine the names of bitfields.
#[cfg_attr(not(feature = "pmbus"), ignore)]
#[test]
fn pmbus_alert_indented() {
    let json = serde_json::from_str::<Object>(PMBUS_ALERT_EREPORT)
        .expect("JSON must parse");
    let displayed =
        Displayer::new(&json).with_initial_indent_spaces(4).to_string();
    assert_contents(
        "tests/output/pmbus_alert_indented.txt",
        displayed.as_ref(),
    );
}

// We only run tests for formatting ereports with PMBus status objects when the
// `pmbus` dependency is enabled. They will be formatted differently when the
// `pmbus` dependency is not enabled, since we require the `pmbus` crate to
// determine the names of bitfields.
#[cfg_attr(not(feature = "pmbus"), ignore)]
#[test]
fn pmbus_alert_missing_fields() {
    let json =
        serde_json::from_str::<Object>(PMBUS_ALERT_MISSING_FIELDS_EREPORT)
            .expect("JSON must parse");
    let displayed = Displayer::new(&json).to_string();
    assert_contents(
        "tests/output/pmbus_alert_missing_fields.txt",
        displayed.as_ref(),
    );
}

#[test]
fn apollo_13_default_indent() {
    let json = serde_json::from_str::<Object>(APOLLO_13_EREPORT)
        .expect("JSON must parse");
    let displayed = Displayer::new(&json).to_string();
    assert_contents(
        "tests/output/apollo_13_default_indent.txt",
        displayed.as_ref(),
    );
}

#[test]
fn apollo_13_8_space_tabs() {
    let json = serde_json::from_str::<Object>(APOLLO_13_EREPORT)
        .expect("JSON must parse");
    let displayed = Displayer::new(&json).with_indent_spaces(8).to_string();
    assert_contents(
        "tests/output/apollo_13_8_space_tabs.txt",
        displayed.as_ref(),
    );
}

#[test]
fn apollo_13_indented() {
    let json = serde_json::from_str::<Object>(APOLLO_13_EREPORT)
        .expect("JSON must parse");
    let displayed =
        Displayer::new(&json).with_initial_indent_spaces(4).to_string();
    assert_contents("tests/output/apollo_13_indented.txt", displayed.as_ref());
}

#[test]
fn bad_ashift() {
    let json = serde_json::from_str::<Object>(BAD_ASHIFT_EREPORT)
        .expect("JSON must parse");
    let displayed = Displayer::new(&json).to_string();
    assert_contents(
        "tests/output/bad_ashift_default_indent.txt",
        displayed.as_ref(),
    );
}

#[test]
fn bad_ashift_8_space_tabs() {
    let json = serde_json::from_str::<Object>(BAD_ASHIFT_EREPORT)
        .expect("JSON must parse");
    let displayed = Displayer::new(&json).with_indent_spaces(8).to_string();
    assert_contents(
        "tests/output/bad_ashift_8_space_tabs.txt",
        displayed.as_ref(),
    );
}

#[test]
fn bad_ashift_indented() {
    let json = serde_json::from_str::<Object>(BAD_ASHIFT_EREPORT)
        .expect("JSON must parse");
    let displayed =
        Displayer::new(&json).with_initial_indent_spaces(4).to_string();
    assert_contents("tests/output/bad_ashift_indented.txt", displayed.as_ref());
}
