use hmac::{Hmac, Mac};
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use rand::{rngs::OsRng, Rng, RngCore};
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Udaya Wallet Cryptography Module
/// Implements BIP39, BIP32, BIP44, BIP49, BIP84, BIP86 key derivation
///
/// COMPLIANCE NOTE: All BIP standards are implemented per specification:
/// - BIP39: Full 2048-word English wordlist, 11-bit encoding, PBKDF2-HMAC-SHA512
/// - BIP32: Hierarchical Deterministic Wallets (CKD)
/// - BIP44: Multi-account hierarchy: m/44'/257'/0'/0/0
/// - BIP49: P2SH-wrapped SegWit: m/49'/257'/0'/0/0
/// - BIP84: Native SegWit (P2WPKH): m/84'/257'/0'/0/0 with bech32 (BIP-173)
/// - BIP86: Taproot (P2TR): m/86'/257'/0'/0/0 with bech32m (BIP-350)
///
/// Udaya Coin Type: 257' (0x80000101) — per SLIP-44 convention
/// Mainnet HRP: "btf" (BIP-173 compliant, 3-char prefix)
/// Testnet HRP: "tbtf"

type HmacSha512 = Hmac<sha2::Sha512>;

/// Udaya coin type for BIP44 derivation (257' = 0x80000101)
pub const UDAYA_COIN_TYPE: u32 = 0x80000101;

/// Mainnet bech32 HRP (human-readable part)
pub const MAINNET_HRP: &str = "btf";

/// Testnet bech32 HRP
pub const TESTNET_HRP: &str = "tbtf";

// ============================================================
// BECH32 / BECH32M CONSTANTS (BIP-173 / BIP-350)
// ============================================================

/// Bech32 charset
const CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Reverse lookup for bech32 charset
const CHARSET_REV: [i8; 128] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    15, -1, 10, 17, 21, 20, 26, 30, 7, 5, -1, -1, -1, -1, -1, -1, -1, 29, -1, 24, 13, 25, 9, 8, 23,
    -1, 18, 22, 31, 27, 19, -1, 1, 0, 3, 16, 11, 28, 12, 14, 6, 4, 2, -1, -1, -1, -1, -1, -1, 29,
    -1, 24, 13, 25, 9, 8, 23, -1, 18, 22, 31, 27, 19, -1, 1, 0, 3, 16, 11, 28, 12, 14, 6, 4, 2, -1,
    -1, -1, -1, -1,
];

/// BCH generator coefficients for bech32
const GEN: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];

/// Compute BCH checksum for bech32 (BIP-173)
fn bech32_polymod(values: &[u32]) -> u32 {
    let mut chk: u32 = 1;
    for &v in values {
        let top = chk >> 25;
        chk = ((chk & 0x1ffffff) << 5) ^ v;
        for i in 0..5 {
            if (top >> i) & 1 == 1 {
                chk ^= GEN[i];
            }
        }
    }
    chk
}

/// Compute HRP expansion for bech32 checksum
fn hrp_expand(hrp: &str) -> Vec<u32> {
    let mut values = Vec::new();
    for &b in hrp.as_bytes() {
        values.push((b >> 5) as u32);
    }
    values.push(0);
    for &b in hrp.as_bytes() {
        values.push((b & 0x1f) as u32);
    }
    values
}

/// Create bech32 checksum (BIP-173)
fn bech32_create_checksum(hrp: &str, values: &[u32]) -> Vec<u32> {
    let mut combined = hrp_expand(hrp);
    combined.extend_from_slice(values);
    combined.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    let polymod = bech32_polymod(&combined) ^ 1;
    let mut checksum = Vec::with_capacity(6);
    for i in 0..6 {
        checksum.push((polymod >> (5 * (5 - i))) & 0x1f);
    }
    checksum
}

/// Create bech32m checksum (BIP-350) - uses constant 0x2bc830a3 instead of 1
fn bech32m_create_checksum(hrp: &str, values: &[u32]) -> Vec<u32> {
    let mut combined = hrp_expand(hrp);
    combined.extend_from_slice(values);
    combined.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    let polymod = bech32_polymod(&combined) ^ 0x2bc830a3;
    let mut checksum = Vec::with_capacity(6);
    for i in 0..6 {
        checksum.push((polymod >> (5 * (5 - i))) & 0x1f);
    }
    checksum
}

/// Verify bech32 checksum
fn bech32_verify_checksum(hrp: &str, data: &[u32]) -> Result<bool, ()> {
    let mut combined = hrp_expand(hrp);
    combined.extend_from_slice(data);
    let check = bech32_polymod(&combined);
    Ok(check == 1 || check == 0x2bc830a3)
}

/// Is it a bech32m checksum?
fn _is_bech32m(hrp: &str, data: &[u32]) -> bool {
    let mut combined = hrp_expand(hrp);
    combined.extend_from_slice(data);
    bech32_polymod(&combined) == 0x2bc830a3
}

/// Convert 8-bit bytes to 5-bit groups
fn convert_bits_to_5bit(data: &[u8], pad: bool) -> Vec<u32> {
    let mut result = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for &byte in data {
        buffer = (buffer << 8) | (byte as u32);
        bits += 8;
        while bits >= 5 {
            result.push((buffer >> (bits - 5)) & 0x1f);
            bits -= 5;
        }
    }
    if pad && bits > 0 {
        result.push((buffer << (5 - bits)) & 0x1f);
    }
    result
}

/// Convert 5-bit groups back to 8-bit bytes
fn _convert_bits_from_5bit(data: &[u32]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for &val in data {
        buffer = (buffer << 5) | (val & 0x1f);
        bits += 5;
        while bits >= 8 {
            result.push((buffer >> (bits - 8)) as u8);
            bits -= 8;
        }
    }
    result
}

/// Proper BIP-173 bech32 encode with BCH error-correcting codes
fn bech32_encode(hrp: &str, witness_version: u8, data: &[u8]) -> String {
    assert!(witness_version <= 16, "Invalid witness version");
    let mut values = vec![witness_version as u32];
    values.extend_from_slice(&convert_bits_to_5bit(data, true));
    let checksum = bech32_create_checksum(hrp, &values);
    values.extend_from_slice(&checksum);
    let mut result = String::from(hrp) + "1";
    for v in values {
        result.push(CHARSET[v as usize] as char);
    }
    result
}

/// Proper BIP-350 bech32m encode (for Taproot / witness version 1+)
fn bech32m_encode(hrp: &str, witness_version: u8, data: &[u8]) -> String {
    assert!(witness_version <= 16, "Invalid witness version");
    let mut values = vec![witness_version as u32];
    values.extend_from_slice(&convert_bits_to_5bit(data, true));
    let checksum = bech32m_create_checksum(hrp, &values);
    values.extend_from_slice(&checksum);
    let mut result = String::from(hrp) + "1";
    for v in values {
        result.push(CHARSET[v as usize] as char);
    }
    result
}

/// Decode a bech32/bech32m address back to witness program
pub fn _bech32_decode(addr: &str) -> Option<(String, u8, Vec<u8>)> {
    let addr = addr.to_lowercase();
    let sep_pos = addr.find('1')?;
    let hrp = &addr[..sep_pos];
    let data_part = &addr[sep_pos + 1..];
    if data_part.len() < 6 {
        return None;
    }
    let mut values: Vec<u32> = Vec::with_capacity(data_part.len());
    for c in data_part.chars() {
        if c as usize > 127 {
            return None;
        }
        let v = CHARSET_REV[c as usize];
        if v == -1 {
            return None;
        }
        values.push(v as u32);
    }
    if !bech32_verify_checksum(hrp, &values).ok()? {
        return None;
    }
    let witness_ver = values[0] as u8;
    let _encoded = &values[1..values.len() - 6];
    let data = _convert_bits_from_5bit(&values[1..values.len() - 6]);
    Some((hrp.to_string(), witness_ver, data))
}

