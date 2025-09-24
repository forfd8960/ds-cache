pub mod handlers;
pub mod hash;
pub mod list;
pub mod set;
pub mod string;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // Core Redis data type operations
    String(StringCommand),
    List(ListCommand),
    Set(SetCommand),
    SortedSet(SortedSetCommand),
    Hash(HashCommand),

    // Basic server operations
    Ping { message: Option<String> },
    Echo { message: String },
    Del { keys: Vec<String> },
    Exists { keys: Vec<String> },
    Keys { pattern: String },
    Type { key: String },

    // Unknown command fallback
    Unknown { command: String, args: Vec<String> },
}

// ========== String Commands ==========
#[derive(Debug, Clone, PartialEq)]
pub enum StringCommand {
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
        options: SetOptions,
    },
    GetSet {
        key: String,
        value: String,
    },
    SetNx {
        key: String,
        value: String,
    },
    SetEx {
        key: String,
        seconds: u64,
        value: String,
    },
    MGet {
        keys: Vec<String>,
    },
    MSet {
        pairs: Vec<(String, String)>,
    },
    MSetNx {
        pairs: Vec<(String, String)>,
    },
    Append {
        key: String,
        value: String,
    },
    Strlen {
        key: String,
    },
    Incr {
        key: String,
    },
    IncrBy {
        key: String,
        increment: i64,
    },
    IncrByFloat {
        key: String,
        increment: f64,
    },
    Decr {
        key: String,
    },
    DecrBy {
        key: String,
        decrement: i64,
    },
    GetRange {
        key: String,
        start: i64,
        end: i64,
    },
    SetRange {
        key: String,
        offset: u64,
        value: String,
    },
}

// ========== List Commands ==========
#[derive(Debug, Clone, PartialEq)]
pub enum ListCommand {
    LPush {
        key: String,
        values: Vec<String>,
    },
    LPushX {
        key: String,
        values: Vec<String>,
    },
    RPush {
        key: String,
        values: Vec<String>,
    },
    RPushX {
        key: String,
        values: Vec<String>,
    },
    LPop {
        key: String,
        count: Option<u64>,
    },
    RPop {
        key: String,
        count: Option<u64>,
    },
    LLen {
        key: String,
    },
    LIndex {
        key: String,
        index: i64,
    },
    LInsert {
        key: String,
        position: ListPosition,
        pivot: String,
        element: String,
    },
    LRange {
        key: String,
        start: i64,
        stop: i64,
    },
    LRem {
        key: String,
        count: i64,
        element: String,
    },
    LSet {
        key: String,
        index: i64,
        element: String,
    },
    LTrim {
        key: String,
        start: i64,
        stop: i64,
    },
    RPopLPush {
        source: String,
        destination: String,
    },
    LMove {
        source: String,
        destination: String,
        source_direction: ListDirection,
        dest_direction: ListDirection,
    },
    BLPop {
        keys: Vec<String>,
        timeout: u64,
    },
    BRPop {
        keys: Vec<String>,
        timeout: u64,
    },
    BRPopLPush {
        source: String,
        destination: String,
        timeout: u64,
    },
}

// ========== Set Commands ==========
#[derive(Debug, Clone, PartialEq)]
pub enum SetCommand {
    SAdd {
        key: String,
        members: Vec<String>,
    },
    SCard {
        key: String,
    },
    SDiff {
        keys: Vec<String>,
    },
    SDiffStore {
        destination: String,
        keys: Vec<String>,
    },
    SInter {
        keys: Vec<String>,
    },
    SInterStore {
        destination: String,
        keys: Vec<String>,
    },
    SIsMember {
        key: String,
        member: String,
    },
    SMIsMember {
        key: String,
        members: Vec<String>,
    },
    SMembers {
        key: String,
    },
    SMove {
        source: String,
        destination: String,
        member: String,
    },
    SPop {
        key: String,
        count: Option<u64>,
    },
    SRandMember {
        key: String,
        count: Option<i64>,
    },
    SRem {
        key: String,
        members: Vec<String>,
    },
    SUnion {
        keys: Vec<String>,
    },
    SUnionStore {
        destination: String,
        keys: Vec<String>,
    },
}

