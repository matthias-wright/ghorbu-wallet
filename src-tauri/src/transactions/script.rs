use crate::transactions::error::ParseScriptTypeError;
use crate::utils::{hex, varint};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::str::FromStr;

lazy_static! {
    static ref OP_CODE_TO_WORD: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "OP_0");
        m.insert(76, "OP_PUSHDATA1");
        m.insert(77, "OP_PUSHDATA2");
        m.insert(78, "OP_PUSHDATA4");
        m.insert(79, "OP_1NEGATE");
        m.insert(81, "OP_1");
        m.insert(82, "OP_2");
        m.insert(83, "OP_3");
        m.insert(84, "OP_4");
        m.insert(85, "OP_5");
        m.insert(86, "OP_6");
        m.insert(87, "OP_7");
        m.insert(88, "OP_8");
        m.insert(89, "OP_9");
        m.insert(90, "OP_10");
        m.insert(91, "OP_11");
        m.insert(92, "OP_12");
        m.insert(93, "OP_13");
        m.insert(94, "OP_14");
        m.insert(95, "OP_15");
        m.insert(96, "OP_16");
        m.insert(97, "OP_NOP");
        m.insert(99, "OP_IF");
        m.insert(100, "OP_NOTIF");
        m.insert(103, "OP_ELSE");
        m.insert(104, "OP_ENDIF");
        m.insert(105, "OP_VERIFY");
        m.insert(106, "OP_RETURN");
        m.insert(107, "OP_TOALTSTACK");
        m.insert(108, "OP_FROMALTSTACK");
        m.insert(109, "OP_2DROP");
        m.insert(110, "OP_2DUP");
        m.insert(111, "OP_3DUP");
        m.insert(112, "OP_2OVER");
        m.insert(113, "OP_2ROT");
        m.insert(114, "OP_2SWAP");
        m.insert(115, "OP_IFDUP");
        m.insert(116, "OP_DEPTH");
        m.insert(117, "OP_DROP");
        m.insert(118, "OP_DUP");
        m.insert(119, "OP_NIP");
        m.insert(120, "OP_OVER");
        m.insert(121, "OP_PICK");
        m.insert(122, "OP_ROLL");
        m.insert(123, "OP_ROT");
        m.insert(124, "OP_SWAP");
        m.insert(125, "OP_TUCK");
        m.insert(130, "OP_SIZE");
        m.insert(135, "OP_EQUAL");
        m.insert(136, "OP_EQUALVERIFY");
        m.insert(139, "OP_1ADD");
        m.insert(140, "OP_1SUB");
        m.insert(143, "OP_NEGATE");
        m.insert(144, "OP_ABS");
        m.insert(145, "OP_NOT");
        m.insert(146, "OP_0NOTEQUAL");
        m.insert(147, "OP_ADD");
        m.insert(148, "OP_SUB");
        m.insert(154, "OP_BOOLAND");
        m.insert(155, "OP_BOOLOR");
        m.insert(156, "OP_NUMEQUAL");
        m.insert(157, "OP_NUMEQUALVERIFY");
        m.insert(158, "OP_NUMNOTEQUAL");
        m.insert(159, "OP_LESSTHAN");
        m.insert(160, "OP_GREATERTHAN");
        m.insert(161, "OP_LESSTHANOREQUAL");
        m.insert(162, "OP_GREATERTHANOREQUAL");
        m.insert(163, "OP_MIN");
        m.insert(164, "OP_MAX");
        m.insert(165, "OP_WITHIN");
        m.insert(166, "OP_RIPEMD160");
        m.insert(167, "OP_SHA1");
        m.insert(168, "OP_SHA256");
        m.insert(169, "OP_HASH160");
        m.insert(170, "OP_HASH256");
        m.insert(171, "OP_CODESEPARATOR");
        m.insert(172, "OP_CHECKSIG");
        m.insert(173, "OP_CHECKSIGVERIFY");
        m.insert(174, "OP_CHECKMULTISIG");
        m.insert(175, "OP_CHECKMULTISIGVERIFY");
        m.insert(176, "OP_NOP1");
        m.insert(177, "OP_CHECKLOCKTIMEVERIFY");
        m.insert(178, "OP_CHECKSEQUENCEVERIFY");
        m.insert(179, "OP_NOP4");
        m.insert(180, "OP_NOP5");
        m.insert(181, "OP_NOP6");
        m.insert(182, "OP_NOP7");
        m.insert(183, "OP_NOP8");
        m.insert(184, "OP_NOP9");
        m.insert(185, "OP_NOP10");
        m
    };
}

