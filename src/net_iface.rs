use std::net::{Ipv4Addr, Ipv6Addr};

use neli::consts::nl::{NlmF, NlmFFlags, Nlmsg};
use neli::consts::rtnl::{Ifa, IfaFFlags, RtAddrFamily, RtScope, Rtm};
use neli::consts::socket::NlFamily;
use neli::nl::{NlPayload, Nlmsghdr};
use neli::rtnl::Ifaddrmsg;
use neli::socket::tokio::NlSocket;
use neli::socket::NlSocketHandle;
use neli::types::RtBuffer;
use neli_wifi::AsyncSocket;
use nix::net::if_::if_nametoindex;
use smart_default::SmartDefault;

use crate::errors::*;

#[derive(Clone, Debug, SmartDefault, PartialEq)]
pub struct NetworkInterface {
    index: u32,
    name: String,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    wifi_info: Option<WirelessInfo>,
    pub stats: Option<InterfaceStats>,
    pub is_up: bool,
}

impl NetworkInterface {
    pub async fn new(name: &str) -> Result<Option<Self>> {
        let if_index: u32 = if_nametoindex(name)
            .or_error(|| format!("Couldn't find index for interface {name}"))?;

        let mut socket = NlSocket::new(
            NlSocketHandle::connect(NlFamily::Route, None, &[]).error("Netlink socket error")?,
        )
        .error("Netlink socket error")?;

        let ipv4 = get_ipv4(&mut socket, if_index as i32).await?;
        let ipv6 = get_ipv6(&mut socket, if_index as i32).await?;

        let wifi_info = WirelessInfo::new(if_index).await?;

        Ok(Some(Self {
            index: if_index,
            name: name.to_owned(),
            ipv4,
            ipv6,
            wifi_info,
            ..Default::default()
        }))
    }

    pub fn ssid(&self) -> Option<String> {
        self.wifi_info.as_ref()?.ssid.clone()
    }

    pub fn signal(&self) -> Option<f64> {
        self.wifi_info.as_ref()?.signal
    }

    pub fn bitrate(&self) -> Option<f64> {
        self.wifi_info.as_ref()?.bitrate
    }

    pub fn frequency(&self) -> Option<f64> {
        self.wifi_info.as_ref()?.frequency
    }
}

async fn get_ip_addr<const T: usize>(
    socket: &mut NlSocket,
    if_index: i32,
    ifa_family: RtAddrFamily,
) -> Result<Option<[u8; T]>, Box<dyn StdError + Send + Sync + 'static>> {
    let ifaddrmsg = Ifaddrmsg {
        ifa_family,
        ifa_prefixlen: 0,
        ifa_flags: IfaFFlags::empty(),
        ifa_scope: 0,
        ifa_index: 0,
        rtattrs: RtBuffer::new(),
    };
    let nl_header = Nlmsghdr::new(
        None,
        Rtm::Getaddr,
        NlmFFlags::new(&[NlmF::Request, NlmF::Dump]),
        None,
        None,
        NlPayload::Payload(ifaddrmsg),
    );
    socket.send(&nl_header).await?;

    let mut res = None;
    let msgs = socket.recv::<u16, Ifaddrmsg>(&mut Vec::new()).await?;
    for msg in msgs {
        if msg.nl_type == u16::from(Nlmsg::Done) {
            break;
        }
        if let NlPayload::Payload(p) = msg.nl_payload {
            if p.ifa_index != if_index || RtScope::from(p.ifa_scope) != RtScope::Universe {
                continue;
            }

            let rtattrs = p.rtattrs.get_attr_handle();
            let Some(attr) = rtattrs
                .get_attribute(Ifa::Local)
                .or_else(|| rtattrs.get_attribute(Ifa::Address)) else { continue; };

            if let Ok(a) = attr.rta_payload.as_ref().try_into() {
                res = Some(a);
            }
        }
    }
    Ok(res)
}

async fn get_ipv4(socket: &mut NlSocket, if_index: i32) -> Result<Option<Ipv4Addr>> {
    Ok(get_ip_addr(socket, if_index, RtAddrFamily::Inet)
        .await
        .map_err(|_e| Error::new("CHANGE_ME"))
        .error("Couldn't get IPv4 address")?
        .map(Ipv4Addr::from))
}

async fn get_ipv6(socket: &mut NlSocket, if_index: i32) -> Result<Option<Ipv6Addr>> {
    Ok(get_ip_addr(socket, if_index, RtAddrFamily::Inet6)
        .await
        .map_err(|_e| Error::new("CHANGE_ME"))
        .error("Couldn't get IPv6 address")?
        .map(Ipv6Addr::from))
}

#[derive(Clone, Debug, SmartDefault, PartialEq)]
pub struct WirelessInfo {
    ssid: Option<String>,
    signal: Option<f64>,
    bitrate: Option<f64>,
    frequency: Option<f64>,
}

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

    // https://github.com/i3/i3status/blob/main/src/print_wireless_info.c#L126
    fn nl80211_xbm_to_percent(xbm: f64) -> f64 {
        const NOISE_FLOOR_DBM: f64 = -90.;
        const SIGNAL_MAX_DBM: f64 = -20.;
        let xbm = xbm.clamp(NOISE_FLOOR_DBM, SIGNAL_MAX_DBM);
        100. - 70. * ((SIGNAL_MAX_DBM - xbm) / (SIGNAL_MAX_DBM - NOISE_FLOOR_DBM))
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

                return Ok(Some(Self {
                    ssid,
                    signal: signal.map(|s| Self::nl80211_xbm_to_percent(s as f64)),
                    // NL80211_RATE_INFO_BITRATE is specified in units of 100 kbit/s, but is
                    // used to specify bits/s, so we convert
                    bitrate: ap.tx_bitrate.map(|br| br as f64 * 1e5),
                    // MHz -> Hz
                    frequency: iface.frequency.map(|f| f as f64 * 1e6),
                }));
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Debug, SmartDefault, Eq, PartialEq)]
pub struct InterfaceStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}
