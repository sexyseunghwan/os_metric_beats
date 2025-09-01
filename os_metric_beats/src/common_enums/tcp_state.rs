#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcpState {
    Established = 0x01,
    SynSent = 0x02,
    SynRecv = 0x03,
    FinWait1 = 0x04,
    FinWait2 = 0x05,
    TimeWait = 0x06,
    Close = 0x07,
    CloseWait = 0x08,
    LastAck = 0x09,
    Listen = 0x0A,
    Closing = 0x0B,
}

impl TcpState {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x01 => Some(TcpState::Established),
            0x02 => Some(TcpState::SynSent),
            0x03 => Some(TcpState::SynRecv),
            0x04 => Some(TcpState::FinWait1),
            0x05 => Some(TcpState::FinWait2),
            0x06 => Some(TcpState::TimeWait),
            0x07 => Some(TcpState::Close),
            0x08 => Some(TcpState::CloseWait),
            0x09 => Some(TcpState::LastAck),
            0x0A => Some(TcpState::Listen),
            0x0B => Some(TcpState::Closing),
            _ => None,
        }
    }
}