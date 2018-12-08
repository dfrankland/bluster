macro_rules! impl_uuid_hash_eq {
    ($struct_with_uuid_member:ident) => {
        impl Hash for $struct_with_uuid_member {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.uuid.hash(state);
            }
        }

        impl PartialEq for $struct_with_uuid_member {
            fn eq(&self, other: &$struct_with_uuid_member) -> bool {
                self.uuid == other.uuid
            }
        }

        impl Eq for $struct_with_uuid_member {}
    };
}
