use neli_wifi::AsyncSocket;
use nix::net::if_::if_nametoindex;

use crate::errors::*;

struct NetworkInterface {
    index: u32,
    name: String,
    ipv4: String,
    ipv6: String,
    wifi_info: WirelessInfo,
}

impl NetworkInterface {
    async fn new(name: &str) -> Result<Self> {
        let if_index: u32 = if_nametoindex(name)
            .or_error(|| format!("Couldn't find index for interface {name}"))?;
        todo!()
    }
}

struct WirelessInfo;

impl WirelessInfo {
    // https://github.com/i3/i3status/blob/main/src/print_wireless_info.c#L140
    fn find_ssid(mut ies: &[u8]) -> Option<String> {
        while ies.len() > 2 && ies[0] != 0 {
            ies = &ies[(ies[1] as usize + 2)..];
        }

        if ies.len() < 2 || ies.len() < (ies[1] as usize) + 2 {
            return None;
        }

        let ssid = &ies[2..][..(ies[1] as usize)];
        Some(String::from_utf8_lossy(ssid).into_owned())
    }

    async fn new(iface_index: u32) -> Result<Option<Self>> {
        let mut socket = match AsyncSocket::connect() {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        let interfaces = socket
            .get_interfaces_info()
            .await
            .error("Couldn't get nl80211 interfaces")?;

        for iface in interfaces {
            if let Some(index) = iface.index {
                if index as u32 != iface_index {
                    continue;
                }

                let Ok(ap) = socket.get_station_info(index).await else { continue; };
                let bss = socket
                    .get_bss_info(index)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .find(|b| b.status == Some(1));

                let signal = ap
                    .signal
                    .or(bss.as_ref().and_then(|b| b.signal.map(|s| (s / 100) as i8)));

                let ssid = iface
                    .ssid
                    .as_deref()
                    .map(|v| String::from_utf8_lossy(v).into_owned())
                    .or_else(|| {
                        bss.as_ref()
                            .and_then(|bss| bss.information_elements.as_deref())
                            .and_then(|elems| Self::find_ssid(elems))
                    });
            }
        }
        todo!()
    }
}
