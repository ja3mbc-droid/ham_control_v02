//! WSJT-X NetworkMessage protocol parser
//!
//! WSJT-X / FreeDV UDP 2237 interface
//! Phase 1 implementation

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum MessageType {
    Heartbeat = 0,
    Status = 1,
    Decode = 2,
    Clear = 3,
    Reply = 4,
    QsoLogged = 5,
    Close = 6,
    WsprDecode = 7,
    LoggedAdif = 12,
    Unknown = 0xffff_ffff,
}

impl From<u32> for MessageType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Heartbeat,
            1 => Self::Status,
            2 => Self::Decode,
            3 => Self::Clear,
            4 => Self::Reply,
            5 => Self::QsoLogged,
            6 => Self::Close,
            7 => Self::WsprDecode,
            12 => Self::LoggedAdif,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct NetworkMessage {
    pub magic: String,
    pub schema: u32,
    pub msg_type: MessageType,
    pub payload: Vec<u8>,
}


pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {

    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
        }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn read_u32(&mut self) -> Result<u32, String> {
        if self.remaining() < 4 {
            return Err("not enough data for u32".into());
        }

        let b = &self.data[self.pos..self.pos + 4];
        self.pos += 4;

        Ok(u32::from_be_bytes([
            b[0], b[1], b[2], b[3]
        ]))
    }

    pub fn read_bytes(
        &mut self,
        len: usize
    ) -> Result<Vec<u8>, String> {

        if self.remaining() < len {
            return Err("not enough data".into());
        }

        let result =
            self.data[self.pos..self.pos + len].to_vec();

        self.pos += len;

        Ok(result)
    }
}


pub fn parse_message(
    data: &[u8]
) -> Result<NetworkMessage, String> {

    let mut reader = ByteReader::new(data);

    let magic_bytes =
        reader.read_bytes(4)?;

    let magic =
        String::from_utf8_lossy(&magic_bytes)
        .to_string();

    let schema =
        reader.read_u32()?;

    let msg_number =
        reader.read_u32()?;

    let msg_type =
        MessageType::from(msg_number);

    let payload =
        reader.read_bytes(reader.remaining())?;

    Ok(NetworkMessage {
        magic,
        schema,
        msg_type,
        payload,
    })
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_message_basic() {

        let mut packet = Vec::new();

        // magic
        packet.extend_from_slice(b"WSJT");

        // schema
        packet.extend_from_slice(&3u32.to_be_bytes());

        // message type = QsoLogged
        packet.extend_from_slice(&5u32.to_be_bytes());

        // dummy payload
        packet.extend_from_slice(b"TEST");


        let result = parse_message(&packet)
            .expect("parse failed");


        assert_eq!(result.magic, "WSJT");
        assert_eq!(result.schema, 3);
        assert_eq!(
            result.msg_type,
            MessageType::QsoLogged
        );

        assert_eq!(
            result.payload,
            b"TEST"
        );
    }
}


#[derive(Debug)]
pub struct QsoLogged {
    pub date_time_off: String,
    pub dx_call: String,
    pub dx_grid: String,
    pub tx_frequency: u64,
    pub mode: String,
    pub report_sent: String,
    pub report_received: String,
    pub comments: String,
    pub name: String,
    pub date_time_on: String,
    pub my_call: String,
    pub my_grid: String,
}


impl<'a> ByteReader<'a> {

    pub fn read_u64(&mut self) -> Result<u64, String> {

        if self.remaining() < 8 {
            return Err("not enough data for u64".into());
        }

        let b = &self.data[self.pos..self.pos + 8];
        self.pos += 8;

        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3],
            b[4], b[5], b[6], b[7],
        ]))
    }


    // QDateTime (8byte JulianDay + 4byte msec + 1byte timespec [+4byte UTC offset])
    pub fn read_qdatetime(&mut self) -> Result<String, String> {
        if self.remaining() < 13 {
            return Err("not enough data for QDateTime".into());
        }
        let jd = self.read_u64()?;
        let msec = self.read_u32()?;
        let timespec_bytes = self.read_bytes(1)?;
        let timespec = timespec_bytes[0];
        if timespec == 2 {
            self.read_bytes(4)?; // UTC offset seconds
        }
        Ok(format!("JD{}+{}ms", jd, msec))
    }

    // Qt QString (UTF-8 converted from QString)
    pub fn read_qstring(&mut self) -> Result<String, String> {

        let len = self.read_u32()? as usize;

        if len == 0 {
            return Ok(String::new());
        }

        let data = self.read_bytes(len)?;

        Ok(
            String::from_utf8_lossy(&data)
            .to_string()
        )
    }
}


impl<'a> ByteReader<'a> {