// ========== Sorted Set Commands ==========
#[derive(Debug, Clone, PartialEq)]
pub enum SortedSetCommand {
    ZAdd {
        key: String,
        options: ZAddOptions,
        members: Vec<(f64, String)>,
    },
    ZCard {
        key: String,
    },
    ZCount {
        key: String,
        min: ZRangeValue,
        max: ZRangeValue,
    },
    ZIncrBy {
        key: String,
        increment: f64,
        member: String,
    },
    ZInter {
        keys: Vec<String>,
        weights: Option<Vec<f64>>,
        aggregate: Option<ZAggregate>,
    },
    ZInterStore {
        destination: String,
        keys: Vec<String>,
        weights: Option<Vec<f64>>,
        aggregate: Option<ZAggregate>,
    },
    ZLexCount {
        key: String,
        min: String,
        max: String,
    },
    ZPopMax {
        key: String,
        count: Option<u64>,
    },
    ZPopMin {
        key: String,
        count: Option<u64>,
    },
    ZRange {
        key: String,
        start: i64,
        stop: i64,
        options: ZRangeOptions,
    },
    ZRangeByLex {
        key: String,
        min: String,
        max: String,
        limit: Option<(u64, u64)>,
    },
    ZRangeByScore {
        key: String,
        min: ZRangeValue,
        max: ZRangeValue,
        options: ZRangeOptions,
        limit: Option<(u64, u64)>,
    },
    ZRank {
        key: String,
        member: String,
    },
    ZRem {
        key: String,
        members: Vec<String>,
    },
    ZRemRangeByLex {
        key: String,
        min: String,
        max: String,
    },
    ZRemRangeByRank {
        key: String,
        start: i64,
        stop: i64,
    },
    ZRemRangeByScore {
        key: String,
        min: ZRangeValue,
        max: ZRangeValue,
    },
    ZRevRange {
        key: String,
        start: i64,
        stop: i64,
        with_scores: bool,
    },
    ZRevRangeByLex {
        key: String,
        max: String,
        min: String,
        limit: Option<(u64, u64)>,
    },
    ZRevRangeByScore {
        key: String,
        max: ZRangeValue,
        min: ZRangeValue,
        options: ZRangeOptions,
        limit: Option<(u64, u64)>,
    },
    ZRevRank {
        key: String,
        member: String,
    },
    ZScore {
        key: String,
        member: String,
    },
    ZUnion {
        keys: Vec<String>,
        weights: Option<Vec<f64>>,
        aggregate: Option<ZAggregate>,
    },
    ZUnionStore {
        destination: String,
        keys: Vec<String>,
        weights: Option<Vec<f64>>,
        aggregate: Option<ZAggregate>,
    },
}

// ========== Hash Commands ==========
#[derive(Debug, Clone, PartialEq)]
pub enum HashCommand {
    HDel {
        key: String,
        fields: Vec<String>,
    },
    HExists {
        key: String,
        field: String,
    },
    HGet {
        key: String,
        field: String,
    },
    HGetAll {
        key: String,
    },
    HIncrBy {
        key: String,
        field: String,
        increment: i64,
    },
    HIncrByFloat {
        key: String,
        field: String,
        increment: f64,
    },
    HKeys {
        key: String,
    },
    HLen {
        key: String,
    },
    HMGet {
        key: String,
        fields: Vec<String>,
    },
    HMSet {
        key: String,
        pairs: Vec<(String, String)>,
    },
    HSet {
        key: String,
        pairs: Vec<(String, String)>,
    },
    HSetNx {
        key: String,
        field: String,
        value: String,
    },
    HStrLen {
        key: String,
        field: String,
    },
    HVals {
        key: String,
    },
}

// ========== String Options ==========
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetOptions {
    pub expire: Option<SetExpire>,
    pub condition: Option<SetCondition>,
    pub get: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetExpire {
    Ex(u64),   // seconds
    Px(u64),   // milliseconds
    ExAt(u64), // unix timestamp seconds
    PxAt(u64), // unix timestamp milliseconds
    KeepTtl,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetCondition {
    Nx, // Only if key doesn't exist
    Xx, // Only if key exists
}

// ========== List Options ==========
#[derive(Debug, Clone, PartialEq)]
pub enum ListPosition {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListDirection {
    Left,
    Right,
}

// ========== Sorted Set Options ==========
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZAddOptions {
    pub condition: Option<ZAddCondition>,
    pub comparison: Option<ZAddComparison>,
    pub change: bool,
    pub increment: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZAddCondition {
    Nx, // Only add new elements
    Xx, // Only update existing elements
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZAddComparison {
    Gt, // Only update if new score is greater
    Lt, // Only update if new score is less
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZRangeValue {
    Score(f64),
    Inclusive(f64),
    Exclusive(f64),
    NegInf,
    PosInf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZAggregate {
    Sum,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZRangeOptions {
    pub with_scores: bool,
    pub rev: bool,
}
