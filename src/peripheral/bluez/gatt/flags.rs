use crate::gatt::{
    characteristic::{self, Properties as CharacteristicProperties},
    descriptor::{self, Properties as DescriptorProperties},
};

pub trait Flags {
    fn flags(self: &Self) -> Vec<String>;
}

impl Flags for CharacteristicProperties {
    fn flags(self: &Self) -> Vec<String> {
        let mut flags = vec![];
        if let Some(ref read) = self.read {
            let read_flag = match read.0 {
                characteristic::Secure::Secure(_) => "secure-read",
                characteristic::Secure::Insecure(_) => "read",
            };
            flags.push(read_flag);
        }

        if let Some(ref write) = self.write {
            let write_flag = match write {
                characteristic::Write::WithResponse(secure) => match secure {
                    characteristic::Secure::Secure(_) => "secure-write",
                    characteristic::Secure::Insecure(_) => "write",
                },
                characteristic::Write::WithoutResponse(_) => "write-without-response",
            };
            flags.push(write_flag);
        }

        if self.notify.is_some() {
            flags.push("notify");
        }

        if self.indicate.is_some() {
            flags.push("indicate");
        }

        flags.iter().map(|s| String::from(*s)).collect()
    }
}

impl Flags for DescriptorProperties {
    fn flags(self: &Self) -> Vec<String> {
        let mut flags = vec![];
        if let Some(ref read) = self.read {
            let read_flag = match read.0 {
                descriptor::Secure::Secure(_) => "secure-read",
                descriptor::Secure::Insecure(_) => "read",
            };
            flags.push(read_flag);
        }

        if let Some(ref write) = self.write {
            let write_flag = match write.0 {
                descriptor::Secure::Secure(_) => "secure-write",
                descriptor::Secure::Insecure(_) => "write",
            };
            flags.push(write_flag);
        }

        flags.iter().map(|s| String::from(*s)).collect()
    }
}
