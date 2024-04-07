use reqwest::blocking::{Client, ClientBuilder};

pub fn user_agent() -> String {
    // PowerShell/$($PSVersionTable.PSVersion.Major).$($PSVersionTable.PSVersion.Minor)
    // (Windows NT $([System.Environment]::OSVersion.Version.Major).$([System.Environment]::OSVersion.Version.Minor)

    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/124.0".to_string()
}

pub fn client() -> Client {
    ClientBuilder::new()
        .user_agent(user_agent())
        .build()
        .unwrap()
}