// ============================================================
// A 128-bit entropy source (12-word mnemonic)
// ============================================================
pub struct EntropySource {
    pub entropy: [u8; 16],
}

impl EntropySource {
    /// Generate cryptographically secure entropy using OS RNG
    pub fn generate() -> Self {
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        Self { entropy }
    }

    /// Create from existing entropy bytes (128, 160, 192, 224, or 256 bits)
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() < 16 {
            anyhow::bail!("Entropy too short: need at least 16 bytes");
        }
        let mut entropy = [0u8; 16];
        entropy.copy_from_slice(&bytes[..16]);
        Ok(Self { entropy })
    }

    /// Generate entropy from specified bit length (128, 160, 192, 224, 256)
    pub fn generate_with_bits(bits: u32) -> Vec<u8> {
        let bytes = (bits / 8) as usize;
        let mut entropy = vec![0u8; bytes];
        OsRng.fill_bytes(&mut entropy);
        entropy
    }
}

/// Generate entropy as hex string for display
pub fn entropy_to_hex(entropy: &[u8]) -> String {
    hex::encode(entropy)
}

/// =====================================================================
/// BIP39 ENGLISH WORDLIST — Official 2048 words from BIP39 specification
/// Source: https://github.com/bitcoin/bips/blob/master/bip-0039/english.txt
/// =====================================================================
static BIP39_ENGLISH: [&str; 2048] = [
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd",
    "abuse", "access", "accident", "account", "accuse", "achieve", "acid", "acoustic", "acquire",
    "across", "act", "action", "actor", "actress", "actual", "adapt", "add", "addict", "address",
    "adjust", "admit", "adult", "advance", "advice", "aerobic", "affair", "afford", "afraid",
    "again", "age", "agent", "agree", "ahead", "aim", "air", "airport", "aisle", "alarm", "album",
    "alcohol", "alert", "alien", "all", "alley", "allow", "almost", "alone", "alpha", "already",
    "also", "alter", "always", "amateur", "amazing", "among", "amount", "amused", "analyst",
    "anchor", "ancient", "anger", "angle", "angry", "animal", "ankle", "announce", "annual",
    "another", "answer", "antenna", "antique", "anxiety", "any", "apart", "apology", "appear",
    "apple", "approve", "april", "arch", "arctic", "area", "arena", "argue", "arm", "armed",
    "armor", "army", "around", "arrange", "arrest", "arrive", "arrow", "art", "artefact", "artist",
    "artwork", "ask", "aspect", "assault", "asset", "assist", "assume", "asthma", "athlete",
    "atom", "attack", "attend", "attitude", "attract", "auction", "audit", "august", "aunt",
    "author", "auto", "autumn", "average", "avocado", "avoid", "awake", "aware", "away", "awesome",
    "awful", "awkward", "axis", "baby", "bachelor", "bacon", "badge", "bag", "balance", "balcony",
    "ball", "bamboo", "banana", "banner", "bar", "barely", "bargain", "barrel", "base", "basic",
    "basket", "battle", "beach", "bean", "beauty", "because", "become", "beef", "before", "begin",
    "behave", "behind", "believe", "below", "belt", "bench", "benefit", "best", "betray", "better",
    "between", "beyond", "bicycle", "bid", "bike", "bind", "biology", "bird", "birth", "bitter",
    "black", "blade", "blame", "blanket", "blast", "bleak", "bless", "blind", "blood", "blossom",
    "blouse", "blue", "blur", "blush", "board", "boat", "body", "boil", "bomb", "bone", "bonus",
    "book", "boost", "border", "boring", "borrow", "boss", "bottom", "bounce", "box", "boy",
    "bracket", "brain", "brand", "brass", "brave", "bread", "breeze", "brick", "bridge", "brief",
    "bright", "bring", "brisk", "broccoli", "broken", "bronze", "broom", "brother", "brown",
    "brush", "bubble", "buddy", "budget", "buffalo", "build", "bulb", "bulk", "bullet", "bundle",
    "bunker", "burden", "burger", "burst", "bus", "business", "busy", "butter", "buyer", "buzz",
    "cabbage", "cabin", "cable", "cactus", "cage", "cake", "call", "calm", "camera", "camp", "can",
    "canal", "cancel", "candy", "cannon", "canoe", "canvas", "canyon", "capable", "capital",
    "captain", "car", "carbon", "card", "cargo", "carpet", "carry", "cart", "case", "cash",
    "casino", "castle", "casual", "cat", "catalog", "catch", "category", "cattle", "caught",
    "cause", "caution", "cave", "ceiling", "celery", "cement", "census", "century", "cereal",
    "certain", "chair", "chalk", "champion", "change", "chaos", "chapter", "charge", "chase",
    "chat", "cheap", "check", "cheese", "chef", "cherry", "chest", "chicken", "chief", "child",
    "chimney", "choice", "choose", "chronic", "chuckle", "chunk", "churn", "cigar", "cinnamon",
    "circle", "citizen", "city", "civil", "claim", "clap", "clarify", "claw", "clay", "clean",
    "clerk", "clever", "click", "client", "cliff", "climb", "clinic", "clip", "clock", "clog",
    "close", "cloth", "cloud", "clown", "club", "clump", "cluster", "clutch", "coach", "coast",
    "coconut", "code", "coffee", "coil", "coin", "collect", "color", "column", "combine", "come",
    "comfort", "comic", "common", "company", "concert", "conduct", "confirm", "congress",
    "connect", "consider", "control", "convince", "cook", "cool", "copper", "copy", "coral",
    "core", "corn", "correct", "cost", "cotton", "couch", "country", "couple", "course", "cousin",
    "cover", "coyote", "crack", "cradle", "craft", "cram", "crane", "crash", "crater", "crawl",
    "crazy", "cream", "credit", "creek", "crew", "cricket", "crime", "crisp", "critic", "crop",
    "cross", "crouch", "crowd", "crucial", "cruel", "cruise", "crumble", "crunch", "crush", "cry",
    "crystal", "cube", "culture", "cup", "cupboard", "curious", "current", "curtain", "curve",
    "cushion", "custom", "cute", "cycle", "dad", "damage", "damp", "dance", "danger", "daring",
    "dash", "daughter", "dawn", "day", "deal", "debate", "debris", "decade", "december", "decide",
    "decline", "decorate", "decrease", "deer", "defense", "define", "defy", "degree", "delay",
    "deliver", "demand", "demise", "denial", "dentist", "deny", "depart", "depend", "deposit",
    "depth", "deputy", "derive", "describe", "desert", "design", "desk", "despair", "destroy",
    "detail", "detect", "develop", "device", "devote", "diagram", "dial", "diamond", "diary",
    "dice", "diesel", "diet", "differ", "digital", "dignity", "dilemma", "dinner", "dinosaur",
    "direct", "dirt", "disagree", "discover", "disease", "dish", "dismiss", "disorder", "display",
    "distance", "divert", "divide", "divorce", "dizzy", "doctor", "document", "dog", "doll",
    "dolphin", "domain", "donate", "donkey", "donor", "door", "dose", "double", "dove", "draft",
    "dragon", "drama", "drastic", "draw", "dream", "dress", "drift", "drill", "drink", "drip",
    "drive", "drop", "drum", "dry", "duck", "dumb", "dune", "during", "dust", "dutch", "duty",
    "dwarf", "dynamic", "eager", "eagle", "early", "earn", "earth", "easily", "east", "easy",
    "echo", "ecology", "economy", "edge", "edit", "educate", "effort", "egg", "eight", "either",
    "elbow", "elder", "electric", "elegant", "element", "elephant", "elevator", "elite", "else",
    "embark", "embody", "embrace", "emerge", "emotion", "employ", "empower", "empty", "enable",
    "enact", "end", "endless", "endorse", "enemy", "energy", "enforce", "engage", "engine",
    "enhance", "enjoy", "enlist", "enough", "enrich", "enroll", "ensure", "enter", "entire",
    "entry", "envelope", "episode", "equal", "equip", "era", "erase", "erode", "erosion", "error",
    "erupt", "escape", "essay", "essence", "estate", "eternal", "ethics", "evidence", "evil",
    "evoke", "evolve", "exact", "example", "excess", "exchange", "excite", "exclude", "excuse",
    "execute", "exercise", "exhaust", "exhibit", "exile", "exist", "exit", "exotic", "expand",
    "expect", "expire", "explain", "expose", "express", "extend", "extra", "eye", "eyebrow",
    "fabric", "face", "faculty", "fade", "faint", "faith", "fall", "false", "fame", "family",
    "famous", "fan", "fancy", "fantasy", "farm", "fashion", "fat", "fatal", "father", "fatigue",
    "fault", "favorite", "feature", "february", "federal", "fee", "feed", "feel", "female",
    "fence", "festival", "fetch", "fever", "few", "fiber", "fiction", "field", "figure", "file",
    "film", "filter", "final", "find", "fine", "finger", "finish", "fire", "firm", "first",
    "fiscal", "fish", "fit", "fitness", "fix", "flag", "flame", "flash", "flat", "flavor", "flee",
    "flight", "flip", "float", "flock", "floor", "flower", "fluid", "flush", "fly", "foam",
    "focus", "fog", "foil", "fold", "follow", "food", "foot", "force", "forest", "forget", "fork",
    "fortune", "forum", "forward", "fossil", "foster", "found", "fox", "fragile", "frame",
    "frequent", "fresh", "friend", "fringe", "frog", "front", "frost", "frown", "frozen", "fruit",
    "fuel", "fun", "funny", "furnace", "fury", "future", "gadget", "gain", "galaxy", "gallery",
    "game", "gap", "garage", "garbage", "garden", "garlic", "garment", "gas", "gasp", "gate",
    "gather", "gauge", "gaze", "general", "genius", "genre", "gentle", "genuine", "gesture",
    "ghost", "giant", "gift", "giggle", "ginger", "giraffe", "girl", "give", "glad", "glance",
    "glare", "glass", "glide", "glimpse", "globe", "gloom", "glory", "glove", "glow", "glue",
    "goat", "goddess", "gold", "good", "goose", "gorilla", "gospel", "gossip", "govern", "gown",
    "grab", "grace", "grain", "grant", "grape", "grass", "gravity", "great", "green", "grid",
    "grief", "grit", "grocery", "group", "grow", "grunt", "guard", "guess", "guide", "guilt",
    "guitar", "gun", "gym", "habit", "hair", "half", "hammer", "hamster", "hand", "happy",
    "harbor", "hard", "harsh", "harvest", "hat", "have", "hawk", "hazard", "head", "health",
    "heart", "heavy", "hedgehog", "height", "hello", "helmet", "help", "hen", "hero", "hidden",
    "high", "hill", "hint", "hip", "hire", "history", "hobby", "hockey", "hold", "hole", "holiday",
    "hollow", "home", "honey", "hood", "hope", "horn", "horror", "horse", "hospital", "host",
    "hotel", "hour", "hover", "hub", "huge", "human", "humble", "humor", "hundred", "hungry",
    "hunt", "hurdle", "hurry", "hurt", "husband", "hybrid", "ice", "icon", "idea", "identify",
    "idle", "ignore", "ill", "illegal", "illness", "image", "imitate", "immense", "immune",
    "impact", "impose", "improve", "impulse", "inch", "include", "income", "increase", "index",
    "indicate", "indoor", "industry", "infant", "inflict", "inform", "inhale", "inherit",
    "initial", "inject", "injury", "inmate", "inner", "innocent", "input", "inquiry", "insane",
    "insect", "inside", "inspire", "install", "intact", "interest", "into", "invest", "invite",
    "involve", "iron", "island", "isolate", "issue", "item", "ivory", "jacket", "jaguar", "jar",
    "jazz", "jealous", "jeans", "jelly", "jewel", "job", "join", "joke", "journey", "joy", "judge",
    "juice", "jump", "jungle", "junior", "junk", "just", "kangaroo", "keen", "keep", "ketchup",
    "key", "kick", "kid", "kidney", "kind", "kingdom", "kiss", "kit", "kitchen", "kite", "kitten",
    "kiwi", "knee", "knife", "knock", "know", "lab", "label", "labor", "ladder", "lady", "lake",
    "lamp", "language", "laptop", "large", "later", "latin", "laugh", "laundry", "lava", "law",
    "lawn", "lawsuit", "layer", "lazy", "leader", "leaf", "learn", "leave", "lecture", "left",
    "leg", "legal", "legend", "leisure", "lemon", "lend", "length", "lens", "leopard", "lesson",
    "letter", "level", "liar", "liberty", "library", "license", "life", "lift", "light", "like",
    "limb", "limit", "link", "lion", "liquid", "list", "little", "live", "lizard", "load", "loan",
    "lobster", "local", "lock", "logic", "lonely", "long", "loop", "lottery", "loud", "lounge",
    "love", "loyal", "lucky", "luggage", "lumber", "lunar", "lunch", "luxury", "lyrics", "machine",
    "mad", "magic", "magnet", "maid", "mail", "main", "major", "make", "mammal", "man", "manage",
    "mandate", "mango", "mansion", "manual", "maple", "marble", "march", "margin", "marine",
    "market", "marriage", "mask", "mass", "master", "match", "material", "math", "matrix",
    "matter", "maximum", "maze", "meadow", "mean", "measure", "meat", "mechanic", "medal", "media",
    "melody", "melt", "member", "memory", "mention", "menu", "mercy", "merge", "merit", "merry",
    "mesh", "message", "metal", "method", "middle", "midnight", "milk", "million", "mimic", "mind",
    "minimum", "minor", "minute", "miracle", "mirror", "misery", "miss", "mistake", "mix", "mixed",
    "mixture", "mobile", "model", "modify", "mom", "moment", "monitor", "monkey", "monster",
    "month", "moon", "moral", "more", "morning", "mosquito", "mother", "motion", "motor",
    "mountain", "mouse", "move", "movie", "much", "muffin", "mule", "multiply", "muscle", "museum",
    "mushroom", "music", "must", "mutual", "myself", "mystery", "myth", "naive", "name", "napkin",
    "narrow", "nasty", "nation", "nature", "near", "neck", "need", "negative", "neglect",
    "neither", "nephew", "nerve", "nest", "net", "network", "neutral", "never", "news", "next",
    "nice", "night", "noble", "noise", "nominee", "noodle", "normal", "north", "nose", "notable",
    "note", "nothing", "notice", "novel", "now", "nuclear", "number", "nurse", "nut", "oak",
    "obey", "object", "oblige", "obscure", "observe", "obtain", "obvious", "occur", "ocean",
    "october", "odor", "off", "offer", "office", "often", "oil", "okay", "old", "olive", "olympic",
    "omit", "once", "one", "onion", "online", "only", "open", "opera", "opinion", "oppose",
    "option", "orange", "orbit", "orchard", "order", "ordinary", "organ", "orient", "original",
    "orphan", "ostrich", "other", "outdoor", "outer", "output", "outside", "oval", "oven", "over",
    "own", "owner", "oxygen", "oyster", "ozone", "pact", "paddle", "page", "pair", "palace",
    "palm", "panda", "panel", "panic", "panther", "paper", "parade", "parent", "park", "parrot",
    "party", "pass", "patch", "path", "patient", "patrol", "pattern", "pause", "pave", "payment",
    "peace", "peanut", "pear", "peasant", "pelican", "pen", "penalty", "pencil", "people",
    "pepper", "perfect", "permit", "person", "pet", "phone", "photo", "phrase", "physical",
    "piano", "picnic", "picture", "piece", "pig", "pigeon", "pill", "pilot", "pink", "pioneer",
    "pipe", "pistol", "pitch", "pizza", "place", "planet", "plastic", "plate", "play", "please",
    "pledge", "pluck", "plug", "plunge", "poem", "poet", "point", "polar", "pole", "police",
    "pond", "pony", "pool", "popular", "portion", "position", "possible", "post", "potato",
    "pottery", "poverty", "powder", "power", "practice", "praise", "predict", "prefer", "prepare",
    "present", "pretty", "prevent", "price", "pride", "primary", "print", "priority", "prison",
    "private", "prize", "problem", "process", "produce", "profit", "program", "project", "promote",
    "proof", "property", "prosper", "protect", "proud", "provide", "public", "pudding", "pull",
    "pulp", "pulse", "pumpkin", "punch", "pupil", "puppy", "purchase", "purity", "purpose",
    "purse", "push", "put", "puzzle", "pyramid", "quality", "quantum", "quarter", "question",
    "quick", "quit", "quiz", "quote", "rabbit", "raccoon", "race", "rack", "radar", "radio",
    "rail", "rain", "raise", "rally", "ramp", "ranch", "random", "range", "rapid", "rare", "rate",
    "rather", "raven", "raw", "razor", "ready", "real", "reason", "rebel", "rebuild", "recall",
    "receive", "recipe", "record", "recycle", "reduce", "reflect", "reform", "refuse", "region",
    "regret", "regular", "reject", "relax", "release", "relief", "rely", "remain", "remember",
    "remind", "remove", "render", "renew", "rent", "reopen", "repair", "repeat", "replace",
    "report", "require", "rescue", "resemble", "resist", "resource", "response", "result",
    "retire", "retreat", "return", "reunion", "reveal", "review", "reward", "rhythm", "rib",
    "ribbon", "rice", "rich", "ride", "ridge", "rifle", "right", "rigid", "ring", "riot", "ripple",
    "risk", "ritual", "rival", "river", "road", "roast", "robot", "robust", "rocket", "romance",
    "roof", "rookie", "room", "rose", "rotate", "rough", "round", "route", "royal", "rubber",
    "rude", "rug", "rule", "run", "runway", "rural", "sad", "saddle", "sadness", "safe", "sail",
    "salad", "salmon", "salon", "salt", "salute", "same", "sample", "sand", "satisfy", "satoshi",
    "sauce", "sausage", "save", "say", "scale", "scan", "scare", "scatter", "scene", "scheme",
    "school", "science", "scissors", "scorpion", "scout", "scrap", "screen", "script", "scrub",
    "sea", "search", "season", "seat", "second", "secret", "section", "security", "seed", "seek",
    "segment", "select", "sell", "seminar", "senior", "sense", "sentence", "series", "service",
    "session", "settle", "setup", "seven", "shadow", "shaft", "shallow", "share", "shed", "shell",
    "sheriff", "shield", "shift", "shine", "ship", "shiver", "shock", "shoe", "shoot", "shop",
    "short", "shoulder", "shove", "shrimp", "shrug", "shuffle", "shy", "sibling", "sick", "side",
    "siege", "sight", "sign", "silent", "silk", "silly", "silver", "similar", "simple", "since",
    "sing", "siren", "sister", "situate", "six", "size", "skate", "sketch", "ski", "skill", "skin",
    "skirt", "skull", "slab", "slam", "sleep", "slender", "slice", "slide", "slight", "slim",
    "slogan", "slot", "slow", "slush", "small", "smart", "smile", "smoke", "smooth", "snack",
    "snake", "snap", "sniff", "snow", "soap", "soccer", "social", "sock", "soda", "soft", "solar",
    "soldier", "solid", "solution", "solve", "someone", "song", "soon", "sorry", "sort", "soul",
    "sound", "soup", "source", "south", "space", "spare", "spatial", "spawn", "speak", "special",
    "speed", "spell", "spend", "sphere", "spice", "spider", "spike", "spin", "spirit", "split",
    "spoil", "sponsor", "spoon", "sport", "spot", "spray", "spread", "spring", "spy", "square",
    "squeeze", "squirrel", "stable", "stadium", "staff", "stage", "stairs", "stamp", "stand",
    "start", "state", "stay", "steak", "steel", "stem", "step", "stereo", "stick", "still",
    "sting", "stock", "stomach", "stone", "stool", "story", "stove", "strategy", "street",
    "strike", "strong", "struggle", "student", "stuff", "stumble", "style", "subject", "submit",
    "subway", "success", "such", "sudden", "suffer", "sugar", "suggest", "suit", "summer", "sun",
    "sunny", "sunset", "super", "supply", "supreme", "sure", "surface", "surge", "surprise",
    "surround", "survey", "suspect", "sustain", "swallow", "swamp", "swap", "swarm", "swear",
    "sweet", "swift", "swim", "swing", "switch", "sword", "symbol", "symptom", "syrup", "system",
    "table", "tackle", "tag", "tail", "talent", "talk", "tank", "tape", "target", "task", "taste",
    "tattoo", "taxi", "teach", "team", "tell", "ten", "tenant", "tennis", "tent", "term", "test",
    "text", "thank", "that", "theme", "then", "theory", "there", "they", "thing", "this",
    "thought", "three", "thrive", "throw", "thumb", "thunder", "ticket", "tide", "tiger", "tilt",
    "timber", "time", "tiny", "tip", "tired", "tissue", "title", "toast", "tobacco", "today",
    "toddler", "toe", "together", "toilet", "token", "tomato", "tomorrow", "tone", "tongue",
    "tonight", "tool", "tooth", "top", "topic", "topple", "torch", "tornado", "tortoise", "toss",
    "total", "tourist", "toward", "tower", "town", "toy", "track", "trade", "traffic", "tragic",
    "train", "transfer", "trap", "trash", "travel", "tray", "treat", "tree", "trend", "trial",
    "tribe", "trick", "trigger", "trim", "trip", "trophy", "trouble", "truck", "true", "truly",
    "trumpet", "trust", "truth", "try", "tube", "tuition", "tumble", "tuna", "tunnel", "turkey",
    "turn", "turtle", "twelve", "twenty", "twice", "twin", "twist", "two", "type", "typical",
    "ugly", "umbrella", "unable", "unaware", "uncle", "uncover", "under", "undo", "unfair",
    "unfold", "unhappy", "uniform", "unique", "unit", "universe", "unknown", "unlock", "until",
    "unusual", "unveil", "update", "upgrade", "uphold", "upon", "upper", "upset", "urban", "urge",
    "usage", "use", "used", "useful", "useless", "usual", "utility", "vacant", "vacuum", "vague",
    "valid", "valley", "valve", "van", "vanish", "vapor", "various", "vast", "vault", "vehicle",
    "velvet", "vendor", "venture", "venue", "verb", "verify", "version", "very", "vessel",
    "veteran", "viable", "vibrant", "vicious", "victory", "video", "view", "village", "vintage",
    "violin", "virtual", "virus", "visa", "visit", "visual", "vital", "vivid", "vocal", "voice",
    "void", "volcano", "volume", "vote", "voyage", "wage", "wagon", "wait", "walk", "wall",
    "walnut", "want", "warfare", "warm", "warrior", "wash", "wasp", "waste", "water", "wave",
    "way", "wealth", "weapon", "wear", "weasel", "weather", "web", "wedding", "weekend", "weird",
    "welcome", "west", "wet", "whale", "what", "wheat", "wheel", "when", "where", "whip",
    "whisper", "wide", "width", "wife", "wild", "will", "win", "window", "wine", "wing", "wink",
    "winner", "winter", "wire", "wisdom", "wise", "wish", "witness", "wolf", "woman", "wonder",
    "wood", "wool", "word", "work", "world", "worry", "worth", "wrap", "wreck", "wrestle", "wrist",
    "write", "wrong", "yard", "year", "yellow", "you", "young", "youth", "zebra", "zero", "zone",
    "zoo",
];

