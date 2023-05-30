//! Utilities to manipulate network interfaces using [`netlink`].
//!
//! This module uses [`neli`] and [`neli-wifi`] to communicate with the Kernel via
//! [`netlink sockets`].
//!
//! [`netlink`]: https://www.ietf.org/rfc/rfc3549.txt
//! [`neli`]: https://docs.rs/neli/latest/neli/index.html
//! [`neli-wifi`]: https://github.com/MaxVerevkin/neli-wifi/
//! [`netlink sockets`]: https://kernel.org/doc/html/next/userspace-api/netlink/intro.html
//!

use std::net::{Ipv4Addr, Ipv6Addr};

use neli::attr::Attribute;
use neli::consts::nl::{NlmF, NlmFFlags, Nlmsg};
use neli::consts::rtnl::{Arphrd, Ifa, IfaFFlags, IffFlags, Ifla, RtAddrFamily, RtScope, Rtm};
use neli::consts::socket::NlFamily;
use neli::nl::{NlPayload, Nlmsghdr};
use neli::rtnl::{Ifaddrmsg, Ifinfomsg};
use neli::socket::{tokio::NlSocket, NlSocketHandle};
use neli::types::RtBuffer;
use neli_wifi::AsyncSocket;
use nix::net::if_::if_nametoindex;
use smart_default::SmartDefault;

use crate::errors::*;

#[derive(Clone, Debug, SmartDefault, PartialEq)]
pub struct NetworkInterface {
    index: i32,
    name: String,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    wifi_info: Option<WirelessInfo>,
    pub stats: Option<RtnlLinkStats>,
}

impl NetworkInterface {
    pub async fn new(name: &str) -> Result<Option<Self>> {
        let if_index = if_nametoindex(name)
            .or_error(|| format!("Couldn't find index for interface {name}"))?
            as i32;

        let mut socket = NlSocket::new(
            NlSocketHandle::connect(NlFamily::Route, None, &[]).error("Netlink socket error")?,
        )
        .error("Netlink socket error")?;

        let ipv4 = get_ipv4(&mut socket, if_index).await?;
        let ipv6 = get_ipv6(&mut socket, if_index).await?;
        let wifi_info = WirelessInfo::new(if_index).await?;
        let stats = RtnlLinkStats::new(&mut socket, if_index)
            .await
            .map_err(BoxedError)
            .error("Couldn't get interface stats")?;
        if stats.is_none() {
            return Ok(None);
        }

        Ok(Some(Self {
            index: if_index,
            name: name.to_owned(),
            ipv4,
            ipv6,
            wifi_info,
            stats,
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

macro_rules! rtnetlink_recv {
    ($socket:ident, $nl_payload:ident, $nl_type:expr, $if_index:expr, $if_attr:ident, $payload:ident : $ptype:ty => $($body:tt)*) => {
        let nl_header = Nlmsghdr::new(
            None,
            $nl_type,
            NlmFFlags::new(&[NlmF::Request, NlmF::Dump]),
            None,
            None,
            NlPayload::Payload($nl_payload),
        );
        $socket.send(&nl_header).await?;

        // Without this loop, I'd get an error `UnexpectedEOB`.
        'msgs: loop {
            let msgs = $socket.recv::<u16, $ptype>(&mut Vec::new()).await?;
            for msg in msgs {
                if msg.nl_type == u16::from(Nlmsg::Done) {
                    break 'msgs;
                }

                if let NlPayload::Payload($payload) = msg.nl_payload {
                    if $payload.$if_attr != $if_index {
                        continue;
                    }

                    $($body)*
                }
            }
        }
    };
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

    let mut res = None;

    rtnetlink_recv!(socket, ifaddrmsg, Rtm::Getaddr, if_index, ifa_index, p: Ifaddrmsg => {
        if RtScope::from(p.ifa_scope) != RtScope::Universe {
            continue;
        }

        let rtattrs = p.rtattrs.get_attr_handle();
        let Some(attr) = rtattrs
            .get_attribute(Ifa::Local)
            .or_else(|| rtattrs.get_attribute(Ifa::Address)) else { continue; };

        if let Ok(a) = attr.rta_payload.as_ref().try_into() {
            res = Some(a);
        }
    });

    Ok(res)
}

async fn get_ipv4(socket: &mut NlSocket, if_index: i32) -> Result<Option<Ipv4Addr>> {
    Ok(get_ip_addr(socket, if_index, RtAddrFamily::Inet)
        .await
        .map_err(BoxedError)
        .error("Couldn't get IPv4 address")?
        .map(Ipv4Addr::from))
}

async fn get_ipv6(socket: &mut NlSocket, if_index: i32) -> Result<Option<Ipv6Addr>> {
    Ok(get_ip_addr(socket, if_index, RtAddrFamily::Inet6)
        .await
        .map_err(BoxedError)
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

    pub async fn new(iface_index: i32) -> Result<Option<Self>> {
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
                if index != iface_index {
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
                            .and_then(Self::find_ssid)
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

#[derive(Clone, Copy, Debug, SmartDefault, Eq, PartialEq)]
pub struct RtnlLinkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

impl RtnlLinkStats {
    pub async fn new(
        socket: &mut NlSocket,
        if_index: i32,
    ) -> Result<Option<Self>, Box<dyn StdError + Send + Sync + 'static>> {
        let ifinfomsg = Ifinfomsg::new(
            RtAddrFamily::Unspecified,
            Arphrd::None,
            0,
            IffFlags::empty(),
            IffFlags::empty(),
            RtBuffer::new(),
        );

        let mut res = None;

        rtnetlink_recv!(socket, ifinfomsg, Rtm::Getlink, if_index, ifi_index, p: Ifinfomsg => {
            for rtattr in p.rtattrs.iter() {
                match rtattr.rta_type {
                    Ifla::Operstate => {
                        let state = rtattr.get_payload_as::<u8>()?;
                        // https://www.kernel.org/doc/html/latest/networking/operstates.html#tlv-ifla-operstate
                        if state != 6 {
                            return Ok(None);
                        }
                    }
                    Ifla::Stats64 => {
                        res = Some(Self::from_rtnl_link_stats64(rtattr.payload().as_ref())?)
                    }
                    _ => (),
                }
            }
        });

        Ok(res)
    }

    // https://www.kernel.org/doc/html/v5.11/networking/statistics.html#c.rtnl_link_stats64 :
    // struct rtnl_link_stats64 {
    //   __u64 rx_packets,
    //   __u64 tx_packets,
    //   __u64 rx_bytes,
    //   __u64 tx_bytes,
    //   -- snip --
    // }
    // For the purpose of upload and download speed, we only care about rx_bytes and tx_bytes.
    pub fn from_rtnl_link_stats64(stats: &[u8]) -> Result<Self> {
        // We need stats to have at least 32 bytes, which will be read as
        // 4 blocks of 8 bytes. We only care about the 3rd anf 4th blocks.
        if stats.len() < 32 {
            return Err(Error::new(format!(
                "Bad contents for interface stats: {stats:?}"
            )));
        }

        let stats_ptr = stats.as_ptr() as *const u64;

        Ok(Self {
            rx_bytes: unsafe { stats_ptr.add(2).read_unaligned() },
            tx_bytes: unsafe { stats_ptr.add(3).read_unaligned() },
        })
    }
}
