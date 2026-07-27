/// Script opcodes for Udaya
pub mod opcodes {
    // Constants
    pub const OP_0: u8 = 0x00;
    pub const OP_PUSHDATA1: u8 = 0x4C;
    pub const OP_PUSHDATA2: u8 = 0x4D;
    pub const OP_PUSHDATA4: u8 = 0x4E;
    pub const OP_1NEGATE: u8 = 0x4F;
    pub const OP_1: u8 = 0x51;
    pub const OP_2: u8 = 0x52;
    pub const OP_3: u8 = 0x53;
    pub const OP_4: u8 = 0x54;
    pub const OP_5: u8 = 0x55;
    pub const OP_6: u8 = 0x56;
    pub const OP_7: u8 = 0x57;
    pub const OP_8: u8 = 0x58;
    pub const OP_9: u8 = 0x59;
    pub const OP_10: u8 = 0x5A;
    pub const OP_11: u8 = 0x5B;
    pub const OP_12: u8 = 0x5C;
    pub const OP_13: u8 = 0x5D;
    pub const OP_14: u8 = 0x5E;
    pub const OP_15: u8 = 0x5F;
    pub const OP_16: u8 = 0x60;

    // Flow control
    pub const OP_NOP: u8 = 0x61;
    pub const OP_IF: u8 = 0x63;
    pub const OP_NOTIF: u8 = 0x64;
    pub const OP_ELSE: u8 = 0x67;
    pub const OP_ENDIF: u8 = 0x68;
    pub const OP_VERIFY: u8 = 0x69;
    pub const OP_RETURN: u8 = 0x6A;

    // Stack operations
    pub const OP_TOALTSTACK: u8 = 0x6B;
    pub const OP_FROMALTSTACK: u8 = 0x6C;
    pub const OP_IFDUP: u8 = 0x73;
    pub const OP_DEPTH: u8 = 0x74;
    pub const OP_DROP: u8 = 0x75;
    pub const OP_DUP: u8 = 0x76;
    pub const OP_NIP: u8 = 0x77;
    pub const OP_OVER: u8 = 0x78;
    pub const OP_PICK: u8 = 0x79;
    pub const OP_ROLL: u8 = 0x7A;
    pub const OP_ROT: u8 = 0x7B;
    pub const OP_SWAP: u8 = 0x7C;
    pub const OP_TUCK: u8 = 0x7D;

    // Arithmetic
    pub const OP_ADD: u8 = 0x93;
    pub const OP_SUB: u8 = 0x94;
    pub const OP_MUL: u8 = 0x95;
    pub const OP_DIV: u8 = 0x96;
    pub const OP_MOD: u8 = 0x97;
    pub const OP_NEGATE: u8 = 0x8F;
    pub const OP_ABS: u8 = 0x90;
    pub const OP_NOT: u8 = 0x91;
    pub const OP_0NOTEQUAL: u8 = 0x92;

    // Bitwise
    pub const OP_AND: u8 = 0x84;
    pub const OP_OR: u8 = 0x85;
    pub const OP_XOR: u8 = 0x86;
    pub const OP_EQUAL: u8 = 0x87;
    pub const OP_EQUALVERIFY: u8 = 0x88;

    // Crypto
    pub const OP_RIPEMD160: u8 = 0xA6;
    pub const OP_SHA1: u8 = 0xA7;
    pub const OP_SHA256: u8 = 0xA8;
    pub const OP_HASH160: u8 = 0xA9;
    pub const OP_HASH256: u8 = 0xAA;
    pub const OP_CODESEPARATOR: u8 = 0xAB;
    pub const OP_CHECKSIG: u8 = 0xAC;
    pub const OP_CHECKSIGVERIFY: u8 = 0xAD;
    pub const OP_CHECKMULTISIG: u8 = 0xAE;
    pub const OP_CHECKMULTISIGVERIFY: u8 = 0xAF;

    // SegWit
    pub const OP_CHECKSEQUENCEVERIFY: u8 = 0xB2;
    pub const OP_CHECKLOCKTIMEVERIFY: u8 = 0xB3;

