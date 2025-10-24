mod common;

use common::{
    assert_case_matches, find_sketchtool, parse_sketchtool_cases, print_alignment_table,
    sketchtool_script,
};
use std::env;
use std::process::Command;

const EXPECTED_SKETCHTOOL_CASES: usize = 1_720;

#[test]
fn sketchtool_in_memory_rect_matrix_matches() {
    let Some(sketchtool) = find_sketchtool() else {
        if env::var_os("SQUIRCLE_FRAME_REQUIRE_SKETCHTOOL").is_some() {
            panic!("未找到 sketchtool，但 SQUIRCLE_FRAME_REQUIRE_SKETCHTOOL 已启用");
        }
        eprintln!("跳过 sketchtool 集成测试：未找到 sketchtool");
        return;
    };

    let output = Command::new(&sketchtool)
        .arg("run-script")
        .arg(sketchtool_script())
        .arg("--without-activating=YES")
        .arg("--timeout=120")
        .output()
        .expect("无法运行 sketchtool");

    assert!(
        output.status.success(),
        "sketchtool 运行失败\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cases = parse_sketchtool_cases(&stdout);
    assert_eq!(
        cases.len(),
        EXPECTED_SKETCHTOOL_CASES,
        "sketchtool 用例数量异常"
    );

    let results = cases
        .into_iter()
        .map(assert_case_matches)
        .collect::<Vec<_>>();
    print_alignment_table(&results);
}