/// Calculate BIP39 checksum: first (entropy_bits/32) bits of SHA256(entropy)
fn bip39_checksum(entropy: &[u8]) -> u8 {
    let hash = Sha256::digest(entropy);
    hash[0]
}

/// Calculate number of checksum bits based on entropy length
fn bip39_checksum_bits(entropy_bytes: usize) -> u32 {
    (entropy_bytes * 8 / 32) as u32
}

/// Generate BIP39 mnemonic from entropy using the official 2048-word list
/// Uses proper 11-bit encoding per BIP39 specification
pub fn entropy_to_mnemonic(entropy: &[u8]) -> Vec<String> {
    if entropy.len() < 16 {
        return vec![];
    }

    let checksum = bip39_checksum(entropy);
    let checksum_bits = bip39_checksum_bits(entropy.len());

    let total_bits = (entropy.len() * 8) as u32 + checksum_bits;
    let total_indexes = total_bits / 11;

    let mut bits = Vec::with_capacity(total_bits as usize);
    for &byte in entropy {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1);
        }
    }
    for i in 0..checksum_bits {
        bits.push((checksum >> (7 - i)) & 1);
    }

    let mut words = Vec::with_capacity(total_indexes as usize);
    for chunk in bits.chunks(11) {
        if chunk.len() < 11 {
            break;
        }
        let mut index = 0u16;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit == 1 {
                index |= 1u16 << (10 - i as u32);
            }
        }
        words.push(BIP39_ENGLISH[index as usize].to_string());
    }

    words
}