    // Reserved words
    pub const OP_RESERVED: u8 = 0x50;
    pub const OP_VER: u8 = 0x62;
    pub const OP_VERIF: u8 = 0x65;
    pub const OP_VERNOTIF: u8 = 0x66;
}

/// Standard script templates
pub mod templates {
    use super::opcodes::*;

    /// Create a P2PKH script: OP_DUP OP_HASH160 <pubkey_hash> OP_EQUALVERIFY OP_CHECKSIG
    pub fn p2pkh(pubkey_hash: &[u8; 20]) -> Vec<u8> {
        let mut script = vec![OP_DUP, OP_HASH160, 0x14]; // 20 bytes
        script.extend_from_slice(pubkey_hash);
        script.push(OP_EQUALVERIFY);
        script.push(OP_CHECKSIG);
        script
    }

    /// Create a P2SH script: OP_HASH160 <script_hash> OP_EQUAL
    pub fn p2sh(script_hash: &[u8; 20]) -> Vec<u8> {
        let mut script = vec![OP_HASH160, 0x14];
        script.extend_from_slice(script_hash);
        script.push(OP_EQUAL);
        script
    }

    /// Create a P2PK script: <pubkey> OP_CHECKSIG
    pub fn p2pk(pubkey: &[u8]) -> Vec<u8> {
        let mut script = vec![pubkey.len() as u8];
        script.extend_from_slice(pubkey);
        script.push(OP_CHECKSIG);
        script
    }

    /// Create a null data output (OP_RETURN)
    pub fn null_data(data: &[u8]) -> Vec<u8> {
        let mut script = vec![OP_RETURN];
        if data.len() <= 220 {
            script.push(data.len() as u8);
            script.extend_from_slice(data);
        }
        script
    }

    /// Create a multisig script: OP_<m> <pub1> ... <pubn> OP_<n> OP_CHECKMULTISIG
    pub fn multisig(pubkeys: &[Vec<u8>], required: u8) -> Vec<u8> {
        let mut script = vec![0x50 + required]; // OP_1 through OP_16
        for pubkey in pubkeys {
            script.push(pubkey.len() as u8);
            script.extend_from_slice(pubkey);
        }
        script.push(0x50 + pubkeys.len() as u8);
        script.push(OP_CHECKMULTISIG);
        script
    }

    /// Parse and display script opcodes
    pub fn format_script(script: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        while i < script.len() {
            if !result.is_empty() {
                result.push(' ');
            }
            match script[i] {
                0x00 => result.push_str("OP_0"),
                0x51..=0x60 => {
                    let val = script[i] - 0x50;
                    result.push_str(&format!("OP_{}", val));
                }
                OP_NOP => result.push_str("OP_NOP"),
                OP_IF => result.push_str("OP_IF"),
                OP_NOTIF => result.push_str("OP_NOTIF"),
                OP_ELSE => result.push_str("OP_ELSE"),
                OP_ENDIF => result.push_str("OP_ENDIF"),
                OP_RETURN => result.push_str("OP_RETURN"),
                OP_DUP => result.push_str("OP_DUP"),
                OP_HASH160 => result.push_str("OP_HASH160"),
                OP_EQUALVERIFY => result.push_str("OP_EQUALVERIFY"),
                OP_CHECKSIG => result.push_str("OP_CHECKSIG"),
                OP_CHECKMULTISIG => result.push_str("OP_CHECKMULTISIG"),
                _ => {
                    if script[i] <= 0x4E {
                        // Push data
                        let data_len = if script[i] == OP_PUSHDATA1 {
                            if i + 1 < script.len() {
                                script[i + 1] as usize
                            } else {
                                0
                            }
                        } else {
                            script[i] as usize
                        };
                        let offset = if script[i] == OP_PUSHDATA1 { 2 } else { 1 };
                        result.push_str(&format!(
                            "0x{}",
                            hex::encode(&script[i + offset..i + offset + data_len.min(32)])
                        ));
                        i += offset + data_len - 1;
                    } else {
                        result.push_str(&format!("0x{:02x}", script[i]));
                    }
                }
            }
            i += 1;
        }
        result
    }
}
