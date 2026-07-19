use std::io::{Read, Write};

/// RADIUS packet types
#[repr(u8)]
pub enum RadiusCode {
    AccessRequest = 1,
    AccessAccept = 2,
    AccessReject = 3,
    AccountingRequest = 4,
    AccountingResponse = 5,
    CoA = 44,
    DisconnectRequest = 46,
}

pub struct RadiusPacket {
    pub code: u8,
    pub identifier: u8,
    pub length: u16,
    pub authenticator: [u8; 16],
    pub attributes: Vec<RadiusAttribute>,
}

pub struct RadiusAttribute {
    pub attribute_type: u8,
    pub length: u8,
    pub value: Vec<u8>,
}

impl RadiusPacket {
    pub fn new(code: u8) -> Self {
        RadiusPacket {
            code,
            identifier: 0,
            length: 20, // Header only
            authenticator: [0; 16],
            attributes: Vec::new(),
        }
    }

    pub fn add_attribute(&mut self, attr_type: u8, value: &[u8]) {
        let attr = RadiusAttribute {
            attribute_type: attr_type,
            length: (value.len() + 2) as u8,
            value: value.to_vec(),
        };
        self.length += attr.length as u16;
        self.attributes.push(attr);
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.length as usize);
        
        buf.push(self.code);
        buf.push(self.identifier);
        buf.extend_from_slice(&self.length.to_be_bytes());
        buf.extend_from_slice(&self.authenticator);

        for attr in &self.attributes {
            buf.push(attr.attribute_type);
            buf.push(attr.length);
            buf.extend_from_slice(&attr.value);
        }

        buf
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 20 {
            return Err("Packet too short".to_string());
        }

        let code = data[0];
        let identifier = data[1];
        let length = u16::from_be_bytes([data[2], data[3]]) as usize;

        if data.len() < length {
            return Err("Incomplete packet".to_string());
        }

        let mut authenticator = [0; 16];
        authenticator.copy_from_slice(&data[4..20]);

        let mut attributes = Vec::new();
        let mut pos = 20;

        while pos < length {
            if pos + 2 > length {
                break;
            }

            let attr_type = data[pos];
            let attr_len = data[pos + 1] as usize;

            if attr_len < 2 || pos + attr_len > length {
                break;
            }

            let value = data[pos + 2..pos + attr_len].to_vec();
            attributes.push(RadiusAttribute {
                attribute_type: attr_type,
                length: attr_len as u8,
                value,
            });

            pos += attr_len;
        }

        Ok(RadiusPacket {
            code,
            identifier,
            length: length as u16,
            authenticator,
            attributes,
        })
    }

    pub fn get_attribute(&self, attr_type: u8) -> Option<&RadiusAttribute> {
        self.attributes.iter().find(|a| a.attribute_type == attr_type)
    }

    pub fn get_attribute_string(&self, attr_type: u8) -> Option<String> {
        self.get_attribute(attr_type)
            .and_then(|attr| String::from_utf8(attr.value.clone()).ok())
    }
}