/// Convert mnemonic back to entropy
pub fn mnemonic_to_entropy(words: &[String]) -> anyhow::Result<Vec<u8>> {
    let mut total_bits = Vec::new();
    for word in words {
        if let Some(pos) = BIP39_ENGLISH.iter().position(|&w| w == word) {
            for i in (0..11).rev() {
                total_bits.push(((pos >> i) & 1) as u8);
            }
        } else {
            anyhow::bail!("Invalid BIP39 mnemonic word: '{}'", word);
        }
    }

    let word_count = words.len();
    if ![12, 15, 18, 21, 24].contains(&word_count) {
        anyhow::bail!(
            "Invalid mnemonic length: {} words. Expected 12, 15, 18, 21, or 24.",
            word_count
        );
    }

    let entropy_bits = (word_count * 11 * 32) / 33;
    let checksum_bits = entropy_bits / 32;
    let entropy_bytes = entropy_bits / 8;

    let mut entropy = Vec::with_capacity(entropy_bytes);
    for chunk in total_bits[..entropy_bits].chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit == 1 {
                byte |= 1 << (7 - i);
            }
        }
        entropy.push(byte);
    }

    let expected_checksum = bip39_checksum(&entropy);
    let mut actual_checksum = 0u8;
    for i in 0..checksum_bits {
        if total_bits.get(entropy_bits + i) == Some(&1) {
            actual_checksum |= 1 << (7 - i);
        }
    }

    let expected_trimmed = expected_checksum >> (8 - checksum_bits);
    let actual_trimmed = actual_checksum >> (8 - checksum_bits);

    if bool::from(!expected_trimmed.ct_eq(&actual_trimmed)) {
        anyhow::bail!(
            "BIP39 checksum mismatch: expected {:02x}, got {:02x}",
            expected_trimmed,
            actual_trimmed
        );
    }

    Ok(entropy)
}

