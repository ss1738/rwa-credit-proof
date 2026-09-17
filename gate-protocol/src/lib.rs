//! Public wire format. No private loan records, blinding nonce or signing keys belong here.
//! Amounts use caller-agreed integer minor units. Configurations are immutable after initialization.

pub const KEY_LEN: usize = 640;
pub const PROOF_LEN: usize = 256;
pub const CONFIG_LEN: usize = 816;
pub const INIT_LEN: usize = 700;
pub const APPROVE_LEN: usize = 332;
pub const MAX_AGE_SECONDS: u64 = 86_400;
pub const MAGIC: [u8; 8] = *b"CLDNGT01";
pub const FR_MODULUS_BE: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00, 0x00, 0x01,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub servicer: [u8; 32],
    pub minimum_threshold: u128,
    pub max_age_seconds: u64,
    pub verifying_key: [u8; KEY_LEN],
}

impl Policy {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.servicer == [0; 32]
            || self.minimum_threshold == 0
            || self.max_age_seconds == 0
            || self.max_age_seconds > MAX_AGE_SECONDS
            || self.verifying_key.iter().all(|b| *b == 0)
        {
            return Err("invalid policy");
        }
        Ok(())
    }
    pub fn instruction(&self) -> Result<Vec<u8>, &'static str> {
        self.validate()?;
        let mut out = Vec::with_capacity(INIT_LEN);
        out.extend_from_slice(&[1, 0, 0, 0]);
        out.extend_from_slice(&self.servicer);
        out.extend_from_slice(&self.minimum_threshold.to_le_bytes());
        out.extend_from_slice(&self.max_age_seconds.to_le_bytes());
        out.extend_from_slice(&self.verifying_key);
        Ok(out)
    }
    pub fn decode_instruction(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() != INIT_LEN || data[..4] != [1, 0, 0, 0] {
            return Err("invalid initialization encoding");
        }
        let value = Self {
            servicer: array(&data[4..36]),
            minimum_threshold: u128::from_le_bytes(array(&data[36..52])),
            max_age_seconds: u64::from_le_bytes(array(&data[52..60])),
            verifying_key: array(&data[60..700]),
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approval {
    pub sequence: u64,
    pub observed_at: i64,
    pub expires_at: i64,
    pub threshold: u128,
    pub commitment: [u8; 32],
    pub proof: [u8; PROOF_LEN],
}
impl Approval {
    pub fn instruction(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(APPROVE_LEN);
        out.extend_from_slice(&[1, 1, 0, 0]);
        out.extend_from_slice(&self.sequence.to_le_bytes());
        out.extend_from_slice(&self.observed_at.to_le_bytes());
        out.extend_from_slice(&self.expires_at.to_le_bytes());
        out.extend_from_slice(&self.threshold.to_le_bytes());
        out.extend_from_slice(&self.commitment);
        out.extend_from_slice(&self.proof);
        out
    }
    pub fn decode_instruction(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() != APPROVE_LEN || data[..4] != [1, 1, 0, 0] {
            return Err("invalid approval encoding");
        }
        let value = Self {
            sequence: u64::from_le_bytes(array(&data[4..12])),
            observed_at: i64::from_le_bytes(array(&data[12..20])),
            expires_at: i64::from_le_bytes(array(&data[20..28])),
            threshold: u128::from_le_bytes(array(&data[28..44])),
            commitment: array(&data[44..76]),
            proof: array(&data[76..332]),
        };
        if value.commitment >= FR_MODULUS_BE {
            return Err("non-canonical commitment");
        }
        Ok(value)
    }
    /// Convert only the pinned key and the two fixed public inputs to the legacy pairing encoding.
    pub fn pairing_instruction(&self, key: &[u8; KEY_LEN]) -> Vec<u8> {
        let mut out = Vec::with_capacity(961);
        out.extend_from_slice(&self.proof);
        out.extend_from_slice(&key[..512]);
        out.push(2);
        out.extend_from_slice(&key[512..576]);
        out.extend_from_slice(&[0; 16]);
        out.extend_from_slice(&self.threshold.to_be_bytes());
        out.extend_from_slice(&key[576..640]);
        out.extend_from_slice(&self.commitment);
        out
    }
}

/// The latest accepted approval, stored alongside immutable policy. A receipt is not a token mint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub authority: [u8; 32],
    pub policy: Policy,
    pub sequence: u64,
    pub observed_at: i64,
    pub expires_at: i64,
    pub verified_slot: u64,
    pub commitment: [u8; 32],
    pub approved_threshold: u128,
}
impl Config {
    pub fn new(authority: [u8; 32], policy: Policy) -> Self {
        Self {
            authority,
            policy,
            sequence: 0,
            observed_at: 0,
            expires_at: 0,
            verified_slot: 0,
            commitment: [0; 32],
            approved_threshold: 0,
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(CONFIG_LEN);
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&self.authority);
        out.extend_from_slice(&self.policy.servicer);
        out.extend_from_slice(&self.policy.minimum_threshold.to_le_bytes());
        out.extend_from_slice(&self.policy.max_age_seconds.to_le_bytes());
        out.extend_from_slice(&self.sequence.to_le_bytes());
        out.extend_from_slice(&self.observed_at.to_le_bytes());
        out.extend_from_slice(&self.expires_at.to_le_bytes());
        out.extend_from_slice(&self.verified_slot.to_le_bytes());
        out.extend_from_slice(&self.commitment);
        out.extend_from_slice(&self.approved_threshold.to_le_bytes());
        out.extend_from_slice(&self.policy.verifying_key);
        out
    }
    pub fn decode(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() != CONFIG_LEN || data[..8] != MAGIC {
            return Err("uninitialized or incompatible account");
        }
        let state = Self {
            authority: array(&data[8..40]),
            policy: Policy {
                servicer: array(&data[40..72]),
                minimum_threshold: u128::from_le_bytes(array(&data[72..88])),
                max_age_seconds: u64::from_le_bytes(array(&data[88..96])),
                verifying_key: array(&data[176..816]),
            },
            sequence: u64::from_le_bytes(array(&data[96..104])),
            observed_at: i64::from_le_bytes(array(&data[104..112])),
            expires_at: i64::from_le_bytes(array(&data[112..120])),
            verified_slot: u64::from_le_bytes(array(&data[120..128])),
            commitment: array(&data[128..160]),
            approved_threshold: u128::from_le_bytes(array(&data[160..176])),
        };
        state.policy.validate()?;
        Ok(state)
    }
}
fn array<const N: usize>(slice: &[u8]) -> [u8; N] {
    // Only called after exact top-level length validation, at fixed documented offsets.
    let mut out = [0; N];
    out.copy_from_slice(slice);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_lengths_and_versions_never_parse() {
        for len in 0..=1200 {
            let data = vec![0; len];
            assert!(Policy::decode_instruction(&data).is_err());
            assert!(Approval::decode_instruction(&data).is_err());
            assert!(Config::decode(&data).is_err());
        }
        let approval = Approval {
            sequence: 1,
            observed_at: 1,
            expires_at: 2,
            threshold: 3,
            commitment: [0; 32],
            proof: [0; 256],
        };
        let bytes = approval.instruction();
        assert_eq!(Approval::decode_instruction(&bytes).unwrap(), approval);
        for i in 0..4 {
            let mut modified = bytes.clone();
            modified[i] = 255;
            assert!(Approval::decode_instruction(&modified).is_err());
        }
        let mut noncanonical = bytes;
        noncanonical[44..76].copy_from_slice(&FR_MODULUS_BE);
        assert!(Approval::decode_instruction(&noncanonical).is_err());
    }
    #[test]
    fn state_and_policy_round_trip() {
        let policy = Policy {
            servicer: [1; 32],
            minimum_threshold: 123,
            max_age_seconds: 300,
            verifying_key: [2; 640],
        };
        assert_eq!(
            policy,
            Policy::decode_instruction(&policy.instruction().unwrap()).unwrap()
        );
        let config = Config::new([3; 32], policy);
        assert_eq!(config.encode().len(), CONFIG_LEN);
        assert_eq!(config, Config::decode(&config.encode()).unwrap());
    }
}
