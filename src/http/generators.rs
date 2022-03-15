use hex::encode as h_encode;
use base64::encode;
use rand::random;


#[derive(Debug, Serialize, Deserialize)]
pub struct SuperProperties<'a> {
    pub os:                         &'a str,
    pub browser:                     &'a str,
    pub device:                      &'a str,
    pub system_locale:               &'a str,
    pub browser_user_agent:          &'a str,
    pub browser_version:            &'a str,
    pub os_version:                 &'a str,
    pub referrer:                    &'a str,
    pub referring_domain:           &'a str,
    pub referrer_current:            &'a str,
    pub referring_domain_current:    &'a str,
    pub release_channel:             &'a str,
    pub client_build_number:        i32,
    pub client_event_source:        Option< &'a str>,
}

impl core::fmt::Display for SuperProperties<'_>  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&encode(match serde_json::to_string(&self) {
            Ok(a) => a,
            Err(_) => return Err(core::fmt::Error)
        }))?;

        Ok(())
    }
}


impl Default for SuperProperties<'_> {
    fn default() -> SuperProperties<'static> {
        SuperProperties {
            os:                         "Windows",
            browser:                    "Chrome",
            device:                     "",
            system_locale:              "en-US",
            browser_user_agent:         "Mozilla/5.0 (Windows NT 5.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/34.0.1847.116 Safari/537.36 Mozilla/5.0 (iPad; U; CPU OS 3_2 like Mac OS X; en-us) AppleWebKit/531.21.10 (KHTML, like Gecko) Version/4.0.4 Mobile/7B334b Safari/531.21.10",
            browser_version:            "34.0.1847.116",
            os_version:                 "10",
            referrer:                   "",
            referring_domain:           "",
            referrer_current:           "",
            referring_domain_current:   "",
            release_channel:            "stable",
            client_build_number:        105691,
            client_event_source:        None,   
        }
    }
}

pub fn build_super_properties(browser_user_agent: impl AsRef<str>) -> String {
    let browser_user_agent = browser_user_agent.as_ref();
    let (os, os_version) = if browser_user_agent.contains("Windows") {
        ("Windows", "10")
    } else if browser_user_agent.contains("Linux") {
        ("Linux", "")
    } else {
        ("Apple", "10_9_3")
    };
    let browser = if browser_user_agent.contains("Chrome") {
        "Chrome"
    } else if browser_user_agent.contains("Firefox") {
        "Firefox"
    } else {
        "Safari"
    };
    let browser_version = browser_user_agent.split("/").collect::<Vec<&str>>()[3]
        .split(' ')
        .collect::<Vec<&str>>()[0];

    let super_properties = SuperProperties {
        os,
        os_version,
        browser,
        browser_user_agent,
        browser_version,
        ..SuperProperties::default()
    };

    let strsuper = serde_json::to_string(&super_properties).unwrap();
    let return_val = encode(strsuper);
    return return_val;
}


pub fn build_cookies() -> String {
    let first = dcfduid();
    return format!("__dcfduid={}; __sdcfduid={}", first, sdcfduid(&first));
}

fn dcfduid() -> String {
    let data: Vec<u8> = (0..16).map(|_| random::<u8>()).collect();
    h_encode(data)
}

fn sdcfduid(base: &str) -> String {
    let data: Vec<u8> = (0..32).map(|_| random::<u8>()).collect();
    format!("{}{}", base, h_encode(data))
}