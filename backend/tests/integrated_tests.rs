use std::fs::File;
use file_diff::{diff_files, diff};
use log::info;

mod common;

#[test]
fn test_compare_created_fmi_file_to_previous_version() {
    common::parse_pbf_to_fmi_file();
    info!("Testing for differences between the old and new fmi file");  
    assert!(diff("./tests_data/output/test-bremen-latest.fmi", "./tests_data/old-bremen-latest.fmi"), 
        "The newly parsed bremen fmi file is different from the previous fmi file. 
        Something probably changed in the parsing method. 
        Update the comparison fmi file if these changes were intended.");
}