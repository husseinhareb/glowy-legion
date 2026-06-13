use std::fs;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DmiInfo {
    pub product_name: Option<String>,
    pub product_version: Option<String>,
    pub sys_vendor: Option<String>,
}

pub fn read_dmi_info() -> DmiInfo {
    DmiInfo {
        product_name: read_dmi_file("/sys/class/dmi/id/product_name"),
        product_version: read_dmi_file("/sys/class/dmi/id/product_version"),
        sys_vendor: read_dmi_file("/sys/class/dmi/id/sys_vendor"),
    }
}

fn read_dmi_file(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