lazy_static! {
    static ref WORD_TO_OP_CODE: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        m.insert("OP_0", 0);
        m.insert("OP_PUSHDATA1", 76);
        m.insert("OP_PUSHDATA2", 77);
        m.insert("OP_PUSHDATA4", 78);
        m.insert("OP_1NEGATE", 79);
        m.insert("OP_1", 81);
        m.insert("OP_2", 82);
        m.insert("OP_3", 83);
        m.insert("OP_4", 84);
        m.insert("OP_5", 85);
        m.insert("OP_6", 86);
        m.insert("OP_7", 87);
        m.insert("OP_8", 88);
        m.insert("OP_9", 89);
        m.insert("OP_10", 90);
        m.insert("OP_11", 91);
        m.insert("OP_12", 92);
        m.insert("OP_13", 93);
        m.insert("OP_14", 94);
        m.insert("OP_15", 95);
        m.insert("OP_16", 96);
        m.insert("OP_NOP", 97);
        m.insert("OP_IF", 99);
        m.insert("OP_NOTIF", 100);
        m.insert("OP_ELSE", 103);
        m.insert("OP_ENDIF", 104);
        m.insert("OP_VERIFY", 105);
        m.insert("OP_RETURN", 106);
        m.insert("OP_TOALTSTACK", 107);
        m.insert("OP_FROMALTSTACK", 108);
        m.insert("OP_2DROP", 109);
        m.insert("OP_2DUP", 110);
        m.insert("OP_3DUP", 111);
        m.insert("OP_2OVER", 112);
        m.insert("OP_2ROT", 113);
        m.insert("OP_2SWAP", 114);
        m.insert("OP_IFDUP", 115);
        m.insert("OP_DEPTH", 116);
        m.insert("OP_DROP", 117);
        m.insert("OP_DUP", 118);
        m.insert("OP_NIP", 119);
        m.insert("OP_OVER", 120);
        m.insert("OP_PICK", 121);
        m.insert("OP_ROLL", 122);
        m.insert("OP_ROT", 123);
        m.insert("OP_SWAP", 124);
        m.insert("OP_TUCK", 125);
        m.insert("OP_SIZE", 130);
        m.insert("OP_EQUAL", 135);
        m.insert("OP_EQUALVERIFY", 136);
        m.insert("OP_1ADD", 139);
        m.insert("OP_1SUB", 140);
        m.insert("OP_NEGATE", 143);
        m.insert("OP_ABS", 144);
        m.insert("OP_NOT", 145);
        m.insert("OP_0NOTEQUAL", 146);
        m.insert("OP_ADD", 147);
        m.insert("OP_SUB", 148);
        m.insert("OP_BOOLAND", 154);
        m.insert("OP_BOOLOR", 155);
        m.insert("OP_NUMEQUAL", 156);
        m.insert("OP_NUMEQUALVERIFY", 157);
        m.insert("OP_NUMNOTEQUAL", 158);
        m.insert("OP_LESSTHAN", 159);
        m.insert("OP_GREATERTHAN", 160);
        m.insert("OP_LESSTHANOREQUAL", 161);
        m.insert("OP_GREATERTHANOREQUAL", 162);
        m.insert("OP_MIN", 163);
        m.insert("OP_MAX", 164);
        m.insert("OP_WITHIN", 165);
        m.insert("OP_RIPEMD160", 166);
        m.insert("OP_SHA1", 167);
        m.insert("OP_SHA256", 168);
        m.insert("OP_HASH160", 169);
        m.insert("OP_HASH256", 170);
        m.insert("OP_CODESEPARATOR", 171);
        m.insert("OP_CHECKSIG", 172);
        m.insert("OP_CHECKSIGVERIFY", 173);
        m.insert("OP_CHECKMULTISIG", 174);
        m.insert("OP_CHECKMULTISIGVERIFY", 175);
        m.insert("OP_NOP1", 176);
        m.insert("OP_CHECKLOCKTIMEVERIFY", 177);
        m.insert("OP_CHECKSEQUENCEVERIFY", 178);
        m.insert("OP_NOP4", 179);
        m.insert("OP_NOP5", 180);
        m.insert("OP_NOP6", 181);
        m.insert("OP_NOP7", 182);
        m.insert("OP_NOP8", 183);
        m.insert("OP_NOP9", 184);
        m.insert("OP_NOP10", 185);
        m
    };
}

pub fn serialize(s: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    for token in s.split(" ") {
        if token == "" {
            continue;
        }
        if WORD_TO_OP_CODE.contains_key(token) {
            let op_code = WORD_TO_OP_CODE.get(token).unwrap();
            bytes.extend(op_code.to_le_bytes());
        } else if token.starts_with("OP_PUSHBYTES") {
            // ignore
        } else {
            let elem_bytes = hex::hex_to_bytes(token).unwrap();
            let len = elem_bytes.len();
            if len < 75 {
                bytes.extend(&len.to_le_bytes()[..1]);
            } else if len >= 75 && len < 0x100 {
                bytes.extend(76u8.to_le_bytes());
                bytes.extend(&len.to_le_bytes()[..1]);
            } else if len >= 0x100 && len <= 520 {
                bytes.extend(77u8.to_le_bytes());
                bytes.extend(&len.to_le_bytes()[..2]);
            } else {
                return None;
            }
            bytes.extend(elem_bytes);
        }
    }
    let mut total_len = varint::encode(bytes.len() as u64);
    total_len.extend(bytes);
    Some(total_len)
}

pub fn p2pkh_script_pub_key(pubkey_hash: &str) -> String {
    format!(
        "OP_DUP OP_HASH160 OP_PUSHBYTES_20 {} OP_EQUALVERIFY OP_CHECKSIG",
        pubkey_hash
    )
}

pub fn p2pkh_script_sig(signature: &str, pubkey: &str) -> String {
    format!("{} {}", signature, pubkey)
}

pub enum ScriptType {
    P2PKH,
    //P2WPKH,
}

impl FromStr for ScriptType {
    type Err = ParseScriptTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p2pkh" => Ok(ScriptType::P2PKH),
            _ => Err(ParseScriptTypeError {}),
        }
    }
}

impl ToString for ScriptType {
    fn to_string(&self) -> String {
        match self {
            ScriptType::P2PKH => String::from("p2pkh"),
        }
    }
}