    /// Qt QString (QDataStream UTF-16BE)
    pub fn read_qstring_fixed(&mut self) -> Result<String, String> {

        let len = self.read_u32()? as usize;

        if len == 0 {
            return Ok(String::new());
        }

        if self.remaining() < len {
            return Err("QString length exceeds payload".into());
        }

        let data = self.read_bytes(len)?;

        let mut chars = Vec::new();

        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                chars.push(
                    u16::from_be_bytes([
                        chunk[0],
                        chunk[1]
                    ])
                );
            }
        }

        Ok(
            String::from_utf16_lossy(&chars)
        )
    }
}



pub fn parse_qso_logged(
    payload: &[u8]
) -> Result<QsoLogged, String> {

    let mut reader =
        ByteReader::new(payload);

    // ペイロード先頭にはクライアントID文字列("FreeDV"等)が
    // QString形式(4byte長+可変長データ)で入っているため、
    // 実データを読む前に読み飛ばす。
    let _client_id = reader.read_qstring()?;

    Ok(QsoLogged {

        date_time_off:
            reader.read_qdatetime()?,

        dx_call:
            reader.read_qstring()?,

        dx_grid:
            reader.read_qstring()?,

        tx_frequency:
            reader.read_u64()?,

        mode:
            reader.read_qstring()?,

        report_sent:
            reader.read_qstring()?,

        report_received:
            reader.read_qstring()?,

        comments:
            reader.read_qstring()?,

        name:
            reader.read_qstring()?,

        date_time_on:
            reader.read_qdatetime()?,

        my_call:
            reader.read_qstring()?,

        my_grid:
            reader.read_qstring()?,
    })
}


#[cfg(test)]
mod qso_logged_tests {

    use super::*;


    #[test]
    fn test_qso_logged_structure() {

        let qso = QsoLogged {

            date_time_off: "20260715_081437".into(),
            dx_call: "JA4UIN".into(),
            dx_grid: "PM65".into(),
            tx_frequency: 145000000,
            mode: "DIGITALVOICE".into(),
            report_sent: "59".into(),
            report_received: "59".into(),
            comments: "".into(),
            name: "".into(),
            date_time_on: "20260715_081437".into(),
            my_call: "JA3MBC".into(),
            my_grid: "PM74qr".into(),
        };


        assert_eq!(qso.dx_call, "JA4UIN");
        assert_eq!(qso.mode, "DIGITALVOICE");
        assert_eq!(qso.report_received, "59");
        assert_eq!(qso.my_call, "JA3MBC");
        assert_eq!(qso.my_grid, "PM74qr");
    }
}


#[cfg(test)]
mod parse_qso_logged_tests {

    use super::*;


    #[test]
    fn test_parse_qso_logged_packet() {

        /*
         * WSJT-X NetworkMessage QSOLogged payload test
         *
         * 実パケット:
         * FreeDV -> UDP 2237
         *
         * 今回は parser の入口確認用
         */


        let mut data: Vec<u8> = Vec::new();


        // client id (先頭にQString "FreeDV" が入る)
        add_qstring(&mut data, "FreeDV");

        // date_time_off (QDateTime: 8byte JD + 4byte msec + 1byte timespec)
        data.extend(&2461236u64.to_be_bytes());
        data.extend(&44238239u32.to_be_bytes());
        data.push(1u8); // Qt::UTC

        // dx_call
        add_qstring(&mut data, "JA4UIN");

        // dx_grid
        add_qstring(&mut data, "PM65");


        // tx_frequency
        data.extend(&145000000u64.to_be_bytes());


        // mode
        add_qstring(&mut data, "DIGITALVOICE");

        // report sent
        add_qstring(&mut data, "59");

        // report received
        add_qstring(&mut data, "59");

        // comments
        add_qstring(&mut data, "");

        // name
        add_qstring(&mut data, "");

        // date_time_on (QDateTime)
        data.extend(&2461236u64.to_be_bytes());
        data.extend(&44238239u32.to_be_bytes());
        data.push(1u8);

        // my_call
        add_qstring(&mut data, "JA3MBC");

        // my_grid
        add_qstring(&mut data, "PM74qr");



        let qso =
            parse_qso_logged(&data)
            .expect("parse failed");


        assert_eq!(qso.dx_call, "JA4UIN");
        assert_eq!(qso.dx_grid, "PM65");
        assert_eq!(qso.mode, "DIGITALVOICE");
        assert_eq!(qso.report_sent, "59");
        assert_eq!(qso.report_received, "59");
        assert_eq!(qso.my_call, "JA3MBC");
        assert_eq!(qso.my_grid, "PM74qr");
    }



    fn add_qstring(
        buf: &mut Vec<u8>,
        s: &str
    ) {

        let bytes = s.as_bytes();

        let len = bytes.len() as u32;

        buf.extend(&len.to_be_bytes());

        buf.extend(bytes);
    }
}