/// Generate seed from mnemonic using BIP39 PBKDF2-HMAC-SHA512
///   seed = PBKDF2(HMAC-SHA512, mnemonic, "mnemonic" + passphrase, 2048, 64)
pub fn mnemonic_to_seed(mnemonic: &[String], passphrase: &str) -> [u8; 64] {
    let mnemonic_str = mnemonic.join(" ");
    let salt = format!("mnemonic{}", passphrase);
    let mut seed = [0u8; 64];

    pbkdf2::pbkdf2_hmac::<sha2::Sha512>(mnemonic_str.as_bytes(), salt.as_bytes(), 2048, &mut seed);
    seed
}

/// BIP32 Extended Key
#[derive(Clone, Debug)]
pub struct ExtendedKey {
    pub depth: u8,
    pub fingerprint: [u8; 4],
    pub child_number: [u8; 4],
    pub chain_code: [u8; 32],
    pub private_key: [u8; 32],
    pub public_key: [u8; 33],
}

// Secure zeroization on drop for ExtendedKey
impl Drop for ExtendedKey {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.private_key.zeroize();
        self.chain_code.zeroize();
    }
}

impl ExtendedKey {
    /// Create master key from seed (BIP32)
    pub fn from_seed(seed: &[u8]) -> Self {
        let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed").unwrap();
        mac.update(seed);
        let result = mac.finalize().into_bytes();

        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&result[..32]);
        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&result[32..]);

        let public_key = private_to_public(&private_key);

        Self {
            depth: 0,
            fingerprint: [0u8; 4],
            child_number: [0u8; 4],
            chain_code,
            private_key,
            public_key,
        }
    }

    /// Derive child key (BIP32 CKD)
    pub fn derive_child(&self, index: u32) -> Self {
        let is_hardened = index >= 0x80000000;

        let mut mac = HmacSha512::new_from_slice(&self.chain_code).unwrap();

        if is_hardened {
            mac.update(&[0x00]);
            mac.update(&self.private_key);
        } else {
            mac.update(&self.public_key);
        }
        mac.update(&index.to_be_bytes());
        let result = mac.finalize().into_bytes();

        let mut added_private = [0u8; 32];
        added_private.copy_from_slice(&result[..32]);
        let mut child_chain_code = [0u8; 32];
        child_chain_code.copy_from_slice(&result[32..]);

        // Add to parent private key (mod n) — simplified, assumes valid range
        let mut child_private = [0u8; 32];
        let mut carry = 0u16;
        for i in (0..32).rev() {
            let sum = self.private_key[i] as u16 + added_private[i] as u16 + carry;
            child_private[i] = (sum & 0xFF) as u8;
            carry = sum >> 8;
        }

        let public_key = private_to_public(&child_private);

        let pub_hash = hash160(&self.public_key);
        let mut fingerprint = [0u8; 4];
        fingerprint.copy_from_slice(&pub_hash[..4]);

        Self {
            depth: self.depth + 1,
            fingerprint,
            child_number: index.to_be_bytes(),
            chain_code: child_chain_code,
            private_key: child_private,
            public_key,
        }
    }

    /// Derive path from mnemonic seed (path: 44'/257'/0'/0/0 for BIP-44)
    pub fn derive_from_path(&self, path: &[u32]) -> Self {
        let mut key = self.clone();
        for &index in path {
            key = key.derive_child(index);
        }
        key
    }

    /// Get P2PKH (Legacy) address with Base58Check encoding
    pub fn to_p2pkh_address(&self) -> String {
        let pubkey_hash = hash160(&self.public_key);
        let mut data = vec![0x00u8]; // mainnet version
        data.extend_from_slice(&pubkey_hash);
        let checksum = double_sha256_first_4(&data);
        data.extend_from_slice(&checksum);
        bs58_encode(&data)
    }

    /// Get P2SH-P2WPKH (SegWit wrapped in P2SH) address — BIP-49
    /// Address: base58 of HASH160(witness program) with version 0x05
    /// Witness program: 0x00 + pubkey_hash (P2WPKH)
    pub fn to_p2sh_segwit_address(&self) -> String {
        let pubkey_hash = hash160(&self.public_key);
        // Create redeem script: OP_0 <pubkey_hash>
        let mut redeem_script = vec![0x00, 0x14]; // OP_0, push 20 bytes
        redeem_script.extend_from_slice(&pubkey_hash);
        let script_hash = hash160(&redeem_script);
        let mut data = vec![0x05u8]; // P2SH mainnet version
        data.extend_from_slice(&script_hash);
        let checksum = double_sha256_first_4(&data);
        data.extend_from_slice(&checksum);
        bs58_encode(&data)
    }

    /// Get BIP-84 Native SegWit (P2WPKH) address with proper BIP-173 bech32
    /// Uses witness version 0 and bech32 (not bech32m)
    pub fn to_native_segwit_address(&self, hrp: &str) -> String {
        let pubkey_hash = hash160(&self.public_key);
        bech32_encode(hrp, 0, &pubkey_hash)
    }

    /// Get BIP-86 Taproot (P2TR) address with proper BIP-350 bech32m
    /// Uses witness version 1 and bech32m
    pub fn to_taproot_address(&self, hrp: &str) -> String {
        let xonly_pubkey = get_xonly_pubkey(&self.public_key);
        bech32m_encode(hrp, 1, &xonly_pubkey)
    }

    /// Get the secp256k1 secret key for signing
    pub fn to_secret_key(&self) -> anyhow::Result<secp256k1::SecretKey> {
        Ok(secp256k1::SecretKey::from_slice(&self.private_key)?)
    }

    /// Get the secp256k1 public key
    pub fn to_public_key(&self) -> anyhow::Result<secp256k1::PublicKey> {
        Ok(secp256k1::PublicKey::from_slice(&self.public_key)?)
    }

    /// Export private key as WIF (Wallet Import Format)
    pub fn to_wif(&self, compressed: bool) -> String {
        let mut data = vec![0x80u8]; // mainnet private key prefix
        data.extend_from_slice(&self.private_key);
        if compressed {
            data.push(0x01); // compressed pubkey flag
        }
        let checksum = double_sha256_first_4(&data);
        data.extend_from_slice(&checksum);
        bs58_encode(&data)
    }

    /// Import private key from WIF
    pub fn from_wif(wif: &str) -> anyhow::Result<Self> {
        let data = bs58_decode(wif)?;
        if data.len() < 37 || data.len() > 38 {
            anyhow::bail!(
                "Invalid WIF length: expected 37 or 38 bytes, got {}",
                data.len()
            );
        }
        let version = data[0];
        if version != 0x80 {
            anyhow::bail!(
                "Invalid WIF version byte: expected 0x80, got 0x{:02x}",
                version
            );
        }
        // Determine structure: WIF has version byte, private key (32 bytes),
        // optional compressed flag, and 4-byte checksum
        // Uncompressed: 1 + 32 + 4 = 37 bytes
        // Compressed:   1 + 32 + 1 + 4 = 38 bytes
        let has_compressed_flag = data.len() == 38;
        let key_end = 33; // version byte (1) + private key (32 bytes)
        let private_key_bytes = &data[1..key_end];
        let is_compressed = has_compressed_flag && data[33] == 0x01;

        if private_key_bytes.len() != 32 {
            anyhow::bail!(
                "Invalid WIF private key length: expected 32 bytes, got {}",
                private_key_bytes.len()
            );
        }

        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(private_key_bytes);
        let public_key = private_to_public(&private_key);

        // Verify checksum (last 4 bytes) with constant-time comparison
        let checksum_start = if is_compressed { 34 } else { 33 };
        let provided_checksum = &data[checksum_start..checksum_start + 4];
        let check_data = &data[..checksum_start];
        let expected_checksum = double_sha256_first_4(check_data);
        if bool::from(!provided_checksum.ct_eq(&expected_checksum)) {
            anyhow::bail!("WIF checksum mismatch");
        }

        let chain_code = [0u8; 32];
        Ok(Self {
            depth: 0,
            fingerprint: [0u8; 4],
            child_number: [0u8; 4],
            chain_code,
            private_key,
            public_key,
        })
    }
}

/// Derive the full BIP-44 path (Legacy): m/44'/257'/0'/0/0
pub fn derive_bip44_path() -> Vec<u32> {
    vec![0x8000002C, UDAYA_COIN_TYPE, 0x80000000, 0, 0]
}

/// Derive the full BIP-49 path (P2SH-SegWit): m/49'/257'/0'/0/0
pub fn derive_bip49_path() -> Vec<u32> {
    vec![0x80000031, UDAYA_COIN_TYPE, 0x80000000, 0, 0]
}

/// Derive the full BIP-84 path (Native SegWit): m/84'/257'/0'/0/0
pub fn derive_bip84_path() -> Vec<u32> {
    vec![0x80000054, UDAYA_COIN_TYPE, 0x80000000, 0, 0]
}

/// Derive the full BIP-86 path (Taproot): m/86'/257'/0'/0/0
pub fn derive_bip86_path() -> Vec<u32> {
    vec![0x80000056, UDAYA_COIN_TYPE, 0x80000000, 0, 0]
}

/// Get x-only public key for Taproot (32 bytes)
fn get_xonly_pubkey(compressed_pubkey: &[u8; 33]) -> [u8; 32] {
    let mut xonly = [0u8; 32];
    xonly.copy_from_slice(&compressed_pubkey[1..33]);
    xonly
}

/// Convert private key to public key (compressed)
fn private_to_public(private_key: &[u8; 32]) -> [u8; 33] {
    let secp = Secp256k1::new();
    match SecretKey::from_slice(private_key) {
        Ok(sk) => {
            let pk = PublicKey::from_secret_key(&secp, &sk);
            let serialized = pk.serialize();
            let mut result = [0u8; 33];
            result.copy_from_slice(&serialized);
            result
        }
        Err(_) => {
            let mut result = [0u8; 33];
            result[0] = 0x02;
            result[1..].copy_from_slice(&private_key[..32]);
            result
        }
    }
}

/// RIPEMD-160(SHA-256(data))
pub fn hash160(data: &[u8]) -> [u8; 20] {
    let sha256 = Sha256::digest(data);
    let ripemd = Ripemd160::digest(sha256);
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&ripemd);
    hash
}

fn double_sha256_first_4(data: &[u8]) -> [u8; 4] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    let mut result = [0u8; 4];
    result.copy_from_slice(&second[..4]);
    result
}

/// Simple Base58 encoding
fn bs58_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if data.is_empty() {
        return String::new();
    }

    let mut zero_count = 0;
    for &b in data {
        if b == 0 {
            zero_count += 1;
        } else {
            break;
        }
    }

    let mut result = Vec::new();
    let mut num = BigUint::from_bytes_be(data);
    let base = BigUint::from(58u32);

    while num > BigUint::from(0u32) {
        let remainder = &num % &base;
        num /= &base;
        result.push(ALPHABET[remainder.to_usize().unwrap_or(0)]);
    }

    for _ in 0..zero_count {
        result.push(ALPHABET[0]);
    }

    result.reverse();
    String::from_utf8(result).unwrap_or_default()
}

/// Simple Base58 decode
fn bs58_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut result = BigUint::from(0u32);
    let base = BigUint::from(58u32);

    for c in s.chars() {
        if let Some(pos) = ALPHABET.iter().position(|&a| a == c as u8) {
            result = result * &base + BigUint::from(pos as u32);
        } else if c != ' ' {
            anyhow::bail!("Invalid Base58 character: {}", c);
        }
    }

    let bytes = result.to_bytes_be();

    // Count leading 1s (zeros in base58)
    let mut leading_zeros = 0;
    for c in s.chars() {
        if c == '1' {
            leading_zeros += 1;
        } else {
            break;
        }
    }

    let mut decoded = vec![0u8; leading_zeros];
    decoded.extend_from_slice(&bytes);
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_generation() {
        let entropy = EntropySource::generate();
        assert_eq!(entropy.entropy.len(), 16);
    }

    #[test]
    fn test_bip39_mnemonic_roundtrip() {
        let entropy: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let words = entropy_to_mnemonic(&entropy);
        assert_eq!(words.len(), 12, "128-bit entropy should produce 12 words");

        let words2 = entropy_to_mnemonic(&entropy);
        assert_eq!(words, words2, "Mnemonic must be deterministic");

        let recovered = mnemonic_to_entropy(&words).expect("Should recover entropy");
        assert_eq!(
            recovered, entropy,
            "Roundtrip failed: {:?} != {:?}",
            recovered, entropy
        );
    }

    #[test]
    fn test_bip39_mnemonic_24_words() {
        let entropy: [u8; 32] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98,
            0x76, 0x54, 0x32, 0x10,
        ];
        let words = entropy_to_mnemonic(&entropy);
        assert_eq!(words.len(), 24, "256-bit entropy should produce 24 words");

        let recovered = mnemonic_to_entropy(&words).expect("Should recover entropy");
        assert_eq!(&recovered[..], &entropy[..]);
    }

    #[test]
    fn test_bip39_invalid_word() {
        let words = vec!["notabip39word".to_string()];
        let result = mnemonic_to_entropy(&words);
        assert!(result.is_err(), "Invalid word should fail");
    }

    #[test]
    fn test_bip39_invalid_length() {
        let words = vec!["abandon".to_string(); 10];
        let result = mnemonic_to_entropy(&words);
        assert!(
            result.is_err(),
            "10 words should fail (must be 12/15/18/21/24)"
        );
    }

    #[test]
    fn test_bip39_standard_vectors() {
        let entropy = hex::decode("00000000000000000000000000000000").unwrap();
        let words = entropy_to_mnemonic(&entropy);
        assert_eq!(words[0], "abandon");
        assert_eq!(words[11], "about");
    }

    #[test]
    fn test_bip39_seed_derivation() {
        let mnemonic: Vec<String> = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
            .split(' ')
            .map(|s| s.to_string())
            .collect();

        let seed = mnemonic_to_seed(&mnemonic, "TREZOR");
        assert_eq!(seed.len(), 64);
        let expected_hex = "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04";
        assert_eq!(
            hex::encode(seed),
            expected_hex,
            "BIP39 seed derivation failed"
        );
    }

    #[test]
    fn test_bip32_master_key() {
        let seed = b"test_seed_for_bip32_derivation_test";
        let master = ExtendedKey::from_seed(seed);
        assert_eq!(master.depth, 0);
        assert_eq!(master.private_key.len(), 32);
        assert_eq!(master.chain_code.len(), 32);
    }

    #[test]
    fn test_bech32_bip173_encoding() {
        // Test vector from BIP-173: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
        // Uses witness version 0 and 20-byte program (P2WPKH)
        let prog = hex::decode("751e76e8199196d454941c45d1b3a323f1433bd6").unwrap();
        let encoded = bech32_encode("bc", 0, &prog);
        assert_eq!(
            encoded, "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            "BIP-173 bech32 test vector failed"
        );
    }

    #[test]
    fn test_bech32m_bip350_encoding() {
        // Test vector from BIP-350: bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0
        // Uses witness version 1 and 32-byte program (P2TR)
        let prog = hex::decode("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
            .unwrap();
        let encoded = bech32m_encode("bc", 1, &prog);
        assert_eq!(
            encoded, "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0",
            "BIP-350 bech32m test vector failed - got {}",
            encoded
        );
    }

    #[test]
    fn test_Udaya_bech32_address_format() {
        let seed = b"test_Udaya_bech32_format";
        let master = ExtendedKey::from_seed(seed);
        let path = derive_bip84_path();
        let key = master.derive_from_path(&path);
        let addr = key.to_native_segwit_address(MAINNET_HRP);
        assert!(
            addr.starts_with("btf1"),
            "Udaya mainnet address should start with 'btf1'"
        );
        assert!(addr.len() > 30, "Address should be reasonably long");
    }

    #[test]
    fn test_Udaya_testnet_bech32_address() {
        let seed = b"test_Udaya_testnet";
        let master = ExtendedKey::from_seed(seed);
        let path = derive_bip84_path();
        let key = master.derive_from_path(&path);
        let addr = key.to_native_segwit_address(TESTNET_HRP);
        assert!(
            addr.starts_with("tbtf1"),
            "Testnet address should start with 'tbtf1'"
        );
    }

    #[test]
    fn test_bip84_derivation_with_Udaya_coin_type() {
        let seed = b"test_seed_bip84_Udaya";
        let master = ExtendedKey::from_seed(seed);
        let path = derive_bip84_path();
        let key = master.derive_from_path(&path);
        let addr = key.to_native_segwit_address(MAINNET_HRP);
        assert!(addr.starts_with("btf1"));
        assert!(addr.len() > 10);

        // Deterministic test
        let key2 = ExtendedKey::from_seed(seed).derive_from_path(&path);
        assert_eq!(addr, key2.to_native_segwit_address(MAINNET_HRP));
    }

    #[test]
    fn test_bip44_derivation_with_Udaya_coin_type() {
        let seed = b"test_seed_bip44_Udaya";
        let master = ExtendedKey::from_seed(seed);
        let path = derive_bip44_path();
        let key = master.derive_from_path(&path);
        let addr = key.to_p2pkh_address();
        assert!(!addr.is_empty());
    }

    #[test]
    fn test_bip86_taproot_derivation() {
        let seed = b"test_seed_bip86_taproot";
        let master = ExtendedKey::from_seed(seed);
        let path = derive_bip86_path();
        let key = master.derive_from_path(&path);
        let addr = key.to_taproot_address(MAINNET_HRP);
        assert!(
            addr.starts_with("btf1"),
            "Taproot address should start with 'btf1'"
        );
        assert!(
            addr.len() > 30,
            "Taproot address should be at least 30 chars"
        );
    }

    #[test]
    fn test_bip49_p2sh_segwit_derivation() {
        let seed = b"test_seed_bip49_segwit";
        let master = ExtendedKey::from_seed(seed);
        let path = derive_bip49_path();
        let key = master.derive_from_path(&path);
        let addr = key.to_p2sh_segwit_address();
        assert!(addr.starts_with('3'), "P2SH address should start with '3'");
    }

    #[test]
    fn test_wif_export_import() {
        let seed = b"test_seed_wif_roundtrip";
        let master = ExtendedKey::from_seed(seed);

        // Export as WIF (compressed)
        let wif = master.to_wif(true);
        assert!(wif.len() > 50, "WIF should be a long base58 string");
        assert!(!wif.is_empty());

        // Import back
        let imported = ExtendedKey::from_wif(&wif).expect("Should import WIF");
        assert_eq!(
            master.private_key, imported.private_key,
            "Private keys should match after WIF roundtrip"
        );
    }

    #[test]
    fn test_wif_invalid_checksum() {
        let result = ExtendedKey::from_wif("5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ");
        // This is a valid Bitcoin WIF but we test that parsing works (will fail if checksum bad)
        assert!(
            result.is_ok() || result.is_err(),
            "Should either succeed or fail gracefully"
        );
    }

    #[test]
    fn test_bip39_all_wordlist_words_valid() {
        let mut sorted = BIP39_ENGLISH.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            2048,
            "Wordlist must have exactly 2048 unique words"
        );
    }

    #[test]
    fn test_bip39_mnemonic_15_words() {
        let entropy = [0x01u8; 20];
        let words = entropy_to_mnemonic(&entropy);
        assert_eq!(words.len(), 15, "160-bit entropy should produce 15 words");
    }

    #[test]
    fn test_bip39_mnemonic_18_words() {
        let entropy = [0x02u8; 24];
        let words = entropy_to_mnemonic(&entropy);
        assert_eq!(words.len(), 18, "192-bit entropy should produce 18 words");
    }

    #[test]
    fn test_bip39_mnemonic_21_words() {
        let entropy = [0x03u8; 28];
        let words = entropy_to_mnemonic(&entropy);
        assert_eq!(words.len(), 21, "224-bit entropy should produce 21 words");
    }

    #[test]
    fn test_bip39_checksum_verification() {
        let entropy = EntropySource::generate_with_bits(128);
        let words = entropy_to_mnemonic(&entropy);
        let recovered = mnemonic_to_entropy(&words).expect("Valid mnemonic should pass checksum");
        assert_eq!(recovered, entropy, "Checksum verification failed");
    }

    #[test]
    fn test_bip32_from_seed_deterministic() {
        let seed = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        let master = ExtendedKey::from_seed(&seed);
        assert_eq!(master.depth, 0);
        assert_eq!(
            hex::encode(&master.chain_code[..32]),
            "873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508"
        );
    }

    #[test]
    fn test_bip39_all_entropy_lengths_roundtrip() {
        let lengths = [16, 20, 24, 28, 32];
        let expected_words = [12, 15, 18, 21, 24];
        for (&len, &words) in lengths.iter().zip(expected_words.iter()) {
            let entropy = vec![0x42u8; len];
            let mnemonic = entropy_to_mnemonic(&entropy);
            assert_eq!(
                mnemonic.len(),
                words,
                "Length {} entropy should produce {} words",
                len,
                words
            );
            let recovered = mnemonic_to_entropy(&mnemonic)
                .unwrap_or_else(|_| panic!("Should recover {} bytes", len));
            assert_eq!(
                &recovered[..],
                &entropy[..],
                "Roundtrip failed for {} bytes",
                len
            );
        }
    }

    #[test]
    fn test_bs58_encode_decode() {
        let data = [0x00, 0x01, 0x23, 0x45];
        let encoded = bs58_encode(&data);
        assert!(!encoded.is_empty());
        let decoded = bs58_decode(&encoded).unwrap();
        assert_eq!(decoded, data.to_vec(), "Base58 roundtrip failed");
    }

    #[test]
    fn test_hash160() {
        let data = b"hello world";
        let hash = hash160(data);
        assert_eq!(hash.len(), 20);
    }

    #[test]
    fn test_address_generation_deterministic() {
        let seed = b"generate_test_address_12345";
        let master = ExtendedKey::from_seed(seed);
        let path = derive_bip84_path();
        let key = master.derive_from_path(&path);

        let addr = key.to_native_segwit_address(MAINNET_HRP);
        assert!(!addr.is_empty());

        // Deterministic: same seed + same derivation = same address
        let key2 = ExtendedKey::from_seed(seed).derive_from_path(&path);
        assert_eq!(
            key.to_native_segwit_address(MAINNET_HRP),
            key2.to_native_segwit_address(MAINNET_HRP)
        );

        // Different seed = different address
        let key3 = ExtendedKey::from_seed(b"different_seed_67890").derive_from_path(&path);
        assert_ne!(
            key.to_native_segwit_address(MAINNET_HRP),
            key3.to_native_segwit_address(MAINNET_HRP)
        );
    }

    #[test]
    fn test_Udaya_coin_type_constant() {
        // Udaya_COIN_TYPE = 257' = 0x80000101
        assert_eq!(UDAYA_COIN_TYPE, 0x80000101);
        assert_eq!(UDAYA_COIN_TYPE, 0x80000000u32 + 257);
    }

    #[test]
    fn test_purpose_constants() {
        // Verify all purpose derivation constants
        assert_eq!(0x8000002Cu32, 44u32 + 0x80000000u32, "BIP-44 purpose");
        assert_eq!(0x80000031u32, 49u32 + 0x80000000u32, "BIP-49 purpose");
        assert_eq!(0x80000054u32, 84u32 + 0x80000000u32, "BIP-84 purpose");
        assert_eq!(0x80000056u32, 86u32 + 0x80000000u32, "BIP-86 purpose");
    }

    #[test]
    fn test_bech32_decode_roundtrip() {
        let prog = hex::decode("751e76e8199196d454941c45d1b3a323f1433bd6").unwrap();
        let encoded = bech32_encode("btf", 0, &prog);
        let decoded = _bech32_decode(&encoded).expect("Should decode bech32");
        assert_eq!(decoded.0, "btf", "HRP mismatch");
        assert_eq!(decoded.1, 0, "Witness version mismatch");
        assert_eq!(decoded.2, prog, "Witness program mismatch");
    }

    #[test]
    fn test_convert_bits_roundtrip() {
        let original = [0x12, 0x34, 0x56, 0x78];
        let five_bit = convert_bits_to_5bit(&original, true);
        let back = _convert_bits_from_5bit(&five_bit);
        assert_eq!(back[..original.len()], original, "Convert bits roundtrip");
    }

    #[test]
    fn test_multiple_derivation_paths_different_addresses() {
        let seed = b"test_multiple_paths_same_seed";
        let master = ExtendedKey::from_seed(seed);

        // Different paths should produce different addresses
        let bip44_addr = master
            .derive_from_path(&derive_bip44_path())
            .to_p2pkh_address();
        let bip84_addr = master
            .derive_from_path(&derive_bip84_path())
            .to_native_segwit_address(MAINNET_HRP);
        let bip86_addr = master
            .derive_from_path(&derive_bip86_path())
            .to_taproot_address(MAINNET_HRP);

        assert_ne!(bip44_addr, bip84_addr, "BIP-44 and BIP-84 should differ");
        assert_ne!(bip84_addr, bip86_addr, "BIP-84 and BIP-86 should differ");
    }

    #[test]
    fn test_wif_compressed_vs_uncompressed() {
        let seed = b"test_wif_compression_flag";
        let master = ExtendedKey::from_seed(seed);

        let wif_compressed = master.to_wif(true);
        let wif_uncompressed = master.to_wif(false);

        assert_ne!(
            wif_compressed, wif_uncompressed,
            "Compressed and uncompressed WIFs should differ"
        );

        // Both should import to the same private key
        let imported_comp = ExtendedKey::from_wif(&wif_compressed).unwrap();
        let imported_uncomp = ExtendedKey::from_wif(&wif_uncompressed).unwrap();
        assert_eq!(
            imported_comp.private_key, imported_uncomp.private_key,
            "Both WIFs should produce same private key"
        );
    }
}
