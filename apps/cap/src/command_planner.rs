// SPEC-MANAGED: apps/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
#![allow(dead_code)]
//! Same-name command planner for `cap <cmd>`.
//!
//! The planner keeps the public command shape familiar (`cap grep`,
//! `cap ls`, ...), then chooses a native implementation for conservative
//! shell-free subsets. Unsupported forms fall back to the original command.

use std::{
    collections::VecDeque,
    env,
    ffi::{CStr, CString},
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    os::unix::{ffi::OsStrExt, fs::MetadataExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};

const SORT_NATIVE_MIN_BYTES: u64 = 1024 * 1024;
// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
const LS_NATIVE_MIN_ENTRIES: usize = 1024;
const FIND_NATIVE_MIN_ENTRIES: usize = 512;
const SED_NATIVE_MIN_BYTES: u64 = 1024 * 1024;
const SED_NATIVE_MIN_SPAN_LINES: usize = 1024;
const GREP_NATIVE_MIN_FILES: usize = 64;
const GREP_NATIVE_MIN_BYTES: u64 = 1024 * 1024;
// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
const WC_NATIVE_MIN_FILES: usize = 64;
const WC_NATIVE_MIN_BYTES: u64 = 1024 * 1024;

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPlan {
    External(ExternalPlan),
    Native(NativePlan),
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPlan {
    pub program: String,
    pub args: Vec<String>,
    pub label: Option<String>,
    pub original: String,
    pub implementation: ExternalImplementation,
    pub reason: String,
    pub fallback: Option<String>,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalImplementation {
    Original,
    Replacement,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePlan {
    pub command: NativeCommand,
    pub label: Option<String>,
    pub original: String,
    pub reason: String,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeCommand {
    True,
    False,
    PipeEmptyProducer(PipeEmptyProducerPlan),
    PipeSideEffectEmptyProducer(PipeSideEffectEmptyProducerPlan),
    PipePredicateEmptyProducer(PipePredicateEmptyProducerPlan),
    PipeWcProducer(PipeWcProducerPlan),
    PipeDuProducer(PipeDuProducerPlan),
    Pwd,
    Echo(EchoPlan),
    Printf(PrintfPlan),
    PrintfLiteral(PrintfLiteralPlan),
    Seq(SeqPlan),
    Whoami,
    Id(IdPlan),
    Uname(UnamePlan),
    Hostname,
    PipeSingleLineProducer(PipeSingleLineProducerPlan),
    PipeSingleLineGrepProducer(PipeSingleLineGrepProducerPlan),
    Test(TestPlan),
    Basename(BasenamePlan),
    Dirname(DirnamePlan),
    Ls(LsPlan),
    Sort(SortPlan),
    Uniq(UniqPlan),
    Cut(CutPlan),
    Tr(TrPlan),
    Cat(CatPlan),
    Head(HeadTailPlan),
    Tail(HeadTailPlan),
    PipeHeadProducer(PipeHeadProducerPlan),
    PipeHeadGrepProducer(PipeHeadGrepProducerPlan),
    PipeTailProducer(PipeTailProducerPlan),
    PipeTailGrepProducer(PipeTailGrepProducerPlan),
    PipeSedProducer(PipeSedProducerPlan),
    PipeSedGrepProducer(PipeSedGrepProducerPlan),
    PipeCutProducer(PipeCutProducerPlan),
    PipeCutGrepProducer(PipeCutGrepProducerPlan),
    PipeCatTrProducer(PipeCatTrProducerPlan),
    PipeCatTrGrepProducer(PipeCatTrGrepProducerPlan),
    Mkdir(MkdirPlan),
    Touch(TouchPlan),
    GrepFile(GrepFilePlan),
    AwkNeedleCount(AwkNeedleCountPlan),
    AwkFirstField(AwkFirstFieldPlan),
    XargsEcho(XargsEchoPlan),
    XargsWcLines(XargsWcLinesPlan),
    PipeXargsEchoProducer(PipeXargsEchoProducerPlan),
    PathLookup(PathLookupPlan),
    PipePathLookupProducer(PipePathLookupProducerPlan),
    Environment(EnvironmentPlan),
    PipeLsWcLines(PipeLsWcLinesPlan),
    PipeLsHead(PipeLsHeadPlan),
    PipeLsTail(PipeLsTailPlan),
    PipeLsSort(PipeLsSortPlan),
    PipeLsSortXargsEcho(PipeLsSortXargsEchoPlan),
    PipeLsSortUniq(PipeLsSortUniqPlan),
    PipeLsSortUniqWcLines(PipeLsSortUniqWcLinesPlan),
    PipeLsSortUniqProducer(PipeLsSortUniqProducerPlan),
    PipeLsSortUniqGrepProducer(PipeLsSortUniqGrepProducerPlan),
    PipeLsGrep(PipeLsGrepPlan),
    PipeLsGrepProducer(PipeLsGrepProducerPlan),
    PipeLsGrepWcLines(PipeLsGrepWcLinesPlan),
    PipeLsGrepXargsEcho(PipeLsGrepXargsEchoPlan),
    PipeLsGrepSortXargsEcho(PipeLsGrepSortXargsEchoPlan),
    PipeLsXargsEcho(PipeLsXargsEchoPlan),
    PipeCatWcLines(PipeCatWcLinesPlan),
    PipeCatHead(PipeCatHeadPlan),
    PipeCatTail(PipeCatTailPlan),
    PipeCatGrep(PipeCatGrepPlan),
    PipeCatGrepPipeline(PipeCatGrepPipelinePlan),
    PipeCatGrepSortUniqProducer(PipeCatGrepSortUniqProducerPlan),
    PipeCatCut(PipeCatCutPlan),
    PipeCatTr(PipeCatTrPlan),
    PipeCatUniq(PipeCatUniqPlan),
    PipeCatUniqWcLines(PipeCatUniqWcLinesPlan),
    PipeCatUniqProducer(PipeCatUniqProducerPlan),
    PipeCatUniqGrepProducer(PipeCatUniqGrepProducerPlan),
    PipeUniqProducer(PipeUniqProducerPlan),
    PipeUniqGrepProducer(PipeUniqGrepProducerPlan),
    PipeCatSort(PipeCatSortPlan),
    PipeCatSortUniq(PipeCatSortUniqPlan),
    PipeCatSortUniqWcLines(PipeCatSortUniqWcLinesPlan),
    PipeCatSortHead(PipeCatSortHeadPlan),
    PipeCatSortTail(PipeCatSortTailPlan),
    PipeCatSortWcLines(PipeCatSortWcLinesPlan),
    PipeCatXargsEcho(PipeCatXargsEchoPlan),
    PipeCatXargsWcLines(PipeCatXargsWcLinesPlan),
    PipeCatXargsWcProducer(PipeCatXargsWcProducerPlan),
    PipeCatGrepXargsEcho(PipeCatGrepXargsEchoPlan),
    PipeCatGrepXargsWcLines(PipeCatGrepXargsWcLinesPlan),
    PipeCatGrepSortXargsEcho(PipeCatGrepSortXargsEchoPlan),
    PipeCatGrepSortXargsWcLines(PipeCatGrepSortXargsWcLinesPlan),
    PipeCatSortXargsEcho(PipeCatSortXargsEchoPlan),
    PipeCatSortXargsWcLines(PipeCatSortXargsWcLinesPlan),
    PipeCatSortXargsWcProducer(PipeCatSortXargsWcProducerPlan),
    PipeGrepHead(PipeGrepHeadPlan),
    PipeGrepTail(PipeGrepTailPlan),
    PipeGrepSort(PipeGrepSortPlan),
    PipeGrepSortUniq(PipeGrepSortUniqPlan),
    PipeGrepSortUniqProducer(PipeGrepSortUniqProducerPlan),
    PipeGrepSortUniqWcLines(PipeGrepSortUniqWcLinesPlan),
    PipeGrepSortHead(PipeGrepSortHeadPlan),
    PipeGrepSortTail(PipeGrepSortTailPlan),
    PipeGrepSortWcLines(PipeGrepSortWcLinesPlan),
    PipeGrepWcLines(PipeGrepWcLinesPlan),
    PipeGrepFile(PipeGrepFilePlan),
    PipeGrepFileSortUniqProducer(PipeGrepFileSortUniqProducerPlan),
    PipeGrepFileCutProducer(PipeGrepFileCutProducerPlan),
    PipeGrepFileCutGrepProducer(PipeGrepFileCutGrepProducerPlan),
    PipeGrepFileAwkProducer(PipeGrepFileAwkProducerPlan),
    PipeGrepFileAwkGrepProducer(PipeGrepFileAwkGrepProducerPlan),
    PipeAwkProducer(PipeAwkProducerPlan),
    PipeAwkGrepProducer(PipeAwkGrepProducerPlan),
    PipeAwkSortUniqProducer(PipeAwkSortUniqProducerPlan),
    PipeAwkXargsEcho(PipeAwkXargsEchoPlan),
    PipeAwkXargsWcLines(PipeAwkXargsWcLinesPlan),
    PipeEchoWcLines(PipeEchoWcLinesPlan),
    PipeEchoHead(PipeEchoHeadPlan),
    PipeEchoTail(PipeEchoTailPlan),
    PipeEchoTr(PipeEchoTrPlan),
    PipeEchoAwkProducer(PipeEchoAwkProducerPlan),
    PipeEchoXargsEcho(PipeEchoXargsEchoPlan),
    PipeEchoXargsWcLines(PipeEchoXargsWcLinesPlan),
    PipePrintfWcLines(PipePrintfWcLinesPlan),
    PipePrintfHead(PipePrintfHeadPlan),
    PipePrintfTail(PipePrintfTailPlan),
    PipePrintfGrep(PipePrintfGrepPlan),
    PipePrintfTr(PipePrintfTrPlan),
    PipePrintfAwkProducer(PipePrintfAwkProducerPlan),
    PipePrintfProducer(PipePrintfProducerPlan),
    PipePrintfLiteralProducer(PipePrintfLiteralProducerPlan),
    PipePrintfGrepProducer(PipePrintfGrepProducerPlan),
    PipePrintfSortUniqProducer(PipePrintfSortUniqProducerPlan),
    PipePrintfGrepSortUniqProducer(PipePrintfGrepSortUniqProducerPlan),
    PipePrintfXargsEcho(PipePrintfXargsEchoPlan),
    PipePrintfXargsWcLines(PipePrintfXargsWcLinesPlan),
    PipeSeqWcLines(PipeSeqWcLinesPlan),
    PipeSeqHead(PipeSeqHeadPlan),
    PipeSeqTail(PipeSeqTailPlan),
    PipeSeqGrepProducer(PipeSeqGrepProducerPlan),
    PipeSeqProducer(PipeSeqProducerPlan),
    PipeSeqSortUniqProducer(PipeSeqSortUniqProducerPlan),
    PipeSeqGrepSortUniqProducer(PipeSeqGrepSortUniqProducerPlan),
    PipeSeqXargsEcho(PipeSeqXargsEchoPlan),
    PipeYesHead(PipeYesHeadPlan),
    PipePathLookupWcLines(PipePathLookupWcLinesPlan),
    PipePathLookupHead(PipePathLookupHeadPlan),
    PipePathLookupTail(PipePathLookupTailPlan),
    PipePathLookupGrepProducer(PipePathLookupGrepProducerPlan),
    PipeEnvironmentWcLines(PipeEnvironmentWcLinesPlan),
    PipeEnvironmentHead(PipeEnvironmentHeadPlan),
    PipeEnvironmentTail(PipeEnvironmentTailPlan),
    PipeEnvironmentGrep(PipeEnvironmentGrepPlan),
    PipeEnvironmentGrepProducer(PipeEnvironmentGrepProducerPlan),
    PipeEnvironmentSort(PipeEnvironmentSortPlan),
    PipeHostnameWcLines(PipeHostnameWcLinesPlan),
    PipeHostnameHead(PipeHostnameHeadPlan),
    PipeHostnameTail(PipeHostnameTailPlan),
    PipeHostnameGrep(PipeHostnameGrepPlan),
    PipeHostnameGrepProducer(PipeHostnameGrepProducerPlan),
    PipeHostnameSort(PipeHostnameSortPlan),
    PipeSortUniq(PipeSortUniqPlan),
    PipeSortUniqWcLines(PipeSortUniqWcLinesPlan),
    PipeSortUniqProducer(PipeSortUniqProducerPlan),
    PipeSortUniqGrepProducer(PipeSortUniqGrepProducerPlan),
    PipeSortGrepProducer(PipeSortGrepProducerPlan),
    PipeSortHead(PipeSortHeadPlan),
    PipeSortTail(PipeSortTailPlan),
    PipeSortWcLines(PipeSortWcLinesPlan),
    PipeSortXargsEcho(PipeSortXargsEchoPlan),
    PipeSortXargsWcLines(PipeSortXargsWcLinesPlan),
    PipeSortXargsWcProducer(PipeSortXargsWcProducerPlan),
    PipeFindXargsEcho(PipeFindXargsEchoPlan),
    PipeFindXargsWcLines(PipeFindXargsWcLinesPlan),
    PipeFindXargsWcProducer(PipeFindXargsWcProducerPlan),
    PipeFindGrepProducer(PipeFindGrepProducerPlan),
    PipeFindGrepXargsEcho(PipeFindGrepXargsEchoPlan),
    PipeFindGrepXargsWcLines(PipeFindGrepXargsWcLinesPlan),
    PipeFindGrepSortXargsEcho(PipeFindGrepSortXargsEchoPlan),
    PipeFindGrepSortXargsWcLines(PipeFindGrepSortXargsWcLinesPlan),
    PipeFindWcLines(PipeFindWcLinesPlan),
    PipeFindHead(PipeFindHeadPlan),
    PipeFindTail(PipeFindTailPlan),
    PipeFindSort(PipeFindSortPlan),
    PipeFindSortUniq(PipeFindSortUniqPlan),
    PipeFindSortUniqWcLines(PipeFindSortUniqWcLinesPlan),
    PipeFindSortUniqProducer(PipeFindSortUniqProducerPlan),
    PipeFindSortUniqGrepProducer(PipeFindSortUniqGrepProducerPlan),
    PipeFindSortXargsEcho(PipeFindSortXargsEchoPlan),
    PipeFindSortXargsWcLines(PipeFindSortXargsWcLinesPlan),
    PipeFindSortWcLines(PipeFindSortWcLinesPlan),
    PipeFindSortHead(PipeFindSortHeadPlan),
    PipeFindSortTail(PipeFindSortTailPlan),
    Find(FindPlan),
    SedPrint(SedPrintPlan),
    WcAll(WcAllPlan),
    WcLines(WcLinesPlan),
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasenamePlan {
    pub path: String,
    pub suffix: Option<String>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirnamePlan {
    pub path: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoPlan {
    pub args: Vec<String>,
    pub newline: bool,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintfFormat {
    String,
    StringNewline,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintfPlan {
    pub format: PrintfFormat,
    pub args: Vec<String>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintfLiteralPlan {
    pub bytes: Vec<u8>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqPlan {
    pub first: i64,
    pub step: i64,
    pub last: i64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdKind {
    Default,
    UserId,
    UserName,
    GroupId,
    GroupName,
    GroupIds,
    GroupNames,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdPlan {
    pub kind: IdKind,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnameField {
    Sysname,
    Nodename,
    Release,
    Version,
    Machine,
    Processor,
    All,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnamePlan {
    pub field: UnameField,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SingleLineProducerSource {
    Pwd,
    Basename(BasenamePlan),
    Dirname(DirnamePlan),
    Whoami,
    Id(IdPlan),
    Uname(UnamePlan),
    Hostname,
    PrintenvName(String),
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSingleLineProducerPlan {
    pub source: SingleLineProducerSource,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSingleLineGrepProducerPlan {
    pub source: SingleLineProducerSource,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestExpr {
    FileExists(String),
    FileRegular(String),
    FileDirectory(String),
    FileNonEmpty(String),
    StringNonEmpty(String),
    StringEmpty(String),
    StringEq(String, String),
    StringNe(String, String),
    IntEq(i64, i64),
    IntNe(i64, i64),
    IntGt(i64, i64),
    IntGe(i64, i64),
    IntLt(i64, i64),
    IntLe(i64, i64),
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestPlan {
    pub expr: TestExpr,
    pub negated: bool,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadTailMode {
    Lines,
    Bytes,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadTailPlan {
    pub file: String,
    pub stdin: bool,
    pub mode: HeadTailMode,
    pub count: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeHeadProducerPlan {
    pub file: String,
    pub stdin: bool,
    pub limit: u64,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeHeadGrepProducerPlan {
    pub file: String,
    pub stdin: bool,
    pub limit: u64,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeTailProducerPlan {
    pub file: String,
    pub stdin: bool,
    pub limit: u64,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeTailGrepProducerPlan {
    pub file: String,
    pub stdin: bool,
    pub limit: u64,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSedProducerPlan {
    pub sed: SedPrintPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSedGrepProducerPlan {
    pub sed: SedPrintPlan,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCutProducerPlan {
    pub cut: CutPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCutGrepProducerPlan {
    pub cut: CutPlan,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatTrProducerPlan {
    pub file: String,
    pub tr: TrPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatTrGrepProducerPlan {
    pub file: String,
    pub tr: TrPlan,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MkdirPlan {
    pub paths: Vec<String>,
    pub parents: bool,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchPlan {
    pub paths: Vec<String>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideEffectEmptyProducer {
    Mkdir(MkdirPlan),
    Touch(TouchPlan),
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSideEffectEmptyProducerPlan {
    pub source: SideEffectEmptyProducer,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePredicateEmptyProducerPlan {
    pub test: TestPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeWcProducerPlan {
    pub wc: WcLinesPlan,
    pub pattern: Option<String>,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuSkPlan {
    pub path: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeDuProducerPlan {
    pub du: DuSkPlan,
    pub pattern: Option<String>,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfLiteralProducerPlan {
    pub printf: PrintfLiteralPlan,
    pub pattern: Option<String>,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwkNeedleCountPlan {
    pub file: String,
    pub stdin: bool,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwkFirstFieldPlan {
    pub file: String,
    pub stdin: bool,
    pub pattern: Option<String>,
    pub field: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XargsEchoPlan {
    pub mode: XargsEchoMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XargsEchoMode {
    OneLine,
    Batch { size: usize },
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XargsWcLinesPlan;

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeXargsEchoProducerPlan {
    pub mode: GrepFilePipeMode,
    pub grep: Option<String>,
    pub source_mode: XargsEchoMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeEmptyProducerPlan {
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathLookupMode {
    Which,
    WhichAll,
    CommandV,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathLookupPlan {
    pub mode: PathLookupMode,
    pub names: Vec<String>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentMode {
    Env,
    Printenv,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentPlan {
    pub mode: EnvironmentMode,
    pub name: Option<String>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LsEntryMode {
    Visible,
    All,
    AlmostAll,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LsPipeSource {
    pub path: String,
    pub mode: LsEntryMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsWcLinesPlan {
    pub source: LsPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsHeadPlan {
    pub source: LsPipeSource,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsTailPlan {
    pub source: LsPipeSource,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsSortPlan {
    pub source: LsPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsSortXargsEchoPlan {
    pub source: LsPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsSortUniqPlan {
    pub source: LsPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsSortUniqWcLinesPlan {
    pub source: LsPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsSortUniqProducerPlan {
    pub source: LsPipeSource,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsSortUniqGrepProducerPlan {
    pub source: LsPipeSource,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsGrepPlan {
    pub source: LsPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsGrepProducerPlan {
    pub source: LsPipeSource,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsGrepWcLinesPlan {
    pub source: LsPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsGrepXargsEchoPlan {
    pub source: LsPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsGrepSortXargsEchoPlan {
    pub source: LsPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeLsXargsEchoPlan {
    pub source: LsPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatWcLinesPlan {
    pub file: String,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatHeadPlan {
    pub file: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatTailPlan {
    pub file: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatGrepPlan {
    pub file: String,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatGrepPipelinePlan {
    pub file: String,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatGrepSortUniqProducerPlan {
    pub file: String,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatGrepXargsEchoPlan {
    pub file: String,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatGrepXargsWcLinesPlan {
    pub file: String,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatGrepSortXargsEchoPlan {
    pub file: String,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatGrepSortXargsWcLinesPlan {
    pub file: String,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatCutPlan {
    pub file: String,
    pub delimiter: u8,
    pub field: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatTrPlan {
    pub file: String,
    pub tr: TrPlan,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatUniqPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatUniqWcLinesPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatUniqProducerPlan {
    pub file: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatUniqGrepProducerPlan {
    pub file: String,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeUniqProducerPlan {
    pub file: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeUniqGrepProducerPlan {
    pub file: String,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortUniqPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortUniqWcLinesPlan {
    pub file: String,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortHeadPlan {
    pub file: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortTailPlan {
    pub file: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortWcLinesPlan {
    pub file: String,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatXargsEchoPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatXargsWcLinesPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatXargsWcProducerPlan {
    pub file: String,
    pub mode: XargsWcOutputMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortXargsEchoPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortXargsWcLinesPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeCatSortXargsWcProducerPlan {
    pub file: String,
    pub mode: XargsWcOutputMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepHeadPlan {
    pub pattern: String,
    pub root: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepTailPlan {
    pub pattern: String,
    pub root: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepSortPlan {
    pub pattern: String,
    pub root: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepSortUniqPlan {
    pub pattern: String,
    pub root: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepSortUniqProducerPlan {
    pub pattern: String,
    pub root: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepSortUniqWcLinesPlan {
    pub pattern: String,
    pub root: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepSortHeadPlan {
    pub pattern: String,
    pub root: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepSortTailPlan {
    pub pattern: String,
    pub root: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepSortWcLinesPlan {
    pub pattern: String,
    pub root: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepWcLinesPlan {
    pub pattern: String,
    pub root: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepFilePlan {
    pub pattern: String,
    pub file: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepFileSortUniqProducerPlan {
    pub pattern: String,
    pub file: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutFilterPlan {
    pub delimiter: u8,
    pub field: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepFileCutProducerPlan {
    pub pattern: String,
    pub file: String,
    pub cut: CutFilterPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepFileCutGrepProducerPlan {
    pub pattern: String,
    pub file: String,
    pub cut: CutFilterPlan,
    pub downstream_pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepFileAwkProducerPlan {
    pub pattern: String,
    pub file: String,
    pub awk_pattern: Option<String>,
    pub awk_field: usize,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeGrepFileAwkGrepProducerPlan {
    pub pattern: String,
    pub file: String,
    pub awk_pattern: Option<String>,
    pub awk_field: usize,
    pub downstream_pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WcCountMode {
    Lines,
    Bytes,
    Words,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
impl WcCountMode {
    fn from_flag(flag: &str) -> Option<Self> {
        match flag {
            "-l" => Some(Self::Lines),
            "-c" => Some(Self::Bytes),
            "-w" => Some(Self::Words),
            _ => None,
        }
    }

    fn flag(self) -> &'static str {
        match self {
            Self::Lines => "-l",
            Self::Bytes => "-c",
            Self::Words => "-w",
        }
    }
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrepFilePipeMode {
    Lines,
    WcLines,
    Wc { mode: WcCountMode },
    Head { limit: u64 },
    Tail { limit: u64 },
    Sort,
    SortUniq,
    SortUniqWcLines,
    SortUniqWc { mode: WcCountMode },
    SortWcLines,
    SortWc { mode: WcCountMode },
    SortHead { limit: u64 },
    SortTail { limit: u64 },
    SortXargsEcho,
    SortXargsEchoBatches { size: usize },
    SortXargsWcLines,
    SortXargsWcOutput { mode: XargsWcOutputMode },
    XargsEcho,
    XargsEchoBatches { size: usize },
    XargsWcLines,
    XargsWcOutput { mode: XargsWcOutputMode },
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeAwkXargsEchoPlan {
    pub file: String,
    pub pattern: Option<String>,
    pub field: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeAwkXargsWcLinesPlan {
    pub file: String,
    pub pattern: Option<String>,
    pub field: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeAwkProducerPlan {
    pub file: String,
    pub pattern: Option<String>,
    pub field: usize,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeAwkGrepProducerPlan {
    pub file: String,
    pub pattern: Option<String>,
    pub field: usize,
    pub downstream_pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeAwkSortUniqProducerPlan {
    pub file: String,
    pub pattern: Option<String>,
    pub field: usize,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEchoWcLinesPlan {
    pub echo: EchoPlan,
    pub newline: bool,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEchoHeadPlan {
    pub echo: EchoPlan,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEchoTailPlan {
    pub echo: EchoPlan,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEchoTrPlan {
    pub echo: EchoPlan,
    pub tr: TrPlan,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEchoAwkProducerPlan {
    pub echo: EchoPlan,
    pub pattern: Option<String>,
    pub field: usize,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEchoXargsEchoPlan {
    pub echo: EchoPlan,
    pub mode: XargsEchoMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEchoXargsWcLinesPlan {
    pub echo: EchoPlan,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfWcLinesPlan {
    pub args: Vec<String>,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfHeadPlan {
    pub args: Vec<String>,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfTailPlan {
    pub args: Vec<String>,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfGrepPlan {
    pub args: Vec<String>,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfTrPlan {
    pub printf: PrintfPlan,
    pub tr: TrPlan,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfAwkProducerPlan {
    pub args: Vec<String>,
    pub pattern: Option<String>,
    pub field: usize,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfProducerPlan {
    pub args: Vec<String>,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfGrepProducerPlan {
    pub args: Vec<String>,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfSortUniqProducerPlan {
    pub args: Vec<String>,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfGrepSortUniqProducerPlan {
    pub args: Vec<String>,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfXargsEchoPlan {
    pub args: Vec<String>,
    pub mode: XargsEchoMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePrintfXargsWcLinesPlan {
    pub args: Vec<String>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeSeqWcLinesPlan {
    pub seq: SeqPlan,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeSeqHeadPlan {
    pub seq: SeqPlan,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSeqTailPlan {
    pub seq: SeqPlan,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSeqGrepProducerPlan {
    pub seq: SeqPlan,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSeqProducerPlan {
    pub seq: SeqPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeSeqSortUniqProducerPlan {
    pub seq: SeqPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSeqGrepSortUniqProducerPlan {
    pub seq: SeqPlan,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeSeqXargsEchoPlan {
    pub seq: SeqPlan,
    pub mode: XargsEchoMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeYesHeadPlan {
    pub value: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePathLookupWcLinesPlan {
    pub lookup: PathLookupPlan,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePathLookupHeadPlan {
    pub lookup: PathLookupPlan,
    pub limit: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePathLookupTailPlan {
    pub lookup: PathLookupPlan,
    pub limit: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePathLookupGrepProducerPlan {
    pub lookup: PathLookupPlan,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipePathLookupProducerPlan {
    pub lookup: PathLookupPlan,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEnvironmentWcLinesPlan {
    pub env: EnvironmentPlan,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEnvironmentHeadPlan {
    pub env: EnvironmentPlan,
    pub limit: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEnvironmentTailPlan {
    pub env: EnvironmentPlan,
    pub limit: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEnvironmentGrepPlan {
    pub env: EnvironmentPlan,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEnvironmentGrepProducerPlan {
    pub env: EnvironmentPlan,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeEnvironmentSortPlan {
    pub env: EnvironmentPlan,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeHostnameWcLinesPlan;

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeHostnameHeadPlan {
    pub limit: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeHostnameTailPlan {
    pub limit: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeHostnameGrepPlan {
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeHostnameGrepProducerPlan {
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeHostnameSortPlan;

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortUniqPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortUniqWcLinesPlan {
    pub file: String,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortUniqProducerPlan {
    pub file: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortUniqGrepProducerPlan {
    pub file: String,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortGrepProducerPlan {
    pub file: String,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortHeadPlan {
    pub file: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortTailPlan {
    pub file: String,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortWcLinesPlan {
    pub file: String,
    pub mode: WcCountMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortXargsEchoPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortXargsWcLinesPlan {
    pub file: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeSortXargsWcProducerPlan {
    pub file: String,
    pub mode: XargsWcOutputMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindPipeSource {
    pub root: String,
    pub name_glob: String,
    pub max_depth: Option<usize>,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindXargsEchoPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindXargsWcLinesPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XargsWcOutputMode {
    WcLines,
    Head { limit: u64 },
    Tail { limit: u64 },
    Sort,
    SortUniq,
    SortUniqWcLines,
    SortWcLines,
    SortHead { limit: u64 },
    SortTail { limit: u64 },
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindXargsWcProducerPlan {
    pub source: FindPipeSource,
    pub pattern: Option<String>,
    pub sort_paths: bool,
    pub uniq_paths: bool,
    pub mode: XargsWcOutputMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindGrepProducerPlan {
    pub source: FindPipeSource,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindGrepXargsEchoPlan {
    pub source: FindPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindGrepXargsWcLinesPlan {
    pub source: FindPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindGrepSortXargsEchoPlan {
    pub source: FindPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindGrepSortXargsWcLinesPlan {
    pub source: FindPipeSource,
    pub pattern: String,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindWcLinesPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindHeadPlan {
    pub source: FindPipeSource,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindTailPlan {
    pub source: FindPipeSource,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortUniqPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortUniqWcLinesPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortUniqProducerPlan {
    pub source: FindPipeSource,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortUniqGrepProducerPlan {
    pub source: FindPipeSource,
    pub pattern: String,
    pub mode: GrepFilePipeMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortXargsEchoPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortXargsWcLinesPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortWcLinesPlan {
    pub source: FindPipeSource,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortHeadPlan {
    pub source: FindPipeSource,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeFindSortTailPlan {
    pub source: FindPipeSource,
    pub limit: u64,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LsPlan {
    pub path: String,
    pub mode: LsEntryMode,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortPlan {
    pub file: String,
    pub stdin: bool,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqPlan {
    pub file: String,
    pub stdin: bool,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutPlan {
    pub file: String,
    pub stdin: bool,
    pub delimiter: u8,
    pub field: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrPlan {
    pub mode: TrMode,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrMode {
    Translate { from: Vec<u8>, to: Vec<u8> },
    Delete { set: Vec<u8> },
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatPlan {
    pub files: Vec<String>,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindPlan {
    pub root: String,
    pub type_filter: Option<FindType>,
    pub name_pattern: Option<String>,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindType {
    File,
    Dir,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SedPrintPlan {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrepFilePlan {
    pub pattern: String,
    pub file: String,
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WcLinesPlan {
    pub files: Vec<String>,
    pub mode: WcCountMode,
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WcAllPlan {
    pub files: Vec<String>,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
impl CommandPlan {
    pub fn explain(&self) -> String {
        match self {
            CommandPlan::External(plan) => {
                let implementation = match plan.implementation {
                    ExternalImplementation::Original => "original",
                    ExternalImplementation::Replacement => "replacement",
                };
                let mut lines = vec![
                    format!("original: {}", plan.original),
                    format!("implementation: {implementation}"),
                    format!("run: {}", render_command(&plan.program, &plan.args)),
                    format!("reason: {}", plan.reason),
                ];
                if let Some(fallback) = &plan.fallback {
                    lines.push(format!("fallback: {fallback}"));
                }
                lines.join("\n")
            }
            CommandPlan::Native(plan) => {
                let native = match &plan.command {
                    NativeCommand::True => "cap-native true",
                    NativeCommand::False => "cap-native false",
                    NativeCommand::PipeEmptyProducer(_) => "cap-native pipe empty producer",
                    NativeCommand::PipeSideEffectEmptyProducer(_) => {
                        "cap-native pipe side-effect empty producer"
                    }
                    NativeCommand::PipePredicateEmptyProducer(_) => {
                        "cap-native pipe predicate empty producer"
                    }
                    NativeCommand::PipeWcProducer(_) => "cap-native pipe wc producer",
                    NativeCommand::PipeDuProducer(_) => "cap-native pipe du producer",
                    NativeCommand::Pwd => "cap-native pwd",
                    NativeCommand::Echo(_) => "cap-native echo",
                    NativeCommand::Printf(_) => "cap-native printf",
                    NativeCommand::PrintfLiteral(_) => "cap-native printf literal",
                    NativeCommand::Seq(_) => "cap-native seq",
                    NativeCommand::Whoami => "cap-native whoami",
                    NativeCommand::Id(_) => "cap-native id",
                    NativeCommand::Uname(_) => "cap-native uname",
                    NativeCommand::Hostname => "cap-native hostname",
                    NativeCommand::PipeSingleLineProducer(_) => {
                        "cap-native pipe single-line producer"
                    }
                    NativeCommand::PipeSingleLineGrepProducer(_) => {
                        "cap-native pipe single-line grep producer"
                    }
                    NativeCommand::Test(_) => "cap-native test",
                    NativeCommand::Basename(_) => "cap-native basename",
                    NativeCommand::Dirname(_) => "cap-native dirname",
                    NativeCommand::Ls(_) => "cap-native ls",
                    NativeCommand::Sort(_) => "cap-native sort",
                    NativeCommand::Uniq(_) => "cap-native uniq",
                    NativeCommand::Cut(_) => "cap-native cut",
                    NativeCommand::Tr(_) => "cap-native tr",
                    NativeCommand::Cat(_) => "cap-native cat",
                    NativeCommand::Head(_) => "cap-native head",
                    NativeCommand::Tail(_) => "cap-native tail",
                    NativeCommand::PipeHeadProducer(_) => "cap-native pipe head producer",
                    NativeCommand::PipeHeadGrepProducer(_) => "cap-native pipe head grep producer",
                    NativeCommand::PipeTailProducer(_) => "cap-native pipe tail producer",
                    NativeCommand::PipeTailGrepProducer(_) => "cap-native pipe tail grep producer",
                    NativeCommand::PipeSedProducer(_) => "cap-native pipe sed producer",
                    NativeCommand::PipeSedGrepProducer(_) => "cap-native pipe sed grep producer",
                    NativeCommand::PipeCutProducer(_) => "cap-native pipe cut producer",
                    NativeCommand::PipeCutGrepProducer(_) => "cap-native pipe cut grep producer",
                    NativeCommand::PipeCatTrProducer(_) => "cap-native pipe cat|tr producer",
                    NativeCommand::PipeCatTrGrepProducer(_) => {
                        "cap-native pipe cat|tr grep producer"
                    }
                    NativeCommand::Mkdir(_) => "cap-native mkdir",
                    NativeCommand::Touch(_) => "cap-native touch",
                    NativeCommand::GrepFile(_) => "cap-native grep file",
                    NativeCommand::AwkNeedleCount(_) => "cap-native awk count",
                    NativeCommand::AwkFirstField(_) => "cap-native awk first-field",
                    NativeCommand::XargsEcho(_) => "cap-native xargs echo",
                    NativeCommand::XargsWcLines(_) => "cap-native xargs wc -l",
                    NativeCommand::PipeXargsEchoProducer(_) => {
                        "cap-native pipe xargs echo producer"
                    }
                    NativeCommand::PathLookup(path) => match path.mode {
                        PathLookupMode::Which => "cap-native which",
                        PathLookupMode::WhichAll => "cap-native which -a",
                        PathLookupMode::CommandV => "cap-native command -v",
                    },
                    NativeCommand::Environment(environment) => match environment.mode {
                        EnvironmentMode::Env => "cap-native env",
                        EnvironmentMode::Printenv => "cap-native printenv",
                    },
                    NativeCommand::PipeLsWcLines(_) => "cap-native pipe ls|wc",
                    NativeCommand::PipeLsHead(_) => "cap-native pipe ls|head",
                    NativeCommand::PipeLsTail(_) => "cap-native pipe ls|tail",
                    NativeCommand::PipeLsSort(_) => "cap-native pipe ls|sort",
                    NativeCommand::PipeLsSortXargsEcho(_) => "cap-native pipe ls|sort|xargs echo",
                    NativeCommand::PipeLsSortUniq(_) => "cap-native pipe ls|sort|uniq",
                    NativeCommand::PipeLsSortUniqWcLines(_) => "cap-native pipe ls|sort|uniq|wc",
                    NativeCommand::PipeLsSortUniqProducer(_) => {
                        "cap-native pipe ls|sort|uniq producer"
                    }
                    NativeCommand::PipeLsSortUniqGrepProducer(_) => {
                        "cap-native pipe ls|sort|uniq grep producer"
                    }
                    NativeCommand::PipeLsGrep(_) => "cap-native pipe ls|grep",
                    NativeCommand::PipeLsGrepProducer(_) => "cap-native pipe ls|grep producer",
                    NativeCommand::PipeLsGrepWcLines(_) => "cap-native pipe ls|grep|wc",
                    NativeCommand::PipeLsGrepXargsEcho(_) => "cap-native pipe ls|grep|xargs echo",
                    NativeCommand::PipeLsGrepSortXargsEcho(_) => {
                        "cap-native pipe ls|grep|sort|xargs echo"
                    }
                    NativeCommand::PipeLsXargsEcho(_) => "cap-native pipe ls|xargs echo",
                    NativeCommand::PipeCatWcLines(_) => "cap-native pipe cat|wc",
                    NativeCommand::PipeCatHead(_) => "cap-native pipe cat|head",
                    NativeCommand::PipeCatTail(_) => "cap-native pipe cat|tail",
                    NativeCommand::PipeCatGrep(_) => "cap-native pipe cat|grep",
                    NativeCommand::PipeCatGrepPipeline(_) => "cap-native pipe cat|grep pipeline",
                    NativeCommand::PipeCatGrepSortUniqProducer(_) => {
                        "cap-native pipe cat|grep|sort|uniq producer"
                    }
                    NativeCommand::PipeCatCut(_) => "cap-native pipe cat|cut",
                    NativeCommand::PipeCatTr(_) => "cap-native pipe cat|tr",
                    NativeCommand::PipeCatUniq(_) => "cap-native pipe cat|uniq",
                    NativeCommand::PipeCatUniqWcLines(_) => "cap-native pipe cat|uniq|wc",
                    NativeCommand::PipeCatUniqProducer(_) => "cap-native pipe cat|uniq producer",
                    NativeCommand::PipeCatUniqGrepProducer(_) => {
                        "cap-native pipe cat|uniq grep producer"
                    }
                    NativeCommand::PipeUniqProducer(_) => "cap-native pipe uniq producer",
                    NativeCommand::PipeUniqGrepProducer(_) => "cap-native pipe uniq grep producer",
                    NativeCommand::PipeCatSort(_) => "cap-native pipe cat|sort",
                    NativeCommand::PipeCatSortUniq(_) => "cap-native pipe cat|sort|uniq",
                    NativeCommand::PipeCatSortUniqWcLines(_) => "cap-native pipe cat|sort|uniq|wc",
                    NativeCommand::PipeCatSortHead(_) => "cap-native pipe cat|sort|head",
                    NativeCommand::PipeCatSortTail(_) => "cap-native pipe cat|sort|tail",
                    NativeCommand::PipeCatSortWcLines(_) => "cap-native pipe cat|sort|wc",
                    NativeCommand::PipeCatXargsEcho(_) => "cap-native pipe cat|xargs echo",
                    NativeCommand::PipeCatXargsWcLines(_) => "cap-native pipe cat|xargs|wc",
                    NativeCommand::PipeCatXargsWcProducer(_) => {
                        "cap-native pipe cat|xargs|wc producer"
                    }
                    NativeCommand::PipeCatGrepXargsEcho(_) => "cap-native pipe cat|grep|xargs echo",
                    NativeCommand::PipeCatGrepXargsWcLines(_) => {
                        "cap-native pipe cat|grep|xargs|wc"
                    }
                    NativeCommand::PipeCatGrepSortXargsEcho(_) => {
                        "cap-native pipe cat|grep|sort|xargs echo"
                    }
                    NativeCommand::PipeCatGrepSortXargsWcLines(_) => {
                        "cap-native pipe cat|grep|sort|xargs|wc"
                    }
                    NativeCommand::PipeCatSortXargsEcho(_) => "cap-native pipe cat|sort|xargs echo",
                    NativeCommand::PipeCatSortXargsWcLines(_) => {
                        "cap-native pipe cat|sort|xargs|wc"
                    }
                    NativeCommand::PipeCatSortXargsWcProducer(_) => {
                        "cap-native pipe cat|sort|xargs|wc producer"
                    }
                    NativeCommand::PipeGrepHead(_) => "cap-native pipe grep|head",
                    NativeCommand::PipeGrepTail(_) => "cap-native pipe grep|tail",
                    NativeCommand::PipeGrepSort(_) => "cap-native pipe grep|sort",
                    NativeCommand::PipeGrepSortUniq(_) => "cap-native pipe grep|sort|uniq",
                    NativeCommand::PipeGrepSortUniqProducer(_) => {
                        "cap-native pipe grep|sort|uniq producer"
                    }
                    NativeCommand::PipeGrepSortUniqWcLines(_) => {
                        "cap-native pipe grep|sort|uniq|wc"
                    }
                    NativeCommand::PipeGrepSortHead(_) => "cap-native pipe grep|sort|head",
                    NativeCommand::PipeGrepSortTail(_) => "cap-native pipe grep|sort|tail",
                    NativeCommand::PipeGrepSortWcLines(_) => "cap-native pipe grep|sort|wc",
                    NativeCommand::PipeGrepWcLines(_) => "cap-native pipe grep|wc",
                    NativeCommand::PipeGrepFile(_) => "cap-native pipe grep-file",
                    NativeCommand::PipeGrepFileSortUniqProducer(_) => {
                        "cap-native pipe grep-file|sort|uniq producer"
                    }
                    NativeCommand::PipeGrepFileCutProducer(_) => {
                        "cap-native pipe grep-file|cut producer"
                    }
                    NativeCommand::PipeGrepFileCutGrepProducer(_) => {
                        "cap-native pipe grep-file|cut grep producer"
                    }
                    NativeCommand::PipeGrepFileAwkProducer(_) => {
                        "cap-native pipe grep-file|awk producer"
                    }
                    NativeCommand::PipeGrepFileAwkGrepProducer(_) => {
                        "cap-native pipe grep-file|awk grep producer"
                    }
                    NativeCommand::PipeAwkProducer(_) => "cap-native pipe awk producer",
                    NativeCommand::PipeAwkGrepProducer(_) => "cap-native pipe awk grep producer",
                    NativeCommand::PipeAwkSortUniqProducer(_) => {
                        "cap-native pipe awk|sort|uniq producer"
                    }
                    NativeCommand::PipeAwkXargsEcho(_) => "cap-native pipe awk|xargs",
                    NativeCommand::PipeAwkXargsWcLines(_) => "cap-native pipe awk|xargs|wc",
                    NativeCommand::PipeEchoWcLines(_) => "cap-native pipe echo|wc",
                    NativeCommand::PipeEchoHead(_) => "cap-native pipe echo|head",
                    NativeCommand::PipeEchoTail(_) => "cap-native pipe echo|tail",
                    NativeCommand::PipeEchoTr(_) => "cap-native pipe echo|tr",
                    NativeCommand::PipeEchoAwkProducer(_) => "cap-native pipe echo|awk producer",
                    NativeCommand::PipeEchoXargsEcho(_) => "cap-native pipe echo|xargs echo",
                    NativeCommand::PipeEchoXargsWcLines(_) => "cap-native pipe echo|xargs|wc",
                    NativeCommand::PipePrintfWcLines(_) => "cap-native pipe printf|wc",
                    NativeCommand::PipePrintfHead(_) => "cap-native pipe printf|head",
                    NativeCommand::PipePrintfTail(_) => "cap-native pipe printf|tail",
                    NativeCommand::PipePrintfGrep(_) => "cap-native pipe printf|grep",
                    NativeCommand::PipePrintfTr(_) => "cap-native pipe printf|tr",
                    NativeCommand::PipePrintfAwkProducer(_) => {
                        "cap-native pipe printf|awk producer"
                    }
                    NativeCommand::PipePrintfProducer(_) => "cap-native pipe printf producer",
                    NativeCommand::PipePrintfLiteralProducer(_) => {
                        "cap-native pipe printf literal producer"
                    }
                    NativeCommand::PipePrintfGrepProducer(_) => {
                        "cap-native pipe printf grep producer"
                    }
                    NativeCommand::PipePrintfSortUniqProducer(_) => {
                        "cap-native pipe printf|sort|uniq producer"
                    }
                    NativeCommand::PipePrintfGrepSortUniqProducer(_) => {
                        "cap-native pipe printf|grep|sort|uniq producer"
                    }
                    NativeCommand::PipePrintfXargsEcho(_) => "cap-native pipe printf|xargs echo",
                    NativeCommand::PipePrintfXargsWcLines(_) => "cap-native pipe printf|xargs|wc",
                    NativeCommand::PipeSeqWcLines(_) => "cap-native pipe seq|wc",
                    NativeCommand::PipeSeqHead(_) => "cap-native pipe seq|head",
                    NativeCommand::PipeSeqTail(_) => "cap-native pipe seq|tail",
                    NativeCommand::PipeSeqGrepProducer(_) => "cap-native pipe seq grep producer",
                    NativeCommand::PipeSeqProducer(_) => "cap-native pipe seq producer",
                    NativeCommand::PipeSeqSortUniqProducer(_) => {
                        "cap-native pipe seq|sort|uniq producer"
                    }
                    NativeCommand::PipeSeqGrepSortUniqProducer(_) => {
                        "cap-native pipe seq|grep|sort|uniq producer"
                    }
                    NativeCommand::PipeSeqXargsEcho(_) => "cap-native pipe seq|xargs echo",
                    NativeCommand::PipeYesHead(_) => "cap-native pipe yes|head",
                    NativeCommand::PipePathLookupWcLines(_) => "cap-native pipe path-lookup|wc",
                    NativeCommand::PipePathLookupHead(_) => "cap-native pipe path-lookup|head",
                    NativeCommand::PipePathLookupTail(_) => "cap-native pipe path-lookup|tail",
                    NativeCommand::PipePathLookupGrepProducer(_) => {
                        "cap-native pipe path-lookup|grep producer"
                    }
                    NativeCommand::PipePathLookupProducer(_) => {
                        "cap-native pipe path-lookup producer"
                    }
                    NativeCommand::PipeEnvironmentWcLines(_) => "cap-native pipe printenv|wc",
                    NativeCommand::PipeEnvironmentHead(_) => "cap-native pipe printenv|head",
                    NativeCommand::PipeEnvironmentTail(_) => "cap-native pipe printenv|tail",
                    NativeCommand::PipeEnvironmentGrep(_) => "cap-native pipe printenv|grep",
                    NativeCommand::PipeEnvironmentGrepProducer(_) => {
                        "cap-native pipe printenv|grep producer"
                    }
                    NativeCommand::PipeEnvironmentSort(_) => "cap-native pipe printenv|sort",
                    NativeCommand::PipeHostnameWcLines(_) => "cap-native pipe hostname|wc",
                    NativeCommand::PipeHostnameHead(_) => "cap-native pipe hostname|head",
                    NativeCommand::PipeHostnameTail(_) => "cap-native pipe hostname|tail",
                    NativeCommand::PipeHostnameGrep(_) => "cap-native pipe hostname|grep",
                    NativeCommand::PipeHostnameGrepProducer(_) => {
                        "cap-native pipe hostname|grep producer"
                    }
                    NativeCommand::PipeHostnameSort(_) => "cap-native pipe hostname|sort",
                    NativeCommand::PipeSortUniq(_) => "cap-native pipe sort|uniq",
                    NativeCommand::PipeSortUniqWcLines(_) => "cap-native pipe sort|uniq|wc",
                    NativeCommand::PipeSortUniqProducer(_) => "cap-native pipe sort|uniq producer",
                    NativeCommand::PipeSortUniqGrepProducer(_) => {
                        "cap-native pipe sort|uniq grep producer"
                    }
                    NativeCommand::PipeSortGrepProducer(_) => "cap-native pipe sort grep producer",
                    NativeCommand::PipeSortHead(_) => "cap-native pipe sort|head",
                    NativeCommand::PipeSortTail(_) => "cap-native pipe sort|tail",
                    NativeCommand::PipeSortWcLines(_) => "cap-native pipe sort|wc",
                    NativeCommand::PipeSortXargsEcho(_) => "cap-native pipe sort|xargs echo",
                    NativeCommand::PipeSortXargsWcLines(_) => "cap-native pipe sort|xargs|wc",
                    NativeCommand::PipeSortXargsWcProducer(_) => {
                        "cap-native pipe sort|xargs|wc producer"
                    }
                    NativeCommand::PipeFindXargsEcho(_) => "cap-native pipe find|xargs echo",
                    NativeCommand::PipeFindXargsWcLines(_) => "cap-native pipe find|xargs|wc",
                    NativeCommand::PipeFindXargsWcProducer(_) => {
                        "cap-native pipe find|xargs|wc producer"
                    }
                    NativeCommand::PipeFindGrepProducer(_) => "cap-native pipe find|grep producer",
                    NativeCommand::PipeFindGrepXargsEcho(_) => {
                        "cap-native pipe find|grep|xargs echo"
                    }
                    NativeCommand::PipeFindGrepXargsWcLines(_) => {
                        "cap-native pipe find|grep|xargs|wc"
                    }
                    NativeCommand::PipeFindGrepSortXargsEcho(_) => {
                        "cap-native pipe find|grep|sort|xargs echo"
                    }
                    NativeCommand::PipeFindGrepSortXargsWcLines(_) => {
                        "cap-native pipe find|grep|sort|xargs|wc"
                    }
                    NativeCommand::PipeFindWcLines(_) => "cap-native pipe find|wc",
                    NativeCommand::PipeFindHead(_) => "cap-native pipe find|head",
                    NativeCommand::PipeFindTail(_) => "cap-native pipe find|tail",
                    NativeCommand::PipeFindSort(_) => "cap-native pipe find|sort",
                    NativeCommand::PipeFindSortUniq(_) => "cap-native pipe find|sort|uniq",
                    NativeCommand::PipeFindSortUniqWcLines(_) => {
                        "cap-native pipe find|sort|uniq|wc"
                    }
                    NativeCommand::PipeFindSortUniqProducer(_) => {
                        "cap-native pipe find|sort|uniq producer"
                    }
                    NativeCommand::PipeFindSortUniqGrepProducer(_) => {
                        "cap-native pipe find|sort|uniq grep producer"
                    }
                    NativeCommand::PipeFindSortXargsEcho(_) => {
                        "cap-native pipe find|sort|xargs echo"
                    }
                    NativeCommand::PipeFindSortXargsWcLines(_) => {
                        "cap-native pipe find|sort|xargs|wc"
                    }
                    NativeCommand::PipeFindSortWcLines(_) => "cap-native pipe find|sort|wc",
                    NativeCommand::PipeFindSortHead(_) => "cap-native pipe find|sort|head",
                    NativeCommand::PipeFindSortTail(_) => "cap-native pipe find|sort|tail",
                    NativeCommand::Find(_) => "cap-native find",
                    NativeCommand::SedPrint(_) => "cap-native sed -n",
                    NativeCommand::WcAll(_) => "cap-native wc",
                    NativeCommand::WcLines(plan) => match plan.mode {
                        WcCountMode::Lines => "cap-native wc -l",
                        WcCountMode::Bytes => "cap-native wc -c",
                        WcCountMode::Words => "cap-native wc -w",
                    },
                };
                [
                    format!("original: {}", plan.original),
                    "implementation: native".to_string(),
                    format!("run: {native}"),
                    format!("reason: {}", plan.reason),
                ]
                .join("\n")
            }
        }
    }
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn plan(command: &[String], label: Option<String>) -> CommandPlan {
    plan_with_tool_resolver(command, label, command_on_path)
}

/// Plan one Bash command string. Simple shell-free strings are parsed into argv
/// and routed through the same replacement planner as `cap <cmd>`; strings that
/// need shell semantics stay under `bash -c`.
/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn plan_shell(command: &str, label: Option<String>) -> CommandPlan {
    let original = command.trim().to_string();
    let planned_label = label.or_else(|| Some(original.clone()));
    if let Some(words) = split_simple_shell_words(&original) {
        if words.iter().any(|word| word == "|") {
            if let Some(plan) = plan_pipe_words(&words, planned_label.clone(), &original) {
                return CommandPlan::Native(plan);
            }
        }
    }
    if !original.is_empty() && !has_shell_control_syntax(&original) {
        if let Some(words) = split_simple_shell_words(&original) {
            if !words.is_empty() && !words_need_shell(&words) {
                return plan(&words, planned_label);
            }
        }
    }

    CommandPlan::External(ExternalPlan {
        program: "bash".to_string(),
        args: vec!["-c".to_string(), original.clone()],
        label: planned_label,
        original,
        implementation: ExternalImplementation::Original,
        reason: "shell command string requires bash semantics; running under bash -c".to_string(),
        fallback: None,
    })
}

fn plan_with_tool_resolver(
    command: &[String],
    label: Option<String>,
    tool_available: impl Fn(&str) -> bool,
) -> CommandPlan {
    let original = render_argv(command);
    let planned_label = label.or_else(|| Some(original.clone()));

    if let Some(plan) = plan_native(command, planned_label.clone(), &original) {
        return CommandPlan::Native(plan);
    }
    if let Some(plan) = plan_grep_replacement(command, planned_label.clone(), &original, |tool| {
        tool_available(tool)
    }) {
        return CommandPlan::External(plan);
    }

    CommandPlan::External(ExternalPlan {
        program: command[0].clone(),
        args: command[1..].to_vec(),
        label: planned_label,
        original,
        implementation: ExternalImplementation::Original,
        reason: "no safe cap replacement matched; running the original command".to_string(),
        fallback: None,
    })
}

fn plan_native(command: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let program = basename(command.first()?);
    match program {
        "true" => Some(NativePlan {
            command: NativeCommand::True,
            label,
            original: original.to_string(),
            reason: "true is a shell-free primitive cap can answer in-process".to_string(),
        }),
        "false" => Some(NativePlan {
            command: NativeCommand::False,
            label,
            original: original.to_string(),
            reason: "false is a shell-free primitive cap can answer in-process".to_string(),
        }),
        "pwd" => plan_pwd(&command[1..], label, original),
        "echo" => plan_echo(&command[1..], label, original),
        "printf" => plan_printf(&command[1..], label, original),
        "seq" => plan_seq(&command[1..], label, original),
        "whoami" => plan_whoami(&command[1..], label, original),
        "id" => plan_id(&command[1..], label, original),
        "uname" => plan_uname(&command[1..], label, original),
        "hostname" => plan_hostname(&command[1..], label, original),
        "test" | "[" => plan_test(program, &command[1..], label, original),
        "basename" => plan_basename(&command[1..], label, original),
        "dirname" => plan_dirname(&command[1..], label, original),
        "ls" => plan_ls(&command[1..], label, original),
        "cat" => plan_cat(&command[1..], label, original),
        "head" => plan_head(&command[1..], label, original),
        "tail" => plan_tail(&command[1..], label, original),
        "mkdir" => plan_mkdir(&command[1..], label, original),
        "touch" => plan_touch(&command[1..], label, original),
        "awk" => plan_awk(&command[1..], label, original),
        "xargs" => plan_xargs(&command[1..], label, original),
        "which" => plan_which(&command[1..], label, original),
        "command" => plan_command_v(&command[1..], label, original),
        "env" => plan_env(&command[1..], label, original),
        "printenv" => plan_printenv(&command[1..], label, original),
        "find" => plan_find(&command[1..], label, original),
        "sort" => plan_sort(&command[1..], label, original),
        "uniq" => plan_uniq(&command[1..], label, original),
        "cut" => plan_cut(&command[1..], label, original),
        "tr" => plan_tr(&command[1..], label, original),
        "sed" => plan_sed(&command[1..], label, original),
        "grep" => plan_grep_file(&command[1..], label, original),
        "wc" => plan_wc(&command[1..], label, original),
        _ => None,
    }
}

fn plan_pwd(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if !args.is_empty() {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::Pwd,
        label,
        original: original.to_string(),
        reason: "pwd without flags can be answered from cap's current directory".to_string(),
    })
}

fn plan_echo(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    Some(NativePlan {
        command: NativeCommand::Echo(parse_echo_args(args)?),
        label,
        original: original.to_string(),
        reason: "plain echo and echo -n can join arguments in-process".to_string(),
    })
}

fn parse_echo_args(args: &[String]) -> Option<EchoPlan> {
    let mut newline = true;
    let mut start = 0;
    if let Some(first) = args.first() {
        if first == "-n" {
            newline = false;
            start = 1;
        } else if first.starts_with('-') {
            return None;
        }
    }
    if !newline && args[start..].iter().any(|arg| arg.starts_with('-')) {
        return None;
    }
    Some(EchoPlan {
        args: args[start..].to_vec(),
        newline,
    })
}

fn plan_printf(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if let Some(literal) = parse_printf_literal_args(args) {
        return Some(NativePlan {
            command: NativeCommand::PrintfLiteral(literal),
            label,
            original: original.to_string(),
            reason: "literal printf format without conversions can emit bytes in-process"
                .to_string(),
        });
    }
    Some(NativePlan {
        command: NativeCommand::Printf(parse_printf_args(args)?),
        label,
        original: original.to_string(),
        reason: "narrow printf %s forms can format arguments in-process".to_string(),
    })
}

fn parse_printf_args(args: &[String]) -> Option<PrintfPlan> {
    if args.len() < 2 {
        return None;
    }
    Some(PrintfPlan {
        format: parse_printf_format(&args[0])?,
        args: args[1..].to_vec(),
    })
}

fn parse_printf_format(format: &str) -> Option<PrintfFormat> {
    match format {
        "%s" => Some(PrintfFormat::String),
        "%s\\n" | "%s\n" => Some(PrintfFormat::StringNewline),
        _ => None,
    }
}

fn parse_printf_literal_args(args: &[String]) -> Option<PrintfLiteralPlan> {
    if args.len() != 1 {
        return None;
    }
    Some(PrintfLiteralPlan {
        bytes: decode_printf_literal_format(&args[0])?,
    })
}

fn decode_printf_literal_format(format: &str) -> Option<Vec<u8>> {
    let bytes = format.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut idx = 0usize;
    while idx < bytes.len() {
        match bytes[idx] {
            b'%' => return None,
            b'\\' => {
                idx += 1;
                let escaped = *bytes.get(idx)?;
                match escaped {
                    b'\\' => out.push(b'\\'),
                    b'n' => out.push(b'\n'),
                    b't' => out.push(b'\t'),
                    b'r' => out.push(b'\r'),
                    _ => return None,
                }
            }
            byte => out.push(byte),
        }
        idx += 1;
    }
    Some(out)
}

fn plan_seq(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    Some(NativePlan {
        command: NativeCommand::Seq(parse_seq_args(args)?),
        label,
        original: original.to_string(),
        reason: "integer seq without flags can generate the range in-process".to_string(),
    })
}

fn parse_seq_args(args: &[String]) -> Option<SeqPlan> {
    let parse = |value: &str| value.parse::<i64>().ok();
    let seq = match args {
        [last] => SeqPlan {
            first: 1,
            step: 1,
            last: parse(last)?,
        },
        [first, last] => SeqPlan {
            first: parse(first)?,
            step: 1,
            last: parse(last)?,
        },
        [first, step, last] => SeqPlan {
            first: parse(first)?,
            step: parse(step)?,
            last: parse(last)?,
        },
        _ => return None,
    };
    (seq.step != 0).then_some(seq)
}

fn plan_whoami(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if !args.is_empty() {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::Whoami,
        label,
        original: original.to_string(),
        reason: "whoami without flags can read the effective user name in-process".to_string(),
    })
}

fn plan_id(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let kind = match args {
        [] => IdKind::Default,
        [flag] if flag == "-u" => IdKind::UserId,
        [flag] if flag == "-un" => IdKind::UserName,
        [flag] if flag == "-g" => IdKind::GroupId,
        [flag] if flag == "-gn" => IdKind::GroupName,
        [flag] if flag == "-G" => IdKind::GroupIds,
        [flag] if flag == "-Gn" => IdKind::GroupNames,
        _ => return None,
    };
    Some(NativePlan {
        command: NativeCommand::Id(IdPlan { kind }),
        label,
        original: original.to_string(),
        reason: "narrow id identity lookups can read process identity in-process".to_string(),
    })
}

fn plan_uname(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let field = match args {
        [] => UnameField::Sysname,
        [flag] if flag == "-s" => UnameField::Sysname,
        [flag] if flag == "-n" => UnameField::Nodename,
        [flag] if flag == "-r" => UnameField::Release,
        [flag] if flag == "-v" => UnameField::Version,
        [flag] if flag == "-m" => UnameField::Machine,
        [flag] if flag == "-p" => UnameField::Processor,
        [flag] if flag == "-a" => UnameField::All,
        _ => return None,
    };
    Some(NativePlan {
        command: NativeCommand::Uname(UnamePlan { field }),
        label,
        original: original.to_string(),
        reason: "simple uname fields can read the kernel utsname in-process".to_string(),
    })
}

fn plan_hostname(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if !args.is_empty() {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::Hostname,
        label,
        original: original.to_string(),
        reason: "hostname without flags can read the kernel hostname in-process".to_string(),
    })
}

fn plan_test(
    program: &str,
    args: &[String],
    label: Option<String>,
    original: &str,
) -> Option<NativePlan> {
    let args = if program == "[" {
        match args.split_last() {
            Some((last, rest)) if last == "]" => rest,
            _ => return None,
        }
    } else {
        args
    };
    Some(NativePlan {
        command: NativeCommand::Test(parse_test_args(args)?),
        label,
        original: original.to_string(),
        reason: "narrow test/[ predicates can evaluate in-process".to_string(),
    })
}

fn parse_test_args(args: &[String]) -> Option<TestPlan> {
    let (negated, args) = if args.first().is_some_and(|arg| arg == "!") {
        (true, &args[1..])
    } else {
        (false, args)
    };
    if args.is_empty() {
        return None;
    }

    let expr = match args {
        [flag, value] if flag == "-e" => TestExpr::FileExists(value.clone()),
        [flag, value] if flag == "-f" => TestExpr::FileRegular(value.clone()),
        [flag, value] if flag == "-d" => TestExpr::FileDirectory(value.clone()),
        [flag, value] if flag == "-s" => TestExpr::FileNonEmpty(value.clone()),
        [flag, value] if flag == "-n" => TestExpr::StringNonEmpty(value.clone()),
        [flag, value] if flag == "-z" => TestExpr::StringEmpty(value.clone()),
        [value] => TestExpr::StringNonEmpty(value.clone()),
        [left, op, right] if op == "=" || op == "==" => {
            TestExpr::StringEq(left.clone(), right.clone())
        }
        [left, op, right] if op == "!=" => TestExpr::StringNe(left.clone(), right.clone()),
        [left, op, right]
            if matches!(op.as_str(), "-eq" | "-ne" | "-gt" | "-ge" | "-lt" | "-le") =>
        {
            let left = left.parse().ok()?;
            let right = right.parse().ok()?;
            match op.as_str() {
                "-eq" => TestExpr::IntEq(left, right),
                "-ne" => TestExpr::IntNe(left, right),
                "-gt" => TestExpr::IntGt(left, right),
                "-ge" => TestExpr::IntGe(left, right),
                "-lt" => TestExpr::IntLt(left, right),
                "-le" => TestExpr::IntLe(left, right),
                _ => unreachable!(),
            }
        }
        _ => return None,
    };
    Some(TestPlan { expr, negated })
}

fn plan_basename(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if !(args.len() == 1 || args.len() == 2) || args.iter().any(|arg| arg.starts_with('-')) {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::Basename(BasenamePlan {
            path: args[0].clone(),
            suffix: args.get(1).cloned(),
        }),
        label,
        original: original.to_string(),
        reason: "basename path plus optional suffix is a shell-free path primitive".to_string(),
    })
}

fn plan_dirname(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if args.len() != 1 || args[0].starts_with('-') {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::Dirname(DirnamePlan {
            path: args[0].clone(),
        }),
        label,
        original: original.to_string(),
        reason: "dirname over one path is a shell-free path primitive".to_string(),
    })
}

fn plan_ls(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let mut mode = LsEntryMode::Visible;
    let mut paths = Vec::new();

    for arg in args {
        if arg == "--" {
            return None;
        }
        if arg.starts_with('-') && arg.len() > 1 {
            for flag in arg[1..].chars() {
                match flag {
                    'a' => mode = LsEntryMode::All,
                    'A' if mode != LsEntryMode::All => mode = LsEntryMode::AlmostAll,
                    'A' => {}
                    '1' => {}
                    _ => return None,
                }
            }
        } else {
            paths.push(arg.clone());
        }
    }

    if paths.len() > 1 {
        return None;
    }
    let path = paths.pop().unwrap_or_else(|| ".".to_string());
    let path_ref = Path::new(&path);
    if !path_ref.exists() {
        return None;
    }
    if !path_ref.is_dir() {
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::Ls(LsPlan { path, mode }),
        label,
        original: original.to_string(),
        reason: "simple non-long ls can be listed in-process".to_string(),
    })
}

fn plan_sort(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let plan = match args {
        [] => SortPlan {
            file: String::new(),
            stdin: true,
        },
        [file] => {
            let meta = fs::metadata(file).ok()?;
            if !meta.is_file() {
                return None;
            }
            SortPlan {
                file: file.clone(),
                stdin: false,
            }
        }
        _ => return None,
    };

    let reason = if plan.stdin {
        "stdin sort can use cap's buffered in-process sorter"
    } else {
        "single-file sort can use cap's buffered in-process sorter"
    };

    Some(NativePlan {
        command: NativeCommand::Sort(plan),
        label,
        original: original.to_string(),
        reason: reason.to_string(),
    })
}

fn sorted_reader_lines<R: Read>(mut reader: R) -> Result<SortedLines> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    Ok(sorted_data_lines(data))
}

fn sorted_data_lines(data: Vec<u8>) -> SortedLines {
    let mut lines = line_spans(&data);

    let mut ascending = true;
    let mut descending = true;
    for pair in lines.windows(2) {
        let previous = &data[pair[0].0..pair[0].1];
        let current = &data[pair[1].0..pair[1].1];
        if previous > current {
            ascending = false;
        }
        if previous < current {
            descending = false;
        }
    }

    if descending && !ascending {
        lines.reverse();
    } else if !ascending {
        lines.sort_unstable_by(|left, right| data[left.0..left.1].cmp(&data[right.0..right.1]));
    }

    SortedLines { data, lines }
}

fn plan_uniq(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let plan = match args {
        [] => UniqPlan {
            file: String::new(),
            stdin: true,
        },
        [file] => UniqPlan {
            file: file.clone(),
            stdin: false,
        },
        _ => return None,
    };
    Some(NativePlan {
        command: NativeCommand::Uniq(plan),
        label,
        original: original.to_string(),
        reason: "adjacent duplicate filtering can stream in-process".to_string(),
    })
}

fn plan_cut(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let plan = parse_cut_args(args)?;
    if !plan.stdin && !Path::new(&plan.file).is_file() {
        return None;
    }
    let source = if plan.stdin {
        "stdin"
    } else {
        "one regular file"
    };
    Some(NativePlan {
        command: NativeCommand::Cut(plan),
        label,
        original: original.to_string(),
        reason: format!("single-field cut over {source} can run in-process"),
    })
}

fn parse_cut_args(args: &[String]) -> Option<CutPlan> {
    parse_cut_args_with_file(args, None)
}

fn parse_cut_filter_args(args: &[String]) -> Option<CutFilterPlan> {
    let mut delimiter = b'\t';
    let mut field = None;
    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--" {
            return None;
        } else if arg == "-d" {
            idx += 1;
            delimiter = parse_cut_delimiter(args.get(idx)?)?;
        } else if let Some(value) = arg.strip_prefix("-d") {
            delimiter = parse_cut_delimiter(value)?;
        } else if arg == "-f" {
            idx += 1;
            field = Some(parse_cut_field(args.get(idx)?)?);
        } else if let Some(value) = arg.strip_prefix("-f") {
            field = Some(parse_cut_field(value)?);
        } else {
            return None;
        }
        idx += 1;
    }

    Some(CutFilterPlan {
        delimiter,
        field: field?,
    })
}

fn parse_cut_args_with_file(args: &[String], forced_file: Option<String>) -> Option<CutPlan> {
    let mut delimiter = b'\t';
    let mut field = None;
    let mut files = Vec::new();
    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--" {
            return None;
        } else if arg == "-d" {
            idx += 1;
            delimiter = parse_cut_delimiter(args.get(idx)?)?;
        } else if let Some(value) = arg.strip_prefix("-d") {
            delimiter = parse_cut_delimiter(value)?;
        } else if arg == "-f" {
            idx += 1;
            field = Some(parse_cut_field(args.get(idx)?)?);
        } else if let Some(value) = arg.strip_prefix("-f") {
            field = Some(parse_cut_field(value)?);
        } else if arg.starts_with('-') {
            return None;
        } else {
            files.push(arg.clone());
        }
        idx += 1;
    }

    let stdin = if let Some(file) = forced_file {
        if !files.is_empty() {
            return None;
        }
        files.push(file);
        false
    } else if files.is_empty() {
        true
    } else {
        false
    };
    if files.len() > 1 {
        return None;
    }
    Some(CutPlan {
        file: files.pop().unwrap_or_default(),
        stdin,
        delimiter,
        field: field?,
    })
}

fn parse_cut_delimiter(value: &str) -> Option<u8> {
    let bytes = value.as_bytes();
    match bytes {
        [delimiter] => Some(*delimiter),
        _ => None,
    }
}

fn parse_cut_field(value: &str) -> Option<usize> {
    let field = value.parse().ok()?;
    if field == 0 {
        None
    } else {
        Some(field)
    }
}

fn plan_tr(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    Some(NativePlan {
        command: NativeCommand::Tr(parse_tr_args(args)?),
        label,
        original: original.to_string(),
        reason: "narrow byte-level tr sets can transform stdin in-process".to_string(),
    })
}

fn parse_tr_args(args: &[String]) -> Option<TrPlan> {
    match args {
        [flag, set] if flag == "-d" => Some(TrPlan {
            mode: TrMode::Delete {
                set: expand_tr_set(set)?,
            },
        }),
        [from, to] if !from.starts_with('-') => {
            let from = expand_tr_set(from)?;
            let to = expand_tr_set(to)?;
            if from.is_empty() || from.len() != to.len() || has_duplicate_bytes(&from) {
                return None;
            }
            Some(TrPlan {
                mode: TrMode::Translate { from, to },
            })
        }
        _ => None,
    }
}

fn expand_tr_set(value: &str) -> Option<Vec<u8>> {
    if let Some(bytes) = expand_tr_class(value) {
        return Some(bytes);
    }
    if value.is_empty()
        || !value.is_ascii()
        || value.as_bytes().iter().any(|byte| {
            matches!(
                *byte,
                b'\\' | b'[' | b']' | b'*' | b':' | b'=' | b'\0' | b'\n' | b'\r'
            )
        })
    {
        return None;
    }

    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if idx + 2 < bytes.len() && bytes[idx + 1] == b'-' {
            let start = bytes[idx];
            let end = bytes[idx + 2];
            if start >= end {
                return None;
            }
            out.extend(start..=end);
            idx += 3;
        } else {
            if bytes[idx] == b'-' {
                return None;
            }
            out.push(bytes[idx]);
            idx += 1;
        }
    }
    Some(out)
}

fn expand_tr_class(value: &str) -> Option<Vec<u8>> {
    match value {
        "[:lower:]" => Some((b'a'..=b'z').collect()),
        "[:upper:]" => Some((b'A'..=b'Z').collect()),
        "[:digit:]" => Some((b'0'..=b'9').collect()),
        _ => None,
    }
}

fn has_duplicate_bytes(bytes: &[u8]) -> bool {
    let mut seen = [false; 256];
    for byte in bytes {
        let slot = &mut seen[usize::from(*byte)];
        if *slot {
            return true;
        }
        *slot = true;
    }
    false
}

fn plan_head(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let plan = parse_head_tail_args(args)?;
    if plan.count == 0 {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::Head(plan),
        label,
        original: original.to_string(),
        reason: "simple head line or byte windows can stream in-process".to_string(),
    })
}

fn plan_tail(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let plan = parse_head_tail_args(args)?;
    Some(NativePlan {
        command: NativeCommand::Tail(plan),
        label,
        original: original.to_string(),
        reason: "simple tail line or byte windows can stream in-process".to_string(),
    })
}

fn parse_mkdir_args(args: &[String]) -> Option<MkdirPlan> {
    if args.is_empty() {
        return None;
    }
    let mut parents = false;
    let mut start = 0;
    if args[0] == "-p" {
        parents = true;
        start = 1;
    } else if args[0].starts_with('-') {
        return None;
    }
    let paths = args[start..].to_vec();
    if paths.is_empty() || paths.iter().any(|path| path.starts_with('-')) {
        return None;
    }
    Some(MkdirPlan { paths, parents })
}

fn plan_mkdir(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let plan = parse_mkdir_args(args)?;
    Some(NativePlan {
        command: NativeCommand::Mkdir(plan),
        label,
        original: original.to_string(),
        reason: "simple mkdir path creation can run in-process".to_string(),
    })
}

fn parse_touch_args(args: &[String]) -> Option<TouchPlan> {
    if args.is_empty() || args.iter().any(|arg| arg.starts_with('-')) {
        return None;
    }
    Some(TouchPlan {
        paths: args.to_vec(),
    })
}

fn plan_touch(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let plan = parse_touch_args(args)?;
    Some(NativePlan {
        command: NativeCommand::Touch(plan),
        label,
        original: original.to_string(),
        reason: "plain touch path updates can run in-process".to_string(),
    })
}

fn plan_awk(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let (script, file, stdin) = match args {
        [script] => (script, String::new(), true),
        [script, file] if Path::new(file).is_file() => (script, file.clone(), false),
        _ => return None,
    };
    if let Some((pattern, field)) = parse_awk_print_field_script(script) {
        return Some(NativePlan {
            command: NativeCommand::AwkFirstField(AwkFirstFieldPlan {
                file,
                stdin,
                pattern,
                field,
            }),
            label,
            original: original.to_string(),
            reason: "narrow awk fixed-field extraction can scan stdin or the file in-process"
                .to_string(),
        });
    }
    if script != "/NEEDLE/ { c++ } END { print c }" {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::AwkNeedleCount(AwkNeedleCountPlan { file, stdin }),
        label,
        original: original.to_string(),
        reason: "narrow awk NEEDLE count can scan stdin or the file in-process".to_string(),
    })
}

fn plan_xargs(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let (command, reason) = match args {
        args if parse_xargs_echo_args(args).is_some() => {
            let mode = parse_xargs_echo_args(args)?;
            let reason = match mode {
                XargsEchoMode::OneLine => "xargs echo can batch stdin tokens in-process",
                XargsEchoMode::Batch { .. } => {
                    "xargs -n echo can emit stdin token batches in-process"
                }
            };
            (NativeCommand::XargsEcho(XargsEchoPlan { mode }), reason)
        }
        [arg, flag] if arg == "wc" && flag == "-l" => (
            NativeCommand::XargsWcLines(XargsWcLinesPlan),
            "xargs wc -l can batch stdin paths in-process",
        ),
        _ => return None,
    };
    Some(NativePlan {
        command,
        label,
        original: original.to_string(),
        reason: reason.to_string(),
    })
}

fn parse_xargs_echo_args(args: &[String]) -> Option<XargsEchoMode> {
    match args {
        [] => Some(XargsEchoMode::OneLine),
        [arg] if arg == "echo" => Some(XargsEchoMode::OneLine),
        [flag, count] if flag == "-n" => {
            parse_xargs_batch_size(count).map(|size| XargsEchoMode::Batch { size })
        }
        [flag] if flag.starts_with("-n") => {
            parse_xargs_compact_batch_flag(flag).map(|size| XargsEchoMode::Batch { size })
        }
        [flag, count, cmd] if flag == "-n" && cmd == "echo" => {
            parse_xargs_batch_size(count).map(|size| XargsEchoMode::Batch { size })
        }
        [flag, cmd] if flag.starts_with("-n") && cmd == "echo" => {
            parse_xargs_compact_batch_flag(flag).map(|size| XargsEchoMode::Batch { size })
        }
        _ => None,
    }
}

fn parse_xargs_batch_size(count: &str) -> Option<usize> {
    let size = count.parse().ok()?;
    (size > 0).then_some(size)
}

fn parse_xargs_compact_batch_flag(flag: &str) -> Option<usize> {
    let count = flag.strip_prefix("-n")?;
    (!count.is_empty()).then_some(())?;
    parse_xargs_batch_size(count)
}

fn parse_xargs_echo_command(words: &[String]) -> Option<XargsEchoMode> {
    let (command, args) = words.split_first()?;
    (command == "xargs").then_some(())?;
    parse_xargs_echo_args(args)
}

fn xargs_echo_pipe_mode(mode: XargsEchoMode) -> GrepFilePipeMode {
    match mode {
        XargsEchoMode::OneLine => GrepFilePipeMode::XargsEcho,
        XargsEchoMode::Batch { size } => GrepFilePipeMode::XargsEchoBatches { size },
    }
}

fn sort_xargs_echo_pipe_mode(mode: XargsEchoMode) -> GrepFilePipeMode {
    match mode {
        XargsEchoMode::OneLine => GrepFilePipeMode::SortXargsEcho,
        XargsEchoMode::Batch { size } => GrepFilePipeMode::SortXargsEchoBatches { size },
    }
}

fn plan_path_lookup(
    mode: PathLookupMode,
    args: &[String],
    label: Option<String>,
    original: &str,
) -> Option<NativePlan> {
    if args.is_empty() || args.iter().any(|arg| arg.starts_with('-')) {
        return None;
    }
    let reason = match mode {
        PathLookupMode::Which => "which path lookup can resolve PATH entries in-process",
        PathLookupMode::WhichAll => "which -a path lookup can resolve all PATH entries in-process",
        PathLookupMode::CommandV => "command -v path lookup can resolve shell words in-process",
    };
    Some(NativePlan {
        command: NativeCommand::PathLookup(PathLookupPlan {
            mode,
            names: args.to_vec(),
        }),
        label,
        original: original.to_string(),
        reason: reason.to_string(),
    })
}

fn plan_which(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    match args {
        [flag, names @ ..] if flag == "-a" => {
            plan_path_lookup(PathLookupMode::WhichAll, names, label, original)
        }
        _ => plan_path_lookup(PathLookupMode::Which, args, label, original),
    }
}

fn plan_command_v(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    match args {
        [flag, names @ ..] if flag == "-v" => {
            plan_path_lookup(PathLookupMode::CommandV, names, label, original)
        }
        _ => None,
    }
}

fn plan_env(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if !args.is_empty() {
        return None;
    }
    Some(NativePlan {
        command: NativeCommand::Environment(EnvironmentPlan {
            mode: EnvironmentMode::Env,
            name: None,
        }),
        label,
        original: original.to_string(),
        reason: "env without assignments can list the current environment in-process".to_string(),
    })
}

fn plan_printenv(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let name = match args {
        [] => None,
        [name] if !name.starts_with('-') => Some(name.clone()),
        _ => return None,
    };
    Some(NativePlan {
        command: NativeCommand::Environment(EnvironmentPlan {
            mode: EnvironmentMode::Printenv,
            name,
        }),
        label,
        original: original.to_string(),
        reason: "printenv without flags can read the current environment in-process".to_string(),
    })
}

fn split_single_pipe(words: &[String]) -> Option<(&[String], &[String])> {
    let pipe = words.iter().position(|word| word == "|")?;
    if pipe == 0 || pipe + 1 == words.len() {
        return None;
    }
    if words[pipe + 1..].iter().any(|word| word == "|") {
        return None;
    }
    Some((&words[..pipe], &words[pipe + 1..]))
}

fn parse_sort_pipe_left(words: &[String]) -> Option<String> {
    match words {
        [cmd, file] if cmd == "sort" && Path::new(file).is_file() => Some(file.clone()),
        _ => None,
    }
}

fn parse_ls_pipe_left(words: &[String]) -> Option<LsPipeSource> {
    if words.first()? != "ls" {
        return None;
    }
    let mut path: Option<String> = None;
    let mut mode = LsEntryMode::Visible;
    for word in &words[1..] {
        if word == "--" {
            return None;
        }
        if word.starts_with('-') && word.len() > 1 {
            for flag in word[1..].chars() {
                match flag {
                    '1' => {}
                    'a' => mode = LsEntryMode::All,
                    'A' if mode != LsEntryMode::All => mode = LsEntryMode::AlmostAll,
                    'A' => {}
                    _ => return None,
                }
            }
        } else if path.replace(word.clone()).is_some() {
            return None;
        }
    }
    let path = path.unwrap_or_else(|| ".".to_string());
    Path::new(&path)
        .is_dir()
        .then_some(LsPipeSource { path, mode })
}

fn parse_grep_file_pipe_left(words: &[String]) -> Option<(String, String)> {
    match words {
        [cmd, pattern]
            if cmd == "grep" && !pattern.starts_with('-') && is_plain_literal_pattern(pattern) =>
        {
            Some((pattern.clone(), String::new()))
        }
        [cmd, pattern, file]
            if cmd == "grep"
                && !pattern.starts_with('-')
                && is_plain_literal_pattern(pattern)
                && !file.starts_with('-')
                && Path::new(file).is_file() =>
        {
            Some((pattern.clone(), file.clone()))
        }
        _ => None,
    }
}

fn parse_path_lookup_pipe_left(words: &[String]) -> Option<PathLookupPlan> {
    match words {
        [cmd, flag, names @ ..] if cmd == "which" && flag == "-a" => {
            if names.is_empty() || names.iter().any(|name| name.starts_with('-')) {
                return None;
            }
            Some(PathLookupPlan {
                mode: PathLookupMode::WhichAll,
                names: names.to_vec(),
            })
        }
        [cmd, names @ ..] if cmd == "which" => {
            if names.is_empty() || names.iter().any(|name| name.starts_with('-')) {
                return None;
            }
            Some(PathLookupPlan {
                mode: PathLookupMode::Which,
                names: names.to_vec(),
            })
        }
        [cmd, flag, names @ ..] if cmd == "command" && flag == "-v" => {
            if names.is_empty() || names.iter().any(|name| name.starts_with('-')) {
                return None;
            }
            Some(PathLookupPlan {
                mode: PathLookupMode::CommandV,
                names: names.to_vec(),
            })
        }
        _ => None,
    }
}

fn parse_environment_pipe_left(words: &[String]) -> Option<EnvironmentPlan> {
    match words {
        [cmd, name] if cmd == "printenv" && !name.starts_with('-') => Some(EnvironmentPlan {
            mode: EnvironmentMode::Printenv,
            name: Some(name.clone()),
        }),
        _ => None,
    }
}

fn parse_hostname_pipe_left(words: &[String]) -> bool {
    matches!(words, [cmd] if cmd == "hostname")
}

fn is_safe_find_name_glob(pattern: &str) -> bool {
    !pattern.is_empty()
        && !pattern.starts_with('-')
        && !pattern.contains('/')
        && !pattern.contains(['[', ']'])
}

fn find_pipe_source(root: &str, name_glob: &str, max_depth: Option<usize>) -> FindPipeSource {
    FindPipeSource {
        root: root.to_string(),
        name_glob: name_glob.to_string(),
        max_depth,
    }
}

fn parse_find_max_depth(value: &str) -> Option<usize> {
    let max_depth = value.parse::<usize>().ok()?;
    (max_depth > 0).then_some(max_depth)
}

fn parse_find_pipe_prefix(words: &[String]) -> Option<(FindPipeSource, usize)> {
    if words.len() < 5 || words[0] != "find" || !Path::new(&words[1]).exists() {
        return None;
    }

    let (type_idx, max_depth) = if words.get(2).is_some_and(|word| word == "-type")
        && words.get(3).is_some_and(|word| word == "f")
    {
        (2, None)
    } else if words.get(2).is_some_and(|word| word == "-maxdepth")
        && words
            .get(3)
            .and_then(|word| parse_find_max_depth(word))
            .is_some()
        && words.get(4).is_some_and(|word| word == "-type")
        && words.get(5).is_some_and(|word| word == "f")
    {
        (4, words.get(3).and_then(|word| parse_find_max_depth(word)))
    } else {
        return None;
    };

    let after_type = type_idx + 2;
    if words.get(after_type).is_some_and(|word| word == "|") {
        return Some((find_pipe_source(&words[1], "*", max_depth), after_type));
    }
    if words.get(after_type).is_some_and(|word| word == "-name")
        && words
            .get(after_type + 1)
            .is_some_and(|word| is_safe_find_name_glob(word))
        && words.get(after_type + 2).is_some_and(|word| word == "|")
    {
        return Some((
            find_pipe_source(&words[1], &words[after_type + 1], max_depth),
            after_type + 2,
        ));
    }
    None
}

fn plan_xargs_wc_output_mode(words: &[String]) -> Option<(XargsWcOutputMode, &'static str)> {
    if words.len() < 5 || words[0] != "xargs" || words[1] != "wc" || words[2] != "-l" {
        return None;
    }
    if words[3] != "|" {
        return None;
    }
    match &words[4..] {
        [cmd, flag] if cmd == "wc" && flag == "-l" => Some((
            XargsWcOutputMode::WcLines,
            "xargs wc -l output piped to wc -l can count result lines in-process",
        )),
        [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                XargsWcOutputMode::Head { limit },
                "xargs wc -l output piped to head can emit the requested result prefix in-process",
            ))
        }
        [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            Some((
                XargsWcOutputMode::Tail { limit },
                "xargs wc -l output piped to tail can emit the requested result suffix in-process",
            ))
        }
        [cmd] if cmd == "sort" => Some((
            XargsWcOutputMode::Sort,
            "xargs wc -l output piped to sort can sort result lines in-process",
        )),
        [cmd, pipe_word, uniq] if cmd == "sort" && pipe_word == "|" && uniq == "uniq" => Some((
            XargsWcOutputMode::SortUniq,
            "xargs wc -l output piped through sort to uniq can de-duplicate result lines in-process",
        )),
        [cmd, pipe_a, uniq, pipe_b, count_cmd, flag]
            if cmd == "sort"
                && pipe_a == "|"
                && uniq == "uniq"
                && pipe_b == "|"
                && count_cmd == "wc"
                && flag == "-l" =>
        {
            Some((
                XargsWcOutputMode::SortUniqWcLines,
                "xargs wc -l output piped through sort and uniq to wc -l can count unique result lines in-process",
            ))
        }
        [cmd, pipe_word, count_cmd, flag]
            if cmd == "sort" && pipe_word == "|" && count_cmd == "wc" && flag == "-l" =>
        {
            Some((
                XargsWcOutputMode::SortWcLines,
                "xargs wc -l output piped through sort to wc -l can count result lines in-process",
            ))
        }
        [cmd, pipe_word, limit_cmd, flag, limit]
            if cmd == "sort" && pipe_word == "|" && limit_cmd == "head" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                XargsWcOutputMode::SortHead { limit },
                "xargs wc -l output piped through sort to head can emit the sorted result prefix in-process",
            ))
        }
        [cmd, pipe_word, limit_cmd, flag, limit]
            if cmd == "sort" && pipe_word == "|" && limit_cmd == "tail" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some((
                XargsWcOutputMode::SortTail { limit },
                "xargs wc -l output piped through sort to tail can emit the sorted result suffix in-process",
            ))
        }
        _ => None,
    }
}

fn find_xargs_wc_output_plan(
    source: FindPipeSource,
    pattern: Option<String>,
    sort_paths: bool,
    uniq_paths: bool,
    mode: XargsWcOutputMode,
    label: Option<String>,
    original: &str,
    reason: &'static str,
) -> NativePlan {
    NativePlan {
        command: NativeCommand::PipeFindXargsWcProducer(PipeFindXargsWcProducerPlan {
            source,
            pattern,
            sort_paths,
            uniq_paths,
            mode,
        }),
        label,
        original: original.to_string(),
        reason: reason.to_string(),
    }
}

fn plan_find_pipe(words: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let (source, pipe) = parse_find_pipe_prefix(words)?;
    let right = &words[pipe + 1..];
    if right.len() >= 4
        && right[0] == "sort"
        && right[1] == "|"
        && right[2] == "uniq"
        && right[3] == "|"
    {
        let downstream = &right[4..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeFindSortUniqGrepProducer(
                    PipeFindSortUniqGrepProducerPlan {
                        source,
                        pattern,
                        mode,
                    },
                ),
                label,
                original: original.to_string(),
                reason:
                    "find/sort/uniq output piped through grep/downstream can be fused in-process"
                        .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeFindSortUniqProducer(PipeFindSortUniqProducerPlan {
                    source,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason:
                    "find/sort/uniq output piped to a supported downstream can be fused in-process"
                        .to_string(),
            });
        }
    }
    if let Some((mode, reason)) = plan_xargs_wc_output_mode(right) {
        return Some(find_xargs_wc_output_plan(
            source, None, false, false, mode, label, original, reason,
        ));
    }
    if right.len() >= 7 && right[0] == "sort" && right[1] == "|" {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&right[2..]) {
            return Some(find_xargs_wc_output_plan(
                source, None, true, false, mode, label, original, reason,
            ));
        }
    }
    if right.len() >= 9
        && right[0] == "sort"
        && right[1] == "|"
        && right[2] == "uniq"
        && right[3] == "|"
    {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&right[4..]) {
            return Some(find_xargs_wc_output_plan(
                source, None, true, true, mode, label, original, reason,
            ));
        }
    }
    if right.len() >= 8
        && right[0] == "grep"
        && !right[1].is_empty()
        && is_plain_literal_pattern(&right[1])
        && right[2] == "|"
    {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&right[3..]) {
            return Some(find_xargs_wc_output_plan(
                source,
                Some(right[1].clone()),
                false,
                false,
                mode,
                label,
                original,
                reason,
            ));
        }
        if right.len() >= 10 && right[3] == "sort" && right[4] == "|" {
            if let Some((mode, reason)) = plan_xargs_wc_output_mode(&right[5..]) {
                return Some(find_xargs_wc_output_plan(
                    source,
                    Some(right[1].clone()),
                    true,
                    false,
                    mode,
                    label,
                    original,
                    reason,
                ));
            }
        }
        if right.len() >= 12
            && right[3] == "sort"
            && right[4] == "|"
            && right[5] == "uniq"
            && right[6] == "|"
        {
            if let Some((mode, reason)) = plan_xargs_wc_output_mode(&right[7..]) {
                return Some(find_xargs_wc_output_plan(
                    source,
                    Some(right[1].clone()),
                    true,
                    true,
                    mode,
                    label,
                    original,
                    reason,
                ));
            }
        }
    }
    match right {
        [xargs, wc, flag] if xargs == "xargs" && wc == "wc" && flag == "-l" => Some(NativePlan {
            command: NativeCommand::PipeFindXargsWcLines(PipeFindXargsWcLinesPlan {
                source,
            }),
            label,
            original: original.to_string(),
            reason: "find piped to xargs wc -l can count lines in-process".to_string(),
        }),
        [xargs] if xargs == "xargs" => Some(NativePlan {
            command: NativeCommand::PipeFindXargsEcho(PipeFindXargsEchoPlan {
                source,
            }),
            label,
            original: original.to_string(),
            reason: "find piped to default xargs echo can batch result tokens in-process"
                .to_string(),
        }),
        [xargs, echo] if xargs == "xargs" && echo == "echo" => Some(NativePlan {
            command: NativeCommand::PipeFindXargsEcho(PipeFindXargsEchoPlan {
                source,
            }),
            label,
            original: original.to_string(),
            reason: "find piped to xargs echo can batch result tokens in-process".to_string(),
        }),
        _ if right.first().is_some_and(|word| word == "grep") => {
            let (pattern, mode, _) = plan_tail_grep_producer_mode(right)?;
            Some(NativePlan {
                command: NativeCommand::PipeFindGrepProducer(PipeFindGrepProducerPlan {
                    source,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason:
                    "find output piped through grep/downstream can be fused in-process"
                        .to_string(),
            })
        }
        [grep, pattern, pipe_word, xargs, echo]
            if grep == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && xargs == "xargs"
                && echo == "echo" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindGrepXargsEcho(PipeFindGrepXargsEchoPlan {
                    source,
                    pattern: pattern.clone(),
                }),
                label,
                original: original.to_string(),
                reason: "find piped through grep to xargs echo can batch matching path tokens in-process"
                    .to_string(),
            })
        }
        [grep, pattern, pipe_word, xargs, wc, flag]
            if grep == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && xargs == "xargs"
                && wc == "wc"
                && flag == "-l" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindGrepXargsWcLines(PipeFindGrepXargsWcLinesPlan {
                    source,
                    pattern: pattern.clone(),
                }),
                label,
                original: original.to_string(),
                reason: "find piped through grep to xargs wc -l can line-count matching path tokens in-process"
                    .to_string(),
            })
        }
        [grep, pattern, pipe_1, sort, pipe_2, xargs, echo]
            if grep == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_1 == "|"
                && sort == "sort"
                && pipe_2 == "|"
                && xargs == "xargs"
                && echo == "echo" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindGrepSortXargsEcho(
                    PipeFindGrepSortXargsEchoPlan {
                        source,
                        pattern: pattern.clone(),
                    },
                ),
                label,
                original: original.to_string(),
                reason: "find piped through grep and sort to xargs echo can batch sorted matching path tokens in-process"
                    .to_string(),
            })
        }
        [grep, pattern, pipe_1, sort, pipe_2, xargs, wc, flag]
            if grep == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_1 == "|"
                && sort == "sort"
                && pipe_2 == "|"
                && xargs == "xargs"
                && wc == "wc"
                && flag == "-l" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindGrepSortXargsWcLines(
                    PipeFindGrepSortXargsWcLinesPlan {
                        source,
                        pattern: pattern.clone(),
                    },
                ),
                label,
                original: original.to_string(),
                reason: "find piped through grep and sort to xargs wc -l can line-count sorted matching path tokens in-process"
                    .to_string(),
            })
        }
        [wc, flag] if wc == "wc" && flag == "-l" => Some(NativePlan {
            command: NativeCommand::PipeFindWcLines(PipeFindWcLinesPlan { source }),
            label,
            original: original.to_string(),
            reason: "find piped to wc -l can count result paths in-process".to_string(),
        }),
        [head, flag, limit] if head == "head" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            (limit > 0).then(|| NativePlan {
                command: NativeCommand::PipeFindHead(PipeFindHeadPlan {
                    source,
                    limit,
                }),
                label,
                original: original.to_string(),
                reason: "find piped to head can stop after the requested paths in-process"
                    .to_string(),
            })
        }
        [tail, flag, limit] if tail == "tail" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            Some(NativePlan {
                command: NativeCommand::PipeFindTail(PipeFindTailPlan {
                    source,
                    limit,
                }),
                label,
                original: original.to_string(),
                reason: "find piped to tail can keep the result suffix in-process".to_string(),
            })
        }
        [sort] if sort == "sort" => Some(NativePlan {
            command: NativeCommand::PipeFindSort(PipeFindSortPlan { source }),
            label,
            original: original.to_string(),
            reason: "find piped to sort can emit sorted result paths in-process".to_string(),
        }),
        [sort, pipe_word, uniq] if sort == "sort" && pipe_word == "|" && uniq == "uniq" => {
            Some(NativePlan {
                command: NativeCommand::PipeFindSortUniq(PipeFindSortUniqPlan {
                    source,
                }),
                label,
                original: original.to_string(),
                reason: "find piped through sort to uniq can emit unique sorted result paths in-process"
                    .to_string(),
            })
        }
        [sort, pipe_word_1, uniq, pipe_word_2, wc, flag]
            if sort == "sort"
                && pipe_word_1 == "|"
                && uniq == "uniq"
                && pipe_word_2 == "|"
                && wc == "wc"
                && flag == "-l" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindSortUniqWcLines(PipeFindSortUniqWcLinesPlan {
                    source,
                }),
                label,
                original: original.to_string(),
                reason: "find piped through sort and uniq to wc -l can count unique sorted result paths in-process"
                    .to_string(),
            })
        }
        [sort, pipe_word, wc, flag]
            if sort == "sort" && pipe_word == "|" && wc == "wc" && flag == "-l" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindSortWcLines(PipeFindSortWcLinesPlan {
                    source,
                }),
                label,
                original: original.to_string(),
                reason: "find piped through sort to wc -l can count sorted result paths in-process"
                    .to_string(),
            })
        }
        [sort, pipe_word, xargs, echo]
            if sort == "sort" && pipe_word == "|" && xargs == "xargs" && echo == "echo" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindSortXargsEcho(PipeFindSortXargsEchoPlan {
                    source,
                }),
                label,
                original: original.to_string(),
                reason: "find piped through sort to xargs echo can batch sorted result tokens in-process"
                    .to_string(),
            })
        }
        [sort, pipe_word, xargs, wc, flag]
            if sort == "sort"
                && pipe_word == "|"
                && xargs == "xargs"
                && wc == "wc"
                && flag == "-l" =>
        {
            Some(NativePlan {
                command: NativeCommand::PipeFindSortXargsWcLines(PipeFindSortXargsWcLinesPlan {
                    source,
                }),
                label,
                original: original.to_string(),
                reason: "find piped through sort to xargs wc -l can line-count sorted paths in-process"
                    .to_string(),
            })
        }
        [sort, pipe_word, head, flag, limit]
            if sort == "sort" && pipe_word == "|" && head == "head" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then(|| NativePlan {
                command: NativeCommand::PipeFindSortHead(PipeFindSortHeadPlan {
                    source,
                    limit,
                }),
                label,
                original: original.to_string(),
                reason: "find piped through sort to head can emit the requested sorted prefix in-process"
                    .to_string(),
            })
        }
        [sort, pipe_word, tail, flag, limit]
            if sort == "sort" && pipe_word == "|" && tail == "tail" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some(NativePlan {
                command: NativeCommand::PipeFindSortTail(PipeFindSortTailPlan {
                    source,
                    limit,
                }),
                label,
                original: original.to_string(),
                reason: "find piped through sort to tail can emit the requested sorted suffix in-process"
                    .to_string(),
            })
        }
        _ => None,
    }
}

fn plan_grep_file_cut_pipe(
    words: &[String],
    label: Option<String>,
    original: &str,
) -> Option<NativePlan> {
    if words.len() < 6 || words.get(3)? != "|" || words.get(4)? != "cut" {
        return None;
    }
    let (pattern, file) = parse_grep_file_pipe_left(&words[..3])?;
    let cut_pipe = words[5..]
        .iter()
        .position(|word| word == "|")
        .map(|idx| idx + 5);
    let cut_end = cut_pipe.unwrap_or(words.len());
    let cut = parse_cut_filter_args(&words[5..cut_end])?;
    if let Some(cut_pipe) = cut_pipe {
        let downstream = &words[cut_pipe + 1..];
        if let Some((downstream_pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeGrepFileCutGrepProducer(
                    PipeGrepFileCutGrepProducerPlan {
                        pattern,
                        file,
                        cut,
                        downstream_pattern,
                        mode,
                    },
                ),
                label,
                original: original.to_string(),
                reason:
                    "grep file output piped through cut and grep/downstream can be fused in-process"
                        .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeGrepFileCutProducer(PipeGrepFileCutProducerPlan {
                    pattern,
                    file,
                    cut,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "grep file output piped through cut/downstream can be fused in-process"
                    .to_string(),
            });
        }
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::PipeGrepFileCutProducer(PipeGrepFileCutProducerPlan {
            pattern,
            file,
            cut,
            mode: GrepFilePipeMode::Lines,
        }),
        label,
        original: original.to_string(),
        reason: "grep file output piped through cut can run in-process".to_string(),
    })
}

fn parse_awk_print_field_script(script: &str) -> Option<(Option<String>, usize)> {
    let script = script.trim();
    if let Some(field) = parse_awk_print_field_action(script) {
        return Some((None, field));
    }
    let rest = script.strip_prefix("/NEEDLE/")?.trim();
    if let Some(field) = parse_awk_print_field_action(rest) {
        return Some((Some("NEEDLE".to_string()), field));
    }
    None
}

fn parse_awk_print_field_action(action: &str) -> Option<usize> {
    let compact = action
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let field = compact.strip_prefix("{print$")?.strip_suffix('}')?;
    if field.is_empty() || !field.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let field = field.parse().ok()?;
    (field > 0).then_some(field)
}

fn plan_xargs_wc_line_producer_mode(
    downstream: &[String],
) -> Option<(GrepFilePipeMode, &'static str)> {
    if let Some((mode, reason)) = plan_xargs_wc_output_mode(downstream) {
        return Some((GrepFilePipeMode::XargsWcOutput { mode }, reason));
    }
    if downstream.len() >= 7 && downstream[0] == "sort" && downstream[1] == "|" {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&downstream[2..]) {
            return Some((GrepFilePipeMode::SortXargsWcOutput { mode }, reason));
        }
    }
    None
}

fn plan_grep_xargs_wc_line_producer_mode(
    downstream: &[String],
) -> Option<(String, GrepFilePipeMode, &'static str)> {
    if downstream.len() < 7
        || downstream[0] != "grep"
        || downstream[1].is_empty()
        || !is_plain_literal_pattern(&downstream[1])
        || downstream[2] != "|"
    {
        return None;
    }
    let pattern = downstream[1].clone();
    if let Some((mode, reason)) = plan_xargs_wc_output_mode(&downstream[3..]) {
        return Some((pattern, GrepFilePipeMode::XargsWcOutput { mode }, reason));
    }
    if downstream.len() >= 9 && downstream[3] == "sort" && downstream[4] == "|" {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&downstream[5..]) {
            return Some((
                pattern,
                GrepFilePipeMode::SortXargsWcOutput { mode },
                reason,
            ));
        }
    }
    None
}

fn plan_grep_file_awk_pipe(
    words: &[String],
    label: Option<String>,
    original: &str,
) -> Option<NativePlan> {
    if words.len() < 6 || words.get(3)? != "|" || words.get(4)? != "awk" {
        return None;
    }
    let (pattern, file) = parse_grep_file_pipe_left(&words[..3])?;
    let (awk_pattern, awk_field) = parse_awk_print_field_script(words.get(5)?)?;
    if words.len() == 6 {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepFileAwkProducer(PipeGrepFileAwkProducerPlan {
                pattern,
                file,
                awk_pattern,
                awk_field,
                mode: GrepFilePipeMode::Lines,
            }),
            label,
            original: original.to_string(),
            reason: "grep file output piped through awk print-field can run in-process".to_string(),
        });
    }
    if words.get(6)? != "|" {
        return None;
    }
    let downstream = &words[7..];
    if let Some((downstream_pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepFileAwkGrepProducer(PipeGrepFileAwkGrepProducerPlan {
                pattern,
                file,
                awk_pattern,
                awk_field,
                downstream_pattern,
                mode,
            }),
            label,
            original: original.to_string(),
            reason:
                "grep file output piped through awk and grep/downstream can be fused in-process"
                    .to_string(),
        });
    }
    if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepFileAwkProducer(PipeGrepFileAwkProducerPlan {
                pattern,
                file,
                awk_pattern,
                awk_field,
                mode,
            }),
            label,
            original: original.to_string(),
            reason: "grep file output piped through awk/downstream can be fused in-process"
                .to_string(),
        });
    }
    None
}

fn plan_finite_awk_pipe(
    words: &[String],
    label: Option<String>,
    original: &str,
) -> Option<NativePlan> {
    let pipe = words.iter().position(|word| word == "|")?;
    if pipe == 0 || words.get(pipe + 1)? != "awk" {
        return None;
    }
    let (pattern, field) = parse_awk_print_field_script(words.get(pipe + 2)?)?;
    let (mode, downstream_reason) = if words.len() == pipe + 3 {
        (
            GrepFilePipeMode::Lines,
            "awk first-field output can be emitted in-process",
        )
    } else {
        if words.get(pipe + 3)? != "|" {
            return None;
        }
        let downstream = &words[pipe + 4..];
        let (mode, reason) = plan_tail_producer_mode(downstream)?;
        (mode, reason)
    };
    if words.first().is_some_and(|word| word == "echo") {
        let echo = parse_echo_args(&words[1..pipe])?;
        return Some(NativePlan {
            command: NativeCommand::PipeEchoAwkProducer(PipeEchoAwkProducerPlan {
                echo,
                pattern,
                field,
                mode,
            }),
            label,
            original: original.to_string(),
            reason: format!(
                "echo output piped through awk can be fused in-process; {downstream_reason}"
            ),
        });
    }
    if words.first().is_some_and(|word| word == "printf") {
        let printf = parse_printf_args(&words[1..pipe])?;
        if printf.format != PrintfFormat::StringNewline {
            return None;
        }
        return Some(NativePlan {
            command: NativeCommand::PipePrintfAwkProducer(PipePrintfAwkProducerPlan {
                args: printf.args,
                pattern,
                field,
                mode,
            }),
            label,
            original: original.to_string(),
            reason: format!(
                "printf %s\\n output piped through awk can be fused in-process; {downstream_reason}"
            ),
        });
    }
    None
}

fn wc_pipe_mode(mode: WcCountMode) -> GrepFilePipeMode {
    match mode {
        WcCountMode::Lines => GrepFilePipeMode::WcLines,
        WcCountMode::Bytes | WcCountMode::Words => GrepFilePipeMode::Wc { mode },
    }
}

fn sort_wc_pipe_mode(mode: WcCountMode) -> GrepFilePipeMode {
    match mode {
        WcCountMode::Lines => GrepFilePipeMode::SortWcLines,
        WcCountMode::Bytes | WcCountMode::Words => GrepFilePipeMode::SortWc { mode },
    }
}

fn sort_uniq_wc_pipe_mode(mode: WcCountMode) -> GrepFilePipeMode {
    match mode {
        WcCountMode::Lines => GrepFilePipeMode::SortUniqWcLines,
        WcCountMode::Bytes | WcCountMode::Words => GrepFilePipeMode::SortUniqWc { mode },
    }
}

fn plan_head_producer_mode(downstream: &[String]) -> Option<(GrepFilePipeMode, &'static str)> {
    if let Some(mode) = plan_xargs_wc_line_producer_mode(downstream) {
        return Some(mode);
    }
    match downstream {
        [cmd, flag] if cmd == "wc" => {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                wc_pipe_mode(mode),
                "head output piped to wc can count emitted output in-process",
            ))
        }
        [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                GrepFilePipeMode::Head { limit },
                "head output piped to head can emit the requested prefix in-process",
            ))
        }
        [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            Some((
                GrepFilePipeMode::Tail { limit },
                "head output piped to tail can emit the requested suffix in-process",
            ))
        }
        [cmd] if cmd == "sort" => Some((
            GrepFilePipeMode::Sort,
            "head output piped to sort can sort emitted lines in-process",
        )),
        [cmd, pipe_word, subcmd] if cmd == "sort" && pipe_word == "|" && subcmd == "uniq" => {
            Some((
                GrepFilePipeMode::SortUniq,
                "head output piped through sort to uniq can de-duplicate emitted lines in-process",
            ))
        }
        [cmd, pipe_a, subcmd, pipe_b, count_cmd, flag]
            if cmd == "sort"
                && pipe_a == "|"
                && subcmd == "uniq"
                && pipe_b == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                sort_uniq_wc_pipe_mode(mode),
                "head output piped through sort and uniq to wc can count unique emitted output in-process",
            ))
        }
        [cmd, pipe_word, count_cmd, flag]
            if cmd == "sort" && pipe_word == "|" && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                sort_wc_pipe_mode(mode),
                "head output piped through sort to wc can count emitted output in-process",
            ))
        }
        [cmd, pipe_word, limit_cmd, flag, limit]
            if cmd == "sort" && pipe_word == "|" && limit_cmd == "head" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                GrepFilePipeMode::SortHead { limit },
                "head output piped through sort to head can emit the sorted prefix in-process",
            ))
        }
        [cmd, pipe_word, limit_cmd, flag, limit]
            if cmd == "sort" && pipe_word == "|" && limit_cmd == "tail" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some((
                GrepFilePipeMode::SortTail { limit },
                "head output piped through sort to tail can emit the sorted suffix in-process",
            ))
        }
        [cmd, pipe_word, subcmd, arg]
            if cmd == "sort" && pipe_word == "|" && subcmd == "xargs" && arg == "echo" =>
        {
            Some((
                GrepFilePipeMode::SortXargsEcho,
                "head output piped through sort to xargs echo can batch sorted emitted tokens in-process",
            ))
        }
        [cmd, pipe_word, subcmd, arg, flag]
            if cmd == "sort"
                && pipe_word == "|"
                && subcmd == "xargs"
                && arg == "wc"
                && flag == "-l" =>
        {
            Some((
                GrepFilePipeMode::SortXargsWcLines,
                "head output piped through sort to xargs wc -l can line-count sorted emitted path tokens in-process",
            ))
        }
        [cmd] if cmd == "xargs" => Some((
            GrepFilePipeMode::XargsEcho,
            "head output piped to default xargs echo can batch emitted tokens in-process",
        )),
        [cmd, subcmd] if cmd == "xargs" && subcmd == "echo" => Some((
            GrepFilePipeMode::XargsEcho,
            "head output piped to xargs echo can batch emitted tokens in-process",
        )),
        [cmd, subcmd, flag] if cmd == "xargs" && subcmd == "wc" && flag == "-l" => Some((
            GrepFilePipeMode::XargsWcLines,
            "head output piped to xargs wc -l can line-count emitted path tokens in-process",
        )),
        _ => None,
    }
}

fn plan_head_grep_producer_mode(
    downstream: &[String],
) -> Option<(String, GrepFilePipeMode, &'static str)> {
    if let Some(mode) = plan_grep_xargs_wc_line_producer_mode(downstream) {
        return Some(mode);
    }
    match downstream {
        [cmd, pattern] if cmd == "grep" && !pattern.is_empty() && is_plain_literal_pattern(pattern) => Some((
            pattern.clone(),
            GrepFilePipeMode::Lines,
            "head output piped to grep can filter emitted lines in-process",
        )),
        [cmd, pattern, pipe_word, count_cmd, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                pattern.clone(),
                wc_pipe_mode(mode),
                "head output piped through grep to wc can count filtered emitted output in-process",
            ))
        }
        [cmd, pattern, pipe_word, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && limit_cmd == "head"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                pattern.clone(),
                GrepFilePipeMode::Head { limit },
                "head output piped through grep to head can emit the filtered prefix in-process",
            ))
        }
        [cmd, pattern, pipe_word, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && limit_cmd == "tail"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some((
                pattern.clone(),
                GrepFilePipeMode::Tail { limit },
                "head output piped through grep to tail can emit the filtered suffix in-process",
            ))
        }
        [cmd, pattern, pipe_word, sort_cmd]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && sort_cmd == "sort" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::Sort,
                "head output piped through grep to sort can sort filtered emitted lines in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && uniq_cmd == "uniq" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::SortUniq,
                "head output piped through grep, sort, and uniq can de-duplicate filtered emitted lines in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd, pipe_c, count_cmd, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && uniq_cmd == "uniq"
                && pipe_c == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                pattern.clone(),
                sort_uniq_wc_pipe_mode(mode),
                "head output piped through grep, sort, uniq, and wc can count unique filtered emitted output in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, count_cmd, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                pattern.clone(),
                sort_wc_pipe_mode(mode),
                "head output piped through grep, sort, and wc can count filtered emitted output in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && limit_cmd == "head"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                pattern.clone(),
                GrepFilePipeMode::SortHead { limit },
                "head output piped through grep, sort, and head can emit the sorted filtered prefix in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && limit_cmd == "tail"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some((
                pattern.clone(),
                GrepFilePipeMode::SortTail { limit },
                "head output piped through grep, sort, and tail can emit the sorted filtered suffix in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, subcmd, arg]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && subcmd == "xargs"
                && arg == "echo" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::SortXargsEcho,
                "head output piped through grep, sort, and xargs echo can batch sorted filtered tokens in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, subcmd, arg, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && subcmd == "xargs"
                && arg == "wc"
                && flag == "-l" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::SortXargsWcLines,
                "head output piped through grep, sort, and xargs wc -l can line-count sorted filtered path tokens in-process",
            ))
        }
        [cmd, pattern, pipe_word, subcmd, arg]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && subcmd == "xargs"
                && arg == "echo" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::XargsEcho,
                "head output piped through grep to xargs echo can batch filtered tokens in-process",
            ))
        }
        [cmd, pattern, pipe_word, subcmd, arg, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && subcmd == "xargs"
                && arg == "wc"
                && flag == "-l" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::XargsWcLines,
                "head output piped through grep to xargs wc -l can line-count filtered path tokens in-process",
            ))
        }
        _ => None,
    }
}

fn plan_tail_producer_mode(downstream: &[String]) -> Option<(GrepFilePipeMode, &'static str)> {
    if let Some(mode) = plan_xargs_wc_line_producer_mode(downstream) {
        return Some(mode);
    }
    if downstream.len() >= 3 && downstream[0] == "sort" && downstream[1] == "|" {
        if let Some(mode) = parse_xargs_echo_command(&downstream[2..]) {
            return Some((
                sort_xargs_echo_pipe_mode(mode),
                match mode {
                    XargsEchoMode::OneLine => {
                        "tail output piped through sort to xargs echo can batch sorted emitted tokens in-process"
                    }
                    XargsEchoMode::Batch { .. } => {
                        "tail output piped through sort to xargs -n echo can emit sorted token batches in-process"
                    }
                },
            ));
        }
    }
    if let Some(mode) = parse_xargs_echo_command(downstream) {
        return Some((
            xargs_echo_pipe_mode(mode),
            match mode {
                XargsEchoMode::OneLine => {
                    "tail output piped to xargs echo can batch emitted tokens in-process"
                }
                XargsEchoMode::Batch { .. } => {
                    "tail output piped to xargs -n echo can emit token batches in-process"
                }
            },
        ));
    }
    match downstream {
        [cmd, flag] if cmd == "wc" => {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                wc_pipe_mode(mode),
                "tail output piped to wc can count emitted output in-process",
            ))
        }
        [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                GrepFilePipeMode::Head { limit },
                "tail output piped to head can emit the requested prefix in-process",
            ))
        }
        [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
            let limit = limit.parse().ok()?;
            Some((
                GrepFilePipeMode::Tail { limit },
                "tail output piped to tail can emit the requested suffix in-process",
            ))
        }
        [cmd] if cmd == "sort" => Some((
            GrepFilePipeMode::Sort,
            "tail output piped to sort can sort emitted lines in-process",
        )),
        [cmd, pipe_word, subcmd] if cmd == "sort" && pipe_word == "|" && subcmd == "uniq" => {
            Some((
                GrepFilePipeMode::SortUniq,
                "tail output piped through sort to uniq can de-duplicate emitted lines in-process",
            ))
        }
        [cmd, pipe_a, subcmd, pipe_b, count_cmd, flag]
            if cmd == "sort"
                && pipe_a == "|"
                && subcmd == "uniq"
                && pipe_b == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                sort_uniq_wc_pipe_mode(mode),
                "tail output piped through sort and uniq to wc can count unique emitted output in-process",
            ))
        }
        [cmd, pipe_word, count_cmd, flag]
            if cmd == "sort" && pipe_word == "|" && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                sort_wc_pipe_mode(mode),
                "tail output piped through sort to wc can count emitted output in-process",
            ))
        }
        [cmd, pipe_word, limit_cmd, flag, limit]
            if cmd == "sort" && pipe_word == "|" && limit_cmd == "head" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                GrepFilePipeMode::SortHead { limit },
                "tail output piped through sort to head can emit the sorted prefix in-process",
            ))
        }
        [cmd, pipe_word, limit_cmd, flag, limit]
            if cmd == "sort" && pipe_word == "|" && limit_cmd == "tail" && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some((
                GrepFilePipeMode::SortTail { limit },
                "tail output piped through sort to tail can emit the sorted suffix in-process",
            ))
        }
        [cmd, pipe_word, subcmd, arg, flag]
            if cmd == "sort"
                && pipe_word == "|"
                && subcmd == "xargs"
                && arg == "wc"
                && flag == "-l" =>
        {
            Some((
                GrepFilePipeMode::SortXargsWcLines,
                "tail output piped through sort to xargs wc -l can line-count sorted emitted path tokens in-process",
            ))
        }
        [cmd, subcmd, flag] if cmd == "xargs" && subcmd == "wc" && flag == "-l" => Some((
            GrepFilePipeMode::XargsWcLines,
            "tail output piped to xargs wc -l can line-count emitted path tokens in-process",
        )),
        _ => None,
    }
}

fn grep_file_pipe_mode_reason(file: &str, mode: GrepFilePipeMode) -> String {
    let source = if file.is_empty() {
        "grep stdin output"
    } else {
        "grep file output"
    };
    match mode {
        GrepFilePipeMode::Lines => {
            format!("{source} can stream matching lines in-process")
        }
        GrepFilePipeMode::WcLines | GrepFilePipeMode::Wc { .. } => {
            format!("{source} piped to wc can count matching output in-process")
        }
        GrepFilePipeMode::Head { .. } => {
            format!("{source} piped to head can emit matching prefix lines in-process")
        }
        GrepFilePipeMode::Tail { .. } => {
            format!("{source} piped to tail can emit matching suffix lines in-process")
        }
        GrepFilePipeMode::Sort => {
            format!("{source} piped to sort can sort matching lines in-process")
        }
        GrepFilePipeMode::SortUniq => {
            format!(
                "{source} piped through sort and uniq can de-duplicate matching lines in-process"
            )
        }
        GrepFilePipeMode::SortUniqWcLines | GrepFilePipeMode::SortUniqWc { .. } => {
            format!("{source} piped through sort and uniq to wc can count unique matching output in-process")
        }
        GrepFilePipeMode::SortWcLines | GrepFilePipeMode::SortWc { .. } => {
            format!("{source} piped through sort to wc can count sorted matching output in-process")
        }
        GrepFilePipeMode::SortHead { .. } => {
            format!("{source} piped through sort to head can emit the sorted matching prefix in-process")
        }
        GrepFilePipeMode::SortTail { .. } => {
            format!("{source} piped through sort to tail can emit the sorted matching suffix in-process")
        }
        GrepFilePipeMode::SortXargsEcho => {
            format!("{source} piped through sort to xargs echo can batch sorted matching tokens in-process")
        }
        GrepFilePipeMode::SortXargsEchoBatches { .. } => {
            format!("{source} piped through sort to xargs -n echo can emit sorted matching token batches in-process")
        }
        GrepFilePipeMode::SortXargsWcLines | GrepFilePipeMode::SortXargsWcOutput { .. } => {
            format!("{source} piped through sort to xargs wc can line-count sorted matching path tokens in-process")
        }
        GrepFilePipeMode::XargsEcho => {
            format!("{source} piped to xargs echo can batch matching tokens in-process")
        }
        GrepFilePipeMode::XargsEchoBatches { .. } => {
            format!("{source} piped to xargs -n echo can emit matching token batches in-process")
        }
        GrepFilePipeMode::XargsWcLines | GrepFilePipeMode::XargsWcOutput { .. } => {
            format!("{source} piped to xargs wc can line-count matching path tokens in-process")
        }
    }
}

fn plan_tail_grep_producer_mode(
    downstream: &[String],
) -> Option<(String, GrepFilePipeMode, &'static str)> {
    if let Some(mode) = plan_grep_xargs_wc_line_producer_mode(downstream) {
        return Some(mode);
    }
    if downstream.len() >= 4
        && downstream[0] == "grep"
        && !downstream[1].is_empty()
        && is_plain_literal_pattern(&downstream[1])
        && downstream[2] == "|"
    {
        if let Some((mode, reason)) = plan_tail_producer_mode(&downstream[3..]) {
            if matches!(
                mode,
                GrepFilePipeMode::XargsEcho
                    | GrepFilePipeMode::XargsEchoBatches { .. }
                    | GrepFilePipeMode::SortXargsEcho
                    | GrepFilePipeMode::SortXargsEchoBatches { .. }
            ) {
                return Some((downstream[1].clone(), mode, reason));
            }
        }
    }
    match downstream {
        [cmd, pattern] if cmd == "grep" && !pattern.is_empty() && is_plain_literal_pattern(pattern) => Some((
            pattern.clone(),
            GrepFilePipeMode::Lines,
            "tail output piped to grep can filter emitted lines in-process",
        )),
        [cmd, pattern, pipe_word, count_cmd, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                pattern.clone(),
                wc_pipe_mode(mode),
                "tail output piped through grep to wc can count filtered emitted output in-process",
            ))
        }
        [cmd, pattern, pipe_word, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && limit_cmd == "head"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                pattern.clone(),
                GrepFilePipeMode::Head { limit },
                "tail output piped through grep to head can emit the filtered prefix in-process",
            ))
        }
        [cmd, pattern, pipe_word, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && limit_cmd == "tail"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some((
                pattern.clone(),
                GrepFilePipeMode::Tail { limit },
                "tail output piped through grep to tail can emit the filtered suffix in-process",
            ))
        }
        [cmd, pattern, pipe_word, sort_cmd]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && sort_cmd == "sort" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::Sort,
                "tail output piped through grep to sort can sort filtered emitted lines in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && uniq_cmd == "uniq" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::SortUniq,
                "tail output piped through grep, sort, and uniq can de-duplicate filtered emitted lines in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd, pipe_c, count_cmd, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && uniq_cmd == "uniq"
                && pipe_c == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                pattern.clone(),
                sort_uniq_wc_pipe_mode(mode),
                "tail output piped through grep, sort, uniq, and wc can count unique filtered emitted output in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, count_cmd, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && count_cmd == "wc" =>
        {
            let mode = WcCountMode::from_flag(flag)?;
            Some((
                pattern.clone(),
                sort_wc_pipe_mode(mode),
                "tail output piped through grep, sort, and wc can count filtered emitted output in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && limit_cmd == "head"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            (limit > 0).then_some((
                pattern.clone(),
                GrepFilePipeMode::SortHead { limit },
                "tail output piped through grep, sort, and head can emit the sorted filtered prefix in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && limit_cmd == "tail"
                && flag == "-n" =>
        {
            let limit = limit.parse().ok()?;
            Some((
                pattern.clone(),
                GrepFilePipeMode::SortTail { limit },
                "tail output piped through grep, sort, and tail can emit the sorted filtered suffix in-process",
            ))
        }
        [cmd, pattern, pipe_a, sort_cmd, pipe_b, subcmd, arg, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_a == "|"
                && sort_cmd == "sort"
                && pipe_b == "|"
                && subcmd == "xargs"
                && arg == "wc"
                && flag == "-l" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::SortXargsWcLines,
                "tail output piped through grep, sort, and xargs wc -l can line-count sorted filtered path tokens in-process",
            ))
        }
        [cmd, pattern, pipe_word, subcmd, arg, flag]
            if cmd == "grep"
                && !pattern.is_empty()
                && is_plain_literal_pattern(pattern)
                && pipe_word == "|"
                && subcmd == "xargs"
                && arg == "wc"
                && flag == "-l" =>
        {
            Some((
                pattern.clone(),
                GrepFilePipeMode::XargsWcLines,
                "tail output piped through grep to xargs wc -l can line-count filtered path tokens in-process",
            ))
        }
        _ => None,
    }
}

/// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
impl SingleLineProducerSource {
    fn pipe_name(&self) -> &'static str {
        match self {
            SingleLineProducerSource::Pwd => "pwd",
            SingleLineProducerSource::Basename(_) => "basename",
            SingleLineProducerSource::Dirname(_) => "dirname",
            SingleLineProducerSource::Whoami => "whoami",
            SingleLineProducerSource::Id(_) => "id",
            SingleLineProducerSource::Uname(_) => "uname",
            SingleLineProducerSource::Hostname => "hostname",
            SingleLineProducerSource::PrintenvName(_) => "printenv",
        }
    }
}

fn parse_single_line_pipe_source(left: &[String]) -> Option<SingleLineProducerSource> {
    match left {
        [cmd] if cmd == "pwd" => Some(SingleLineProducerSource::Pwd),
        [cmd] if cmd == "whoami" => Some(SingleLineProducerSource::Whoami),
        [cmd] if cmd == "hostname" => Some(SingleLineProducerSource::Hostname),
        [cmd, name] if cmd == "printenv" && !name.starts_with('-') => {
            Some(SingleLineProducerSource::PrintenvName(name.clone()))
        }
        [cmd] if cmd == "uname" => Some(SingleLineProducerSource::Uname(UnamePlan {
            field: UnameField::Sysname,
        })),
        [cmd, flag] if cmd == "uname" => {
            let field = match flag.as_str() {
                "-s" => UnameField::Sysname,
                "-n" => UnameField::Nodename,
                "-r" => UnameField::Release,
                "-v" => UnameField::Version,
                "-m" => UnameField::Machine,
                "-p" => UnameField::Processor,
                "-a" => UnameField::All,
                _ => return None,
            };
            Some(SingleLineProducerSource::Uname(UnamePlan { field }))
        }
        [cmd, flag] if cmd == "id" => {
            let kind = match flag.as_str() {
                "-u" => IdKind::UserId,
                "-un" => IdKind::UserName,
                "-g" => IdKind::GroupId,
                "-gn" => IdKind::GroupName,
                "-G" => IdKind::GroupIds,
                "-Gn" => IdKind::GroupNames,
                _ => return None,
            };
            Some(SingleLineProducerSource::Id(IdPlan { kind }))
        }
        [cmd] if cmd == "id" => Some(SingleLineProducerSource::Id(IdPlan {
            kind: IdKind::Default,
        })),
        [cmd, path] if cmd == "basename" && !path.starts_with('-') => {
            Some(SingleLineProducerSource::Basename(BasenamePlan {
                path: path.clone(),
                suffix: None,
            }))
        }
        [cmd, path, suffix]
            if cmd == "basename" && !path.starts_with('-') && !suffix.starts_with('-') =>
        {
            Some(SingleLineProducerSource::Basename(BasenamePlan {
                path: path.clone(),
                suffix: Some(suffix.clone()),
            }))
        }
        [cmd, path] if cmd == "dirname" && !path.starts_with('-') => {
            Some(SingleLineProducerSource::Dirname(DirnamePlan {
                path: path.clone(),
            }))
        }
        _ => None,
    }
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_sed_pipe_source(left: &[String]) -> Option<SedPrintPlan> {
    if left.len() != 4 || left[0] != "sed" || left[1] != "-n" {
        return None;
    }
    let (start_line, end_line) = parse_sed_print_script(&left[2])?;
    let path = Path::new(&left[3]);
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    Some(SedPrintPlan {
        file: left[3].clone(),
        start_line,
        end_line,
    })
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_cat_sed_direct_source(words: &[String]) -> Option<SedPrintPlan> {
    if words.len() != 6
        || words[0] != "cat"
        || words[2] != "|"
        || words[3] != "sed"
        || words[4] != "-n"
    {
        return None;
    }
    let (start_line, end_line) = parse_sed_print_script(&words[5])?;
    let meta = fs::metadata(&words[1]).ok()?;
    if !meta.is_file() {
        return None;
    }
    Some(SedPrintPlan {
        file: words[1].clone(),
        start_line,
        end_line,
    })
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_cat_sed_pipe_source(words: &[String]) -> Option<(SedPrintPlan, usize)> {
    if words.len() < 8
        || words[0] != "cat"
        || words[2] != "|"
        || words[3] != "sed"
        || words[4] != "-n"
        || words[6] != "|"
    {
        return None;
    }
    let (start_line, end_line) = parse_sed_print_script(&words[5])?;
    let meta = fs::metadata(&words[1]).ok()?;
    if !meta.is_file() {
        return None;
    }
    Some((
        SedPrintPlan {
            file: words[1].clone(),
            start_line,
            end_line,
        },
        7,
    ))
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_awk_print_field_pipe_source(
    words: &[String],
) -> Option<(String, Option<String>, usize, usize)> {
    if words.len() >= 4 && words[0] == "awk" && words[2] == "|" {
        let (pattern, field) = parse_awk_print_field_script(&words[1])?;
        return Some((String::new(), pattern, field, 3));
    }

    if words.len() >= 5 && words[0] == "awk" && words[3] == "|" && Path::new(&words[2]).is_file() {
        let (pattern, field) = parse_awk_print_field_script(&words[1])?;
        return Some((words[2].clone(), pattern, field, 4));
    }

    if words.len() == 5
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "awk"
        && Path::new(&words[1]).is_file()
    {
        let (pattern, field) = parse_awk_print_field_script(&words[4])?;
        return Some((words[1].clone(), pattern, field, 5));
    }

    if words.len() >= 7
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "awk"
        && words[5] == "|"
        && Path::new(&words[1]).is_file()
    {
        let (pattern, field) = parse_awk_print_field_script(&words[4])?;
        return Some((words[1].clone(), pattern, field, 6));
    }

    None
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_cat_head_tail_pipe_source(
    words: &[String],
    command: &str,
    require_positive: bool,
) -> Option<(String, u64, Option<usize>)> {
    if words.len() < 4 || words[0] != "cat" || words[2] != "|" || words[3] != command {
        return None;
    }
    let meta = fs::metadata(&words[1]).ok()?;
    if !meta.is_file() {
        return None;
    }

    let validate_limit = |limit| {
        if require_positive && limit == 0 {
            None
        } else {
            Some(limit)
        }
    };

    if words.len() == 4 {
        return Some((words[1].clone(), 10, None));
    }
    if words.len() >= 6 && words[4] == "|" {
        return Some((words[1].clone(), 10, Some(5)));
    }
    if words[4] == "-n" {
        if words.len() < 6 {
            return None;
        }
        let limit = validate_limit(words[5].parse().ok()?)?;
        if words.len() == 6 {
            return Some((words[1].clone(), limit, None));
        }
        if words.len() >= 8 && words[6] == "|" {
            return Some((words[1].clone(), limit, Some(7)));
        }
    }
    if let Some(limit) = words[4].strip_prefix('-') {
        if !limit.is_empty() && limit.chars().all(|ch| ch.is_ascii_digit()) {
            let limit = validate_limit(limit.parse().ok()?)?;
            if words.len() == 5 {
                return Some((words[1].clone(), limit, None));
            }
            if words.len() >= 7 && words[5] == "|" {
                return Some((words[1].clone(), limit, Some(6)));
            }
        }
    }

    None
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_cut_pipe_source(left: &[String]) -> Option<CutPlan> {
    if left.first().map(|cmd| cmd != "cut").unwrap_or(true) {
        return None;
    }
    let cut = parse_cut_args(&left[1..])?;
    if cut.stdin {
        return Some(cut);
    }
    if !fs::metadata(&cut.file).ok()?.is_file() {
        return None;
    }
    Some(cut)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_cat_cut_pipe_source(words: &[String]) -> Option<(CutPlan, usize)> {
    if words.len() < 7 || words[0] != "cat" || words[2] != "|" || words[3] != "cut" {
        return None;
    }
    let downstream_pipe = words[4..].iter().position(|word| word == "|")? + 4;
    if downstream_pipe + 1 >= words.len() {
        return None;
    }
    let cut = parse_cut_args_with_file(&words[4..downstream_pipe], Some(words[1].clone()))?;
    let meta = fs::metadata(&cut.file).ok()?;
    if !meta.is_file() {
        return None;
    }
    Some((cut, downstream_pipe + 1))
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_cat_tr_pipe_source(words: &[String]) -> Option<(String, TrPlan, usize)> {
    if words.len() < 7 || words[0] != "cat" || words[2] != "|" || words[3] != "tr" {
        return None;
    }
    let downstream_pipe = words[4..].iter().position(|word| word == "|")? + 4;
    if downstream_pipe + 1 >= words.len() {
        return None;
    }
    let file = words[1].clone();
    let meta = fs::metadata(&file).ok()?;
    if !meta.is_file() {
        return None;
    }
    let tr = parse_tr_args(&words[4..downstream_pipe])?;
    Some((file, tr, downstream_pipe + 1))
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_cat_uniq_pipe_source(words: &[String]) -> Option<(String, usize)> {
    if words.len() < 6
        || words[0] != "cat"
        || words[2] != "|"
        || words[3] != "uniq"
        || words[4] != "|"
    {
        return None;
    }
    let file = words[1].clone();
    let meta = fs::metadata(&file).ok()?;
    if !meta.is_file() {
        return None;
    }
    Some((file, 5))
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_uniq_pipe_source(words: &[String]) -> Option<(String, usize)> {
    if words.len() < 4 || words[0] != "uniq" || words[2] != "|" {
        return None;
    }
    let file = words[1].clone();
    let meta = fs::metadata(&file).ok()?;
    if !meta.is_file() {
        return None;
    }
    Some((file, 3))
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_sort_uniq_pipe_source(words: &[String]) -> Option<(String, usize)> {
    if words.len() >= 6
        && words[0] == "sort"
        && words[2] == "|"
        && words[3] == "uniq"
        && words[4] == "|"
    {
        let file = words[1].clone();
        if fs::metadata(&file).ok()?.is_file() {
            return Some((file, 5));
        }
    }
    if words.len() >= 8
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && words[5] == "uniq"
        && words[6] == "|"
    {
        let file = words[1].clone();
        if fs::metadata(&file).ok()?.is_file() {
            return Some((file, 7));
        }
    }
    None
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_sort_pipe_source(words: &[String]) -> Option<(String, usize)> {
    if words.len() >= 4 && words[0] == "sort" && words[2] == "|" {
        let file = words[1].clone();
        if fs::metadata(&file).ok()?.is_file() {
            return Some((file, 3));
        }
    }
    if words.len() >= 6
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
    {
        let file = words[1].clone();
        if fs::metadata(&file).ok()?.is_file() {
            return Some((file, 5));
        }
    }
    None
}

fn parse_xargs_echo_pipe_source(left: &[String]) -> Option<XargsEchoMode> {
    parse_xargs_echo_command(left)
}

fn parse_empty_output_pipe_source(left: &[String]) -> bool {
    matches!(left, [cmd] if cmd == "true" || cmd == "false")
}

fn parse_side_effect_empty_pipe_source(left: &[String]) -> Option<SideEffectEmptyProducer> {
    let (command, args) = left.split_first()?;
    match command.as_str() {
        "mkdir" => parse_mkdir_args(args).map(SideEffectEmptyProducer::Mkdir),
        "touch" => parse_touch_args(args).map(SideEffectEmptyProducer::Touch),
        _ => None,
    }
}

fn parse_predicate_empty_pipe_source(left: &[String]) -> Option<TestPlan> {
    let (command, args) = left.split_first()?;
    let args = if command == "[" {
        match args.split_last() {
            Some((last, rest)) if last == "]" => rest,
            _ => return None,
        }
    } else if command == "test" {
        args
    } else {
        return None;
    };
    parse_test_args(args)
}

fn parse_printf_literal_pipe_source(left: &[String]) -> Option<PrintfLiteralPlan> {
    let (command, args) = left.split_first()?;
    if command != "printf" {
        return None;
    }
    parse_printf_literal_args(args)
}

fn parse_wc_pipe_source(left: &[String]) -> Option<WcLinesPlan> {
    let (command, args) = left.split_first()?;
    (command == "wc").then_some(())?;
    parse_wc_args(args, true)
}

fn parse_du_pipe_source(left: &[String]) -> Option<DuSkPlan> {
    match left {
        [cmd, flag, path] if cmd == "du" && flag == "-sk" && !path.starts_with('-') => {
            fs::symlink_metadata(path).ok()?;
            Some(DuSkPlan { path: path.clone() })
        }
        _ => None,
    }
}

fn plan_pipe_words(words: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if let Some(plan) = plan_grep_file_cut_pipe(words, label.clone(), original) {
        return Some(plan);
    }
    if let Some(plan) = plan_grep_file_awk_pipe(words, label.clone(), original) {
        return Some(plan);
    }
    if let Some(plan) = plan_finite_awk_pipe(words, label.clone(), original) {
        return Some(plan);
    }

    if let Some(pipe) = words.iter().position(|word| word == "|") {
        if pipe > 0 && pipe + 1 < words.len() {
            if parse_empty_output_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((_pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeEmptyProducer(PipeEmptyProducerPlan { mode }),
                        label,
                        original: original.to_string(),
                        reason:
                            "empty primitive output piped through grep/downstream can be fused in-process"
                                .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeEmptyProducer(PipeEmptyProducerPlan { mode }),
                        label,
                        original: original.to_string(),
                        reason:
                            "empty primitive output piped to a supported downstream can be fused in-process"
                                .to_string(),
                        });
                }
            }
            if let Some(source) = parse_side_effect_empty_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((_pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSideEffectEmptyProducer(
                            PipeSideEffectEmptyProducerPlan { source, mode },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "side-effect command with empty stdout piped through grep/downstream can run in-process"
                                .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSideEffectEmptyProducer(
                            PipeSideEffectEmptyProducerPlan { source, mode },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "side-effect command with empty stdout piped to a supported downstream can run in-process"
                                .to_string(),
                    });
                }
            }
            if let Some(test) = parse_predicate_empty_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((_pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipePredicateEmptyProducer(
                            PipePredicateEmptyProducerPlan { test, mode },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "predicate command with empty stdout piped through grep/downstream can run in-process"
                                .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipePredicateEmptyProducer(
                            PipePredicateEmptyProducerPlan { test, mode },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "predicate command with empty stdout piped to a supported downstream can run in-process"
                                .to_string(),
                        });
                }
            }
            if let Some(printf) = parse_printf_literal_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipePrintfLiteralProducer(
                            PipePrintfLiteralProducerPlan {
                                printf,
                                pattern: Some(pattern),
                                mode,
                            },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "literal printf output piped through grep/downstream can run in-process"
                                .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipePrintfLiteralProducer(
                            PipePrintfLiteralProducerPlan {
                                printf,
                                pattern: None,
                                mode,
                            },
                        ),
                        label,
                        original: original.to_string(),
                        reason: "literal printf output piped to a supported downstream can run in-process"
                            .to_string(),
                    });
                }
            }
            if let Some(wc) = parse_wc_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeWcProducer(PipeWcProducerPlan {
                            wc,
                            pattern: Some(pattern),
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "wc output piped through grep/downstream can run in-process"
                            .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeWcProducer(PipeWcProducerPlan {
                            wc,
                            pattern: None,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "wc output piped to a supported downstream can run in-process"
                            .to_string(),
                    });
                }
            }
            if let Some(du) = parse_du_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeDuProducer(PipeDuProducerPlan {
                            du,
                            pattern: Some(pattern),
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "du -sk output piped through grep/downstream can run in-process"
                            .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeDuProducer(PipeDuProducerPlan {
                            du,
                            pattern: None,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "du -sk output piped to a supported downstream can run in-process"
                            .to_string(),
                    });
                }
            }
            if let Some(lookup) = parse_path_lookup_pipe_left(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipePathLookupGrepProducer(
                            PipePathLookupGrepProducerPlan {
                                lookup,
                                pattern,
                                mode,
                            },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "path lookup output piped through literal grep/downstream can be fused in-process"
                                .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipePathLookupProducer(PipePathLookupProducerPlan {
                            lookup,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "path lookup output piped to a supported downstream can be fused in-process"
                                .to_string(),
                    });
                }
            }
            if let Some(env) = parse_environment_pipe_left(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeEnvironmentGrepProducer(
                            PipeEnvironmentGrepProducerPlan { env, pattern, mode },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "environment output piped through literal grep/downstream can be fused in-process"
                            .to_string(),
                    });
                }
            }
            if parse_hostname_pipe_left(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeHostnameGrepProducer(
                            PipeHostnameGrepProducerPlan { pattern, mode },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "hostname output piped through literal grep/downstream can be fused in-process"
                                .to_string(),
                    });
                }
            }
        }
    }

    if let Some(sed) = parse_cat_sed_direct_source(words) {
        return Some(NativePlan {
            command: NativeCommand::SedPrint(sed),
            label,
            original: original.to_string(),
            reason: "cat piped to sed -n line print can be served as an in-process ranged read"
                .to_string(),
        });
    }

    if let Some((sed, downstream_start)) = parse_cat_sed_pipe_source(words) {
        let downstream = &words[downstream_start..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeSedGrepProducer(PipeSedGrepProducerPlan {
                    sed,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "cat/sed output piped through grep/downstream can be fused in-process"
                    .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeSedProducer(PipeSedProducerPlan { sed, mode }),
                label,
                original: original.to_string(),
                reason: "cat/sed output piped to a supported downstream can be fused in-process"
                    .to_string(),
            });
        }
    }

    if let Some(pipe) = words.iter().position(|word| word == "|") {
        if pipe > 0 && pipe + 1 < words.len() {
            if let Some(sed) = parse_sed_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSedGrepProducer(PipeSedGrepProducerPlan {
                            sed,
                            pattern,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "sed -n output piped through grep/downstream can be fused in-process"
                                .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSedProducer(PipeSedProducerPlan { sed, mode }),
                        label,
                        original: original.to_string(),
                        reason:
                            "sed -n output piped to a supported downstream can be fused in-process"
                                .to_string(),
                    });
                }
            }
        }
    }

    if let Some(pipe) = words.iter().position(|word| word == "|") {
        if pipe > 0 && pipe + 1 < words.len() {
            if let Some(cut) = parse_cut_pipe_source(&words[..pipe]) {
                let downstream = &words[pipe + 1..];
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeCutGrepProducer(PipeCutGrepProducerPlan {
                            cut,
                            pattern,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "cut output piped through grep/downstream can be fused in-process"
                            .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeCutProducer(PipeCutProducerPlan { cut, mode }),
                        label,
                        original: original.to_string(),
                        reason:
                            "cut output piped to a supported downstream can be fused in-process"
                                .to_string(),
                    });
                }
            }
        }
    }

    if let Some((cut, downstream_start)) = parse_cat_cut_pipe_source(words) {
        let downstream = &words[downstream_start..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeCutGrepProducer(PipeCutGrepProducerPlan {
                    cut,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "cat/cut output piped through grep/downstream can be fused in-process"
                    .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeCutProducer(PipeCutProducerPlan { cut, mode }),
                label,
                original: original.to_string(),
                reason: "cat/cut output piped to a supported downstream can be fused in-process"
                    .to_string(),
            });
        }
    }

    if let Some((file, tr, downstream_start)) = parse_cat_tr_pipe_source(words) {
        let downstream = &words[downstream_start..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeCatTrGrepProducer(PipeCatTrGrepProducerPlan {
                    file,
                    tr,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "cat/tr output piped through grep/downstream can be fused in-process"
                    .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeCatTrProducer(PipeCatTrProducerPlan { file, tr, mode }),
                label,
                original: original.to_string(),
                reason: "cat/tr output piped to a supported downstream can be fused in-process"
                    .to_string(),
            });
        }
    }

    if let Some((file, downstream_start)) = parse_cat_uniq_pipe_source(words) {
        let downstream = &words[downstream_start..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeCatUniqGrepProducer(PipeCatUniqGrepProducerPlan {
                    file,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "cat/uniq output piped through grep/downstream can be fused in-process"
                    .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeCatUniqProducer(PipeCatUniqProducerPlan { file, mode }),
                label,
                original: original.to_string(),
                reason: "cat/uniq output piped to a supported downstream can be fused in-process"
                    .to_string(),
            });
        }
    }

    if let Some((file, downstream_start)) = parse_uniq_pipe_source(words) {
        let downstream = &words[downstream_start..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeUniqGrepProducer(PipeUniqGrepProducerPlan {
                    file,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "uniq output piped through grep/downstream can be fused in-process"
                    .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeUniqProducer(PipeUniqProducerPlan { file, mode }),
                label,
                original: original.to_string(),
                reason: "uniq output piped to a supported downstream can be fused in-process"
                    .to_string(),
            });
        }
    }

    if let Some((file, downstream_start)) = parse_sort_uniq_pipe_source(words) {
        let downstream = &words[downstream_start..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeSortUniqGrepProducer(PipeSortUniqGrepProducerPlan {
                    file,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "sort/uniq output piped through grep/downstream can be fused in-process"
                    .to_string(),
            });
        }
        if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeSortUniqProducer(PipeSortUniqProducerPlan {
                    file,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "sort/uniq output piped to a supported downstream can be fused in-process"
                    .to_string(),
            });
        }
    }

    if let Some((file, downstream_start)) = parse_sort_pipe_source(words) {
        let downstream = &words[downstream_start..];
        if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeSortGrepProducer(PipeSortGrepProducerPlan {
                    file,
                    pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "sort output piped through grep/downstream can be fused in-process"
                    .to_string(),
            });
        }
    }

    if let Some(pipe) = words.iter().position(|word| word == "|") {
        if pipe > 0 && pipe + 1 < words.len() {
            if let Some(source_mode) = parse_xargs_echo_pipe_source(&words[..pipe]) {
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(&words[pipe + 1..]) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeXargsEchoProducer(PipeXargsEchoProducerPlan {
                            mode,
                            grep: Some(pattern),
                            source_mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "xargs echo output piped through grep/downstream can be fused in-process"
                                .to_string(),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(&words[pipe + 1..]) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeXargsEchoProducer(PipeXargsEchoProducerPlan {
                            mode,
                            grep: None,
                            source_mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "xargs echo output piped to a supported downstream can be fused in-process"
                                .to_string(),
                    });
                }
            }
        }
    }

    if let Some(pipe) = words.iter().position(|word| word == "|") {
        if pipe > 0 && pipe + 1 < words.len() {
            let source = parse_single_line_pipe_source(&words[..pipe]);
            if let Some(source) = source {
                let downstream = &words[pipe + 1..];
                let source_name = source.pipe_name();
                if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSingleLineGrepProducer(
                            PipeSingleLineGrepProducerPlan {
                                source,
                                pattern,
                                mode,
                            },
                        ),
                        label,
                        original: original.to_string(),
                        reason: format!(
                            "{source_name} output piped through grep/downstream can be fused in-process"
                        ),
                    });
                }
                if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSingleLineProducer(
                            PipeSingleLineProducerPlan { source, mode },
                        ),
                        label,
                        original: original.to_string(),
                        reason: format!(
                            "{source_name} output piped to a supported downstream can be fused in-process"
                        ),
                    });
                }
            }
        }
    }

    if words.first().is_some_and(|word| word == "head") {
        if let Some(pipe) = words.iter().position(|word| word == "|") {
            if pipe > 0 && pipe + 1 < words.len() {
                let head = parse_head_tail_args(&words[1..pipe])?;
                if head.mode == HeadTailMode::Lines && head.count > 0 {
                    let downstream = &words[pipe + 1..];
                    if let Some((pattern, mode, reason)) = plan_head_grep_producer_mode(downstream)
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipeHeadGrepProducer(
                                PipeHeadGrepProducerPlan {
                                    file: head.file,
                                    stdin: head.stdin,
                                    limit: head.count,
                                    pattern,
                                    mode,
                                },
                            ),
                            label,
                            original: original.to_string(),
                            reason: reason.to_string(),
                        });
                    }
                    if let Some((mode, reason)) = plan_head_producer_mode(downstream) {
                        return Some(NativePlan {
                            command: NativeCommand::PipeHeadProducer(PipeHeadProducerPlan {
                                file: head.file,
                                stdin: head.stdin,
                                limit: head.count,
                                mode,
                            }),
                            label,
                            original: original.to_string(),
                            reason: reason.to_string(),
                        });
                    }
                }
            }
        }
    }

    if words.first().is_some_and(|word| word == "tail") {
        if let Some(pipe) = words.iter().position(|word| word == "|") {
            if pipe > 0 && pipe + 1 < words.len() {
                let tail = parse_head_tail_args(&words[1..pipe])?;
                if tail.mode == HeadTailMode::Lines {
                    let downstream = &words[pipe + 1..];
                    if let Some((pattern, mode, reason)) = plan_tail_grep_producer_mode(downstream)
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipeTailGrepProducer(
                                PipeTailGrepProducerPlan {
                                    file: tail.file,
                                    stdin: tail.stdin,
                                    limit: tail.count,
                                    pattern,
                                    mode,
                                },
                            ),
                            label,
                            original: original.to_string(),
                            reason: reason.to_string(),
                        });
                    }
                    if let Some((mode, reason)) = plan_tail_producer_mode(downstream) {
                        return Some(NativePlan {
                            command: NativeCommand::PipeTailProducer(PipeTailProducerPlan {
                                file: tail.file,
                                stdin: tail.stdin,
                                limit: tail.count,
                                mode,
                            }),
                            label,
                            original: original.to_string(),
                            reason: reason.to_string(),
                        });
                    }
                }
            }
        }
    }

    if words.first().is_some_and(|word| word == "printf") {
        if let Some(pipe) = words.iter().position(|word| word == "|") {
            if pipe > 0 && pipe + 1 < words.len() {
                let printf = parse_printf_args(&words[1..pipe])?;
                if printf.format == PrintfFormat::StringNewline {
                    let downstream = &words[pipe + 1..];
                    if let [grep_cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd, pipe_c, rest @ ..] =
                        downstream
                    {
                        if grep_cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_a == "|"
                            && sort_cmd == "sort"
                            && pipe_b == "|"
                            && uniq_cmd == "uniq"
                            && pipe_c == "|"
                        {
                            if let Some((mode, _)) = plan_tail_producer_mode(rest) {
                                return Some(NativePlan {
                                    command: NativeCommand::PipePrintfGrepSortUniqProducer(
                                        PipePrintfGrepSortUniqProducerPlan {
                                            args: printf.args,
                                            pattern: pattern.clone(),
                                            mode,
                                        },
                                    ),
                                    label,
                                    original: original.to_string(),
                                    reason:
                                        "printf %s\\n piped through grep, sort, uniq, and a downstream producer can run in-process"
                                            .to_string(),
                                });
                            }
                        }
                    }
                    let printf_grep = match downstream {
                        [cmd, pattern, pipe_word, count_cmd, flag]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_word == "|"
                                && count_cmd == "wc" =>
                        {
                            let mode = WcCountMode::from_flag(flag)?;
                            Some((
                                pattern.clone(),
                                wc_pipe_mode(mode),
                                "printf %s\\n piped through grep to wc can count filtered generated output in-process",
                            ))
                        }
                        [cmd, pattern, pipe_word, limit_cmd, flag, limit]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_word == "|"
                                && limit_cmd == "head"
                                && flag == "-n" =>
                        {
                            let limit = limit.parse().ok()?;
                            (limit > 0).then_some((
                                pattern.clone(),
                                GrepFilePipeMode::Head { limit },
                                "printf %s\\n piped through grep to head can emit the filtered generated prefix in-process",
                            ))
                        }
                        [cmd, pattern, pipe_word, limit_cmd, flag, limit]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_word == "|"
                                && limit_cmd == "tail"
                                && flag == "-n" =>
                        {
                            let limit = limit.parse().ok()?;
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::Tail { limit },
                                "printf %s\\n piped through grep to tail can emit the filtered generated suffix in-process",
                            ))
                        }
                        [cmd, pattern, pipe_word, sort_cmd]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_word == "|"
                                && sort_cmd == "sort" =>
                        {
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::Sort,
                                "printf %s\\n piped through grep to sort can sort filtered generated lines in-process",
                            ))
                        }
                        [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_a == "|"
                                && sort_cmd == "sort"
                                && pipe_b == "|"
                                && uniq_cmd == "uniq" =>
                        {
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::SortUniq,
                                "printf %s\\n piped through grep, sort, and uniq can de-duplicate filtered generated lines in-process",
                            ))
                        }
                        [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd, pipe_c, count_cmd, flag]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_a == "|"
                                && sort_cmd == "sort"
                                && pipe_b == "|"
                                && uniq_cmd == "uniq"
                                && pipe_c == "|"
                                && count_cmd == "wc"
                                && flag == "-l" =>
                        {
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::SortUniqWcLines,
                                "printf %s\\n piped through grep, sort, uniq, and wc -l can count unique filtered generated lines in-process",
                            ))
                        }
                        [cmd, pattern, pipe_a, sort_cmd, pipe_b, count_cmd, flag]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_a == "|"
                                && sort_cmd == "sort"
                                && pipe_b == "|"
                                && count_cmd == "wc"
                                && flag == "-l" =>
                        {
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::SortWcLines,
                                "printf %s\\n piped through grep, sort, and wc -l can count filtered generated lines in-process",
                            ))
                        }
                        [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_a == "|"
                                && sort_cmd == "sort"
                                && pipe_b == "|"
                                && limit_cmd == "head"
                                && flag == "-n" =>
                        {
                            let limit = limit.parse().ok()?;
                            (limit > 0).then_some((
                                pattern.clone(),
                                GrepFilePipeMode::SortHead { limit },
                                "printf %s\\n piped through grep, sort, and head can emit the sorted filtered generated prefix in-process",
                            ))
                        }
                        [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_a == "|"
                                && sort_cmd == "sort"
                                && pipe_b == "|"
                                && limit_cmd == "tail"
                                && flag == "-n" =>
                        {
                            let limit = limit.parse().ok()?;
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::SortTail { limit },
                                "printf %s\\n piped through grep, sort, and tail can emit the sorted filtered generated suffix in-process",
                            ))
                        }
                        [cmd, pattern, pipe_a, sort_cmd, pipe_b, subcmd, arg]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_a == "|"
                                && sort_cmd == "sort"
                                && pipe_b == "|"
                                && subcmd == "xargs"
                                && arg == "echo" =>
                        {
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::SortXargsEcho,
                                "printf %s\\n piped through grep, sort, and xargs echo can batch sorted filtered generated tokens in-process",
                            ))
                        }
                        [cmd, pattern, pipe_word, subcmd, arg]
                            if cmd == "grep"
                                && !pattern.is_empty()
                                && is_plain_literal_pattern(pattern)
                                && pipe_word == "|"
                                && subcmd == "xargs"
                                && arg == "echo" =>
                        {
                            Some((
                                pattern.clone(),
                                GrepFilePipeMode::XargsEcho,
                                "printf %s\\n piped through grep to xargs echo can batch filtered generated tokens in-process",
                            ))
                        }
                        _ => None,
                    };
                    if let Some((pattern, mode, reason)) = printf_grep {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfGrepProducer(
                                PipePrintfGrepProducerPlan {
                                    args: printf.args,
                                    pattern,
                                    mode,
                                },
                            ),
                            label,
                            original: original.to_string(),
                            reason: reason.to_string(),
                        });
                    }

                    if let [sort_cmd, pipe_a, uniq_cmd, pipe_b, rest @ ..] = downstream {
                        if sort_cmd == "sort"
                            && pipe_a == "|"
                            && uniq_cmd == "uniq"
                            && pipe_b == "|"
                        {
                            if let Some((mode, _)) = plan_tail_producer_mode(rest) {
                                return Some(NativePlan {
                                    command: NativeCommand::PipePrintfSortUniqProducer(
                                        PipePrintfSortUniqProducerPlan {
                                            args: printf.args,
                                            mode,
                                        },
                                    ),
                                    label,
                                    original: original.to_string(),
                                    reason:
                                        "printf %s\\n piped through sort, uniq, and a downstream producer can run in-process"
                                            .to_string(),
                                });
                            }
                        }
                    }

                    let planned = match downstream {
                        [cmd] if cmd == "sort" => Some((
                            GrepFilePipeMode::Sort,
                            "printf %s\\n piped to sort can sort generated lines in-process",
                        )),
                        [cmd, pipe_word, subcmd]
                            if cmd == "sort" && pipe_word == "|" && subcmd == "uniq" =>
                        {
                            Some((
                                GrepFilePipeMode::SortUniq,
                                "printf %s\\n piped through sort to uniq can de-duplicate generated lines in-process",
                            ))
                        }
                        [cmd, pipe_a, subcmd, pipe_b, count_cmd, flag]
                            if cmd == "sort"
                                && pipe_a == "|"
                                && subcmd == "uniq"
                                && pipe_b == "|"
                                && count_cmd == "wc"
                                && flag == "-l" =>
                        {
                            Some((
                                GrepFilePipeMode::SortUniqWcLines,
                                "printf %s\\n piped through sort and uniq to wc -l can count unique generated lines in-process",
                            ))
                        }
                        [cmd, pipe_word, count_cmd, flag]
                            if cmd == "sort"
                                && pipe_word == "|"
                                && count_cmd == "wc"
                                && flag == "-l" =>
                        {
                            Some((
                                GrepFilePipeMode::SortWcLines,
                                "printf %s\\n piped through sort to wc -l can count generated lines in-process",
                            ))
                        }
                        [cmd, pipe_word, limit_cmd, flag, limit]
                            if cmd == "sort"
                                && pipe_word == "|"
                                && limit_cmd == "head"
                                && flag == "-n" =>
                        {
                            let limit = limit.parse().ok()?;
                            (limit > 0).then_some((
                                GrepFilePipeMode::SortHead { limit },
                                "printf %s\\n piped through sort to head can emit the sorted generated prefix in-process",
                            ))
                        }
                        [cmd, pipe_word, limit_cmd, flag, limit]
                            if cmd == "sort"
                                && pipe_word == "|"
                                && limit_cmd == "tail"
                                && flag == "-n" =>
                        {
                            let limit = limit.parse().ok()?;
                            Some((
                                GrepFilePipeMode::SortTail { limit },
                                "printf %s\\n piped through sort to tail can emit the sorted generated suffix in-process",
                            ))
                        }
                        [cmd, pipe_word, rest @ ..]
                            if cmd == "sort"
                                && pipe_word == "|"
                                && parse_xargs_echo_command(rest).is_some() =>
                        {
                            let mode = sort_xargs_echo_pipe_mode(parse_xargs_echo_command(rest)?);
                            Some((
                                mode,
                                "printf %s\\n piped through sort to xargs echo can batch sorted generated tokens in-process",
                            ))
                        }
                        [cmd, pipe_word, subcmd, arg, flag]
                            if cmd == "sort"
                                && pipe_word == "|"
                                && subcmd == "xargs"
                                && arg == "wc"
                                && flag == "-l" =>
                        {
                            Some((
                                GrepFilePipeMode::SortXargsWcLines,
                                "printf %s\\n piped through sort to xargs wc -l can line-count sorted generated path tokens in-process",
                            ))
                        }
                        _ => None,
                    };
                    if let Some((mode, reason)) = planned {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode,
                            }),
                            label,
                            original: original.to_string(),
                            reason: reason.to_string(),
                        });
                    }
                }
            }
        }
    }

    if words.first().is_some_and(|word| word == "seq") {
        if let Some(pipe) = words.iter().position(|word| word == "|") {
            if pipe > 0 && pipe + 1 < words.len() {
                let seq = parse_seq_args(&words[1..pipe])?;
                let downstream = &words[pipe + 1..];
                if let [grep_cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd, pipe_c, rest @ ..] =
                    downstream
                {
                    if grep_cmd == "grep"
                        && !pattern.is_empty()
                        && is_plain_literal_pattern(pattern)
                        && pipe_a == "|"
                        && sort_cmd == "sort"
                        && pipe_b == "|"
                        && uniq_cmd == "uniq"
                        && pipe_c == "|"
                    {
                        if let Some((mode, _)) = plan_tail_producer_mode(rest) {
                            return Some(NativePlan {
                                command: NativeCommand::PipeSeqGrepSortUniqProducer(
                                    PipeSeqGrepSortUniqProducerPlan {
                                        seq,
                                        pattern: pattern.clone(),
                                        mode,
                                    },
                                ),
                                label,
                                original: original.to_string(),
                                reason:
                                    "integer seq piped through grep, sort, uniq, and a downstream producer can run in-process"
                                        .to_string(),
                            });
                        }
                    }
                }
                let seq_grep = match downstream {
                    [cmd, pattern]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern) =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::Lines,
                            "integer seq piped to grep can filter generated lines in-process",
                        ))
                    }
                    [cmd, pattern, pipe_word, count_cmd, flag]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_word == "|"
                            && count_cmd == "wc"
                            && flag == "-l" =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::WcLines,
                            "integer seq piped through grep to wc -l can count filtered lines in-process",
                        ))
                    }
                    [cmd, pattern, pipe_word, limit_cmd, flag, limit]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_word == "|"
                            && limit_cmd == "head"
                            && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        (limit > 0).then_some((
                            pattern.clone(),
                            GrepFilePipeMode::Head { limit },
                            "integer seq piped through grep to head can emit the filtered prefix in-process",
                        ))
                    }
                    [cmd, pattern, pipe_word, limit_cmd, flag, limit]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_word == "|"
                            && limit_cmd == "tail"
                            && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::Tail { limit },
                            "integer seq piped through grep to tail can emit the filtered suffix in-process",
                        ))
                    }
                    [cmd, pattern, pipe_word, sort_cmd]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_word == "|"
                            && sort_cmd == "sort" =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::Sort,
                            "integer seq piped through grep to sort can sort filtered lines in-process",
                        ))
                    }
                    [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_a == "|"
                            && sort_cmd == "sort"
                            && pipe_b == "|"
                            && uniq_cmd == "uniq" =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::SortUniq,
                            "integer seq piped through grep, sort, and uniq can de-duplicate filtered lines in-process",
                        ))
                    }
                    [cmd, pattern, pipe_a, sort_cmd, pipe_b, uniq_cmd, pipe_c, count_cmd, flag]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_a == "|"
                            && sort_cmd == "sort"
                            && pipe_b == "|"
                            && uniq_cmd == "uniq"
                            && pipe_c == "|"
                            && count_cmd == "wc"
                            && flag == "-l" =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::SortUniqWcLines,
                            "integer seq piped through grep, sort, uniq, and wc -l can count unique filtered lines in-process",
                        ))
                    }
                    [cmd, pattern, pipe_a, sort_cmd, pipe_b, count_cmd, flag]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_a == "|"
                            && sort_cmd == "sort"
                            && pipe_b == "|"
                            && count_cmd == "wc"
                            && flag == "-l" =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::SortWcLines,
                            "integer seq piped through grep, sort, and wc -l can count filtered lines in-process",
                        ))
                    }
                    [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_a == "|"
                            && sort_cmd == "sort"
                            && pipe_b == "|"
                            && limit_cmd == "head"
                            && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        (limit > 0).then_some((
                            pattern.clone(),
                            GrepFilePipeMode::SortHead { limit },
                            "integer seq piped through grep, sort, and head can emit the sorted filtered prefix in-process",
                        ))
                    }
                    [cmd, pattern, pipe_a, sort_cmd, pipe_b, limit_cmd, flag, limit]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_a == "|"
                            && sort_cmd == "sort"
                            && pipe_b == "|"
                            && limit_cmd == "tail"
                            && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::SortTail { limit },
                            "integer seq piped through grep, sort, and tail can emit the sorted filtered suffix in-process",
                        ))
                    }
                    [cmd, pattern, pipe_a, sort_cmd, pipe_b, subcmd, arg]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_a == "|"
                            && sort_cmd == "sort"
                            && pipe_b == "|"
                            && subcmd == "xargs"
                            && arg == "echo" =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::SortXargsEcho,
                            "integer seq piped through grep, sort, and xargs echo can batch sorted filtered tokens in-process",
                        ))
                    }
                    [cmd, pattern, pipe_word, subcmd, arg]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern)
                            && pipe_word == "|"
                            && subcmd == "xargs"
                            && arg == "echo" =>
                    {
                        Some((
                            pattern.clone(),
                            GrepFilePipeMode::XargsEcho,
                            "integer seq piped through grep to xargs echo can batch filtered tokens in-process",
                        ))
                    }
                    _ => None,
                };
                if let Some((pattern, mode, reason)) = seq_grep {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSeqGrepProducer(PipeSeqGrepProducerPlan {
                            seq,
                            pattern,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: reason.to_string(),
                    });
                }
                if let [sort_cmd, pipe_a, uniq_cmd, pipe_b, rest @ ..] = downstream {
                    if sort_cmd == "sort" && pipe_a == "|" && uniq_cmd == "uniq" && pipe_b == "|" {
                        if let Some((mode, _)) = plan_tail_producer_mode(rest) {
                            return Some(NativePlan {
                                command: NativeCommand::PipeSeqSortUniqProducer(
                                    PipeSeqSortUniqProducerPlan { seq, mode },
                                ),
                                label,
                                original: original.to_string(),
                                reason:
                                    "integer seq piped through sort, uniq, and a downstream producer can run in-process"
                                        .to_string(),
                            });
                        }
                    }
                }
                let planned = match downstream {
                    [cmd] if cmd == "sort" => Some((
                        GrepFilePipeMode::Sort,
                        "integer seq piped to sort can sort generated lines in-process",
                    )),
                    [cmd, pipe_word, subcmd]
                        if cmd == "sort" && pipe_word == "|" && subcmd == "uniq" =>
                    {
                        Some((
                            GrepFilePipeMode::SortUniq,
                            "integer seq piped through sort to uniq can de-duplicate generated lines in-process",
                        ))
                    }
                    [cmd, pipe_a, subcmd, pipe_b, count_cmd, flag]
                        if cmd == "sort"
                            && pipe_a == "|"
                            && subcmd == "uniq"
                            && pipe_b == "|"
                            && count_cmd == "wc"
                            && flag == "-l" =>
                    {
                        Some((
                            GrepFilePipeMode::SortUniqWcLines,
                            "integer seq piped through sort and uniq to wc -l can count unique generated lines in-process",
                        ))
                    }
                    [cmd, pipe_word, count_cmd, flag]
                        if cmd == "sort"
                            && pipe_word == "|"
                            && count_cmd == "wc"
                            && flag == "-l" =>
                    {
                        Some((
                            GrepFilePipeMode::SortWcLines,
                            "integer seq piped through sort to wc -l can count generated lines in-process",
                        ))
                    }
                    [cmd, pipe_word, limit_cmd, flag, limit]
                        if cmd == "sort"
                            && pipe_word == "|"
                            && limit_cmd == "head"
                            && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        (limit > 0).then_some((
                            GrepFilePipeMode::SortHead { limit },
                            "integer seq piped through sort to head can emit the sorted prefix in-process",
                        ))
                    }
                    [cmd, pipe_word, limit_cmd, flag, limit]
                        if cmd == "sort"
                            && pipe_word == "|"
                            && limit_cmd == "tail"
                            && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        Some((
                            GrepFilePipeMode::SortTail { limit },
                            "integer seq piped through sort to tail can emit the sorted suffix in-process",
                        ))
                    }
                    args
                        if args.len() >= 3
                            && args[0] == "sort"
                            && args[1] == "|"
                            && parse_xargs_echo_command(&args[2..]).is_some() =>
                    {
                        let xargs_mode = parse_xargs_echo_command(&args[2..])?;
                        Some((
                            sort_xargs_echo_pipe_mode(xargs_mode),
                            match xargs_mode {
                                XargsEchoMode::OneLine => {
                                    "integer seq piped through sort to xargs echo can batch sorted generated tokens in-process"
                                }
                                XargsEchoMode::Batch { .. } => {
                                    "integer seq piped through sort to xargs -n echo can emit sorted generated token batches in-process"
                                }
                            },
                        ))
                    }
                    _ => None,
                };
                if let Some((mode, reason)) = planned {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSeqProducer(PipeSeqProducerPlan { seq, mode }),
                        label,
                        original: original.to_string(),
                        reason: reason.to_string(),
                    });
                }
            }
        }
    }

    if let Some((left, right)) = split_single_pipe(words) {
        if left.first().is_some_and(|word| word == "echo") {
            let echo = parse_echo_args(&left[1..])?;
            if right.first().is_some_and(|word| word == "tr") {
                let tr = parse_tr_args(&right[1..])?;
                return Some(NativePlan {
                    command: NativeCommand::PipeEchoTr(PipeEchoTrPlan { echo, tr }),
                    label,
                    original: original.to_string(),
                    reason: "echo piped to tr can transform generated bytes in-process".to_string(),
                });
            }
            match right {
                [cmd, flag] if cmd == "wc" => {
                    let mode = WcCountMode::from_flag(flag)?;
                    let newline = echo.newline;
                    return Some(NativePlan {
                        command: NativeCommand::PipeEchoWcLines(PipeEchoWcLinesPlan {
                            echo,
                            newline,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "echo piped to wc can count generated output in-process"
                            .to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeEchoHead(PipeEchoHeadPlan { echo, limit }),
                            label,
                            original: original.to_string(),
                            reason:
                                "echo piped to head can forward the one generated line in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeEchoTail(PipeEchoTailPlan { echo, limit }),
                        label,
                        original: original.to_string(),
                        reason: "echo piped to tail can forward the generated line in-process"
                            .to_string(),
                    });
                }
                args if parse_xargs_echo_command(args).is_some() => {
                    let mode = parse_xargs_echo_command(args)?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeEchoXargsEcho(PipeEchoXargsEchoPlan {
                            echo,
                            mode,
                        }),
                        label,
                        original: original.to_string(),
                        reason: match mode {
                            XargsEchoMode::OneLine => {
                                "echo piped to xargs echo can batch generated tokens in-process"
                            }
                            XargsEchoMode::Batch { .. } => {
                                "echo piped to xargs -n echo can emit generated token batches in-process"
                            }
                        }
                        .to_string(),
                    });
                }
                [cmd, subcmd, flag] if cmd == "xargs" && subcmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeEchoXargsWcLines(PipeEchoXargsWcLinesPlan {
                            echo,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "echo piped to xargs wc -l can line-count generated path tokens in-process"
                            .to_string(),
                    });
                }
                _ => {}
            }
        }

        if left.first().is_some_and(|word| word == "printf") {
            let printf = parse_printf_args(&left[1..])?;
            if right.first().is_some_and(|word| word == "tr") {
                let tr = parse_tr_args(&right[1..])?;
                return Some(NativePlan {
                    command: NativeCommand::PipePrintfTr(PipePrintfTrPlan { printf, tr }),
                    label,
                    original: original.to_string(),
                    reason: "printf piped to tr can transform generated bytes in-process"
                        .to_string(),
                });
            }
            if printf.format == PrintfFormat::StringNewline {
                match right {
                    [cmd, flag] if cmd == "wc" => {
                        let mode = WcCountMode::from_flag(flag)?;
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfWcLines(PipePrintfWcLinesPlan {
                                args: printf.args,
                                mode,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped to wc can count generated output in-process"
                                    .to_string(),
                        });
                    }
                    [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                        let limit = limit.parse().ok()?;
                        if limit > 0 {
                            return Some(NativePlan {
                                command: NativeCommand::PipePrintfHead(PipePrintfHeadPlan {
                                    args: printf.args,
                                    limit,
                                }),
                                label,
                                original: original.to_string(),
                                reason:
                                    "printf %s\\n piped to head can emit the requested generated lines in-process"
                                        .to_string(),
                            });
                        }
                    }
                    [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                        let limit = limit.parse().ok()?;
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfTail(PipePrintfTailPlan {
                                args: printf.args,
                                limit,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped to tail can emit the requested generated suffix in-process"
                                    .to_string(),
                        });
                    }
                    [cmd, pattern]
                        if cmd == "grep"
                            && !pattern.is_empty()
                            && is_plain_literal_pattern(pattern) =>
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfGrep(PipePrintfGrepPlan {
                                args: printf.args,
                                pattern: pattern.clone(),
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped to grep can filter generated lines in-process"
                                    .to_string(),
                        });
                    }
                    [cmd] if cmd == "sort" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode: GrepFilePipeMode::Sort,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped to sort can sort generated lines in-process"
                                    .to_string(),
                        });
                    }
                    [cmd, pipe, subcmd] if cmd == "sort" && pipe == "|" && subcmd == "uniq" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode: GrepFilePipeMode::SortUniq,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped through sort to uniq can de-duplicate generated lines in-process"
                                    .to_string(),
                        });
                    }
                    [cmd, pipe_a, subcmd, pipe_b, count_cmd, flag]
                        if cmd == "sort"
                            && pipe_a == "|"
                            && subcmd == "uniq"
                            && pipe_b == "|"
                            && count_cmd == "wc"
                            && flag == "-l" =>
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode: GrepFilePipeMode::SortUniqWcLines,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped through sort and uniq to wc -l can count unique generated lines in-process"
                                    .to_string(),
                        });
                    }
                    [cmd, pipe, count_cmd, flag]
                        if cmd == "sort" && pipe == "|" && count_cmd == "wc" && flag == "-l" =>
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode: GrepFilePipeMode::SortWcLines,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped through sort to wc -l can count generated lines in-process"
                                    .to_string(),
                        });
                    }
                    [cmd, pipe, limit_cmd, flag, limit]
                        if cmd == "sort" && pipe == "|" && limit_cmd == "head" && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        if limit > 0 {
                            return Some(NativePlan {
                                command: NativeCommand::PipePrintfProducer(
                                    PipePrintfProducerPlan {
                                        args: printf.args,
                                        mode: GrepFilePipeMode::SortHead { limit },
                                    },
                                ),
                                label,
                                original: original.to_string(),
                                reason:
                                    "printf %s\\n piped through sort to head can emit the sorted generated prefix in-process"
                                        .to_string(),
                            });
                        }
                    }
                    [cmd, pipe, limit_cmd, flag, limit]
                        if cmd == "sort" && pipe == "|" && limit_cmd == "tail" && flag == "-n" =>
                    {
                        let limit = limit.parse().ok()?;
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode: GrepFilePipeMode::SortTail { limit },
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped through sort to tail can emit the sorted generated suffix in-process"
                                    .to_string(),
                        });
                    }
                    args if args.len() >= 3
                        && args[0] == "sort"
                        && args[1] == "|"
                        && parse_xargs_echo_command(&args[2..]).is_some() =>
                    {
                        let xargs_mode = parse_xargs_echo_command(&args[2..])?;
                        let mode = sort_xargs_echo_pipe_mode(xargs_mode);
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode,
                            }),
                            label,
                            original: original.to_string(),
                            reason: match xargs_mode {
                                XargsEchoMode::OneLine => {
                                    "printf %s\\n piped through sort to xargs echo can batch sorted generated tokens in-process"
                                }
                                XargsEchoMode::Batch { .. } => {
                                    "printf %s\\n piped through sort to xargs -n echo can emit sorted generated token batches in-process"
                                }
                            }
                            .to_string(),
                        });
                    }
                    [cmd, pipe, subcmd, arg, flag]
                        if cmd == "sort"
                            && pipe == "|"
                            && subcmd == "xargs"
                            && arg == "wc"
                            && flag == "-l" =>
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfProducer(PipePrintfProducerPlan {
                                args: printf.args,
                                mode: GrepFilePipeMode::SortXargsWcLines,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped through sort to xargs wc -l can line-count sorted generated path tokens in-process"
                                    .to_string(),
                        });
                    }
                    args if parse_xargs_echo_command(args).is_some() => {
                        let mode = parse_xargs_echo_command(args)?;
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfXargsEcho(
                                PipePrintfXargsEchoPlan {
                                    args: printf.args,
                                    mode,
                                },
                            ),
                            label,
                            original: original.to_string(),
                            reason: match mode {
                                XargsEchoMode::OneLine => {
                                    "printf %s\\n piped to xargs echo can batch generated tokens in-process"
                                }
                                XargsEchoMode::Batch { .. } => {
                                    "printf %s\\n piped to xargs -n echo can emit generated token batches in-process"
                                }
                            }
                            .to_string(),
                        });
                    }
                    [cmd, subcmd, flag] if cmd == "xargs" && subcmd == "wc" && flag == "-l" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipePrintfXargsWcLines(
                                PipePrintfXargsWcLinesPlan { args: printf.args },
                            ),
                            label,
                            original: original.to_string(),
                            reason:
                                "printf %s\\n piped to xargs wc -l can line-count generated path tokens in-process"
                                    .to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        if left.first().is_some_and(|word| word == "seq") {
            let seq = parse_seq_args(&left[1..])?;
            match right {
                [cmd, flag] if cmd == "wc" => {
                    let mode = WcCountMode::from_flag(flag)?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeSeqWcLines(PipeSeqWcLinesPlan { seq, mode }),
                        label,
                        original: original.to_string(),
                        reason: "integer seq piped to wc can count generated output in-process"
                            .to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeSeqHead(PipeSeqHeadPlan { seq, limit }),
                            label,
                            original: original.to_string(),
                            reason:
                                "integer seq piped to head can emit only the requested prefix in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeSeqTail(PipeSeqTailPlan { seq, limit }),
                        label,
                        original: original.to_string(),
                        reason:
                            "integer seq piped to tail can emit only the requested suffix in-process"
                                .to_string(),
                    });
                }
                args if parse_xargs_echo_command(args).is_some() => {
                    let mode = parse_xargs_echo_command(args)?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeSeqXargsEcho(PipeSeqXargsEchoPlan { seq, mode }),
                        label,
                        original: original.to_string(),
                        reason: match mode {
                            XargsEchoMode::OneLine => {
                                "integer seq piped to xargs echo can batch generated tokens in-process"
                            }
                            XargsEchoMode::Batch { .. } => {
                                "integer seq piped to xargs -n echo can emit generated token batches in-process"
                            }
                        }
                        .to_string(),
                    });
                }
                _ => {}
            }
        }

        if left.first().is_some_and(|word| word == "yes") {
            let value = match left {
                [cmd] if cmd == "yes" => "y".to_string(),
                [cmd, value] if cmd == "yes" && !value.starts_with('-') => value.clone(),
                _ => return None,
            };
            if let [cmd, flag, limit] = right {
                if cmd == "head" && flag == "-n" {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeYesHead(PipeYesHeadPlan { value, limit }),
                            label,
                            original: original.to_string(),
                            reason:
                                "yes piped to head can generate only the requested lines in-process"
                                    .to_string(),
                        });
                    }
                }
            }
        }

        if let Some(lookup) = parse_path_lookup_pipe_left(left) {
            match right {
                [cmd, flag] if cmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipePathLookupWcLines(PipePathLookupWcLinesPlan {
                            lookup,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "path lookup piped to wc -l can count generated lines in-process"
                            .to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipePathLookupHead(PipePathLookupHeadPlan {
                                lookup,
                                limit,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "path lookup piped to head can emit the requested prefix in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipePathLookupTail(PipePathLookupTailPlan {
                            lookup,
                            limit,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "path lookup piped to tail can emit the requested suffix in-process"
                                .to_string(),
                    });
                }
                _ => {}
            }
        }

        if let Some(env) = parse_environment_pipe_left(left) {
            if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(right) {
                return Some(NativePlan {
                    command: NativeCommand::PipeEnvironmentGrepProducer(
                        PipeEnvironmentGrepProducerPlan { env, pattern, mode },
                    ),
                    label,
                    original: original.to_string(),
                    reason:
                        "environment output piped through literal grep/downstream can be fused in-process"
                            .to_string(),
                });
            }
            match right {
                [cmd, flag] if cmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeEnvironmentWcLines(
                            PipeEnvironmentWcLinesPlan { env },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "environment output piped to wc -l can count generated lines in-process"
                                .to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeEnvironmentHead(PipeEnvironmentHeadPlan {
                                env,
                                limit,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "environment output piped to head can emit the requested prefix in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeEnvironmentTail(PipeEnvironmentTailPlan {
                            env,
                            limit,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "environment output piped to tail can emit the requested suffix in-process"
                                .to_string(),
                    });
                }
                [cmd, pattern] if cmd == "grep" && is_plain_literal_pattern(pattern) => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeEnvironmentGrep(PipeEnvironmentGrepPlan {
                            env,
                            pattern: pattern.clone(),
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "environment output piped to literal grep can filter generated lines in-process"
                                .to_string(),
                    });
                }
                [cmd] if cmd == "sort" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeEnvironmentSort(PipeEnvironmentSortPlan {
                            env,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "environment output piped to sort can order generated lines in-process"
                                .to_string(),
                    });
                }
                _ => {}
            }
        }

        if parse_hostname_pipe_left(left) {
            match right {
                [cmd, flag] if cmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeHostnameWcLines(PipeHostnameWcLinesPlan),
                        label,
                        original: original.to_string(),
                        reason:
                            "hostname piped to wc -l can count its single output line in-process"
                                .to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeHostnameHead(PipeHostnameHeadPlan {
                                limit,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "hostname piped to head can emit the requested prefix in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeHostnameTail(PipeHostnameTailPlan { limit }),
                        label,
                        original: original.to_string(),
                        reason: "hostname piped to tail can emit the requested suffix in-process"
                            .to_string(),
                    });
                }
                [cmd, pattern] if cmd == "grep" && is_plain_literal_pattern(pattern) => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeHostnameGrep(PipeHostnameGrepPlan {
                            pattern: pattern.clone(),
                        }),
                        label,
                        original: original.to_string(),
                        reason: "hostname piped to literal grep can filter its generated line in-process"
                            .to_string(),
                    });
                }
                [cmd] if cmd == "sort" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeHostnameSort(PipeHostnameSortPlan),
                        label,
                        original: original.to_string(),
                        reason:
                            "hostname piped to sort can keep the single generated line in-process"
                                .to_string(),
                    });
                }
                _ => {}
            }
        }

        if left.first().is_some_and(|word| word == "ls") {
            let source = parse_ls_pipe_left(left)?;
            match right {
                [cmd, flag] if cmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeLsWcLines(PipeLsWcLinesPlan { source }),
                        label,
                        original: original.to_string(),
                        reason: "ls piped to wc -l can count listed entries in-process".to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsHead(PipeLsHeadPlan { source, limit }),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped to head can emit the requested entry prefix in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeLsTail(PipeLsTailPlan { source, limit }),
                        label,
                        original: original.to_string(),
                        reason: "ls piped to tail can emit the requested entry suffix in-process"
                            .to_string(),
                    });
                }
                [cmd] if cmd == "sort" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeLsSort(PipeLsSortPlan { source }),
                        label,
                        original: original.to_string(),
                        reason: "ls piped to sort can keep directory listing in one process"
                            .to_string(),
                    });
                }
                [cmd, pattern]
                    if cmd == "grep"
                        && !pattern.is_empty()
                        && is_plain_literal_pattern(pattern) =>
                {
                    return Some(NativePlan {
                        command: NativeCommand::PipeLsGrep(PipeLsGrepPlan {
                            source,
                            pattern: pattern.clone(),
                        }),
                        label,
                        original: original.to_string(),
                        reason: "ls piped to grep can filter listed entries by literal in-process"
                            .to_string(),
                    });
                }
                [cmd, subcmd] if cmd == "xargs" && subcmd == "echo" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeLsXargsEcho(PipeLsXargsEchoPlan { source }),
                        label,
                        original: original.to_string(),
                        reason: "ls piped to xargs echo can batch listed entries in-process"
                            .to_string(),
                    });
                }
                _ => {}
            }
        }

        if left.len() == 2
            && left.first().is_some_and(|word| word == "cat")
            && Path::new(&left[1]).is_file()
        {
            match right {
                [cmd, subcmd] if cmd == "xargs" && subcmd == "echo" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeCatXargsEcho(PipeCatXargsEchoPlan {
                            file: left[1].clone(),
                        }),
                        label,
                        original: original.to_string(),
                        reason: "cat piped to xargs echo can batch file tokens in-process"
                            .to_string(),
                    });
                }
                [cmd, subcmd, flag] if cmd == "xargs" && subcmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeCatXargsWcLines(PipeCatXargsWcLinesPlan {
                            file: left[1].clone(),
                        }),
                        label,
                        original: original.to_string(),
                        reason: "cat piped to xargs wc -l can line-count file tokens in-process"
                            .to_string(),
                    });
                }
                _ => {}
            }
        }

        if let Some((pattern, file)) = parse_grep_file_pipe_left(left) {
            if let Some((mode, _reason)) = plan_tail_producer_mode(right) {
                let reason = grep_file_pipe_mode_reason(&file, mode);
                return Some(NativePlan {
                    command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                        pattern,
                        file,
                        mode,
                    }),
                    label,
                    original: original.to_string(),
                    reason,
                });
            }
            match right {
                [cmd, flag] if cmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                            pattern,
                            file,
                            mode: GrepFilePipeMode::WcLines,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "grep file piped to wc -l can count matching lines in-process"
                            .to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                                pattern,
                                file,
                                mode: GrepFilePipeMode::Head { limit },
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "grep file piped to head can emit matching prefix lines in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                            pattern,
                            file,
                            mode: GrepFilePipeMode::Tail { limit },
                        }),
                        label,
                        original: original.to_string(),
                        reason: "grep file piped to tail can emit matching suffix lines in-process"
                            .to_string(),
                    });
                }
                [cmd] if cmd == "sort" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                            pattern,
                            file,
                            mode: GrepFilePipeMode::Sort,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "grep file piped to sort can sort matching lines in-process"
                            .to_string(),
                    });
                }
                [cmd, arg] if cmd == "xargs" && arg == "echo" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                            pattern,
                            file,
                            mode: GrepFilePipeMode::XargsEcho,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "grep file piped to xargs echo can batch matching tokens in-process"
                                .to_string(),
                    });
                }
                [cmd, arg, flag] if cmd == "xargs" && arg == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                            pattern,
                            file,
                            mode: GrepFilePipeMode::XargsWcLines,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "grep file piped to xargs wc -l can line-count matching path tokens in-process"
                            .to_string(),
                    });
                }
                _ => {}
            }
        }

        if left.first().is_some_and(|word| word == "sort") {
            let file = parse_sort_pipe_left(left)?;
            match right {
                [cmd] if cmd == "uniq" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSortUniq(PipeSortUniqPlan { file }),
                        label,
                        original: original.to_string(),
                        reason:
                            "sort piped to uniq can sort and de-duplicate in one in-process stage"
                                .to_string(),
                    });
                }
                [cmd, flag] if cmd == "wc" => {
                    let mode = WcCountMode::from_flag(flag)?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeSortWcLines(PipeSortWcLinesPlan { file, mode }),
                        label,
                        original: original.to_string(),
                        reason: "sort piped to wc can count sorted output in-process".to_string(),
                    });
                }
                [cmd, flag, limit] if cmd == "head" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    if limit > 0 {
                        return Some(NativePlan {
                            command: NativeCommand::PipeSortHead(PipeSortHeadPlan { file, limit }),
                            label,
                            original: original.to_string(),
                            reason:
                                "sort piped to head can emit only the requested sorted prefix in-process"
                                    .to_string(),
                        });
                    }
                }
                [cmd, flag, limit] if cmd == "tail" && flag == "-n" => {
                    let limit = limit.parse().ok()?;
                    return Some(NativePlan {
                        command: NativeCommand::PipeSortTail(PipeSortTailPlan { file, limit }),
                        label,
                        original: original.to_string(),
                        reason:
                            "sort piped to tail can emit only the requested sorted suffix in-process"
                                .to_string(),
                    });
                }
                [cmd, subcmd] if cmd == "xargs" && subcmd == "echo" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSortXargsEcho(PipeSortXargsEchoPlan { file }),
                        label,
                        original: original.to_string(),
                        reason: "sort piped to xargs echo can batch sorted tokens in-process"
                            .to_string(),
                    });
                }
                [cmd, subcmd, flag] if cmd == "xargs" && subcmd == "wc" && flag == "-l" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeSortXargsWcLines(PipeSortXargsWcLinesPlan {
                            file,
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "sort piped to xargs wc -l can line-count sorted path tokens in-process"
                                .to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    if words.len() >= 4 && words.first().is_some_and(|word| word == "ls") {
        if let Some(sort_pos) = words.iter().position(|word| word == "|") {
            if sort_pos + 1 < words.len() && words[sort_pos + 1] == "sort" {
                let source = parse_ls_pipe_left(&words[..sort_pos])?;
                let after_sort = &words[sort_pos + 2..];
                if after_sort.len() >= 4
                    && after_sort[0] == "|"
                    && after_sort[1] == "uniq"
                    && after_sort[2] == "|"
                {
                    let downstream = &after_sort[3..];
                    if let Some((pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsSortUniqGrepProducer(
                                PipeLsSortUniqGrepProducerPlan {
                                    source,
                                    pattern,
                                    mode,
                                },
                            ),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls/sort/uniq output piped through grep/downstream can be fused in-process"
                                    .to_string(),
                        });
                    }
                    if let Some((mode, _)) = plan_tail_producer_mode(downstream) {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsSortUniqProducer(
                                PipeLsSortUniqProducerPlan { source, mode },
                            ),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls/sort/uniq output piped to a supported downstream can be fused in-process"
                                    .to_string(),
                        });
                    }
                }
                match &words[sort_pos + 2..] {
                    [pipe, cmd] if pipe == "|" && cmd == "uniq" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsSortUniq(PipeLsSortUniqPlan { source }),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped through sort to uniq can emit unique listed entries in-process"
                                    .to_string(),
                        });
                    }
                    [pipe, cmd, pipe2, wc, flag]
                        if pipe == "|"
                            && cmd == "uniq"
                            && pipe2 == "|"
                            && wc == "wc"
                            && flag == "-l" =>
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsSortUniqWcLines(
                                PipeLsSortUniqWcLinesPlan { source },
                            ),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped through sort and uniq to wc -l can count unique listed entries in-process"
                                    .to_string(),
                        });
                    }
                    [pipe, wc, flag] if pipe == "|" && wc == "wc" && flag == "-l" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsWcLines(PipeLsWcLinesPlan { source }),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped through sort to wc -l can count listed entries in-process"
                                    .to_string(),
                        });
                    }
                    [pipe, xargs, echo] if pipe == "|" && xargs == "xargs" && echo == "echo" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsSortXargsEcho(
                                PipeLsSortXargsEchoPlan { source },
                            ),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped through sort to xargs echo can batch sorted entries in-process"
                                    .to_string(),
                        });
                    }
                    [pipe, cmd, flag, limit] if pipe == "|" && flag == "-n" => {
                        let limit = limit.parse().ok()?;
                        match cmd.as_str() {
                            "head" if limit > 0 => {
                                return Some(NativePlan {
                                    command: NativeCommand::PipeLsHead(PipeLsHeadPlan {
                                        source,
                                        limit,
                                    }),
                                    label,
                                    original: original.to_string(),
                                    reason:
                                        "ls piped through sort to head can emit the requested entry prefix in-process"
                                            .to_string(),
                                });
                            }
                            "tail" => {
                                return Some(NativePlan {
                                    command: NativeCommand::PipeLsTail(PipeLsTailPlan {
                                        source,
                                        limit,
                                    }),
                                    label,
                                    original: original.to_string(),
                                    reason:
                                        "ls piped through sort to tail can emit the requested entry suffix in-process"
                                            .to_string(),
                                });
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if words.len() >= 6 && words.first().is_some_and(|word| word == "ls") {
        if let Some(grep_pos) = words.iter().position(|word| word == "|") {
            if grep_pos + 3 < words.len()
                && words[grep_pos + 1] == "grep"
                && !words[grep_pos + 2].is_empty()
                && is_plain_literal_pattern(&words[grep_pos + 2])
            {
                let source = parse_ls_pipe_left(&words[..grep_pos])?;
                let pattern = words[grep_pos + 2].clone();
                if words.len() == grep_pos + 3 {
                    return Some(NativePlan {
                        command: NativeCommand::PipeLsGrepProducer(PipeLsGrepProducerPlan {
                            source,
                            pattern,
                            mode: GrepFilePipeMode::Lines,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "ls output piped through grep can be fused in-process".to_string(),
                    });
                }
                if words[grep_pos + 3] == "|" {
                    if let Some((mode, _)) = plan_tail_producer_mode(&words[grep_pos + 4..]) {
                        if !matches!(
                            mode,
                            GrepFilePipeMode::XargsWcLines | GrepFilePipeMode::SortXargsWcLines
                        ) {
                            return Some(NativePlan {
                                command: NativeCommand::PipeLsGrepProducer(
                                    PipeLsGrepProducerPlan {
                                        source,
                                        pattern,
                                        mode,
                                    },
                                ),
                                label,
                                original: original.to_string(),
                                reason:
                                    "ls output piped through grep/downstream can be fused in-process"
                                        .to_string(),
                            });
                        }
                    }
                }
                match &words[grep_pos + 3..] {
                    [pipe, wc, flag] if pipe == "|" && wc == "wc" && flag == "-l" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsGrepWcLines(PipeLsGrepWcLinesPlan {
                                source,
                                pattern,
                            }),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped through grep to wc -l can count literal entry matches in-process"
                                    .to_string(),
                        });
                    }
                    [pipe, xargs, echo] if pipe == "|" && xargs == "xargs" && echo == "echo" => {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsGrepXargsEcho(
                                PipeLsGrepXargsEchoPlan { source, pattern },
                            ),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped through grep to xargs echo can batch matching entries in-process"
                                    .to_string(),
                        });
                    }
                    [pipe_1, sort, pipe_2, xargs, echo]
                        if pipe_1 == "|"
                            && sort == "sort"
                            && pipe_2 == "|"
                            && xargs == "xargs"
                            && echo == "echo" =>
                    {
                        return Some(NativePlan {
                            command: NativeCommand::PipeLsGrepSortXargsEcho(
                                PipeLsGrepSortXargsEchoPlan { source, pattern },
                            ),
                            label,
                            original: original.to_string(),
                            reason:
                                "ls piped through grep and sort to xargs echo can batch sorted matching entries in-process"
                                    .to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    if words.len() > 4 && words.get(3).is_some_and(|word| word == "|") {
        if let Some((pattern, file)) = parse_grep_file_pipe_left(&words[..3]) {
            if let Some((mode, reason)) = plan_tail_producer_mode(&words[4..]) {
                return Some(NativePlan {
                    command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                        pattern,
                        file,
                        mode,
                    }),
                    label,
                    original: original.to_string(),
                    reason: reason.to_string(),
                });
            }
        }
    }

    if words.len() > 6 && words[0] == "cat" && words[2] == "|" && Path::new(&words[1]).is_file() {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&words[3..]) {
            return Some(NativePlan {
                command: NativeCommand::PipeCatXargsWcProducer(PipeCatXargsWcProducerPlan {
                    file: words[1].clone(),
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: reason.to_string(),
            });
        }
    }

    if words.len() > 6 && words[0] == "sort" && words[2] == "|" && Path::new(&words[1]).is_file() {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&words[3..]) {
            return Some(NativePlan {
                command: NativeCommand::PipeSortXargsWcProducer(PipeSortXargsWcProducerPlan {
                    file: words[1].clone(),
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: reason.to_string(),
            });
        }
    }

    if words.len() > 8
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && Path::new(&words[1]).is_file()
    {
        if let Some((mode, reason)) = plan_xargs_wc_output_mode(&words[5..]) {
            return Some(NativePlan {
                command: NativeCommand::PipeCatSortXargsWcProducer(
                    PipeCatSortXargsWcProducerPlan {
                        file: words[1].clone(),
                        mode,
                    },
                ),
                label,
                original: original.to_string(),
                reason: reason.to_string(),
            });
        }
    }

    if let Some(sort_pos) = words.iter().position(|word| word == "sort") {
        if sort_pos > 0 && words.get(sort_pos - 1).is_some_and(|word| word == "|") {
            if let Some((pattern, file)) = parse_grep_file_pipe_left(&words[..sort_pos - 1]) {
                let mode = match &words[sort_pos + 1..] {
                    [pipe, cmd] if pipe == "|" && cmd == "uniq" => GrepFilePipeMode::SortUniq,
                    [pipe1, cmd1, pipe2, rest @ ..]
                        if pipe1 == "|" && cmd1 == "uniq" && pipe2 == "|" =>
                    {
                        if let Some((mode, _)) = plan_tail_producer_mode(rest) {
                            return Some(NativePlan {
                                command: NativeCommand::PipeGrepFileSortUniqProducer(
                                    PipeGrepFileSortUniqProducerPlan {
                                        pattern,
                                        file,
                                        mode,
                                    },
                                ),
                                label,
                                original: original.to_string(),
                                reason:
                                    "grep file output piped through sort, uniq, and a downstream producer can run in-process"
                                        .to_string(),
                            });
                        }
                        return None;
                    }
                    [pipe1, cmd1, pipe2, cmd2, flag]
                        if pipe1 == "|"
                            && cmd1 == "uniq"
                            && pipe2 == "|"
                            && cmd2 == "wc"
                            && flag == "-l" =>
                    {
                        GrepFilePipeMode::SortUniqWcLines
                    }
                    [pipe, cmd, flag] if pipe == "|" && cmd == "wc" && flag == "-l" => {
                        GrepFilePipeMode::SortWcLines
                    }
                    [pipe, cmd, flag, limit] if pipe == "|" && cmd == "head" && flag == "-n" => {
                        let limit = limit.parse().ok()?;
                        if limit == 0 {
                            return None;
                        }
                        GrepFilePipeMode::SortHead { limit }
                    }
                    [pipe, cmd, flag, limit] if pipe == "|" && cmd == "tail" && flag == "-n" => {
                        GrepFilePipeMode::SortTail {
                            limit: limit.parse().ok()?,
                        }
                    }
                    [pipe, cmd, arg] if pipe == "|" && cmd == "xargs" && arg == "echo" => {
                        GrepFilePipeMode::SortXargsEcho
                    }
                    [pipe, cmd, arg, flag]
                        if pipe == "|" && cmd == "xargs" && arg == "wc" && flag == "-l" =>
                    {
                        GrepFilePipeMode::SortXargsWcLines
                    }
                    _ => return None,
                };
                return Some(NativePlan {
                    command: NativeCommand::PipeGrepFile(PipeGrepFilePlan {
                        pattern,
                        file,
                        mode,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "grep file piped through sort can be fused in-process".to_string(),
                });
            }
        }
    }

    if words.len() == 7
        && words[0] == "sort"
        && words[2] == "|"
        && words[3] == "uniq"
        && words[4] == "|"
        && words[5] == "wc"
        && WcCountMode::from_flag(&words[6]).is_some()
        && Path::new(&words[1]).is_file()
    {
        let mode = WcCountMode::from_flag(&words[6])?;
        return Some(NativePlan {
            command: NativeCommand::PipeSortUniqWcLines(PipeSortUniqWcLinesPlan {
                file: words[1].clone(),
                mode,
            }),
            label,
            original: original.to_string(),
            reason: "sort piped through uniq to wc can count unique sorted output in-process"
                .to_string(),
        });
    }

    if words.len() == 5
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "wc"
        && WcCountMode::from_flag(&words[4]).is_some()
        && Path::new(&words[1]).is_file()
    {
        let mode = WcCountMode::from_flag(&words[4])?;
        return Some(NativePlan {
            command: NativeCommand::PipeCatWcLines(PipeCatWcLinesPlan {
                file: words[1].clone(),
                mode,
            }),
            label,
            original: original.to_string(),
            reason: "cat piped to wc can count file output in-process".to_string(),
        });
    }

    if let Some((file, limit, downstream_start)) =
        parse_cat_head_tail_pipe_source(words, "head", true)
    {
        if let Some(downstream_start) = downstream_start {
            let downstream = &words[downstream_start..];
            if let Some((pattern, mode, reason)) = plan_head_grep_producer_mode(downstream) {
                return Some(NativePlan {
                    command: NativeCommand::PipeHeadGrepProducer(PipeHeadGrepProducerPlan {
                        file,
                        stdin: false,
                        limit,
                        pattern,
                        mode,
                    }),
                    label,
                    original: original.to_string(),
                    reason: reason.to_string(),
                });
            }
            if let Some((mode, reason)) = plan_head_producer_mode(downstream) {
                return Some(NativePlan {
                    command: NativeCommand::PipeHeadProducer(PipeHeadProducerPlan {
                        file,
                        stdin: false,
                        limit,
                        mode,
                    }),
                    label,
                    original: original.to_string(),
                    reason: reason.to_string(),
                });
            }
        } else {
            return Some(NativePlan {
                command: NativeCommand::PipeCatHead(PipeCatHeadPlan { file, limit }),
                label,
                original: original.to_string(),
                reason: "cat piped to head can stop file streaming in-process".to_string(),
            });
        }
    }

    if let Some((file, limit, downstream_start)) =
        parse_cat_head_tail_pipe_source(words, "tail", false)
    {
        if let Some(downstream_start) = downstream_start {
            let downstream = &words[downstream_start..];
            if let Some((pattern, mode, reason)) = plan_tail_grep_producer_mode(downstream) {
                return Some(NativePlan {
                    command: NativeCommand::PipeTailGrepProducer(PipeTailGrepProducerPlan {
                        file,
                        stdin: false,
                        limit,
                        pattern,
                        mode,
                    }),
                    label,
                    original: original.to_string(),
                    reason: reason.to_string(),
                });
            }
            if let Some((mode, reason)) = plan_tail_producer_mode(downstream) {
                return Some(NativePlan {
                    command: NativeCommand::PipeTailProducer(PipeTailProducerPlan {
                        file,
                        stdin: false,
                        limit,
                        mode,
                    }),
                    label,
                    original: original.to_string(),
                    reason: reason.to_string(),
                });
            }
        } else {
            return Some(NativePlan {
                command: NativeCommand::PipeCatTail(PipeCatTailPlan { file, limit }),
                label,
                original: original.to_string(),
                reason: "cat piped to tail can keep the line window in-process".to_string(),
            });
        }
    }

    if words.len() == 6
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "head"
        && words[4] == "-n"
        && Path::new(&words[1]).is_file()
    {
        let limit = words[5].parse().ok()?;
        if limit > 0 {
            return Some(NativePlan {
                command: NativeCommand::PipeCatHead(PipeCatHeadPlan {
                    file: words[1].clone(),
                    limit,
                }),
                label,
                original: original.to_string(),
                reason: "cat piped to head can stop file streaming in-process".to_string(),
            });
        }
    }

    if words.len() == 6
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "tail"
        && words[4] == "-n"
        && Path::new(&words[1]).is_file()
    {
        let limit = words[5].parse().ok()?;
        return Some(NativePlan {
            command: NativeCommand::PipeCatTail(PipeCatTailPlan {
                file: words[1].clone(),
                limit,
            }),
            label,
            original: original.to_string(),
            reason: "cat piped to tail can keep the line window in-process".to_string(),
        });
    }

    if words.len() == 5
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "grep"
        && !words[4].is_empty()
        && is_plain_literal_pattern(&words[4])
        && Path::new(&words[1]).is_file()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeCatGrep(PipeCatGrepPlan {
                file: words[1].clone(),
                pattern: words[4].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "cat piped to grep can scan matching lines in-process".to_string(),
        });
    }

    if words.len() >= 7
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "grep"
        && !words[4].is_empty()
        && is_plain_literal_pattern(&words[4])
        && words[5] == "|"
        && Path::new(&words[1]).is_file()
    {
        match &words[6..] {
            [wc, flag] if wc == "wc" && flag == "-l" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                        mode: GrepFilePipeMode::WcLines,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep to wc -l can count matching lines in-process"
                        .to_string(),
                });
            }
            [head, flag, limit] if head == "head" && flag == "-n" => {
                let limit = limit.parse().ok()?;
                if limit > 0 {
                    return Some(NativePlan {
                        command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                            file: words[1].clone(),
                            pattern: words[4].clone(),
                            mode: GrepFilePipeMode::Head { limit },
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "cat piped through grep to head can emit matching prefix lines in-process"
                                .to_string(),
                    });
                }
            }
            [tail, flag, limit] if tail == "tail" && flag == "-n" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                        mode: GrepFilePipeMode::Tail {
                            limit: limit.parse().ok()?,
                        },
                    }),
                    label,
                    original: original.to_string(),
                    reason:
                        "cat piped through grep to tail can emit matching suffix lines in-process"
                            .to_string(),
                });
            }
            [sort] if sort == "sort" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                        mode: GrepFilePipeMode::Sort,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep to sort can sort matching lines in-process"
                        .to_string(),
                });
            }
            [sort, pipe, uniq] if sort == "sort" && pipe == "|" && uniq == "uniq" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                        mode: GrepFilePipeMode::SortUniq,
                    }),
                    label,
                    original: original.to_string(),
                    reason:
                        "cat piped through grep, sort, and uniq can de-duplicate matching lines in-process"
                        .to_string(),
                });
            }
            [sort, pipe1, uniq, pipe2, rest @ ..]
                if sort == "sort" && pipe1 == "|" && uniq == "uniq" && pipe2 == "|" =>
            {
                if let Some((mode, _)) = plan_tail_producer_mode(rest) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeCatGrepSortUniqProducer(
                            PipeCatGrepSortUniqProducerPlan {
                                file: words[1].clone(),
                                pattern: words[4].clone(),
                                mode,
                            },
                        ),
                        label,
                        original: original.to_string(),
                        reason: "cat piped through grep, sort, uniq, and a downstream producer can run in-process"
                            .to_string(),
                    });
                }
            }
            [sort, pipe1, uniq, pipe2, wc, flag]
                if sort == "sort"
                    && pipe1 == "|"
                    && uniq == "uniq"
                    && pipe2 == "|"
                    && wc == "wc"
                    && flag == "-l" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                        mode: GrepFilePipeMode::SortUniqWcLines,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep, sort, uniq, and wc -l can count unique matching lines in-process"
                        .to_string(),
                });
            }
            [sort, pipe, wc, flag]
                if sort == "sort" && pipe == "|" && wc == "wc" && flag == "-l" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                        mode: GrepFilePipeMode::SortWcLines,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep, sort, and wc -l can count matching lines in-process"
                        .to_string(),
                });
            }
            [sort, pipe, head, flag, limit]
                if sort == "sort" && pipe == "|" && head == "head" && flag == "-n" =>
            {
                let limit = limit.parse().ok()?;
                if limit > 0 {
                    return Some(NativePlan {
                        command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                            file: words[1].clone(),
                            pattern: words[4].clone(),
                            mode: GrepFilePipeMode::SortHead { limit },
                        }),
                        label,
                        original: original.to_string(),
                        reason: "cat piped through grep, sort, and head can emit sorted matching prefix lines in-process"
                            .to_string(),
                    });
                }
            }
            [sort, pipe, tail, flag, limit]
                if sort == "sort" && pipe == "|" && tail == "tail" && flag == "-n" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepPipeline(PipeCatGrepPipelinePlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                        mode: GrepFilePipeMode::SortTail {
                            limit: limit.parse().ok()?,
                        },
                    }),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep, sort, and tail can emit sorted matching suffix lines in-process"
                        .to_string(),
                });
            }
            [xargs, echo] if xargs == "xargs" && echo == "echo" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepXargsEcho(PipeCatGrepXargsEchoPlan {
                        file: words[1].clone(),
                        pattern: words[4].clone(),
                    }),
                    label,
                    original: original.to_string(),
                    reason:
                        "cat piped through grep to xargs echo can batch matching tokens in-process"
                            .to_string(),
                });
            }
            [xargs, wc, flag] if xargs == "xargs" && wc == "wc" && flag == "-l" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepXargsWcLines(
                        PipeCatGrepXargsWcLinesPlan {
                            file: words[1].clone(),
                            pattern: words[4].clone(),
                        },
                    ),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep to xargs wc -l can line-count matching tokens in-process"
                        .to_string(),
                });
            }
            [sort, pipe, xargs, echo]
                if sort == "sort" && pipe == "|" && xargs == "xargs" && echo == "echo" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepSortXargsEcho(
                        PipeCatGrepSortXargsEchoPlan {
                            file: words[1].clone(),
                            pattern: words[4].clone(),
                        },
                    ),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep and sort to xargs echo can batch sorted matching tokens in-process"
                        .to_string(),
                });
            }
            [sort, pipe, xargs, wc, flag]
                if sort == "sort"
                    && pipe == "|"
                    && xargs == "xargs"
                    && wc == "wc"
                    && flag == "-l" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatGrepSortXargsWcLines(
                        PipeCatGrepSortXargsWcLinesPlan {
                            file: words[1].clone(),
                            pattern: words[4].clone(),
                        },
                    ),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through grep and sort to xargs wc -l can line-count sorted matching tokens in-process"
                        .to_string(),
                });
            }
            _ => {}
        }
    }

    if words.len() >= 5
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "cut"
        && Path::new(&words[1]).is_file()
    {
        let cut = parse_cut_args_with_file(&words[4..], Some(words[1].clone()))?;
        return Some(NativePlan {
            command: NativeCommand::PipeCatCut(PipeCatCutPlan {
                file: cut.file,
                delimiter: cut.delimiter,
                field: cut.field,
            }),
            label,
            original: original.to_string(),
            reason: "cat piped to cut can extract one field in-process".to_string(),
        });
    }

    if words.len() >= 5
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "tr"
        && Path::new(&words[1]).is_file()
    {
        let tr = parse_tr_args(&words[4..])?;
        return Some(NativePlan {
            command: NativeCommand::PipeCatTr(PipeCatTrPlan {
                file: words[1].clone(),
                tr,
            }),
            label,
            original: original.to_string(),
            reason: "cat piped to tr can transform file bytes in-process".to_string(),
        });
    }

    if words.len() == 4
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "uniq"
        && Path::new(&words[1]).is_file()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeCatUniq(PipeCatUniqPlan {
                file: words[1].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "cat piped to uniq can de-duplicate adjacent file lines in-process".to_string(),
        });
    }

    if words.len() == 7
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "uniq"
        && words[4] == "|"
        && words[5] == "wc"
        && words[6] == "-l"
        && Path::new(&words[1]).is_file()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeCatUniqWcLines(PipeCatUniqWcLinesPlan {
                file: words[1].clone(),
            }),
            label,
            original: original.to_string(),
            reason:
                "cat piped through uniq to wc -l can count adjacent unique file lines in-process"
                    .to_string(),
        });
    }

    if words.len() == 4
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && Path::new(&words[1]).is_file()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeCatSort(PipeCatSortPlan {
                file: words[1].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "cat piped to sort can sort the file in-process".to_string(),
        });
    }

    if words.len() == 6
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && words[5] == "uniq"
        && Path::new(&words[1]).is_file()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeCatSortUniq(PipeCatSortUniqPlan {
                file: words[1].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "cat piped through sort to uniq can de-duplicate in-process".to_string(),
        });
    }

    if words.len() == 9
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && words[5] == "uniq"
        && words[6] == "|"
        && words[7] == "wc"
        && WcCountMode::from_flag(&words[8]).is_some()
        && Path::new(&words[1]).is_file()
    {
        let mode = WcCountMode::from_flag(&words[8])?;
        return Some(NativePlan {
            command: NativeCommand::PipeCatSortUniqWcLines(PipeCatSortUniqWcLinesPlan {
                file: words[1].clone(),
                mode,
            }),
            label,
            original: original.to_string(),
            reason:
                "cat piped through sort and uniq to wc can count unique sorted output in-process"
                    .to_string(),
        });
    }

    if words.len() == 8
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && words[5] == "xargs"
        && words[6] == "wc"
        && words[7] == "-l"
        && Path::new(&words[1]).is_file()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeCatSortXargsWcLines(PipeCatSortXargsWcLinesPlan {
                file: words[1].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "cat piped through sort to xargs wc -l can line-count sorted tokens in-process"
                .to_string(),
        });
    }

    if words.len() == 7
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && words[5] == "xargs"
        && words[6] == "echo"
        && Path::new(&words[1]).is_file()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeCatSortXargsEcho(PipeCatSortXargsEchoPlan {
                file: words[1].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "cat piped through sort to xargs echo can batch sorted tokens in-process"
                .to_string(),
        });
    }

    if words.len() == 7
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && words[5] == "wc"
        && WcCountMode::from_flag(&words[6]).is_some()
        && Path::new(&words[1]).is_file()
    {
        let mode = WcCountMode::from_flag(&words[6])?;
        return Some(NativePlan {
            command: NativeCommand::PipeCatSortWcLines(PipeCatSortWcLinesPlan {
                file: words[1].clone(),
                mode,
            }),
            label,
            original: original.to_string(),
            reason: "cat piped through sort to wc can count sorted output in-process".to_string(),
        });
    }

    if words.len() == 8
        && words[0] == "cat"
        && words[2] == "|"
        && words[3] == "sort"
        && words[4] == "|"
        && words[6] == "-n"
        && Path::new(&words[1]).is_file()
    {
        let limit = words[7].parse().ok()?;
        match words[5].as_str() {
            "head" if limit > 0 => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatSortHead(PipeCatSortHeadPlan {
                        file: words[1].clone(),
                        limit,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through sort to head can emit the sorted prefix in-process"
                        .to_string(),
                });
            }
            "tail" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeCatSortTail(PipeCatSortTailPlan {
                        file: words[1].clone(),
                        limit,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "cat piped through sort to tail can emit the sorted suffix in-process"
                        .to_string(),
                });
            }
            _ => {}
        }
    }

    if words.len() == 8
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "head"
        && words[6] == "-n"
    {
        let limit = words[7].parse().ok()?;
        if limit > 0
            && !words[2].is_empty()
            && is_plain_literal_pattern(&words[2])
            && Path::new(&words[3]).exists()
        {
            return Some(NativePlan {
                command: NativeCommand::PipeGrepHead(PipeGrepHeadPlan {
                    pattern: words[2].clone(),
                    root: words[3].clone(),
                    limit,
                }),
                label,
                original: original.to_string(),
                reason: "grep -R piped to head can stop after the requested matches in-process"
                    .to_string(),
            });
        }
    }

    if words.len() == 8
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "tail"
        && words[6] == "-n"
    {
        let limit = words[7].parse().ok()?;
        if limit > 0
            && !words[2].is_empty()
            && is_plain_literal_pattern(&words[2])
            && Path::new(&words[3]).exists()
        {
            return Some(NativePlan {
                command: NativeCommand::PipeGrepTail(PipeGrepTailPlan {
                    pattern: words[2].clone(),
                    root: words[3].clone(),
                    limit,
                }),
                label,
                original: original.to_string(),
                reason: "grep -R piped to tail can keep the matching line window in-process"
                    .to_string(),
            });
        }
    }

    if words.len() == 6
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "sort"
        && !words[2].is_empty()
        && is_plain_literal_pattern(&words[2])
        && Path::new(&words[3]).exists()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepSort(PipeGrepSortPlan {
                pattern: words[2].clone(),
                root: words[3].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "grep -R piped to sort can collect and sort matching lines in-process"
                .to_string(),
        });
    }

    if words.len() == 8
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "sort"
        && words[6] == "|"
        && words[7] == "uniq"
        && !words[2].is_empty()
        && is_plain_literal_pattern(&words[2])
        && Path::new(&words[3]).exists()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepSortUniq(PipeGrepSortUniqPlan {
                pattern: words[2].clone(),
                root: words[3].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "grep -R piped through sort to uniq can emit unique sorted matches in-process"
                .to_string(),
        });
    }

    if words.len() == 9
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "sort"
        && words[6] == "|"
        && words[7] == "wc"
        && words[8] == "-l"
        && !words[2].is_empty()
        && is_plain_literal_pattern(&words[2])
        && Path::new(&words[3]).exists()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepSortWcLines(PipeGrepSortWcLinesPlan {
                pattern: words[2].clone(),
                root: words[3].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "grep -R piped through sort to wc -l can count matching lines in-process"
                .to_string(),
        });
    }

    if words.len() == 11
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "sort"
        && words[6] == "|"
        && words[7] == "uniq"
        && words[8] == "|"
        && words[9] == "wc"
        && words[10] == "-l"
        && !words[2].is_empty()
        && is_plain_literal_pattern(&words[2])
        && Path::new(&words[3]).exists()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepSortUniqWcLines(PipeGrepSortUniqWcLinesPlan {
                pattern: words[2].clone(),
                root: words[3].clone(),
            }),
            label,
            original: original.to_string(),
            reason:
                "grep -R piped through sort and uniq to wc -l can count unique sorted matches in-process"
                    .to_string(),
        });
    }

    if words.len() >= 10
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "sort"
        && words[6] == "|"
        && words[7] == "uniq"
        && words[8] == "|"
        && !words[2].is_empty()
        && is_plain_literal_pattern(&words[2])
        && Path::new(&words[3]).exists()
    {
        if let Some((mode, _)) = plan_tail_producer_mode(&words[9..]) {
            return Some(NativePlan {
                command: NativeCommand::PipeGrepSortUniqProducer(
                    PipeGrepSortUniqProducerPlan {
                        pattern: words[2].clone(),
                        root: words[3].clone(),
                        mode,
                    },
                ),
                label,
                original: original.to_string(),
                reason:
                    "grep -R output piped through sort, uniq, and a downstream producer can run in-process"
                        .to_string(),
            });
        }
    }

    if words.len() == 10
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "sort"
        && words[6] == "|"
        && words[8] == "-n"
    {
        let limit = words[9].parse().ok()?;
        if limit > 0
            && !words[2].is_empty()
            && is_plain_literal_pattern(&words[2])
            && Path::new(&words[3]).exists()
        {
            match words[7].as_str() {
                "head" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeGrepSortHead(PipeGrepSortHeadPlan {
                            pattern: words[2].clone(),
                            root: words[3].clone(),
                            limit,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "grep -R piped through sort to head can emit the sorted prefix in-process"
                            .to_string(),
                    });
                }
                "tail" => {
                    return Some(NativePlan {
                        command: NativeCommand::PipeGrepSortTail(PipeGrepSortTailPlan {
                            pattern: words[2].clone(),
                            root: words[3].clone(),
                            limit,
                        }),
                        label,
                        original: original.to_string(),
                        reason: "grep -R piped through sort to tail can emit the sorted suffix in-process"
                            .to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    if words.len() == 6
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "wc"
    {
        return None;
    }

    if words.len() == 7
        && words[0] == "grep"
        && words[1] == "-R"
        && words[4] == "|"
        && words[5] == "wc"
        && words[6] == "-l"
        && !words[2].is_empty()
        && is_plain_literal_pattern(&words[2])
        && Path::new(&words[3]).exists()
    {
        return Some(NativePlan {
            command: NativeCommand::PipeGrepWcLines(PipeGrepWcLinesPlan {
                pattern: words[2].clone(),
                root: words[3].clone(),
            }),
            label,
            original: original.to_string(),
            reason: "grep -R piped to wc -l can count matching output lines in-process".to_string(),
        });
    }

    if let Some((awk_file, awk_pattern, awk_field, downstream_start)) =
        parse_awk_print_field_pipe_source(words)
    {
        let downstream = &words[downstream_start..];
        if downstream.is_empty() {
            return Some(NativePlan {
                command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                    file: awk_file.clone(),
                    pattern: awk_pattern.clone(),
                    field: awk_field,
                    mode: GrepFilePipeMode::Lines,
                }),
                label,
                original: original.to_string(),
                reason: "cat piped to awk print-field can emit fields in-process".to_string(),
            });
        }
        if let Some((mode, reason)) = plan_tail_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                    file: awk_file.clone(),
                    pattern: awk_pattern.clone(),
                    field: awk_field,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: reason.to_string(),
            });
        }
        if let Some((downstream_pattern, mode, _)) = plan_tail_grep_producer_mode(downstream) {
            return Some(NativePlan {
                command: NativeCommand::PipeAwkGrepProducer(PipeAwkGrepProducerPlan {
                    file: awk_file.clone(),
                    pattern: awk_pattern.clone(),
                    field: awk_field,
                    downstream_pattern,
                    mode,
                }),
                label,
                original: original.to_string(),
                reason: "awk print-field output piped through grep/downstream can run in-process"
                    .to_string(),
            });
        }
        if let [sort_cmd, pipe_a, uniq_cmd, pipe_b, rest @ ..] = downstream {
            if sort_cmd == "sort" && pipe_a == "|" && uniq_cmd == "uniq" && pipe_b == "|" {
                if let Some((mode, _)) = plan_tail_producer_mode(rest) {
                    return Some(NativePlan {
                        command: NativeCommand::PipeAwkSortUniqProducer(
                            PipeAwkSortUniqProducerPlan {
                                file: awk_file.clone(),
                                pattern: awk_pattern.clone(),
                                field: awk_field,
                                mode,
                            },
                        ),
                        label,
                        original: original.to_string(),
                        reason:
                            "awk print-field output piped through sort, uniq, and a downstream producer can run in-process"
                                .to_string(),
                    });
                }
            }
        }
        match downstream {
            [wc, flag] if wc == "wc" && flag == "-l" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::WcLines,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "awk print-field piped to wc -l can count emitted fields in-process"
                        .to_string(),
                });
            }
            [head, flag, limit] if head == "head" && flag == "-n" => {
                let limit = limit.parse().ok()?;
                if limit > 0 {
                    return Some(NativePlan {
                        command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                            file: awk_file.clone(),
                            pattern: awk_pattern.clone(),
                            field: awk_field,
                            mode: GrepFilePipeMode::Head { limit },
                        }),
                        label,
                        original: original.to_string(),
                        reason: "awk print-field piped to head can emit field prefix in-process"
                            .to_string(),
                    });
                }
            }
            [tail, flag, limit] if tail == "tail" && flag == "-n" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::Tail {
                            limit: limit.parse().ok()?,
                        },
                    }),
                    label,
                    original: original.to_string(),
                    reason: "awk print-field piped to tail can emit field suffix in-process"
                        .to_string(),
                });
            }
            [sort] if sort == "sort" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::Sort,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "awk print-field piped to sort can sort emitted fields in-process"
                        .to_string(),
                });
            }
            [sort, pipe, uniq] if sort == "sort" && pipe == "|" && uniq == "uniq" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::SortUniq,
                    }),
                    label,
                    original: original.to_string(),
                    reason:
                        "awk print-field piped through sort and uniq can de-duplicate fields in-process"
                            .to_string(),
                });
            }
            [sort, pipe1, uniq, pipe2, wc, flag]
                if sort == "sort"
                    && pipe1 == "|"
                    && uniq == "uniq"
                    && pipe2 == "|"
                    && wc == "wc"
                    && flag == "-l" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::SortUniqWcLines,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "awk print-field piped through sort, uniq, and wc -l can count unique fields in-process"
                        .to_string(),
                });
            }
            [sort, pipe, wc, flag]
                if sort == "sort" && pipe == "|" && wc == "wc" && flag == "-l" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::SortWcLines,
                    }),
                    label,
                    original: original.to_string(),
                    reason:
                        "awk print-field piped through sort and wc -l can count fields in-process"
                            .to_string(),
                });
            }
            [sort, pipe, head, flag, limit]
                if sort == "sort" && pipe == "|" && head == "head" && flag == "-n" =>
            {
                let limit = limit.parse().ok()?;
                if limit > 0 {
                    return Some(NativePlan {
                        command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                            file: awk_file.clone(),
                            pattern: awk_pattern.clone(),
                            field: awk_field,
                            mode: GrepFilePipeMode::SortHead { limit },
                        }),
                        label,
                        original: original.to_string(),
                        reason:
                            "awk print-field piped through sort and head can emit sorted field prefix in-process"
                                .to_string(),
                    });
                }
            }
            [sort, pipe, tail, flag, limit]
                if sort == "sort" && pipe == "|" && tail == "tail" && flag == "-n" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::SortTail {
                            limit: limit.parse().ok()?,
                        },
                    }),
                    label,
                    original: original.to_string(),
                    reason:
                        "awk print-field piped through sort and tail can emit sorted field suffix in-process"
                            .to_string(),
                });
            }
            [xargs, echo] if xargs == "xargs" && echo == "echo" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkXargsEcho(PipeAwkXargsEchoPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "awk print-field piped to xargs echo can stream tokens in-process"
                        .to_string(),
                });
            }
            [xargs, wc, flag] if xargs == "xargs" && wc == "wc" && flag == "-l" => {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkXargsWcLines(PipeAwkXargsWcLinesPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "awk print-field piped to xargs wc -l can count emitted path tokens in-process"
                        .to_string(),
                });
            }
            [sort, pipe, xargs, echo]
                if sort == "sort" && pipe == "|" && xargs == "xargs" && echo == "echo" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::SortXargsEcho,
                    }),
                    label,
                    original: original.to_string(),
                    reason:
                        "awk print-field piped through sort to xargs echo can batch sorted fields in-process"
                            .to_string(),
                });
            }
            [sort, pipe, xargs, wc, flag]
                if sort == "sort"
                    && pipe == "|"
                    && xargs == "xargs"
                    && wc == "wc"
                    && flag == "-l" =>
            {
                return Some(NativePlan {
                    command: NativeCommand::PipeAwkProducer(PipeAwkProducerPlan {
                        file: awk_file.clone(),
                        pattern: awk_pattern.clone(),
                        field: awk_field,
                        mode: GrepFilePipeMode::SortXargsWcLines,
                    }),
                    label,
                    original: original.to_string(),
                    reason: "awk print-field piped through sort to xargs wc -l can line-count sorted path fields in-process"
                        .to_string(),
                });
            }
            _ => {}
        }
    }

    if let Some(plan) = plan_find_pipe(words, label.clone(), original) {
        return Some(plan);
    }

    None
}

fn parse_head_tail_args(args: &[String]) -> Option<HeadTailPlan> {
    match args {
        [] => Some(HeadTailPlan {
            file: String::new(),
            stdin: true,
            mode: HeadTailMode::Lines,
            count: 10,
        }),
        [lines]
            if lines.strip_prefix('-').is_some_and(|value| {
                !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
            }) =>
        {
            Some(HeadTailPlan {
                file: String::new(),
                stdin: true,
                mode: HeadTailMode::Lines,
                count: lines.strip_prefix('-')?.parse().ok()?,
            })
        }
        [file] => Some(HeadTailPlan {
            file: file.clone(),
            stdin: false,
            mode: HeadTailMode::Lines,
            count: 10,
        }),
        [flag, count] if flag == "-c" || flag == "-n" => Some(HeadTailPlan {
            file: String::new(),
            stdin: true,
            mode: if flag == "-c" {
                HeadTailMode::Bytes
            } else {
                HeadTailMode::Lines
            },
            count: count.parse().ok()?,
        }),
        [flag, count, file] if flag == "-c" || flag == "-n" => Some(HeadTailPlan {
            file: file.clone(),
            stdin: false,
            mode: if flag == "-c" {
                HeadTailMode::Bytes
            } else {
                HeadTailMode::Lines
            },
            count: count.parse().ok()?,
        }),
        [lines, file]
            if lines.strip_prefix('-').is_some_and(|value| {
                !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
            }) =>
        {
            Some(HeadTailPlan {
                file: file.clone(),
                stdin: false,
                mode: HeadTailMode::Lines,
                count: lines.strip_prefix('-')?.parse().ok()?,
            })
        }
        _ => None,
    }
}

fn plan_cat(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if args.is_empty() || args.iter().any(|arg| arg.starts_with('-')) {
        return None;
    }
    if args.iter().any(|path| !Path::new(path).is_file()) {
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::Cat(CatPlan {
            files: args.to_vec(),
        }),
        label,
        original: original.to_string(),
        reason: "plain cat over regular files can stream in-process".to_string(),
    })
}

fn plan_find(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let mut idx = 0;
    let root = if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        idx = 1;
        args[0].clone()
    } else {
        ".".to_string()
    };
    let root_ref = Path::new(&root);
    if !root_ref.exists() {
        return None;
    }

    let mut type_filter = None;
    let mut name_pattern = None;
    while idx < args.len() {
        match args[idx].as_str() {
            "-type" => {
                idx += 1;
                let kind = args.get(idx)?;
                type_filter = match kind.as_str() {
                    "f" => Some(FindType::File),
                    "d" => Some(FindType::Dir),
                    _ => return None,
                };
            }
            "-name" => {
                idx += 1;
                let pattern = args.get(idx)?;
                if pattern.contains(['[', ']']) {
                    return None;
                }
                name_pattern = Some(pattern.clone());
            }
            _ => return None,
        }
        idx += 1;
    }

    Some(NativePlan {
        command: NativeCommand::Find(FindPlan {
            root,
            type_filter,
            name_pattern,
        }),
        label,
        original: original.to_string(),
        reason: "simple find predicates can be walked in-process".to_string(),
    })
}

fn plan_sed(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if args.len() != 3 || args[0] != "-n" {
        return None;
    }
    let (start_line, end_line) = parse_sed_print_script(&args[1])?;
    let path = Path::new(&args[2]);
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::SedPrint(SedPrintPlan {
            file: args[2].clone(),
            start_line,
            end_line,
        }),
        label,
        original: original.to_string(),
        reason: "sed -n line print can be served as an in-process ranged read".to_string(),
    })
}

fn plan_grep_file(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    match args {
        [pattern] if !pattern.starts_with('-') && is_plain_literal_pattern(pattern) => {
            Some(NativePlan {
                command: NativeCommand::GrepFile(GrepFilePlan {
                    pattern: pattern.clone(),
                    file: String::new(),
                }),
                label,
                original: original.to_string(),
                reason: "plain literal grep over stdin can scan in-process".to_string(),
            })
        }
        [pattern, file]
            if !pattern.starts_with('-')
                && is_plain_literal_pattern(pattern)
                && !file.starts_with('-')
                && Path::new(file).is_file() =>
        {
            Some(NativePlan {
                command: NativeCommand::GrepFile(GrepFilePlan {
                    pattern: pattern.clone(),
                    file: file.clone(),
                }),
                label,
                original: original.to_string(),
                reason: "plain literal grep over one regular file can scan in-process".to_string(),
            })
        }
        _ => None,
    }
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_wc_args(args: &[String], allow_stdin: bool) -> Option<WcLinesPlan> {
    if args.is_empty() {
        return None;
    }
    let mode = WcCountMode::from_flag(&args[0])?;

    let files = args[1..].to_vec();
    if !allow_stdin && files.is_empty() {
        return None;
    }
    if files.iter().any(|path| path.starts_with('-')) {
        return None;
    }

    for file in &files {
        let meta = fs::metadata(file).ok()?;
        if !meta.is_file() {
            return None;
        }
    }

    Some(WcLinesPlan { files, mode })
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn parse_wc_all_args(args: &[String], allow_stdin: bool) -> Option<WcAllPlan> {
    if !allow_stdin && args.is_empty() {
        return None;
    }
    if args.iter().any(|path| path.starts_with('-')) {
        return None;
    }

    for file in args {
        let meta = fs::metadata(file).ok()?;
        if !meta.is_file() {
            return None;
        }
    }

    Some(WcAllPlan {
        files: args.to_vec(),
    })
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn plan_wc(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if let Some(plan) = parse_wc_args(args, true) {
        let mode = plan.mode;
        let stdin = plan.files.is_empty();
        return Some(NativePlan {
            command: NativeCommand::WcLines(plan),
            label,
            original: original.to_string(),
            reason: if stdin {
                format!("wc {} stdin count can stream in-process", mode.flag())
            } else {
                format!(
                    "wc {} regular-file aggregate can count in-process",
                    mode.flag()
                )
            },
        });
    }

    let plan = parse_wc_all_args(args, true)?;
    let stdin = plan.files.is_empty();
    Some(NativePlan {
        command: NativeCommand::WcAll(plan),
        label,
        original: original.to_string(),
        reason: if stdin {
            "wc stdin line/word/byte counts can stream in-process".to_string()
        } else {
            "wc regular-file line/word/byte aggregate can count in-process".to_string()
        },
    })
}

fn plan_grep_replacement(
    command: &[String],
    label: Option<String>,
    original: &str,
    tool_available: impl Fn(&str) -> bool,
) -> Option<ExternalPlan> {
    if !tool_available("rg") || command.first().map(|w| basename(w)) != Some("grep") {
        return None;
    }

    let mut recursive = false;
    let mut rg_args = vec!["--hidden".to_string(), "--no-ignore".to_string()];
    let mut positional = Vec::new();
    let mut flags_done = false;

    for word in command.iter().skip(1) {
        if !flags_done && word == "--" {
            flags_done = true;
            continue;
        }
        if !flags_done && word.starts_with("--") {
            match word.as_str() {
                "--recursive" | "--dereference-recursive" => recursive = true,
                "--line-number" => rg_args.push("--line-number".to_string()),
                "--ignore-case" => rg_args.push("--ignore-case".to_string()),
                "--fixed-strings" => rg_args.push("--fixed-strings".to_string()),
                "--word-regexp" => rg_args.push("--word-regexp".to_string()),
                _ => return None,
            }
            continue;
        }
        if !flags_done && word.starts_with('-') && word.len() > 1 {
            for flag in word[1..].chars() {
                match flag {
                    'R' | 'r' => recursive = true,
                    'n' => rg_args.push("-n".to_string()),
                    'i' => rg_args.push("-i".to_string()),
                    'F' => rg_args.push("-F".to_string()),
                    'w' => rg_args.push("-w".to_string()),
                    'H' => rg_args.push("-H".to_string()),
                    'l' => rg_args.push("-l".to_string()),
                    'q' => rg_args.push("-q".to_string()),
                    _ => return None,
                }
            }
            continue;
        }
        positional.push(word.clone());
    }

    if !recursive || positional.len() < 2 {
        return None;
    }
    if positional
        .iter()
        .any(|arg| arg.is_empty() || arg.contains(['*', '?', '[', ']']))
    {
        return None;
    }
    if positional.iter().skip(1).any(|path| path.starts_with('-')) {
        return None;
    }
    rg_args.push("--".to_string());
    rg_args.extend(positional);

    let optimized = render_command("rg", &rg_args);
    let fallback = render_argv(command);
    let script = format!("{optimized} || {fallback}");

    Some(ExternalPlan {
        program: "bash".to_string(),
        args: vec!["-c".to_string(), script],
        label,
        original: original.to_string(),
        implementation: ExternalImplementation::Replacement,
        reason: "recursive grep safe subset can use rg with original-command fallback".to_string(),
        fallback: Some(fallback),
    })
}

/// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
fn dir_entries_at_least(path: &Path, min: usize, include_hidden: bool) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    let mut count = 0usize;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !include_hidden && name.to_string_lossy().starts_with('.') {
            continue;
        }
        count += 1;
        if count >= min {
            return true;
        }
    }
    false
}

/// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
fn tree_entries_at_least(root: &Path, min: usize) -> bool {
    let mut count = 0usize;
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if !path.as_os_str().is_empty() {
            count += 1;
            if count >= min {
                return true;
            }
        }
        if meta.file_type().is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                stack.push(entry.path());
            }
        }
    }
    false
}

/// @spec apps/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
fn grep_workload_at_least(root: &Path, min_files: usize, min_bytes: u64) -> bool {
    let mut files = 0usize;
    let mut bytes = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_file() {
            files += 1;
            bytes = bytes.saturating_add(meta.len());
            if files >= min_files || bytes >= min_bytes {
                return true;
            }
        } else if meta.file_type().is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                stack.push(entry.path());
            }
        }
    }
    false
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn run_native(plan: &NativePlan) -> Result<ExitCode> {
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let code = run_native_to(plan, &mut stdout, &mut stderr)?;
    Ok(exit_code_from_i32(code))
}

pub(crate) fn run_native_to(
    plan: &NativePlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    match &plan.command {
        NativeCommand::True => Ok(0),
        NativeCommand::False => Ok(1),
        NativeCommand::PipeEmptyProducer(pipe) => run_pipe_empty_producer(pipe, stdout, stderr),
        NativeCommand::PipeSideEffectEmptyProducer(pipe) => {
            run_pipe_side_effect_empty_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipePredicateEmptyProducer(pipe) => {
            run_pipe_predicate_empty_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeWcProducer(pipe) => run_pipe_wc_producer(pipe, stdout, stderr),
        NativeCommand::PipeDuProducer(pipe) => run_pipe_du_producer(pipe, stdout, stderr),
        NativeCommand::Pwd => run_pwd(stdout, stderr),
        NativeCommand::Echo(echo) => run_echo(echo, stdout),
        NativeCommand::Printf(printf) => run_printf(printf, stdout),
        NativeCommand::PrintfLiteral(printf) => run_printf_literal(printf, stdout),
        NativeCommand::Seq(seq) => run_seq(seq, stdout),
        NativeCommand::Whoami => run_whoami(stdout, stderr),
        NativeCommand::Id(id) => run_id(id, stdout, stderr),
        NativeCommand::Uname(uname) => run_uname(uname, stdout, stderr),
        NativeCommand::Hostname => run_hostname(stdout, stderr),
        NativeCommand::PipeSingleLineProducer(pipe) => {
            run_pipe_single_line_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSingleLineGrepProducer(pipe) => {
            run_pipe_single_line_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::Test(test) => run_test(test),
        NativeCommand::Basename(base) => run_basename(base, stdout),
        NativeCommand::Dirname(dir) => run_dirname(dir, stdout),
        NativeCommand::Ls(ls) => run_ls(ls, stdout, stderr),
        NativeCommand::Sort(sort) => run_sort(sort, stdout),
        NativeCommand::Uniq(uniq) => run_uniq(uniq, stdout, stderr),
        NativeCommand::Cut(cut) => run_cut(cut, stdout, stderr),
        NativeCommand::Tr(tr) => run_tr(tr, stdout),
        NativeCommand::Cat(cat) => run_cat(cat, stdout, stderr),
        NativeCommand::Head(head) => run_head(head, stdout, stderr),
        NativeCommand::Tail(tail) => run_tail(tail, stdout, stderr),
        NativeCommand::PipeHeadProducer(pipe) => run_pipe_head_producer(pipe, stdout, stderr),
        NativeCommand::PipeHeadGrepProducer(pipe) => {
            run_pipe_head_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeTailProducer(pipe) => run_pipe_tail_producer(pipe, stdout, stderr),
        NativeCommand::PipeTailGrepProducer(pipe) => {
            run_pipe_tail_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSedProducer(pipe) => run_pipe_sed_producer(pipe, stdout, stderr),
        NativeCommand::PipeSedGrepProducer(pipe) => {
            run_pipe_sed_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeCutProducer(pipe) => run_pipe_cut_producer(pipe, stdout, stderr),
        NativeCommand::PipeCutGrepProducer(pipe) => {
            run_pipe_cut_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatTrProducer(pipe) => run_pipe_cat_tr_producer(pipe, stdout, stderr),
        NativeCommand::PipeCatTrGrepProducer(pipe) => {
            run_pipe_cat_tr_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::Mkdir(mkdir) => run_mkdir(mkdir, stderr),
        NativeCommand::Touch(touch) => run_touch(touch, stderr),
        NativeCommand::GrepFile(grep) => run_grep_file(grep, stdout, stderr),
        NativeCommand::AwkNeedleCount(awk) => run_awk_needle_count(awk, stdout, stderr),
        NativeCommand::AwkFirstField(awk) => run_awk_first_field(awk, stdout, stderr),
        NativeCommand::XargsEcho(xargs) => run_xargs_echo(xargs, stdout, stderr),
        NativeCommand::XargsWcLines(xargs) => run_xargs_wc_lines(xargs, stdout, stderr),
        NativeCommand::PipeXargsEchoProducer(pipe) => {
            run_pipe_xargs_echo_producer(pipe, stdout, stderr)
        }
        NativeCommand::PathLookup(path_lookup) => run_path_lookup(path_lookup, stdout),
        NativeCommand::Environment(environment) => run_environment(environment, stdout),
        NativeCommand::PipeLsWcLines(pipe) => run_pipe_ls_wc_lines(pipe, stdout, stderr),
        NativeCommand::PipeLsHead(pipe) => run_pipe_ls_head(pipe, stdout, stderr),
        NativeCommand::PipeLsTail(pipe) => run_pipe_ls_tail(pipe, stdout, stderr),
        NativeCommand::PipeLsSort(pipe) => run_pipe_ls_sort(pipe, stdout, stderr),
        NativeCommand::PipeLsSortXargsEcho(pipe) => {
            run_pipe_ls_sort_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeLsSortUniq(pipe) => run_pipe_ls_sort_uniq(pipe, stdout, stderr),
        NativeCommand::PipeLsSortUniqWcLines(pipe) => {
            run_pipe_ls_sort_uniq_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeLsSortUniqProducer(pipe) => {
            run_pipe_ls_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeLsSortUniqGrepProducer(pipe) => {
            run_pipe_ls_sort_uniq_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeLsGrep(pipe) => run_pipe_ls_grep(pipe, stdout, stderr),
        NativeCommand::PipeLsGrepProducer(pipe) => run_pipe_ls_grep_producer(pipe, stdout, stderr),
        NativeCommand::PipeLsGrepWcLines(pipe) => run_pipe_ls_grep_wc_lines(pipe, stdout, stderr),
        NativeCommand::PipeLsGrepXargsEcho(pipe) => {
            run_pipe_ls_grep_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeLsGrepSortXargsEcho(pipe) => {
            run_pipe_ls_grep_sort_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeLsXargsEcho(pipe) => run_pipe_ls_xargs_echo(pipe, stdout, stderr),
        NativeCommand::PipeCatWcLines(pipe) => run_pipe_cat_wc_lines(pipe, stdout, stderr),
        NativeCommand::PipeCatHead(pipe) => run_pipe_cat_head(pipe, stdout, stderr),
        NativeCommand::PipeCatTail(pipe) => run_pipe_cat_tail(pipe, stdout, stderr),
        NativeCommand::PipeCatGrep(pipe) => run_pipe_cat_grep(pipe, stdout, stderr),
        NativeCommand::PipeCatGrepPipeline(pipe) => {
            run_pipe_cat_grep_pipeline(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatGrepSortUniqProducer(pipe) => {
            run_pipe_cat_grep_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatCut(pipe) => run_pipe_cat_cut(pipe, stdout, stderr),
        NativeCommand::PipeCatTr(pipe) => run_pipe_cat_tr(pipe, stdout, stderr),
        NativeCommand::PipeCatUniq(pipe) => run_pipe_cat_uniq(pipe, stdout),
        NativeCommand::PipeCatUniqWcLines(pipe) => run_pipe_cat_uniq_wc_lines(pipe, stdout),
        NativeCommand::PipeCatUniqProducer(pipe) => {
            run_pipe_cat_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatUniqGrepProducer(pipe) => {
            run_pipe_cat_uniq_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeUniqProducer(pipe) => run_pipe_uniq_producer(pipe, stdout, stderr),
        NativeCommand::PipeUniqGrepProducer(pipe) => {
            run_pipe_uniq_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatSort(pipe) => run_pipe_cat_sort(pipe, stdout),
        NativeCommand::PipeCatSortUniq(pipe) => run_pipe_cat_sort_uniq(pipe, stdout),
        NativeCommand::PipeCatSortUniqWcLines(pipe) => {
            run_pipe_cat_sort_uniq_wc_lines(pipe, stdout)
        }
        NativeCommand::PipeCatSortHead(pipe) => run_pipe_cat_sort_head(pipe, stdout),
        NativeCommand::PipeCatSortTail(pipe) => run_pipe_cat_sort_tail(pipe, stdout),
        NativeCommand::PipeCatSortWcLines(pipe) => run_pipe_cat_sort_wc_lines(pipe, stdout),
        NativeCommand::PipeCatXargsEcho(pipe) => run_pipe_cat_xargs_echo(pipe, stdout),
        NativeCommand::PipeCatXargsWcLines(pipe) => {
            run_pipe_cat_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatXargsWcProducer(pipe) => {
            run_pipe_cat_xargs_wc_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatGrepXargsEcho(pipe) => {
            run_pipe_cat_grep_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatGrepXargsWcLines(pipe) => {
            run_pipe_cat_grep_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatGrepSortXargsEcho(pipe) => {
            run_pipe_cat_grep_sort_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatGrepSortXargsWcLines(pipe) => {
            run_pipe_cat_grep_sort_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatSortXargsEcho(pipe) => run_pipe_cat_sort_xargs_echo(pipe, stdout),
        NativeCommand::PipeCatSortXargsWcLines(pipe) => {
            run_pipe_cat_sort_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeCatSortXargsWcProducer(pipe) => {
            run_pipe_cat_sort_xargs_wc_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepHead(pipe) => run_pipe_grep_head(pipe, stdout, stderr),
        NativeCommand::PipeGrepTail(pipe) => run_pipe_grep_tail(pipe, stdout, stderr),
        NativeCommand::PipeGrepSort(pipe) => run_pipe_grep_sort(pipe, stdout, stderr),
        NativeCommand::PipeGrepSortUniq(pipe) => run_pipe_grep_sort_uniq(pipe, stdout, stderr),
        NativeCommand::PipeGrepSortUniqProducer(pipe) => {
            run_pipe_grep_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepSortUniqWcLines(pipe) => {
            run_pipe_grep_sort_uniq_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepSortHead(pipe) => run_pipe_grep_sort_head(pipe, stdout, stderr),
        NativeCommand::PipeGrepSortTail(pipe) => run_pipe_grep_sort_tail(pipe, stdout, stderr),
        NativeCommand::PipeGrepSortWcLines(pipe) => {
            run_pipe_grep_sort_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepWcLines(pipe) => run_pipe_grep_wc_lines(pipe, stdout, stderr),
        NativeCommand::PipeGrepFile(pipe) => run_pipe_grep_file(pipe, stdout, stderr),
        NativeCommand::PipeGrepFileSortUniqProducer(pipe) => {
            run_pipe_grep_file_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepFileCutProducer(pipe) => {
            run_pipe_grep_file_cut_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepFileCutGrepProducer(pipe) => {
            run_pipe_grep_file_cut_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepFileAwkProducer(pipe) => {
            run_pipe_grep_file_awk_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeGrepFileAwkGrepProducer(pipe) => {
            run_pipe_grep_file_awk_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeAwkProducer(pipe) => run_pipe_awk_producer(pipe, stdout, stderr),
        NativeCommand::PipeAwkGrepProducer(pipe) => {
            run_pipe_awk_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeAwkSortUniqProducer(pipe) => {
            run_pipe_awk_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeAwkXargsEcho(pipe) => run_pipe_awk_xargs_echo(pipe, stdout, stderr),
        NativeCommand::PipeAwkXargsWcLines(pipe) => {
            run_pipe_awk_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeEchoWcLines(pipe) => run_pipe_echo_wc_lines(pipe, stdout),
        NativeCommand::PipeEchoHead(pipe) => run_pipe_echo_head(pipe, stdout),
        NativeCommand::PipeEchoTail(pipe) => run_pipe_echo_tail(pipe, stdout),
        NativeCommand::PipeEchoTr(pipe) => run_pipe_echo_tr(pipe, stdout),
        NativeCommand::PipeEchoAwkProducer(pipe) => {
            run_pipe_echo_awk_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeEchoXargsEcho(pipe) => run_pipe_echo_xargs_echo(pipe, stdout),
        NativeCommand::PipeEchoXargsWcLines(pipe) => {
            run_pipe_echo_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipePrintfWcLines(pipe) => run_pipe_printf_wc_lines(pipe, stdout),
        NativeCommand::PipePrintfHead(pipe) => run_pipe_printf_head(pipe, stdout),
        NativeCommand::PipePrintfTail(pipe) => run_pipe_printf_tail(pipe, stdout),
        NativeCommand::PipePrintfGrep(pipe) => run_pipe_printf_grep(pipe, stdout),
        NativeCommand::PipePrintfTr(pipe) => run_pipe_printf_tr(pipe, stdout),
        NativeCommand::PipePrintfAwkProducer(pipe) => {
            run_pipe_printf_awk_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipePrintfProducer(pipe) => run_pipe_printf_producer(pipe, stdout, stderr),
        NativeCommand::PipePrintfLiteralProducer(pipe) => {
            run_pipe_printf_literal_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipePrintfGrepProducer(pipe) => {
            run_pipe_printf_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipePrintfSortUniqProducer(pipe) => {
            run_pipe_printf_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipePrintfGrepSortUniqProducer(pipe) => {
            run_pipe_printf_grep_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipePrintfXargsEcho(pipe) => run_pipe_printf_xargs_echo(pipe, stdout),
        NativeCommand::PipePrintfXargsWcLines(pipe) => {
            run_pipe_printf_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeSeqWcLines(pipe) => run_pipe_seq_wc_lines(pipe, stdout),
        NativeCommand::PipeSeqHead(pipe) => run_pipe_seq_head(pipe, stdout),
        NativeCommand::PipeSeqTail(pipe) => run_pipe_seq_tail(pipe, stdout),
        NativeCommand::PipeSeqGrepProducer(pipe) => {
            run_pipe_seq_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSeqProducer(pipe) => run_pipe_seq_producer(pipe, stdout, stderr),
        NativeCommand::PipeSeqSortUniqProducer(pipe) => {
            run_pipe_seq_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSeqGrepSortUniqProducer(pipe) => {
            run_pipe_seq_grep_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSeqXargsEcho(pipe) => run_pipe_seq_xargs_echo(pipe, stdout),
        NativeCommand::PipeYesHead(pipe) => run_pipe_yes_head(pipe, stdout),
        NativeCommand::PipePathLookupWcLines(pipe) => run_pipe_path_lookup_wc_lines(pipe, stdout),
        NativeCommand::PipePathLookupHead(pipe) => run_pipe_path_lookup_head(pipe, stdout),
        NativeCommand::PipePathLookupTail(pipe) => run_pipe_path_lookup_tail(pipe, stdout),
        NativeCommand::PipePathLookupGrepProducer(pipe) => {
            run_pipe_path_lookup_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipePathLookupProducer(pipe) => {
            run_pipe_path_lookup_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeEnvironmentWcLines(pipe) => run_pipe_environment_wc_lines(pipe, stdout),
        NativeCommand::PipeEnvironmentHead(pipe) => run_pipe_environment_head(pipe, stdout),
        NativeCommand::PipeEnvironmentTail(pipe) => run_pipe_environment_tail(pipe, stdout),
        NativeCommand::PipeEnvironmentGrep(pipe) => run_pipe_environment_grep(pipe, stdout),
        NativeCommand::PipeEnvironmentGrepProducer(pipe) => {
            run_pipe_environment_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeEnvironmentSort(pipe) => run_pipe_environment_sort(pipe, stdout),
        NativeCommand::PipeHostnameWcLines(pipe) => {
            run_pipe_hostname_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeHostnameHead(pipe) => run_pipe_hostname_head(pipe, stdout, stderr),
        NativeCommand::PipeHostnameTail(pipe) => run_pipe_hostname_tail(pipe, stdout, stderr),
        NativeCommand::PipeHostnameGrep(pipe) => run_pipe_hostname_grep(pipe, stdout, stderr),
        NativeCommand::PipeHostnameGrepProducer(pipe) => {
            run_pipe_hostname_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeHostnameSort(pipe) => run_pipe_hostname_sort(pipe, stdout, stderr),
        NativeCommand::PipeSortUniq(pipe) => run_pipe_sort_uniq(pipe, stdout),
        NativeCommand::PipeSortUniqWcLines(pipe) => run_pipe_sort_uniq_wc_lines(pipe, stdout),
        NativeCommand::PipeSortUniqProducer(pipe) => {
            run_pipe_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSortUniqGrepProducer(pipe) => {
            run_pipe_sort_uniq_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSortGrepProducer(pipe) => {
            run_pipe_sort_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeSortHead(pipe) => run_pipe_sort_head(pipe, stdout),
        NativeCommand::PipeSortTail(pipe) => run_pipe_sort_tail(pipe, stdout),
        NativeCommand::PipeSortWcLines(pipe) => run_pipe_sort_wc_lines(pipe, stdout),
        NativeCommand::PipeSortXargsEcho(pipe) => run_pipe_sort_xargs_echo(pipe, stdout),
        NativeCommand::PipeSortXargsWcLines(pipe) => {
            run_pipe_sort_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeSortXargsWcProducer(pipe) => {
            run_pipe_sort_xargs_wc_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindXargsEcho(pipe) => run_pipe_find_xargs_echo(pipe, stdout, stderr),
        NativeCommand::PipeFindXargsWcLines(pipe) => {
            run_pipe_find_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindXargsWcProducer(pipe) => {
            run_pipe_find_xargs_wc_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindGrepProducer(pipe) => {
            run_pipe_find_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindGrepXargsEcho(pipe) => {
            run_pipe_find_grep_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindGrepXargsWcLines(pipe) => {
            run_pipe_find_grep_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindGrepSortXargsEcho(pipe) => {
            run_pipe_find_grep_sort_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindGrepSortXargsWcLines(pipe) => {
            run_pipe_find_grep_sort_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindWcLines(pipe) => run_pipe_find_wc_lines(pipe, stdout, stderr),
        NativeCommand::PipeFindHead(pipe) => run_pipe_find_head(pipe, stdout, stderr),
        NativeCommand::PipeFindTail(pipe) => run_pipe_find_tail(pipe, stdout, stderr),
        NativeCommand::PipeFindSort(pipe) => run_pipe_find_sort(pipe, stdout, stderr),
        NativeCommand::PipeFindSortUniq(pipe) => run_pipe_find_sort_uniq(pipe, stdout, stderr),
        NativeCommand::PipeFindSortUniqWcLines(pipe) => {
            run_pipe_find_sort_uniq_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindSortUniqProducer(pipe) => {
            run_pipe_find_sort_uniq_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindSortUniqGrepProducer(pipe) => {
            run_pipe_find_sort_uniq_grep_producer(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindSortXargsEcho(pipe) => {
            run_pipe_find_sort_xargs_echo(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindSortXargsWcLines(pipe) => {
            run_pipe_find_sort_xargs_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindSortWcLines(pipe) => {
            run_pipe_find_sort_wc_lines(pipe, stdout, stderr)
        }
        NativeCommand::PipeFindSortHead(pipe) => run_pipe_find_sort_head(pipe, stdout, stderr),
        NativeCommand::PipeFindSortTail(pipe) => run_pipe_find_sort_tail(pipe, stdout, stderr),
        NativeCommand::Find(find) => run_find(find, stdout, stderr),
        NativeCommand::SedPrint(sed) => run_sed_print(sed, stdout, stderr),
        NativeCommand::WcAll(wc) => run_wc_all(wc, stdout, stderr),
        NativeCommand::WcLines(wc) => run_wc_lines(wc, stdout, stderr),
    }
}

fn run_pwd(stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    match env::current_dir() {
        Ok(cwd) => {
            writeln!(stdout, "{}", cwd.display())?;
            Ok(0)
        }
        Err(e) => {
            writeln!(stderr, "pwd: {e}")?;
            Ok(1)
        }
    }
}

fn run_echo(plan: &EchoPlan, stdout: &mut dyn Write) -> Result<i32> {
    write_echo(plan, stdout)?;
    Ok(0)
}

fn write_echo(plan: &EchoPlan, stdout: &mut dyn Write) -> Result<()> {
    for (idx, arg) in plan.args.iter().enumerate() {
        if idx > 0 {
            write!(stdout, " ")?;
        }
        write!(stdout, "{arg}")?;
    }
    if plan.newline {
        writeln!(stdout)?;
    }
    Ok(())
}

fn run_printf(plan: &PrintfPlan, stdout: &mut dyn Write) -> Result<i32> {
    write_printf(plan.format, &plan.args, stdout)?;
    Ok(0)
}

fn run_printf_literal(plan: &PrintfLiteralPlan, stdout: &mut dyn Write) -> Result<i32> {
    stdout.write_all(&plan.bytes)?;
    Ok(0)
}

fn write_printf(format: PrintfFormat, args: &[String], stdout: &mut dyn Write) -> Result<()> {
    match format {
        PrintfFormat::String => {
            for arg in args {
                write!(stdout, "{arg}")?;
            }
        }
        PrintfFormat::StringNewline => {
            for arg in args {
                writeln!(stdout, "{arg}")?;
            }
        }
    }
    Ok(())
}

fn run_seq(plan: &SeqPlan, stdout: &mut dyn Write) -> Result<i32> {
    write_seq(plan, stdout, u64::MAX)?;
    Ok(0)
}

fn seq_count(plan: &SeqPlan) -> u64 {
    let first = plan.first as i128;
    let last = plan.last as i128;
    let step = plan.step as i128;
    if step > 0 {
        if first > last {
            return 0;
        }
        (((last - first) / step) + 1) as u64
    } else {
        if first < last {
            return 0;
        }
        (((first - last) / -step) + 1) as u64
    }
}

fn write_seq(plan: &SeqPlan, stdout: &mut dyn Write, limit: u64) -> Result<()> {
    let mut current = plan.first as i128;
    let step = plan.step as i128;
    let last = plan.last as i128;
    let mut remaining = limit.min(seq_count(plan));
    while remaining > 0 {
        writeln!(stdout, "{current}")?;
        current += step;
        remaining -= 1;
        if (step > 0 && current > last) || (step < 0 && current < last) {
            break;
        }
    }
    Ok(())
}

fn write_seq_tail(plan: &SeqPlan, stdout: &mut dyn Write, limit: u64) -> Result<()> {
    let count = seq_count(plan);
    let emit = count.min(limit);
    let skip = count.saturating_sub(emit);
    let mut current = plan.first as i128 + (plan.step as i128 * skip as i128);
    let step = plan.step as i128;
    let mut remaining = emit;
    while remaining > 0 {
        writeln!(stdout, "{current}")?;
        current += step;
        remaining -= 1;
    }
    Ok(())
}

fn run_whoami(stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    match effective_user_name() {
        Some(name) => {
            writeln!(stdout, "{name}")?;
            Ok(0)
        }
        None => {
            writeln!(stderr, "whoami: cannot find name for user ID")?;
            Ok(1)
        }
    }
}

fn run_id(plan: &IdPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    match plan.kind {
        IdKind::Default => match default_id_value() {
            Ok(value) => {
                writeln!(stdout, "{value}")?;
                Ok(0)
            }
            Err(message) => {
                writeln!(stderr, "{message}")?;
                Ok(1)
            }
        },
        IdKind::UserId => {
            writeln!(stdout, "{}", unsafe { libc::geteuid() })?;
            Ok(0)
        }
        IdKind::UserName => match effective_user_name() {
            Some(name) => {
                writeln!(stdout, "{name}")?;
                Ok(0)
            }
            None => {
                writeln!(stderr, "id: cannot find name for user ID")?;
                Ok(1)
            }
        },
        IdKind::GroupId => {
            writeln!(stdout, "{}", unsafe { libc::getegid() })?;
            Ok(0)
        }
        IdKind::GroupName => match effective_group_name() {
            Some(name) => {
                writeln!(stdout, "{name}")?;
                Ok(0)
            }
            None => {
                writeln!(stderr, "id: cannot find name for group ID")?;
                Ok(1)
            }
        },
        IdKind::GroupIds => match effective_group_id_list() {
            Ok(value) => {
                writeln!(stdout, "{value}")?;
                Ok(0)
            }
            Err(message) => {
                writeln!(stderr, "{message}")?;
                Ok(1)
            }
        },
        IdKind::GroupNames => match effective_group_name_list() {
            Ok(value) => {
                writeln!(stdout, "{value}")?;
                Ok(0)
            }
            Err(message) => {
                writeln!(stderr, "{message}")?;
                Ok(1)
            }
        },
    }
}

fn effective_user_name() -> Option<String> {
    let uid = unsafe { libc::geteuid() };
    let pw = unsafe { libc::getpwuid(uid) };
    if pw.is_null() {
        return None;
    }
    let name = unsafe { (*pw).pw_name };
    if name.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .to_string(),
    )
}

fn effective_group_name() -> Option<String> {
    let gid = unsafe { libc::getegid() };
    let group = unsafe { libc::getgrgid(gid) };
    if group.is_null() {
        return None;
    }
    let name = unsafe { (*group).gr_name };
    if name.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .to_string(),
    )
}

fn effective_group_ids() -> std::result::Result<Vec<libc::gid_t>, &'static str> {
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count < 0 {
        return Err("id: cannot get groups");
    }
    let mut groups = vec![0 as libc::gid_t; count as usize];
    let read = unsafe { libc::getgroups(count, groups.as_mut_ptr()) };
    if read < 0 {
        return Err("id: cannot get groups");
    }
    groups.truncate(read as usize);
    Ok(groups)
}

fn effective_group_id_list() -> std::result::Result<String, &'static str> {
    Ok(effective_group_ids()?
        .into_iter()
        .map(|gid| gid.to_string())
        .collect::<Vec<_>>()
        .join(" "))
}

fn effective_group_name_list() -> std::result::Result<String, &'static str> {
    let mut names = Vec::new();
    for gid in effective_group_ids()? {
        let group = unsafe { libc::getgrgid(gid) };
        if group.is_null() {
            return Err("id: cannot find name for group ID");
        }
        let name = unsafe { (*group).gr_name };
        if name.is_null() {
            return Err("id: cannot find name for group ID");
        }
        names.push(
            unsafe { CStr::from_ptr(name) }
                .to_string_lossy()
                .to_string(),
        );
    }
    Ok(names.join(" "))
}

fn default_id_value() -> std::result::Result<String, &'static str> {
    let uid = unsafe { libc::geteuid() };
    let gid = unsafe { libc::getegid() };
    let user = effective_user_name()
        .map(|name| format!("{uid}({name})"))
        .unwrap_or_else(|| uid.to_string());
    let group = effective_group_name()
        .map(|name| format!("{gid}({name})"))
        .unwrap_or_else(|| gid.to_string());
    let groups = effective_group_ids()?
        .into_iter()
        .map(|group_id| {
            group_name_for_id(group_id)
                .map(|name| format!("{group_id}({name})"))
                .unwrap_or_else(|| group_id.to_string())
        })
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("uid={user} gid={group} groups={groups}"))
}

fn group_name_for_id(gid: libc::gid_t) -> Option<String> {
    let group = unsafe { libc::getgrgid(gid) };
    if group.is_null() {
        return None;
    }
    let name = unsafe { (*group).gr_name };
    if name.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .to_string(),
    )
}

fn run_uname(plan: &UnamePlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let Some(fields) = uname_fields() else {
        writeln!(stderr, "uname: {}", io::Error::last_os_error())?;
        return Ok(1);
    };
    match plan.field {
        UnameField::Sysname => writeln!(stdout, "{}", fields.sysname)?,
        UnameField::Nodename => writeln!(stdout, "{}", fields.nodename)?,
        UnameField::Release => writeln!(stdout, "{}", fields.release)?,
        UnameField::Version => writeln!(stdout, "{}", fields.version)?,
        UnameField::Machine => writeln!(stdout, "{}", fields.machine)?,
        UnameField::Processor => writeln!(stdout, "{}", processor_from_machine(&fields.machine))?,
        UnameField::All => writeln!(
            stdout,
            "{} {} {} {} {}",
            fields.sysname, fields.nodename, fields.release, fields.version, fields.machine
        )?,
    }
    Ok(0)
}

fn run_hostname(stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let Some(hostname) = hostname_value() else {
        writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
        return Ok(1);
    };
    writeln!(stdout, "{hostname}")?;
    Ok(0)
}

fn hostname_value() -> Option<String> {
    let mut buf = [0 as libc::c_char; 256];
    if unsafe { libc::gethostname(buf.as_mut_ptr(), buf.len()) } != 0 {
        return None;
    }
    let end = buf
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(buf.len().saturating_sub(1));
    let bytes = buf[..end].iter().map(|ch| *ch as u8).collect::<Vec<_>>();
    Some(String::from_utf8_lossy(&bytes).to_string())
}

struct UnameFields {
    sysname: String,
    nodename: String,
    release: String,
    version: String,
    machine: String,
}

fn uname_fields() -> Option<UnameFields> {
    let mut uts = std::mem::MaybeUninit::<libc::utsname>::zeroed();
    if unsafe { libc::uname(uts.as_mut_ptr()) } != 0 {
        return None;
    }
    let uts = unsafe { uts.assume_init() };
    Some(UnameFields {
        sysname: c_char_array_to_string(&uts.sysname),
        nodename: c_char_array_to_string(&uts.nodename),
        release: c_char_array_to_string(&uts.release),
        version: c_char_array_to_string(&uts.version),
        machine: c_char_array_to_string(&uts.machine),
    })
}

fn processor_from_machine(machine: &str) -> String {
    if cfg!(target_os = "macos") {
        match machine {
            "arm64" | "aarch64" => "arm".to_string(),
            "x86_64" => "i386".to_string(),
            other => other.to_string(),
        }
    } else {
        machine.to_string()
    }
}

fn c_char_array_to_string(buf: &[libc::c_char]) -> String {
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .to_string()
}

fn run_test(plan: &TestPlan) -> Result<i32> {
    let value = eval_test_expr(&plan.expr);
    Ok(if value ^ plan.negated { 0 } else { 1 })
}

fn eval_test_expr(expr: &TestExpr) -> bool {
    match expr {
        TestExpr::FileExists(path) => Path::new(path).exists(),
        TestExpr::FileRegular(path) => Path::new(path).is_file(),
        TestExpr::FileDirectory(path) => Path::new(path).is_dir(),
        TestExpr::FileNonEmpty(path) => fs::metadata(path).is_ok_and(|meta| meta.len() > 0),
        TestExpr::StringNonEmpty(value) => !value.is_empty(),
        TestExpr::StringEmpty(value) => value.is_empty(),
        TestExpr::StringEq(left, right) => left == right,
        TestExpr::StringNe(left, right) => left != right,
        TestExpr::IntEq(left, right) => left == right,
        TestExpr::IntNe(left, right) => left != right,
        TestExpr::IntGt(left, right) => left > right,
        TestExpr::IntGe(left, right) => left >= right,
        TestExpr::IntLt(left, right) => left < right,
        TestExpr::IntLe(left, right) => left <= right,
    }
}

fn run_basename(plan: &BasenamePlan, stdout: &mut dyn Write) -> Result<i32> {
    writeln!(
        stdout,
        "{}",
        basename_value(&plan.path, plan.suffix.as_deref())
    )?;
    Ok(0)
}

fn run_dirname(plan: &DirnamePlan, stdout: &mut dyn Write) -> Result<i32> {
    writeln!(stdout, "{}", dirname_value(&plan.path))?;
    Ok(0)
}

fn run_ls(plan: &LsPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let path = Path::new(&plan.path);
    if path.is_dir() {
        let names = collect_ls_names(&plan.path, plan.mode)?;
        for name in names {
            writeln!(stdout, "{name}")?;
        }
        return Ok(0);
    }

    if path.exists() {
        writeln!(stdout, "{}", plan.path)?;
        Ok(0)
    } else {
        writeln!(stderr, "ls: {}: No such file or directory", plan.path)?;
        Ok(1)
    }
}

fn collect_ls_names(path: &str, mode: LsEntryMode) -> Result<Vec<String>> {
    let mut names = Vec::new();
    if mode == LsEntryMode::All {
        names.push(".".to_string());
        names.push("..".to_string());
    }
    for entry in fs::read_dir(path).with_context(|| format!("reading {path}"))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if mode != LsEntryMode::Visible || !name.starts_with('.') {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

fn write_names<'a>(
    names: impl IntoIterator<Item = &'a String>,
    stdout: &mut dyn Write,
) -> Result<()> {
    let mut buffered = io::BufWriter::new(stdout);
    for name in names {
        writeln!(buffered, "{name}")?;
    }
    Ok(())
}

fn run_pipe_ls_wc_lines(
    plan: &PipeLsWcLinesPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    writeln!(stdout, "{:8}", names.len())?;
    Ok(0)
}

fn run_pipe_ls_head(
    plan: &PipeLsHeadPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    write_names(
        names
            .iter()
            .take(usize::try_from(plan.limit).unwrap_or(usize::MAX)),
        stdout,
    )?;
    Ok(0)
}

fn run_pipe_ls_tail(
    plan: &PipeLsTailPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    let limit = usize::try_from(plan.limit).unwrap_or(usize::MAX);
    let start = names.len().saturating_sub(limit);
    write_names(names.iter().skip(start), stdout)?;
    Ok(0)
}

fn run_pipe_ls_sort(
    plan: &PipeLsSortPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    write_names(names.iter(), stdout)?;
    Ok(0)
}

fn run_pipe_ls_sort_xargs_echo(
    plan: &PipeLsSortXargsEchoPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    write_xargs_echo_paths(&names, stdout)?;
    Ok(0)
}

fn run_pipe_ls_sort_uniq(
    plan: &PipeLsSortUniqPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let mut names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    names.dedup();
    write_names(names.iter(), stdout)?;
    Ok(0)
}

fn run_pipe_ls_sort_uniq_wc_lines(
    plan: &PipeLsSortUniqWcLinesPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let mut names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    names.dedup();
    writeln!(stdout, "{:8}", names.len())?;
    Ok(0)
}

fn collect_ls_sort_uniq_lines(source: &LsPipeSource) -> Result<Vec<Vec<u8>>> {
    let mut names = collect_ls_names(&source.path, source.mode)?;
    names.dedup();
    Ok(names
        .into_iter()
        .map(|name| {
            let mut line = name.into_bytes();
            line.push(b'\n');
            line
        })
        .collect())
}

fn collect_ls_sort_uniq_grep_lines(source: &LsPipeSource, pattern: &str) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_ls_sort_uniq_lines(source)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn collect_ls_grep_lines(source: &LsPipeSource, pattern: &str) -> Result<Vec<Vec<u8>>> {
    Ok(matching_ls_names(source, pattern)?
        .into_iter()
        .map(|name| {
            let mut line = name.into_bytes();
            line.push(b'\n');
            line
        })
        .collect())
}

fn run_pipe_ls_sort_uniq_producer(
    plan: &PipeLsSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_ls_sort_uniq_lines(&plan.source)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_ls_sort_uniq_grep_producer(
    plan: &PipeLsSortUniqGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_ls_sort_uniq_grep_lines(&plan.source, &plan.pattern)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_ls_grep(
    plan: &PipeLsGrepPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    let matched = names
        .iter()
        .filter(|name| name.contains(&plan.pattern))
        .collect::<Vec<_>>();
    write_names(matched.iter().copied(), stdout)?;
    Ok(if matched.is_empty() { 1 } else { 0 })
}

fn run_pipe_ls_grep_producer(
    plan: &PipeLsGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_ls_grep_lines(&plan.source, &plan.pattern)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_ls_grep_wc_lines(
    plan: &PipeLsGrepWcLinesPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    let matches = names
        .iter()
        .filter(|name| name.contains(&plan.pattern))
        .count();
    writeln!(stdout, "{matches:8}")?;
    Ok(0)
}

fn run_pipe_ls_grep_xargs_echo(
    plan: &PipeLsGrepXargsEchoPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = matching_ls_names(&plan.source, &plan.pattern)?;
    write_xargs_echo_paths(&names, stdout)?;
    Ok(0)
}

fn run_pipe_ls_grep_sort_xargs_echo(
    plan: &PipeLsGrepSortXargsEchoPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = matching_ls_names(&plan.source, &plan.pattern)?;
    write_xargs_echo_paths(&names, stdout)?;
    Ok(0)
}

fn matching_ls_names(source: &LsPipeSource, pattern: &str) -> Result<Vec<String>> {
    Ok(collect_ls_names(&source.path, source.mode)?
        .into_iter()
        .filter(|name| name.contains(pattern))
        .collect())
}

fn run_pipe_ls_xargs_echo(
    plan: &PipeLsXargsEchoPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let names = collect_ls_names(&plan.source.path, plan.source.mode)?;
    write_xargs_echo_paths(&names, stdout)?;
    Ok(0)
}

fn run_sort(plan: &SortPlan, stdout: &mut dyn Write) -> Result<i32> {
    let sorted = if plan.stdin {
        let stdin = io::stdin();
        sorted_reader_lines(stdin.lock())?
    } else {
        sorted_file_lines(&plan.file)?
    };
    let mut buffered = io::BufWriter::new(stdout);
    for span in sorted.lines.iter().copied() {
        write_line_span(&sorted.data, span, &mut buffered)?;
    }
    Ok(0)
}

struct SortedLines {
    data: Vec<u8>,
    lines: Vec<(usize, usize)>,
}

fn sorted_file_lines(file: &str) -> Result<SortedLines> {
    let data = fs::read(file).with_context(|| format!("reading {file}"))?;
    Ok(sorted_data_lines(data))
}

fn run_uniq(plan: &UniqPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    if plan.stdin {
        let stdin = io::stdin();
        run_uniq_reader(stdin.lock(), stdout)?;
        return Ok(0);
    }
    let file = match fs::File::open(&plan.file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "uniq: {}: {e}", plan.file)?;
            return Ok(1);
        }
    };
    run_uniq_reader(file, stdout)?;
    Ok(0)
}

fn run_uniq_reader<R: Read>(reader: R, stdout: &mut dyn Write) -> Result<()> {
    let mut reader = BufReader::new(reader);
    let mut previous: Option<Vec<u8>> = None;
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = reader.read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        if previous.as_deref() != Some(line.as_slice()) {
            stdout.write_all(&line)?;
            previous = Some(line.clone());
        }
    }
    Ok(())
}

fn file_lines(file: &str) -> Result<SortedLines> {
    let data = fs::read(file).with_context(|| format!("reading {file}"))?;
    let lines = line_spans(&data);
    Ok(SortedLines { data, lines })
}

fn line_spans(data: &[u8]) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut start = 0;
    while start < data.len() {
        let mut end = start;
        while end < data.len() && data[end] != b'\n' {
            end += 1;
        }
        let next = if end < data.len() { end + 1 } else { end };
        lines.push((start, next));
        start = next;
    }
    lines
}

fn write_line_span(data: &[u8], span: (usize, usize), stdout: &mut dyn Write) -> Result<()> {
    let line = &data[span.0..span.1];
    stdout.write_all(line)?;
    if !line.ends_with(b"\n") {
        stdout.write_all(b"\n")?;
    }
    Ok(())
}

fn line_without_trailing_newline(data: &[u8], span: (usize, usize)) -> &[u8] {
    let line = &data[span.0..span.1];
    line.strip_suffix(b"\n").unwrap_or(line)
}

fn unique_line_span_count(lines: &SortedLines) -> usize {
    let mut previous: Option<(usize, usize)> = None;
    let mut count = 0usize;
    for span in lines.lines.iter().copied() {
        let duplicate = previous.is_some_and(|prev| {
            line_without_trailing_newline(&lines.data, prev)
                == line_without_trailing_newline(&lines.data, span)
        });
        if !duplicate {
            count += 1;
            previous = Some(span);
        }
    }
    count
}

fn run_pipe_sort_uniq(plan: &PipeSortUniqPlan, stdout: &mut dyn Write) -> Result<i32> {
    let sorted = sorted_file_lines(&plan.file)?;
    let mut buffered = io::BufWriter::new(stdout);
    let mut previous: Option<(usize, usize)> = None;
    for span in sorted.lines.iter().copied() {
        let duplicate = previous.is_some_and(|prev| {
            line_without_trailing_newline(&sorted.data, prev)
                == line_without_trailing_newline(&sorted.data, span)
        });
        if !duplicate {
            write_line_span(&sorted.data, span, &mut buffered)?;
            previous = Some(span);
        }
    }
    Ok(0)
}

fn run_pipe_sort_uniq_wc_lines(
    plan: &PipeSortUniqWcLinesPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_sort_uniq_lines(&plan.file)?;
    writeln!(
        stdout,
        "{:8}",
        count_wc_byte_lines(&lines, plan.mode, false)
    )?;
    Ok(0)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_sort_uniq_lines(file: &str) -> Result<Vec<Vec<u8>>> {
    let sorted = sorted_file_lines(file)?;
    let mut previous: Option<(usize, usize)> = None;
    let mut lines = Vec::new();
    for span in sorted.lines.iter().copied() {
        let duplicate = previous.is_some_and(|prev| {
            line_without_trailing_newline(&sorted.data, prev)
                == line_without_trailing_newline(&sorted.data, span)
        });
        if !duplicate {
            let mut line = sorted.data[span.0..span.1].to_vec();
            if !line.ends_with(b"\n") {
                line.push(b'\n');
            }
            lines.push(line);
            previous = Some(span);
        }
    }
    Ok(lines)
}

fn collect_sort_uniq_grep_lines(file: &str, pattern: &str) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_sort_uniq_lines(file)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_sort_lines(file: &str) -> Result<Vec<Vec<u8>>> {
    let sorted = sorted_file_lines(file)?;
    let mut lines = Vec::new();
    for span in sorted.lines.iter().copied() {
        let mut line = sorted.data[span.0..span.1].to_vec();
        if !line.ends_with(b"\n") {
            line.push(b'\n');
        }
        lines.push(line);
    }
    Ok(lines)
}

fn collect_sort_grep_lines(file: &str, pattern: &str) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_sort_lines(file)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn run_pipe_sort_uniq_producer(
    plan: &PipeSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_sort_uniq_lines(&plan.file)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_sort_uniq_grep_producer(
    plan: &PipeSortUniqGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_sort_uniq_grep_lines(&plan.file, &plan.pattern)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_sort_grep_producer(
    plan: &PipeSortGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_sort_grep_lines(&plan.file, &plan.pattern)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_sort_head(plan: &PipeSortHeadPlan, stdout: &mut dyn Write) -> Result<i32> {
    let sorted = sorted_file_lines(&plan.file)?;
    let mut buffered = io::BufWriter::new(stdout);
    for span in sorted
        .lines
        .iter()
        .copied()
        .take(usize::try_from(plan.limit).unwrap_or(usize::MAX))
    {
        write_line_span(&sorted.data, span, &mut buffered)?;
    }
    Ok(0)
}

fn run_pipe_sort_tail(plan: &PipeSortTailPlan, stdout: &mut dyn Write) -> Result<i32> {
    let sorted = sorted_file_lines(&plan.file)?;
    let mut buffered = io::BufWriter::new(stdout);
    let limit = usize::try_from(plan.limit).unwrap_or(usize::MAX);
    let start = sorted.lines.len().saturating_sub(limit);
    for span in sorted.lines.iter().copied().skip(start) {
        write_line_span(&sorted.data, span, &mut buffered)?;
    }
    Ok(0)
}

fn run_pipe_sort_wc_lines(plan: &PipeSortWcLinesPlan, stdout: &mut dyn Write) -> Result<i32> {
    let lines = collect_sort_lines(&plan.file)?;
    writeln!(
        stdout,
        "{:8}",
        count_wc_byte_lines(&lines, plan.mode, false)
    )?;
    Ok(0)
}

fn run_pipe_sort_xargs_echo(plan: &PipeSortXargsEchoPlan, stdout: &mut dyn Write) -> Result<i32> {
    let sorted = sorted_file_lines(&plan.file)?;
    let mut first = true;
    for span in sorted.lines.iter().copied() {
        let text = String::from_utf8_lossy(&sorted.data[span.0..span.1]);
        write_xargs_echo_path_tokens(&text, stdout, &mut first)?;
    }
    if !first {
        writeln!(stdout)?;
    }
    Ok(0)
}

fn run_pipe_sort_xargs_wc_lines(
    plan: &PipeSortXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let sorted = sorted_file_lines(&plan.file)?;
    let mut paths = Vec::new();
    for span in sorted.lines.iter().copied() {
        paths.extend(xargs_tokens_from_bytes(&sorted.data[span.0..span.1]));
    }
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn run_pipe_sort_xargs_wc_producer(
    plan: &PipeSortXargsWcProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let sorted = sorted_file_lines(&plan.file)?;
    let mut paths = Vec::new();
    for span in sorted.lines.iter().copied() {
        paths.extend(xargs_tokens_from_bytes(&sorted.data[span.0..span.1]));
    }
    write_xargs_wc_output_paths(&paths, plan.mode, stdout, stderr)
}

fn run_pipe_cat_sort(plan: &PipeCatSortPlan, stdout: &mut dyn Write) -> Result<i32> {
    run_sort(
        &SortPlan {
            file: plan.file.clone(),
            stdin: false,
        },
        stdout,
    )
}

fn run_pipe_cat_sort_uniq(plan: &PipeCatSortUniqPlan, stdout: &mut dyn Write) -> Result<i32> {
    run_pipe_sort_uniq(
        &PipeSortUniqPlan {
            file: plan.file.clone(),
        },
        stdout,
    )
}

fn run_pipe_cat_sort_uniq_wc_lines(
    plan: &PipeCatSortUniqWcLinesPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    run_pipe_sort_uniq_wc_lines(
        &PipeSortUniqWcLinesPlan {
            file: plan.file.clone(),
            mode: plan.mode,
        },
        stdout,
    )
}

fn run_pipe_cat_sort_head(plan: &PipeCatSortHeadPlan, stdout: &mut dyn Write) -> Result<i32> {
    run_pipe_sort_head(
        &PipeSortHeadPlan {
            file: plan.file.clone(),
            limit: plan.limit,
        },
        stdout,
    )
}

fn run_pipe_cat_sort_tail(plan: &PipeCatSortTailPlan, stdout: &mut dyn Write) -> Result<i32> {
    run_pipe_sort_tail(
        &PipeSortTailPlan {
            file: plan.file.clone(),
            limit: plan.limit,
        },
        stdout,
    )
}

fn run_pipe_cat_sort_wc_lines(
    plan: &PipeCatSortWcLinesPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    run_pipe_sort_wc_lines(
        &PipeSortWcLinesPlan {
            file: plan.file.clone(),
            mode: plan.mode,
        },
        stdout,
    )
}

fn run_pipe_cat_xargs_echo(plan: &PipeCatXargsEchoPlan, stdout: &mut dyn Write) -> Result<i32> {
    let data = fs::read(&plan.file).with_context(|| format!("reading {}", plan.file))?;
    write_xargs_echo_byte_tokens(&data, stdout)?;
    Ok(0)
}

fn run_pipe_cat_xargs_wc_lines(
    plan: &PipeCatXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let data = fs::read(&plan.file).with_context(|| format!("reading {}", plan.file))?;
    let paths = xargs_tokens_from_bytes(&data);
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn run_pipe_cat_xargs_wc_producer(
    plan: &PipeCatXargsWcProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let data = fs::read(&plan.file).with_context(|| format!("reading {}", plan.file))?;
    let paths = xargs_tokens_from_bytes(&data);
    write_xargs_wc_output_paths(&paths, plan.mode, stdout, stderr)
}

fn run_pipe_cat_sort_xargs_echo(
    plan: &PipeCatSortXargsEchoPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    run_pipe_sort_xargs_echo(
        &PipeSortXargsEchoPlan {
            file: plan.file.clone(),
        },
        stdout,
    )
}

fn run_pipe_cat_sort_xargs_wc_lines(
    plan: &PipeCatSortXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    run_pipe_sort_xargs_wc_lines(
        &PipeSortXargsWcLinesPlan {
            file: plan.file.clone(),
        },
        stdout,
        stderr,
    )
}

fn run_pipe_cat_sort_xargs_wc_producer(
    plan: &PipeCatSortXargsWcProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    run_pipe_sort_xargs_wc_producer(
        &PipeSortXargsWcProducerPlan {
            file: plan.file.clone(),
            mode: plan.mode,
        },
        stdout,
        stderr,
    )
}

fn run_pipe_cat_uniq(plan: &PipeCatUniqPlan, stdout: &mut dyn Write) -> Result<i32> {
    let lines = file_lines(&plan.file)?;
    let mut buffered = io::BufWriter::new(stdout);
    let mut previous: Option<(usize, usize)> = None;
    for span in lines.lines.iter().copied() {
        let duplicate = previous.is_some_and(|prev| {
            line_without_trailing_newline(&lines.data, prev)
                == line_without_trailing_newline(&lines.data, span)
        });
        if !duplicate {
            write_line_span(&lines.data, span, &mut buffered)?;
            previous = Some(span);
        }
    }
    Ok(0)
}

fn run_pipe_cat_uniq_wc_lines(
    plan: &PipeCatUniqWcLinesPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let lines = file_lines(&plan.file)?;
    writeln!(stdout, "{:8}", unique_line_span_count(&lines))?;
    Ok(0)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_uniq_lines_for_command(
    command_name: &str,
    file: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let file_handle = match fs::File::open(file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "{command_name}: {file}: {e}")?;
            return Ok(Vec::new());
        }
    };
    let mut reader = BufReader::new(file_handle);
    let mut line = Vec::new();
    let mut previous: Option<Vec<u8>> = None;
    let mut lines = Vec::new();
    loop {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "{command_name}: {file}: {e}")?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        let comparable = line
            .strip_suffix(b"\n")
            .map_or_else(|| line.clone(), |line| line.to_vec());
        if previous.as_deref() != Some(comparable.as_slice()) {
            if !line.ends_with(b"\n") {
                line.push(b'\n');
            }
            lines.push(std::mem::take(&mut line));
            previous = Some(comparable);
        }
    }
    Ok(lines)
}

fn collect_cat_uniq_lines(file: &str, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    collect_uniq_lines_for_command("cat", file, stderr)
}

fn collect_cat_uniq_grep_lines(
    file: &str,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_cat_uniq_lines(file, stderr)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn collect_uniq_lines(file: &str, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    collect_uniq_lines_for_command("uniq", file, stderr)
}

fn collect_uniq_grep_lines(
    file: &str,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_uniq_lines(file, stderr)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn run_pipe_cat_uniq_producer(
    plan: &PipeCatUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cat_uniq_lines(&plan.file, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_cat_uniq_grep_producer(
    plan: &PipeCatUniqGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cat_uniq_grep_lines(&plan.file, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_uniq_producer(
    plan: &PipeUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_uniq_lines(&plan.file, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_uniq_grep_producer(
    plan: &PipeUniqGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_uniq_grep_lines(&plan.file, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_cut(plan: &CutPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    if plan.stdin {
        let stdin = io::stdin();
        return run_cut_reader(stdin.lock(), plan.delimiter, plan.field, stdout).map(|_| 0);
    }
    run_cut_file(
        &plan.file,
        plan.delimiter,
        plan.field,
        "cut",
        stdout,
        stderr,
    )
}

fn run_pipe_cat_cut(
    plan: &PipeCatCutPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    run_cut_file(
        &plan.file,
        plan.delimiter,
        plan.field,
        "cat",
        stdout,
        stderr,
    )
    .map(|_| 0)
}

fn run_cut_file(
    file: &str,
    delimiter: u8,
    field: usize,
    error_command: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let file_handle = match fs::File::open(file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "{error_command}: {file}: {e}")?;
            return Ok(1);
        }
    };
    run_cut_reader(BufReader::new(file_handle), delimiter, field, stdout)?;
    Ok(0)
}

fn run_cut_reader<R: BufRead>(
    mut reader: R,
    delimiter: u8,
    field: usize,
    stdout: &mut dyn Write,
) -> Result<()> {
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = reader.read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        write_cut_line(&line, delimiter, field, stdout)?;
    }
    Ok(())
}

fn write_cut_line(line: &[u8], delimiter: u8, field: usize, stdout: &mut dyn Write) -> Result<()> {
    let content = line.strip_suffix(b"\n").unwrap_or(line);
    if !content.contains(&delimiter) {
        stdout.write_all(content)?;
        stdout.write_all(b"\n")?;
        return Ok(());
    }
    let mut current_field = 1usize;
    let mut start = 0usize;
    for (idx, byte) in content.iter().enumerate() {
        if *byte != delimiter {
            continue;
        }
        if current_field == field {
            stdout.write_all(&content[start..idx])?;
            stdout.write_all(b"\n")?;
            return Ok(());
        }
        current_field += 1;
        start = idx + 1;
    }
    if current_field == field {
        stdout.write_all(&content[start..])?;
    }
    stdout.write_all(b"\n")?;
    Ok(())
}

fn run_tr(plan: &TrPlan, stdout: &mut dyn Write) -> Result<i32> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    transform_reader(&mut input, &plan.mode, stdout)?;
    Ok(0)
}

fn run_pipe_cat_tr(
    plan: &PipeCatTrPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut file = match fs::File::open(&plan.file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "cat: {}: {e}", plan.file)?;
            return Ok(0);
        }
    };
    transform_reader(&mut file, &plan.tr.mode, stdout)?;
    Ok(0)
}

fn transform_reader(input: &mut dyn Read, mode: &TrMode, stdout: &mut dyn Write) -> Result<()> {
    let mut buf = [0u8; 8192];
    loop {
        let read = input.read(&mut buf)?;
        if read == 0 {
            return Ok(());
        }
        transform_bytes(&buf[..read], mode, stdout)?;
    }
}

fn transform_bytes(input: &[u8], mode: &TrMode, stdout: &mut dyn Write) -> Result<()> {
    stdout.write_all(&transform_bytes_to_vec(input, mode))?;
    Ok(())
}

fn transform_bytes_to_vec(input: &[u8], mode: &TrMode) -> Vec<u8> {
    match mode {
        TrMode::Translate { from, to } => {
            let mut table = [0u8; 256];
            for (idx, slot) in table.iter_mut().enumerate() {
                *slot = idx as u8;
            }
            for (from, to) in from.iter().zip(to.iter()) {
                table[usize::from(*from)] = *to;
            }
            let mut out = Vec::with_capacity(input.len());
            out.extend(input.iter().map(|byte| table[usize::from(*byte)]));
            out
        }
        TrMode::Delete { set } => {
            let mut delete = [false; 256];
            for byte in set {
                delete[usize::from(*byte)] = true;
            }
            let mut out = Vec::with_capacity(input.len());
            out.extend(
                input
                    .iter()
                    .copied()
                    .filter(|byte| !delete[usize::from(*byte)]),
            );
            out
        }
    }
}

fn run_cat(plan: &CatPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let mut exit = 0;
    for file in &plan.files {
        match fs::File::open(file) {
            Ok(mut f) => {
                io::copy(&mut f, stdout)?;
            }
            Err(e) => {
                writeln!(stderr, "cat: {file}: {e}")?;
                exit = 1;
            }
        }
    }
    Ok(exit)
}

fn run_head(plan: &HeadTailPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    if plan.stdin {
        let stdin = io::stdin();
        run_head_reader(stdin.lock(), plan.mode, plan.count, stdout)?;
        return Ok(0);
    }

    let mut file = match fs::File::open(&plan.file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "head: {}: {e}", plan.file)?;
            return Ok(1);
        }
    };
    run_head_reader(&mut file, plan.mode, plan.count, stdout)?;
    Ok(0)
}

fn run_head_reader<R: Read>(
    mut reader: R,
    mode: HeadTailMode,
    count: u64,
    stdout: &mut dyn Write,
) -> Result<()> {
    match mode {
        HeadTailMode::Bytes => {
            let mut remaining = count;
            let mut buf = [0u8; 8192];
            while remaining > 0 {
                let want = remaining.min(buf.len() as u64) as usize;
                let read = reader.read(&mut buf[..want])?;
                if read == 0 {
                    break;
                }
                stdout.write_all(&buf[..read])?;
                remaining -= read as u64;
            }
        }
        HeadTailMode::Lines => {
            if count == 0 {
                return Ok(());
            }
            let mut remaining = count;
            let mut reader = BufReader::new(reader);
            let mut line = Vec::new();
            while remaining > 0 {
                line.clear();
                let read = reader.read_until(b'\n', &mut line)?;
                if read == 0 {
                    break;
                }
                stdout.write_all(&line)?;
                remaining -= 1;
            }
        }
    }
    Ok(())
}

fn run_tail(plan: &HeadTailPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    if plan.stdin {
        let stdin = io::stdin();
        run_tail_reader(stdin.lock(), plan.mode, plan.count, stdout)?;
        return Ok(0);
    }

    let mut file = match fs::File::open(&plan.file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "tail: {}: {e}", plan.file)?;
            return Ok(1);
        }
    };
    match plan.mode {
        HeadTailMode::Bytes => {
            let len = file.metadata()?.len();
            let start = len.saturating_sub(plan.count);
            file.seek(SeekFrom::Start(start))?;
            io::copy(&mut file, stdout)?;
        }
        HeadTailMode::Lines => {
            if plan.count == 0 {
                return Ok(0);
            }
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            let mut start = 0usize;
            let mut pos = data.len();
            if pos > 0 && data[pos - 1] == b'\n' {
                pos -= 1;
            }
            let mut seen = 0u64;
            while pos > 0 {
                if data[pos - 1] == b'\n' {
                    seen += 1;
                    if seen == plan.count {
                        start = pos;
                        break;
                    }
                }
                pos -= 1;
            }
            stdout.write_all(&data[start..])?;
        }
    }
    Ok(0)
}

fn run_tail_reader<R: Read>(
    reader: R,
    mode: HeadTailMode,
    count: u64,
    stdout: &mut dyn Write,
) -> Result<()> {
    if count == 0 {
        return Ok(());
    }
    let limit = usize::try_from(count).unwrap_or(usize::MAX);
    match mode {
        HeadTailMode::Bytes => {
            let mut reader = BufReader::new(reader);
            let mut buf = [0u8; 8192];
            let mut tail = VecDeque::new();
            loop {
                let read = reader.read(&mut buf)?;
                if read == 0 {
                    break;
                }
                for byte in &buf[..read] {
                    if tail.len() == limit {
                        tail.pop_front();
                    }
                    tail.push_back(*byte);
                }
            }
            let (front, back) = tail.as_slices();
            stdout.write_all(front)?;
            stdout.write_all(back)?;
        }
        HeadTailMode::Lines => {
            let mut reader = BufReader::new(reader);
            let mut tail: VecDeque<Vec<u8>> = VecDeque::new();
            let mut line = Vec::new();
            loop {
                line.clear();
                let read = reader.read_until(b'\n', &mut line)?;
                if read == 0 {
                    break;
                }
                if tail.len() == limit {
                    tail.pop_front();
                }
                tail.push_back(line.clone());
            }
            for line in tail {
                stdout.write_all(&line)?;
            }
        }
    }
    Ok(())
}

fn collect_head_lines(file: &str, limit: u64, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    let file_handle = match fs::File::open(file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "head: {file}: {e}")?;
            return Ok(Vec::new());
        }
    };
    collect_head_reader_lines(BufReader::new(file_handle), limit, stderr, "head", file)
}

fn collect_head_reader_lines<R: BufRead>(
    mut reader: R,
    limit: u64,
    stderr: &mut dyn Write,
    command_name: &str,
    source_name: &str,
) -> Result<Vec<Vec<u8>>> {
    let mut line = Vec::new();
    let mut lines = Vec::new();
    let mut remaining = limit;
    while remaining > 0 {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "{command_name}: {source_name}: {e}")?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        lines.push(std::mem::take(&mut line));
        remaining -= 1;
    }
    Ok(lines)
}

fn collect_head_plan_lines(
    file: &str,
    stdin: bool,
    limit: u64,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    if stdin {
        let stdin = io::stdin();
        return collect_head_reader_lines(stdin.lock(), limit, stderr, "head", "stdin");
    }
    collect_head_lines(file, limit, stderr)
}

fn collect_head_grep_lines(
    file: &str,
    limit: u64,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let file_handle = match fs::File::open(file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "head: {file}: {e}")?;
            return Ok(Vec::new());
        }
    };
    collect_head_grep_reader_lines(
        BufReader::new(file_handle),
        limit,
        pattern,
        stderr,
        "head",
        file,
    )
}

fn collect_head_grep_reader_lines<R: BufRead>(
    mut reader: R,
    limit: u64,
    pattern: &str,
    stderr: &mut dyn Write,
    command_name: &str,
    source_name: &str,
) -> Result<Vec<Vec<u8>>> {
    let needle = pattern.as_bytes();
    let mut line = Vec::new();
    let mut lines = Vec::new();
    let mut remaining = limit;
    while remaining > 0 {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "{command_name}: {source_name}: {e}")?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        if needle.is_empty() || line.windows(needle.len()).any(|window| window == needle) {
            if !line.ends_with(b"\n") {
                line.push(b'\n');
            }
            lines.push(std::mem::take(&mut line));
        }
        remaining -= 1;
    }
    Ok(lines)
}

fn collect_head_grep_plan_lines(
    file: &str,
    stdin: bool,
    limit: u64,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    if stdin {
        let stdin = io::stdin();
        return collect_head_grep_reader_lines(
            stdin.lock(),
            limit,
            pattern,
            stderr,
            "head",
            "stdin",
        );
    }
    collect_head_grep_lines(file, limit, pattern, stderr)
}

fn byte_lines_from_slice(data: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (idx, byte) in data.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(data[start..=idx].to_vec());
            start = idx + 1;
        }
    }
    if start < data.len() {
        lines.push(data[start..].to_vec());
    }
    lines
}

fn collect_tail_lines(file: &str, limit: u64, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    let mut file_handle = match fs::File::open(file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "tail: {file}: {e}")?;
            return Ok(Vec::new());
        }
    };
    if limit == 0 {
        return Ok(Vec::new());
    }
    let mut data = Vec::new();
    if let Err(e) = file_handle.read_to_end(&mut data) {
        writeln!(stderr, "tail: {file}: {e}")?;
        return Ok(Vec::new());
    }
    let mut start = 0usize;
    let mut pos = data.len();
    if pos > 0 && data[pos - 1] == b'\n' {
        pos -= 1;
    }
    let mut seen = 0u64;
    while pos > 0 {
        if data[pos - 1] == b'\n' {
            seen += 1;
            if seen == limit {
                start = pos;
                break;
            }
        }
        pos -= 1;
    }
    Ok(byte_lines_from_slice(&data[start..]))
}

fn collect_tail_reader_lines<R: BufRead>(
    mut reader: R,
    limit: u64,
    stderr: &mut dyn Write,
    command_name: &str,
    source_name: &str,
) -> Result<Vec<Vec<u8>>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    let mut tail = VecDeque::new();
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "{command_name}: {source_name}: {e}")?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        if tail.len() == limit {
            tail.pop_front();
        }
        tail.push_back(line.clone());
    }
    Ok(tail.into_iter().collect())
}

fn collect_tail_plan_lines(
    file: &str,
    stdin: bool,
    limit: u64,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    if stdin {
        let stdin = io::stdin();
        return collect_tail_reader_lines(stdin.lock(), limit, stderr, "tail", "stdin");
    }
    collect_tail_lines(file, limit, stderr)
}

fn count_tail_stdin_wc_lines(limit: u64, stderr: &mut dyn Write) -> Result<u64> {
    if limit == 0 {
        return Ok(0);
    }
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buf = [0u8; 8192];
    let mut newline_count = 0u64;
    let mut saw_input = false;
    let mut last_was_newline = false;
    loop {
        let read = match reader.read(&mut buf) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "tail: stdin: {e}")?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        saw_input = true;
        last_was_newline = buf[read - 1] == b'\n';
        newline_count += buf[..read].iter().filter(|byte| **byte == b'\n').count() as u64;
    }
    if saw_input && !last_was_newline {
        if newline_count.saturating_add(1) <= limit {
            Ok(newline_count)
        } else {
            Ok(newline_count.min(limit.saturating_sub(1)))
        }
    } else {
        Ok(newline_count.min(limit))
    }
}

fn collect_tail_grep_lines(
    file: &str,
    limit: u64,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_tail_lines(file, limit, stderr)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn collect_tail_grep_plan_lines(
    file: &str,
    stdin: bool,
    limit: u64,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let source = if stdin {
        let stdin = io::stdin();
        collect_tail_reader_lines(stdin.lock(), limit, stderr, "tail", "stdin")?
    } else {
        collect_tail_lines(file, limit, stderr)?
    };
    let mut lines = filter_byte_lines_by_literal(source, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn run_xargs_wc_output_from_line_tokens(
    mut lines: Vec<Vec<u8>>,
    sort_input: bool,
    mode: XargsWcOutputMode,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    if sort_input {
        ensure_byte_lines_end_with_newline(&mut lines);
        lines.sort_unstable();
    }
    let mut paths = Vec::new();
    for line in &lines {
        paths.extend(xargs_tokens_from_bytes(line));
    }
    let (wc_lines, _xargs_exit) = collect_xargs_wc_path_lines(&paths, stderr)?;
    write_xargs_wc_output_lines(wc_lines, mode, stdout)?;
    Ok(0)
}

fn run_head_line_producer(
    mut lines: Vec<Vec<u8>>,
    mode: GrepFilePipeMode,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    grep_exit_for_empty: bool,
    wc_counts_newlines: bool,
) -> Result<i32> {
    let matched = !lines.is_empty();
    if matches!(
        mode,
        GrepFilePipeMode::Sort
            | GrepFilePipeMode::SortUniq
            | GrepFilePipeMode::SortUniqWcLines
            | GrepFilePipeMode::SortUniqWc { .. }
            | GrepFilePipeMode::SortWcLines
            | GrepFilePipeMode::SortWc { .. }
            | GrepFilePipeMode::SortHead { .. }
            | GrepFilePipeMode::SortTail { .. }
            | GrepFilePipeMode::SortXargsEcho
            | GrepFilePipeMode::SortXargsEchoBatches { .. }
            | GrepFilePipeMode::SortXargsWcLines
            | GrepFilePipeMode::SortXargsWcOutput { .. }
    ) {
        ensure_byte_lines_end_with_newline(&mut lines);
    }
    match mode {
        GrepFilePipeMode::Lines => {
            for line in lines {
                stdout.write_all(&line)?;
            }
            return Ok(if grep_exit_for_empty && !matched {
                1
            } else {
                0
            });
        }
        GrepFilePipeMode::WcLines => {
            let count = count_wc_byte_lines(&lines, WcCountMode::Lines, wc_counts_newlines);
            writeln!(stdout, "{count:8}")?;
        }
        GrepFilePipeMode::Wc { mode } => {
            let count = count_wc_byte_lines(&lines, mode, wc_counts_newlines);
            writeln!(stdout, "{count:8}")?;
        }
        GrepFilePipeMode::Head { limit } => {
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Tail { limit } => {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        GrepFilePipeMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        GrepFilePipeMode::SortUniqWc { mode } => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", count_wc_unique_byte_lines(&lines, mode))?;
        }
        GrepFilePipeMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::SortWc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortXargsEcho => {
            lines.sort_unstable();
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::SortXargsEchoBatches { size } => {
            lines.sort_unstable();
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::SortXargsWcLines => {
            lines.sort_unstable();
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::SortXargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, true, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsEcho => {
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::XargsEchoBatches { size } => {
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::XargsWcLines => {
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::XargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, false, mode, stdout, stderr);
        }
    }
    Ok(0)
}

fn run_pipe_head_producer(
    plan: &PipeHeadProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_head_plan_lines(&plan.file, plan.stdin, plan.limit, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_head_grep_producer(
    plan: &PipeHeadGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines =
        collect_head_grep_plan_lines(&plan.file, plan.stdin, plan.limit, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_tail_producer(
    plan: &PipeTailProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    if plan.stdin
        && matches!(
            plan.mode,
            GrepFilePipeMode::WcLines
                | GrepFilePipeMode::Wc {
                    mode: WcCountMode::Lines
                }
        )
    {
        let count = count_tail_stdin_wc_lines(plan.limit, stderr)?;
        writeln!(stdout, "{count:8}")?;
        return Ok(0);
    }
    let lines = collect_tail_plan_lines(&plan.file, plan.stdin, plan.limit, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_tail_grep_producer(
    plan: &PipeTailGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines =
        collect_tail_grep_plan_lines(&plan.file, plan.stdin, plan.limit, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_sed_print_lines(plan: &SedPrintPlan, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    let file_handle = match fs::File::open(&plan.file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "sed: {}: {e}", plan.file)?;
            return Ok(Vec::new());
        }
    };
    let mut reader = BufReader::new(file_handle);
    let mut line = Vec::new();
    let mut lines = Vec::new();
    for line_no in 1usize.. {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "sed: {}: {e}", plan.file)?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        if line_no >= plan.start_line && line_no <= plan.end_line {
            lines.push(std::mem::take(&mut line));
        }
        if line_no > plan.end_line {
            break;
        }
    }
    Ok(lines)
}

fn collect_sed_grep_lines(
    plan: &SedPrintPlan,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_sed_print_lines(plan, stderr)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn run_pipe_sed_producer(
    plan: &PipeSedProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_sed_print_lines(&plan.sed, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_sed_grep_producer(
    plan: &PipeSedGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_sed_grep_lines(&plan.sed, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn cut_line_bytes(line: &[u8], delimiter: u8, field: usize) -> Vec<u8> {
    let content = line.strip_suffix(b"\n").unwrap_or(line);
    let mut output = Vec::new();
    if !content.contains(&delimiter) {
        output.extend_from_slice(content);
        output.push(b'\n');
        return output;
    }
    let mut current_field = 1usize;
    let mut start = 0usize;
    for (idx, byte) in content.iter().enumerate() {
        if *byte != delimiter {
            continue;
        }
        if current_field == field {
            output.extend_from_slice(&content[start..idx]);
            output.push(b'\n');
            return output;
        }
        current_field += 1;
        start = idx + 1;
    }
    if current_field == field {
        output.extend_from_slice(&content[start..]);
    }
    output.push(b'\n');
    output
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_cut_lines(plan: &CutPlan, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    if plan.stdin {
        let stdin = io::stdin();
        return collect_cut_reader(
            stdin.lock(),
            plan.delimiter,
            plan.field,
            stderr,
            "cut",
            "stdin",
        );
    }
    let file_handle = match fs::File::open(&plan.file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "cut: {}: {e}", plan.file)?;
            return Ok(Vec::new());
        }
    };
    collect_cut_reader(
        BufReader::new(file_handle),
        plan.delimiter,
        plan.field,
        stderr,
        "cut",
        &plan.file,
    )
}

fn collect_cut_reader<R: BufRead>(
    mut reader: R,
    delimiter: u8,
    field: usize,
    stderr: &mut dyn Write,
    command_name: &str,
    source_name: &str,
) -> Result<Vec<Vec<u8>>> {
    let mut line = Vec::new();
    let mut lines = Vec::new();
    loop {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "{command_name}: {source_name}: {e}")?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        lines.push(cut_line_bytes(&line, delimiter, field));
    }
    Ok(lines)
}

fn count_cut_input_records(plan: &CutPlan, stderr: &mut dyn Write) -> Result<u64> {
    if plan.stdin {
        let stdin = io::stdin();
        return count_cut_reader_records(stdin.lock(), stderr, "cut", "stdin");
    }
    let file_handle = match fs::File::open(&plan.file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "cut: {}: {e}", plan.file)?;
            return Ok(0);
        }
    };
    count_cut_reader_records(BufReader::new(file_handle), stderr, "cut", &plan.file)
}

fn count_cut_reader_records<R: BufRead>(
    mut reader: R,
    stderr: &mut dyn Write,
    command_name: &str,
    source_name: &str,
) -> Result<u64> {
    let mut line = Vec::new();
    let mut count = 0u64;
    loop {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "{command_name}: {source_name}: {e}")?;
                break;
            }
        };
        if read == 0 {
            break;
        }
        count = count.saturating_add(1);
    }
    Ok(count)
}

fn collect_cut_grep_lines(
    plan: &CutPlan,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_cut_lines(plan, stderr)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn run_pipe_cut_producer(
    plan: &PipeCutProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    if matches!(plan.mode, GrepFilePipeMode::WcLines) {
        writeln!(stdout, "{:8}", count_cut_input_records(&plan.cut, stderr)?)?;
        return Ok(0);
    }
    let lines = collect_cut_lines(&plan.cut, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_cut_grep_producer(
    plan: &PipeCutGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cut_grep_lines(&plan.cut, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_cat_tr_lines(file: &str, tr: &TrPlan, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    let mut file_handle = match fs::File::open(file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "cat: {file}: {e}")?;
            return Ok(Vec::new());
        }
    };
    let mut data = Vec::new();
    if let Err(e) = file_handle.read_to_end(&mut data) {
        writeln!(stderr, "cat: {file}: {e}")?;
        return Ok(Vec::new());
    }
    let transformed = transform_bytes_to_vec(&data, &tr.mode);
    Ok(byte_lines_from_slice(&transformed))
}

fn collect_cat_tr_grep_lines(
    file: &str,
    tr: &TrPlan,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = filter_byte_lines_by_literal(collect_cat_tr_lines(file, tr, stderr)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn run_pipe_cat_tr_producer(
    plan: &PipeCatTrProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cat_tr_lines(&plan.file, &plan.tr, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_cat_tr_grep_producer(
    plan: &PipeCatTrGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cat_tr_grep_lines(&plan.file, &plan.tr, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn single_line_source_line(
    source: &SingleLineProducerSource,
    stderr: &mut dyn Write,
) -> Result<Option<Vec<u8>>> {
    let value = match source {
        SingleLineProducerSource::Pwd => match env::current_dir() {
            Ok(cwd) => cwd.display().to_string(),
            Err(e) => {
                writeln!(stderr, "pwd: {e}")?;
                return Ok(None);
            }
        },
        SingleLineProducerSource::Basename(plan) => {
            basename_value(&plan.path, plan.suffix.as_deref())
        }
        SingleLineProducerSource::Dirname(plan) => dirname_value(&plan.path),
        SingleLineProducerSource::Whoami => match effective_user_name() {
            Some(name) => name,
            None => {
                writeln!(stderr, "whoami: cannot find name for user ID")?;
                return Ok(None);
            }
        },
        SingleLineProducerSource::Id(plan) => match id_value(plan) {
            Ok(value) => value,
            Err(message) => {
                writeln!(stderr, "{message}")?;
                return Ok(None);
            }
        },
        SingleLineProducerSource::Uname(plan) => match uname_value(plan) {
            Some(value) => value,
            None => {
                writeln!(stderr, "uname: {}", io::Error::last_os_error())?;
                return Ok(None);
            }
        },
        SingleLineProducerSource::Hostname => match hostname_value() {
            Some(value) => value,
            None => {
                writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
                return Ok(None);
            }
        },
        SingleLineProducerSource::PrintenvName(name) => {
            if let Some(value) = env::var_os(name) {
                let mut line = value.as_os_str().as_bytes().to_vec();
                line.push(b'\n');
                return Ok(Some(line));
            } else {
                return Ok(Some(Vec::new()));
            }
        }
    };
    let mut line = value.into_bytes();
    line.push(b'\n');
    Ok(Some(line))
}

fn single_line_source_lines(
    source: &SingleLineProducerSource,
    stderr: &mut dyn Write,
) -> Result<Option<Vec<Vec<u8>>>> {
    match single_line_source_line(source, stderr)? {
        Some(line) if line.is_empty() => Ok(Some(Vec::new())),
        Some(line) => Ok(Some(vec![line])),
        None => Ok(None),
    }
}

fn id_value(plan: &IdPlan) -> std::result::Result<String, &'static str> {
    match plan.kind {
        IdKind::Default => default_id_value(),
        IdKind::UserId => Ok(unsafe { libc::geteuid() }.to_string()),
        IdKind::UserName => effective_user_name().ok_or("id: cannot find name for user ID"),
        IdKind::GroupId => Ok(unsafe { libc::getegid() }.to_string()),
        IdKind::GroupName => effective_group_name().ok_or("id: cannot find name for group ID"),
        IdKind::GroupIds => effective_group_id_list(),
        IdKind::GroupNames => effective_group_name_list(),
    }
}

fn uname_value(plan: &UnamePlan) -> Option<String> {
    let fields = uname_fields()?;
    Some(match plan.field {
        UnameField::Sysname => fields.sysname,
        UnameField::Nodename => fields.nodename,
        UnameField::Release => fields.release,
        UnameField::Version => fields.version,
        UnameField::Machine => fields.machine,
        UnameField::Processor => processor_from_machine(&fields.machine),
        UnameField::All => format!(
            "{} {} {} {} {}",
            fields.sysname, fields.nodename, fields.release, fields.version, fields.machine
        ),
    })
}

fn run_pipe_single_line_producer(
    plan: &PipeSingleLineProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let Some(lines) = single_line_source_lines(&plan.source, stderr)? else {
        return Ok(1);
    };
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_single_line_grep_producer(
    plan: &PipeSingleLineGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let Some(lines) = single_line_source_lines(&plan.source, stderr)? else {
        return Ok(1);
    };
    let mut lines = filter_byte_lines_by_literal(lines, &plan.pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_empty_producer(
    plan: &PipeEmptyProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    run_head_line_producer(Vec::new(), plan.mode, stdout, stderr, true, false)
}

fn run_side_effect_empty_source(
    source: &SideEffectEmptyProducer,
    stderr: &mut dyn Write,
) -> Result<i32> {
    match source {
        SideEffectEmptyProducer::Mkdir(plan) => run_mkdir(plan, stderr),
        SideEffectEmptyProducer::Touch(plan) => run_touch(plan, stderr),
    }
}

fn run_pipe_side_effect_empty_producer(
    plan: &PipeSideEffectEmptyProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let _ = run_side_effect_empty_source(&plan.source, stderr)?;
    run_head_line_producer(Vec::new(), plan.mode, stdout, stderr, true, false)
}

fn run_pipe_predicate_empty_producer(
    plan: &PipePredicateEmptyProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let _ = run_test(&plan.test)?;
    run_head_line_producer(Vec::new(), plan.mode, stdout, stderr, true, false)
}

fn collect_wc_output_lines(plan: &WcLinesPlan, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    if plan.files.is_empty() {
        let stdin = io::stdin();
        return match count_reader_wc(stdin.lock(), plan.mode) {
            Ok(count) => Ok(vec![format!("{count:8}\n").into_bytes()]),
            Err(e) => {
                writeln!(stderr, "wc: stdin: {e}")?;
                Ok(Vec::new())
            }
        };
    }

    let mut lines = Vec::new();
    let mut total = 0u64;
    for file in &plan.files {
        match count_file_wc(file, plan.mode) {
            Ok(count) => {
                total = total.saturating_add(count);
                lines.push(format!("{count:8} {file}\n").into_bytes());
            }
            Err(e) => {
                writeln!(stderr, "wc: {file}: {e}")?;
            }
        }
    }
    if plan.files.len() > 1 {
        lines.push(format!("{total:8} total\n").into_bytes());
    }
    Ok(lines)
}

fn run_pipe_wc_producer(
    plan: &PipeWcProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = collect_wc_output_lines(&plan.wc, stderr)?;
    let grep_exit_for_empty = plan.pattern.is_some();
    if let Some(pattern) = &plan.pattern {
        lines = filter_byte_lines_by_literal(lines, pattern);
    }
    run_head_line_producer(lines, plan.mode, stdout, stderr, grep_exit_for_empty, true)
}

fn collect_du_sk_blocks(
    path: &Path,
    blocks: &mut u64,
    saw_countable: &mut bool,
    stderr: &mut dyn Write,
) -> Result<()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "du: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    *saw_countable = true;
    *blocks = blocks.saturating_add(meta.blocks());
    if !meta.file_type().is_dir() {
        return Ok(());
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            writeln!(stderr, "du: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    for entry in entries {
        match entry {
            Ok(entry) => collect_du_sk_blocks(&entry.path(), blocks, saw_countable, stderr)?,
            Err(e) => {
                writeln!(stderr, "du: {}: {e}", path.display())?;
            }
        }
    }
    Ok(())
}

fn collect_du_sk_output_lines(plan: &DuSkPlan, stderr: &mut dyn Write) -> Result<Vec<Vec<u8>>> {
    let mut blocks = 0u64;
    let mut saw_countable = false;
    collect_du_sk_blocks(
        Path::new(&plan.path),
        &mut blocks,
        &mut saw_countable,
        stderr,
    )?;
    if saw_countable {
        Ok(vec![
            format!("{}\t{}\n", (blocks + 1) / 2, plan.path).into_bytes()
        ])
    } else {
        Ok(Vec::new())
    }
}

fn run_pipe_du_producer(
    plan: &PipeDuProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = collect_du_sk_output_lines(&plan.du, stderr)?;
    let grep_exit_for_empty = plan.pattern.is_some();
    if let Some(pattern) = &plan.pattern {
        lines = filter_byte_lines_by_literal(lines, pattern);
    }
    run_head_line_producer(lines, plan.mode, stdout, stderr, grep_exit_for_empty, true)
}

fn ensure_byte_lines_end_with_newline(lines: &mut [Vec<u8>]) {
    for line in lines {
        if !line.ends_with(b"\n") {
            line.push(b'\n');
        }
    }
}

fn count_words_in_byte_chunks<'a>(chunks: impl IntoIterator<Item = &'a [u8]>) -> u64 {
    let mut words = 0u64;
    let mut in_word = false;
    for chunk in chunks {
        for byte in chunk {
            if byte.is_ascii_whitespace() {
                in_word = false;
            } else if !in_word {
                words = words.saturating_add(1);
                in_word = true;
            }
        }
    }
    words
}

fn count_wc_byte_lines(lines: &[Vec<u8>], mode: WcCountMode, wc_counts_newlines: bool) -> u64 {
    match mode {
        WcCountMode::Lines => {
            if wc_counts_newlines {
                lines.iter().filter(|line| line.ends_with(b"\n")).count() as u64
            } else {
                lines.len() as u64
            }
        }
        WcCountMode::Bytes => lines
            .iter()
            .fold(0u64, |acc, line| acc.saturating_add(line.len() as u64)),
        WcCountMode::Words => count_words_in_byte_chunks(lines.iter().map(Vec::as_slice)),
    }
}

fn count_wc_unique_byte_lines(lines: &[Vec<u8>], mode: WcCountMode) -> u64 {
    match mode {
        WcCountMode::Lines => unique_byte_line_count(lines) as u64,
        WcCountMode::Bytes => {
            let mut bytes = 0u64;
            for (idx, line) in lines.iter().enumerate() {
                let duplicate = idx > 0
                    && byte_line_without_trailing_newline(&lines[idx - 1])
                        == byte_line_without_trailing_newline(line);
                if !duplicate {
                    bytes = bytes.saturating_add(line.len() as u64);
                }
            }
            bytes
        }
        WcCountMode::Words => {
            let chunks = lines.iter().enumerate().filter_map(|(idx, line)| {
                let duplicate = idx > 0
                    && byte_line_without_trailing_newline(&lines[idx - 1])
                        == byte_line_without_trailing_newline(line);
                (!duplicate).then_some(line.as_slice())
            });
            count_words_in_byte_chunks(chunks)
        }
    }
}

fn run_mkdir(plan: &MkdirPlan, stderr: &mut dyn Write) -> Result<i32> {
    let mut exit = 0;
    for path in &plan.paths {
        let result = if plan.parents {
            fs::create_dir_all(path)
        } else {
            fs::create_dir(path)
        };
        if let Err(e) = result {
            writeln!(stderr, "mkdir: {path}: {e}")?;
            exit = 1;
        }
    }
    Ok(exit)
}

fn run_touch(plan: &TouchPlan, stderr: &mut dyn Write) -> Result<i32> {
    let mut exit = 0;
    for path in &plan.paths {
        match touch_now(Path::new(path)) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                match OpenOptions::new().create(true).write(true).open(path) {
                    Ok(_) => {
                        if let Err(e) = touch_now(Path::new(path)) {
                            writeln!(stderr, "touch: {path}: {e}")?;
                            exit = 1;
                        }
                    }
                    Err(e) => {
                        writeln!(stderr, "touch: {path}: {e}")?;
                        exit = 1;
                    }
                }
            }
            Err(e) => {
                writeln!(stderr, "touch: {path}: {e}")?;
                exit = 1;
            }
        }
    }
    Ok(exit)
}

fn run_awk_needle_count(
    plan: &AwkNeedleCountPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    if plan.stdin {
        let stdin = io::stdin();
        run_awk_needle_count_reader(stdin.lock(), stdout)?;
        return Ok(0);
    }
    match fs::File::open(&plan.file) {
        Ok(file) => {
            run_awk_needle_count_reader(file, stdout)?;
            Ok(0)
        }
        Err(e) => {
            writeln!(stderr, "awk: {}: {e}", plan.file)?;
            Ok(2)
        }
    }
}

fn run_awk_needle_count_reader<R: Read>(reader: R, stdout: &mut dyn Write) -> Result<()> {
    let reader = BufReader::new(reader);
    let mut count = 0usize;
    for line in reader.lines() {
        if line?.contains("NEEDLE") {
            count += 1;
        }
    }
    if count > 0 {
        writeln!(stdout, "{count}")?;
    } else {
        writeln!(stdout)?;
    }
    Ok(())
}

fn run_xargs_echo(
    plan: &XargsEchoPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    match plan.mode {
        XargsEchoMode::OneLine => {
            let mut first = true;
            for token in input.split_whitespace() {
                if !first {
                    write!(stdout, " ")?;
                }
                write!(stdout, "{token}")?;
                first = false;
            }
            if !first {
                writeln!(stdout)?;
            }
        }
        XargsEchoMode::Batch { size } => {
            write_xargs_echo_batched_tokens(input.split_whitespace(), size, stdout)?;
        }
    }
    Ok(0)
}

fn run_xargs_wc_lines(
    _plan: &XargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let paths = input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn collect_xargs_echo_stdin_lines(mode: XargsEchoMode) -> Result<Vec<Vec<u8>>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(xargs_echo_lines_from_bytes(input.as_bytes(), mode))
}

fn xargs_echo_lines_from_bytes(input: &[u8], mode: XargsEchoMode) -> Vec<Vec<u8>> {
    let tokens = xargs_tokens_from_bytes(input);
    match mode {
        XargsEchoMode::OneLine => {
            if tokens.is_empty() {
                Vec::new()
            } else {
                let mut line = tokens.join(" ").into_bytes();
                line.push(b'\n');
                vec![line]
            }
        }
        XargsEchoMode::Batch { size } => xargs_echo_batched_token_lines(tokens, size),
    }
}

fn xargs_echo_batched_token_lines(tokens: Vec<String>, size: usize) -> Vec<Vec<u8>> {
    if size == 0 {
        return Vec::new();
    }
    tokens
        .chunks(size)
        .map(|chunk| {
            let mut line = chunk.join(" ").into_bytes();
            line.push(b'\n');
            line
        })
        .collect()
}

fn count_xargs_echo_stdin_wc_lines(mode: XargsEchoMode) -> Result<u64> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buf = [0u8; 8192];
    let mut in_token = false;
    let mut saw_token = false;
    let mut token_count = 0u64;
    loop {
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        for byte in &buf[..read] {
            if byte.is_ascii_whitespace() {
                in_token = false;
            } else if !in_token {
                saw_token = true;
                in_token = true;
                token_count = token_count.saturating_add(1);
            }
        }
    }
    Ok(match mode {
        XargsEchoMode::OneLine => {
            if saw_token {
                1
            } else {
                0
            }
        }
        XargsEchoMode::Batch { size } => {
            let size = u64::try_from(size).unwrap_or(u64::MAX);
            if token_count == 0 {
                0
            } else {
                1 + ((token_count - 1) / size)
            }
        }
    })
}

fn run_pipe_xargs_echo_producer(
    plan: &PipeXargsEchoProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    if plan.grep.is_none()
        && matches!(
            plan.mode,
            GrepFilePipeMode::WcLines
                | GrepFilePipeMode::Wc {
                    mode: WcCountMode::Lines
                }
        )
    {
        let count = count_xargs_echo_stdin_wc_lines(plan.source_mode)?;
        writeln!(stdout, "{count:8}")?;
        return Ok(0);
    }
    let mut lines = collect_xargs_echo_stdin_lines(plan.source_mode)?;
    let grep_exit_for_empty = if let Some(pattern) = &plan.grep {
        let needle = pattern.as_bytes();
        lines.retain(|line| byte_line_contains(line, needle));
        true
    } else {
        false
    };
    run_head_line_producer(lines, plan.mode, stdout, stderr, grep_exit_for_empty, true)
}

fn write_xargs_wc_paths(
    paths: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let (lines, exit) = collect_xargs_wc_path_lines(paths, stderr)?;
    for line in lines {
        stdout.write_all(&line)?;
    }
    Ok(exit)
}

fn write_xargs_wc_output_paths(
    paths: &[String],
    mode: XargsWcOutputMode,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let (lines, _xargs_exit) = collect_xargs_wc_path_lines(paths, stderr)?;
    write_xargs_wc_output_lines(lines, mode, stdout)?;
    Ok(0)
}

fn collect_xargs_wc_path_lines(
    paths: &[String],
    stderr: &mut dyn Write,
) -> Result<(Vec<Vec<u8>>, i32)> {
    let mut output = Vec::new();
    let mut total = 0u64;
    let mut exit = 0;
    for file in paths {
        match count_newlines(file) {
            Ok(lines) => {
                total = total.saturating_add(lines);
                output.push(format!("{lines:8} {file}\n").into_bytes());
            }
            Err(e) => {
                writeln!(stderr, "wc: {file}: {e}")?;
                exit = 1;
            }
        }
    }
    if paths.len() > 1 {
        output.push(format!("{total:8} total\n").into_bytes());
    }
    Ok((output, exit))
}

fn write_xargs_wc_output_lines(
    mut lines: Vec<Vec<u8>>,
    mode: XargsWcOutputMode,
    stdout: &mut dyn Write,
) -> Result<()> {
    match mode {
        XargsWcOutputMode::WcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        XargsWcOutputMode::Head { limit } => {
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        XargsWcOutputMode::Tail { limit } => {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        XargsWcOutputMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        XargsWcOutputMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        XargsWcOutputMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        XargsWcOutputMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        XargsWcOutputMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        XargsWcOutputMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
    }
    Ok(())
}

fn xargs_tokens_from_bytes(input: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(input)
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn run_path_lookup(plan: &PathLookupPlan, stdout: &mut dyn Write) -> Result<i32> {
    let (lines, found, missing) = path_lookup_output(plan);
    for line in lines {
        writeln!(stdout, "{line}")?;
    }
    Ok(match plan.mode {
        PathLookupMode::Which | PathLookupMode::WhichAll => {
            if missing {
                1
            } else {
                0
            }
        }
        PathLookupMode::CommandV => {
            if found {
                0
            } else {
                1
            }
        }
    })
}

fn run_environment(plan: &EnvironmentPlan, stdout: &mut dyn Write) -> Result<i32> {
    let (lines, found) = environment_output(plan);
    write_byte_lines(stdout, &lines)?;
    Ok(match plan.mode {
        EnvironmentMode::Env => 0,
        EnvironmentMode::Printenv => {
            if plan.name.is_some() && !found {
                1
            } else {
                0
            }
        }
    })
}

fn run_pipe_path_lookup_wc_lines(
    plan: &PipePathLookupWcLinesPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let (lines, _, _) = path_lookup_output(&plan.lookup);
    writeln!(stdout, "{:8}", lines.len())?;
    Ok(0)
}

fn run_pipe_path_lookup_head(plan: &PipePathLookupHeadPlan, stdout: &mut dyn Write) -> Result<i32> {
    let (lines, _, _) = path_lookup_output(&plan.lookup);
    for line in lines.into_iter().take(plan.limit) {
        writeln!(stdout, "{line}")?;
    }
    Ok(0)
}

fn run_pipe_path_lookup_tail(plan: &PipePathLookupTailPlan, stdout: &mut dyn Write) -> Result<i32> {
    let (lines, _, _) = path_lookup_output(&plan.lookup);
    let start = lines.len().saturating_sub(plan.limit);
    for line in lines.into_iter().skip(start) {
        writeln!(stdout, "{line}")?;
    }
    Ok(0)
}

fn run_pipe_path_lookup_grep_producer(
    plan: &PipePathLookupGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let (lines, _, _) = path_lookup_output(&plan.lookup);
    let lines = lines
        .into_iter()
        .map(|line| {
            let mut bytes = line.into_bytes();
            bytes.push(b'\n');
            bytes
        })
        .collect::<Vec<_>>();
    let mut filtered = filter_byte_lines_by_literal(lines, &plan.pattern);
    ensure_byte_lines_end_with_newline(&mut filtered);
    run_head_line_producer(filtered, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_path_lookup_producer(
    plan: &PipePathLookupProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let (lines, _, _) = path_lookup_output(&plan.lookup);
    let lines = lines
        .into_iter()
        .map(|line| {
            let mut bytes = line.into_bytes();
            bytes.push(b'\n');
            bytes
        })
        .collect::<Vec<_>>();
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_environment_wc_lines(
    plan: &PipeEnvironmentWcLinesPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let (lines, _) = environment_output(&plan.env);
    writeln!(stdout, "{:8}", lines.len())?;
    Ok(0)
}

fn run_pipe_environment_head(
    plan: &PipeEnvironmentHeadPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let (lines, _) = environment_output(&plan.env);
    for line in lines.into_iter().take(plan.limit) {
        write_byte_line(stdout, &line)?;
    }
    Ok(0)
}

fn run_pipe_environment_tail(
    plan: &PipeEnvironmentTailPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let (lines, _) = environment_output(&plan.env);
    let start = lines.len().saturating_sub(plan.limit);
    for line in lines.into_iter().skip(start) {
        write_byte_line(stdout, &line)?;
    }
    Ok(0)
}

fn run_pipe_environment_grep(
    plan: &PipeEnvironmentGrepPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let pattern = plan.pattern.as_bytes();
    let (lines, _) = environment_output(&plan.env);
    let mut matched = false;
    for line in lines {
        if byte_line_contains(&line, pattern) {
            matched = true;
            write_byte_line(stdout, &line)?;
        }
    }
    Ok(if matched { 0 } else { 1 })
}

fn run_pipe_environment_grep_producer(
    plan: &PipeEnvironmentGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let pattern = plan.pattern.as_bytes();
    let (lines, _) = environment_output(&plan.env);
    let mut matched = lines
        .into_iter()
        .filter(|line| byte_line_contains(line, pattern))
        .collect::<Vec<_>>();
    ensure_byte_lines_end_with_newline(&mut matched);
    run_head_line_producer(matched, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_environment_sort(
    plan: &PipeEnvironmentSortPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    let (mut lines, _) = environment_output(&plan.env);
    lines.sort_unstable();
    write_byte_lines(stdout, &lines)?;
    Ok(0)
}

fn run_pipe_hostname_wc_lines(
    _plan: &PipeHostnameWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    if hostname_value().is_some() {
        writeln!(stdout, "{:8}", 1)?;
        Ok(0)
    } else {
        writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
        Ok(1)
    }
}

fn run_pipe_hostname_head(
    plan: &PipeHostnameHeadPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let Some(hostname) = hostname_value() else {
        writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
        return Ok(1);
    };
    if plan.limit > 0 {
        writeln!(stdout, "{hostname}")?;
    }
    Ok(0)
}

fn run_pipe_hostname_tail(
    plan: &PipeHostnameTailPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let Some(hostname) = hostname_value() else {
        writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
        return Ok(1);
    };
    if plan.limit > 0 {
        writeln!(stdout, "{hostname}")?;
    }
    Ok(0)
}

fn run_pipe_hostname_grep(
    plan: &PipeHostnameGrepPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let Some(hostname) = hostname_value() else {
        writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
        return Ok(1);
    };
    if hostname.contains(&plan.pattern) {
        writeln!(stdout, "{hostname}")?;
        Ok(0)
    } else {
        Ok(1)
    }
}

fn run_pipe_hostname_grep_producer(
    plan: &PipeHostnameGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let Some(hostname) = hostname_value() else {
        writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
        return Ok(1);
    };
    let mut line = hostname.into_bytes();
    line.push(b'\n');
    let mut lines = filter_byte_lines_by_literal(vec![line], &plan.pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_hostname_sort(
    _plan: &PipeHostnameSortPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let Some(hostname) = hostname_value() else {
        writeln!(stderr, "hostname: {}", io::Error::last_os_error())?;
        return Ok(1);
    };
    writeln!(stdout, "{hostname}")?;
    Ok(0)
}

fn path_lookup_output(plan: &PathLookupPlan) -> (Vec<String>, bool, bool) {
    let mut lines = Vec::new();
    let mut found_any = false;
    let mut missing_any = false;
    for name in &plan.names {
        match resolve_shell_or_path_words(name, plan.mode) {
            Some(mut resolved) => {
                lines.append(&mut resolved);
                found_any = true;
            }
            None => {
                missing_any = true;
            }
        }
    }
    (lines, found_any, missing_any)
}

fn environment_output(plan: &EnvironmentPlan) -> (Vec<Vec<u8>>, bool) {
    match (&plan.mode, &plan.name) {
        (EnvironmentMode::Env, None) | (EnvironmentMode::Printenv, None) => {
            (environment_all_lines(), true)
        }
        (EnvironmentMode::Printenv, Some(name)) => {
            if let Some(value) = env::var_os(name) {
                (vec![value.as_os_str().as_bytes().to_vec()], true)
            } else {
                (Vec::new(), false)
            }
        }
        (EnvironmentMode::Env, Some(_)) => (Vec::new(), false),
    }
}

fn environment_all_lines() -> Vec<Vec<u8>> {
    env::vars_os()
        .map(|(key, value)| {
            let mut line = Vec::new();
            line.extend_from_slice(key.as_os_str().as_bytes());
            line.push(b'=');
            line.extend_from_slice(value.as_os_str().as_bytes());
            line
        })
        .collect()
}

fn write_byte_lines(stdout: &mut dyn Write, lines: &[Vec<u8>]) -> Result<()> {
    for line in lines {
        write_byte_line(stdout, line)?;
    }
    Ok(())
}

fn write_byte_line(stdout: &mut dyn Write, line: &[u8]) -> Result<()> {
    stdout.write_all(line)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

fn byte_line_contains(line: &[u8], pattern: &[u8]) -> bool {
    !pattern.is_empty() && line.windows(pattern.len()).any(|window| window == pattern)
}

fn resolve_shell_or_path_words(name: &str, mode: PathLookupMode) -> Option<Vec<String>> {
    if name.contains('/') {
        return path_matches_lookup(name, PathLookupPathMode::Executable)
            .then(|| vec![name.to_string()]);
    }
    match mode {
        PathLookupMode::Which => {
            find_path_lookup_candidate(name, PathLookupPathMode::Executable).map(|line| vec![line])
        }
        PathLookupMode::WhichAll => {
            let lines = find_path_lookup_candidates(name, PathLookupPathMode::Executable);
            (!lines.is_empty()).then_some(lines)
        }
        PathLookupMode::CommandV => shell_word_kind(name)
            .map(|_| vec![name.to_string()])
            .or_else(|| {
                find_path_lookup_candidate(name, PathLookupPathMode::AnyFile).map(|line| vec![line])
            }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellWordKind {
    Builtin,
    Reserved,
}

fn shell_word_kind(name: &str) -> Option<ShellWordKind> {
    if matches!(
        name,
        "alias"
            | "bg"
            | "bind"
            | "break"
            | "builtin"
            | "caller"
            | "cd"
            | "command"
            | "compgen"
            | "complete"
            | "compopt"
            | "continue"
            | "declare"
            | "dirs"
            | "disown"
            | "echo"
            | "enable"
            | "eval"
            | "exec"
            | "exit"
            | "export"
            | "false"
            | "fc"
            | "fg"
            | "getopts"
            | "hash"
            | "help"
            | "history"
            | "jobs"
            | "kill"
            | "let"
            | "local"
            | "logout"
            | "mapfile"
            | "popd"
            | "printf"
            | "pushd"
            | "pwd"
            | "read"
            | "readarray"
            | "readonly"
            | "return"
            | "set"
            | "shift"
            | "shopt"
            | "source"
            | "suspend"
            | "test"
            | "times"
            | "trap"
            | "true"
            | "type"
            | "typeset"
            | "ulimit"
            | "umask"
            | "unalias"
            | "unset"
            | "wait"
            | "["
    ) {
        return Some(ShellWordKind::Builtin);
    }
    if matches!(
        name,
        "!" | "[["
            | "]]"
            | "{"
            | "}"
            | "case"
            | "do"
            | "done"
            | "elif"
            | "else"
            | "esac"
            | "fi"
            | "for"
            | "function"
            | "if"
            | "in"
            | "select"
            | "then"
            | "time"
            | "until"
            | "while"
    ) {
        return Some(ShellWordKind::Reserved);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathLookupPathMode {
    Executable,
    AnyFile,
}

fn find_path_lookup_candidate(name: &str, mode: PathLookupPathMode) -> Option<String> {
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths).find_map(|dir| {
        let path = dir.join(name);
        path_matches_lookup(&path, mode).then(|| path.display().to_string())
    })
}

fn find_path_lookup_candidates(name: &str, mode: PathLookupPathMode) -> Vec<String> {
    let Some(paths) = env::var_os("PATH") else {
        return Vec::new();
    };
    env::split_paths(&paths)
        .filter_map(|dir| {
            let path = dir.join(name);
            path_matches_lookup(&path, mode).then(|| path.display().to_string())
        })
        .collect()
}

fn path_matches_lookup(path: impl AsRef<Path>, mode: PathLookupPathMode) -> bool {
    let Ok(meta) = fs::metadata(path.as_ref()) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    match mode {
        PathLookupPathMode::Executable => meta.permissions().mode() & 0o111 != 0,
        PathLookupPathMode::AnyFile => true,
    }
}

fn run_pipe_cat_wc_lines(
    plan: &PipeCatWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    match count_file_wc(&plan.file, plan.mode) {
        Ok(count) => {
            writeln!(stdout, "{count:8}")?;
            Ok(0)
        }
        Err(e) => {
            writeln!(stderr, "cat: {}: {e}", plan.file)?;
            writeln!(stdout, "{:8}", 0)?;
            Ok(0)
        }
    }
}

fn run_pipe_cat_head(
    plan: &PipeCatHeadPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let file = match fs::File::open(&plan.file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "cat: {}: {e}", plan.file)?;
            return Ok(0);
        }
    };
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut remaining = plan.limit;
    while remaining > 0 {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "cat: {}: {e}", plan.file)?;
                return Ok(0);
            }
        };
        if read == 0 {
            break;
        }
        stdout.write_all(&line)?;
        remaining -= 1;
    }
    Ok(0)
}

fn run_pipe_cat_tail(
    plan: &PipeCatTailPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let file = match fs::File::open(&plan.file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "cat: {}: {e}", plan.file)?;
            return Ok(0);
        }
    };
    if plan.limit == 0 {
        return Ok(0);
    }
    let limit = usize::try_from(plan.limit).unwrap_or(usize::MAX);
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut tail = VecDeque::new();
    loop {
        line.clear();
        let read = match reader.read_until(b'\n', &mut line) {
            Ok(read) => read,
            Err(e) => {
                writeln!(stderr, "cat: {}: {e}", plan.file)?;
                return Ok(0);
            }
        };
        if read == 0 {
            break;
        }
        tail.push_back(std::mem::take(&mut line));
        if tail.len() > limit {
            tail.pop_front();
        }
    }
    for line in tail {
        stdout.write_all(&line)?;
    }
    Ok(0)
}

fn run_pipe_cat_grep(
    plan: &PipeCatGrepPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let file = match fs::File::open(&plan.file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "cat: {}: {e}", plan.file)?;
            return Ok(1);
        }
    };
    let needle = plan.pattern.as_bytes();
    let mut matched = false;
    for line in BufReader::new(file).split(b'\n') {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                writeln!(stderr, "cat: {}: {e}", plan.file)?;
                return Ok(if matched { 0 } else { 1 });
            }
        };
        if line.windows(needle.len()).any(|window| window == needle) {
            stdout.write_all(&line)?;
            writeln!(stdout)?;
            matched = true;
        }
    }
    Ok(if matched { 0 } else { 1 })
}

fn run_pipe_cat_grep_pipeline(
    plan: &PipeCatGrepPipelinePlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = collect_cat_grep_lines(&plan.file, &plan.pattern, stderr)?;
    match plan.mode {
        GrepFilePipeMode::Lines => {
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::WcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::Wc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::Head { limit } => {
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Tail { limit } => {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        GrepFilePipeMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        GrepFilePipeMode::SortUniqWc { mode } => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", count_wc_unique_byte_lines(&lines, mode))?;
        }
        GrepFilePipeMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::SortWc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortXargsEcho => {
            lines.sort_unstable();
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::SortXargsEchoBatches { size } => {
            lines.sort_unstable();
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::SortXargsWcLines => {
            lines.sort_unstable();
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::SortXargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, true, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsEcho => {
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::XargsEchoBatches { size } => {
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::XargsWcLines => {
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::XargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, false, mode, stdout, stderr);
        }
    }
    Ok(0)
}

fn run_pipe_cat_grep_sort_uniq_producer(
    plan: &PipeCatGrepSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cat_grep_lines(&plan.file, &plan.pattern, stderr)?;
    let lines = sort_unique_byte_lines(lines);
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_cat_grep_xargs_echo(
    plan: &PipeCatGrepXargsEchoPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cat_grep_lines(&plan.file, &plan.pattern, stderr)?;
    write_xargs_echo_line_tokens(&lines, stdout)?;
    Ok(0)
}

fn run_pipe_cat_grep_xargs_wc_lines(
    plan: &PipeCatGrepXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_cat_grep_lines(&plan.file, &plan.pattern, stderr)?;
    let mut paths = Vec::new();
    for line in &lines {
        paths.extend(xargs_tokens_from_bytes(line));
    }
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn run_pipe_cat_grep_sort_xargs_echo(
    plan: &PipeCatGrepSortXargsEchoPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = collect_cat_grep_lines(&plan.file, &plan.pattern, stderr)?;
    lines.sort_unstable();
    write_xargs_echo_line_tokens(&lines, stdout)?;
    Ok(0)
}

fn run_pipe_cat_grep_sort_xargs_wc_lines(
    plan: &PipeCatGrepSortXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = collect_cat_grep_lines(&plan.file, &plan.pattern, stderr)?;
    lines.sort_unstable();
    let mut paths = Vec::new();
    for line in &lines {
        paths.extend(xargs_tokens_from_bytes(line));
    }
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn collect_cat_grep_lines(
    file: &str,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let file_handle = match fs::File::open(file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "cat: {file}: {e}")?;
            return Ok(Vec::new());
        }
    };
    let needle = pattern.as_bytes();
    let mut lines = Vec::new();
    for line in BufReader::new(file_handle).split(b'\n') {
        let mut line = match line {
            Ok(line) => line,
            Err(e) => {
                writeln!(stderr, "cat: {file}: {e}")?;
                return Ok(lines);
            }
        };
        if line.windows(needle.len()).any(|window| window == needle) {
            line.push(b'\n');
            lines.push(line);
        }
    }
    Ok(lines)
}

fn run_pipe_grep_head(
    plan: &PipeGrepHeadPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut remaining = plan.limit;
    visit_grep_head(
        PathBuf::from(&plan.root),
        &plan.pattern,
        stdout,
        stderr,
        &mut remaining,
    )?;
    Ok(0)
}

fn visit_grep_head(
    path: PathBuf,
    pattern: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    remaining: &mut u64,
) -> Result<()> {
    if *remaining == 0 {
        return Ok(());
    }
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "grep: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for line in BufReader::new(file).split(b'\n') {
            let mut line = line?;
            let had_newline = true;
            if line
                .windows(pattern.len())
                .any(|window| window == pattern.as_bytes())
            {
                write!(stdout, "{}:", path.display())?;
                stdout.write_all(&line)?;
                if had_newline {
                    writeln!(stdout)?;
                }
                *remaining = remaining.saturating_sub(1);
                if *remaining == 0 {
                    break;
                }
            }
            line.clear();
        }
    } else if meta.file_type().is_dir() {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            visit_grep_head(entry.path(), pattern, stdout, stderr, remaining)?;
            if *remaining == 0 {
                break;
            }
        }
    }
    Ok(())
}

fn run_pipe_grep_tail(
    plan: &PipeGrepTailPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let limit = usize::try_from(plan.limit).unwrap_or(usize::MAX);
    let mut tail = VecDeque::new();
    visit_grep_tail(
        PathBuf::from(&plan.root),
        &plan.pattern,
        stderr,
        limit,
        &mut tail,
    )?;
    for line in tail {
        stdout.write_all(&line)?;
    }
    Ok(0)
}

fn visit_grep_tail(
    path: PathBuf,
    pattern: &str,
    stderr: &mut dyn Write,
    limit: usize,
    tail: &mut VecDeque<Vec<u8>>,
) -> Result<()> {
    if limit == 0 {
        return Ok(());
    }
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "grep: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        let needle = pattern.as_bytes();
        for line in BufReader::new(file).split(b'\n') {
            let line = line?;
            if line.windows(needle.len()).any(|window| window == needle) {
                let mut out = Vec::new();
                write!(&mut out, "{}:", path.display())?;
                out.extend_from_slice(&line);
                out.push(b'\n');
                tail.push_back(out);
                if tail.len() > limit {
                    tail.pop_front();
                }
            }
        }
    } else if meta.file_type().is_dir() {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            visit_grep_tail(entry.path(), pattern, stderr, limit, tail)?;
        }
    }
    Ok(())
}

fn run_pipe_grep_sort(
    plan: &PipeGrepSortPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = Vec::new();
    collect_grep_output_lines(PathBuf::from(&plan.root), &plan.pattern, stderr, &mut lines)?;
    lines.sort_unstable();
    for line in lines {
        stdout.write_all(&line)?;
    }
    Ok(0)
}

fn run_pipe_grep_sort_uniq(
    plan: &PipeGrepSortUniqPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = Vec::new();
    collect_grep_output_lines(PathBuf::from(&plan.root), &plan.pattern, stderr, &mut lines)?;
    lines.sort_unstable();
    write_unique_byte_lines(&lines, stdout)?;
    Ok(0)
}

fn run_pipe_grep_sort_uniq_producer(
    plan: &PipeGrepSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = Vec::new();
    collect_grep_output_lines(PathBuf::from(&plan.root), &plan.pattern, stderr, &mut lines)?;
    let lines = sort_unique_byte_lines(lines);
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_grep_sort_head(
    plan: &PipeGrepSortHeadPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = Vec::new();
    collect_grep_output_lines(PathBuf::from(&plan.root), &plan.pattern, stderr, &mut lines)?;
    lines.sort_unstable();
    for line in lines
        .into_iter()
        .take(usize::try_from(plan.limit).unwrap_or(usize::MAX))
    {
        stdout.write_all(&line)?;
    }
    Ok(0)
}

fn run_pipe_grep_sort_tail(
    plan: &PipeGrepSortTailPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = Vec::new();
    collect_grep_output_lines(PathBuf::from(&plan.root), &plan.pattern, stderr, &mut lines)?;
    lines.sort_unstable();
    let limit = usize::try_from(plan.limit).unwrap_or(usize::MAX);
    let start = lines.len().saturating_sub(limit);
    for line in lines.into_iter().skip(start) {
        stdout.write_all(&line)?;
    }
    Ok(0)
}

fn run_pipe_grep_sort_uniq_wc_lines(
    plan: &PipeGrepSortUniqWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = Vec::new();
    collect_grep_output_lines(PathBuf::from(&plan.root), &plan.pattern, stderr, &mut lines)?;
    lines.sort_unstable();
    writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
    Ok(0)
}

fn run_pipe_grep_sort_wc_lines(
    plan: &PipeGrepSortWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut matches = 0u64;
    visit_grep_count(
        PathBuf::from(&plan.root),
        &plan.pattern,
        stderr,
        &mut matches,
    )?;
    writeln!(stdout, "{matches:8}")?;
    Ok(0)
}

fn write_unique_byte_lines(lines: &[Vec<u8>], stdout: &mut dyn Write) -> Result<()> {
    for (idx, line) in lines.iter().enumerate() {
        let duplicate = idx > 0
            && byte_line_without_trailing_newline(&lines[idx - 1])
                == byte_line_without_trailing_newline(line);
        if !duplicate {
            stdout.write_all(line)?;
        }
    }
    Ok(())
}

fn unique_byte_line_count(lines: &[Vec<u8>]) -> usize {
    let mut count = 0usize;
    for (idx, line) in lines.iter().enumerate() {
        let duplicate = idx > 0
            && byte_line_without_trailing_newline(&lines[idx - 1])
                == byte_line_without_trailing_newline(line);
        if !duplicate {
            count += 1;
        }
    }
    count
}

fn byte_line_without_trailing_newline(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\n").unwrap_or(line)
}

fn collect_grep_file_plain_lines(
    file: &str,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<(Vec<Vec<u8>>, bool)> {
    if file.is_empty() {
        let stdin = io::stdin();
        return collect_grep_plain_lines_reader(stdin.lock(), pattern, stderr, "stdin");
    }

    let file_handle = match fs::File::open(file) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            writeln!(stderr, "grep: {file}: {e}")?;
            return Ok((Vec::new(), true));
        }
    };
    collect_grep_plain_lines_reader(BufReader::new(file_handle), pattern, stderr, file)
}

fn collect_grep_plain_lines_reader<R: BufRead>(
    reader: R,
    pattern: &str,
    stderr: &mut dyn Write,
    source_label: &str,
) -> Result<(Vec<Vec<u8>>, bool)> {
    let needle = pattern.as_bytes();
    let mut lines = Vec::new();
    let mut had_error = false;
    for line in reader.split(b'\n') {
        match line {
            Ok(line) => {
                if line.windows(needle.len()).any(|window| window == needle) {
                    let mut out = line;
                    out.push(b'\n');
                    lines.push(out);
                }
            }
            Err(e) => {
                writeln!(stderr, "grep: {source_label}: {e}")?;
                had_error = true;
                break;
            }
        }
    }
    Ok((lines, had_error))
}

fn run_grep_file(
    plan: &GrepFilePlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let (lines, had_error) = collect_grep_file_plain_lines(&plan.file, &plan.pattern, stderr)?;
    for line in &lines {
        stdout.write_all(line)?;
    }
    if had_error {
        Ok(2)
    } else if lines.is_empty() {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn run_pipe_grep_file(
    plan: &PipeGrepFilePlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let (mut lines, _) = collect_grep_file_plain_lines(&plan.file, &plan.pattern, stderr)?;
    match plan.mode {
        GrepFilePipeMode::Lines => {
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::WcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::Wc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::Head { limit } => {
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Tail { limit } => {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        GrepFilePipeMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        GrepFilePipeMode::SortUniqWc { mode } => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", count_wc_unique_byte_lines(&lines, mode))?;
        }
        GrepFilePipeMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::SortWc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortXargsEcho => {
            lines.sort_unstable();
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::SortXargsEchoBatches { size } => {
            lines.sort_unstable();
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::SortXargsWcLines => {
            lines.sort_unstable();
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::SortXargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, true, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsEcho => {
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::XargsEchoBatches { size } => {
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::XargsWcLines => {
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::XargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, false, mode, stdout, stderr);
        }
    }
    Ok(0)
}

fn run_pipe_grep_file_sort_uniq_producer(
    plan: &PipeGrepFileSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let (lines, _) = collect_grep_file_plain_lines(&plan.file, &plan.pattern, stderr)?;
    let lines = sort_unique_byte_lines(lines);
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_grep_file_cut_lines(
    file: &str,
    pattern: &str,
    cut: &CutFilterPlan,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let (lines, _) = collect_grep_file_plain_lines(file, pattern, stderr)?;
    Ok(lines
        .iter()
        .map(|line| cut_line_bytes(line, cut.delimiter, cut.field))
        .collect())
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn run_pipe_grep_file_cut_producer(
    plan: &PipeGrepFileCutProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_grep_file_cut_lines(&plan.file, &plan.pattern, &plan.cut, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn run_pipe_grep_file_cut_grep_producer(
    plan: &PipeGrepFileCutGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = filter_byte_lines_by_literal(
        collect_grep_file_cut_lines(&plan.file, &plan.pattern, &plan.cut, stderr)?,
        &plan.downstream_pattern,
    );
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn whitespace_field_line(line: &[u8], field: usize) -> Vec<u8> {
    let text = String::from_utf8_lossy(line);
    let token = text
        .split_whitespace()
        .nth(field.saturating_sub(1))
        .unwrap_or("");
    let mut out = token.as_bytes().to_vec();
    out.push(b'\n');
    out
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn collect_grep_file_awk_lines(
    file: &str,
    pattern: &str,
    awk_pattern: Option<&str>,
    awk_field: usize,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let (mut lines, _) = collect_grep_file_plain_lines(file, pattern, stderr)?;
    if let Some(awk_pattern) = awk_pattern {
        lines = filter_byte_lines_by_literal(lines, awk_pattern);
    }
    Ok(lines
        .iter()
        .map(|line| whitespace_field_line(line, awk_field))
        .collect())
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn run_pipe_grep_file_awk_producer(
    plan: &PipeGrepFileAwkProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = collect_grep_file_awk_lines(
        &plan.file,
        &plan.pattern,
        plan.awk_pattern.as_deref(),
        plan.awk_field,
        stderr,
    )?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn run_pipe_grep_file_awk_grep_producer(
    plan: &PipeGrepFileAwkGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = filter_byte_lines_by_literal(
        collect_grep_file_awk_lines(
            &plan.file,
            &plan.pattern,
            plan.awk_pattern.as_deref(),
            plan.awk_field,
            stderr,
        )?,
        &plan.downstream_pattern,
    );
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn write_xargs_echo_line_tokens(lines: &[Vec<u8>], stdout: &mut dyn Write) -> Result<()> {
    let mut first = true;
    for line in lines {
        let text = String::from_utf8_lossy(line);
        write_xargs_echo_path_tokens(&text, stdout, &mut first)?;
    }
    if !first {
        writeln!(stdout)?;
    }
    Ok(())
}

fn write_xargs_echo_batched_lines(
    lines: &[Vec<u8>],
    size: usize,
    stdout: &mut dyn Write,
) -> Result<()> {
    let tokens = lines
        .iter()
        .flat_map(|line| xargs_tokens_from_bytes(line))
        .collect::<Vec<_>>();
    for line in xargs_echo_batched_token_lines(tokens, size) {
        stdout.write_all(&line)?;
    }
    Ok(())
}

fn write_xargs_echo_batched_tokens<'a, I>(
    tokens: I,
    size: usize,
    stdout: &mut dyn Write,
) -> Result<()>
where
    I: IntoIterator<Item = &'a str>,
{
    if size == 0 {
        return Ok(());
    }
    let mut batch = Vec::new();
    for token in tokens {
        batch.push(token);
        if batch.len() == size {
            writeln!(stdout, "{}", batch.join(" "))?;
            batch.clear();
        }
    }
    if !batch.is_empty() {
        writeln!(stdout, "{}", batch.join(" "))?;
    }
    Ok(())
}

fn collect_grep_output_lines(
    path: PathBuf,
    pattern: &str,
    stderr: &mut dyn Write,
    lines: &mut Vec<Vec<u8>>,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "grep: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        let needle = pattern.as_bytes();
        for line in BufReader::new(file).split(b'\n') {
            let line = line?;
            if line.windows(needle.len()).any(|window| window == needle) {
                let mut out = Vec::new();
                write!(&mut out, "{}:", path.display())?;
                out.extend_from_slice(&line);
                out.push(b'\n');
                lines.push(out);
            }
        }
    } else if meta.file_type().is_dir() {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            collect_grep_output_lines(entry.path(), pattern, stderr, lines)?;
        }
    }
    Ok(())
}

fn run_pipe_grep_wc_lines(
    plan: &PipeGrepWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut matches = 0u64;
    visit_grep_count(
        PathBuf::from(&plan.root),
        &plan.pattern,
        stderr,
        &mut matches,
    )?;
    writeln!(stdout, "{matches:8}")?;
    Ok(0)
}

fn visit_grep_count(
    path: PathBuf,
    pattern: &str,
    stderr: &mut dyn Write,
    matches: &mut u64,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "grep: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        let needle = pattern.as_bytes();
        for line in BufReader::new(file).split(b'\n') {
            let line = line?;
            if line.windows(needle.len()).any(|window| window == needle) {
                *matches = matches.saturating_add(1);
            }
        }
    } else if meta.file_type().is_dir() {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "grep: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            visit_grep_count(entry.path(), pattern, stderr, matches)?;
        }
    }
    Ok(())
}

fn run_pipe_awk_xargs_echo(
    plan: &PipeAwkXargsEchoPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let tokens = awk_first_fields(&plan.file, plan.pattern.as_deref(), plan.field, stderr)?;
    write_xargs_echo_paths(&tokens, stdout)?;
    Ok(0)
}

fn run_pipe_awk_xargs_wc_lines(
    plan: &PipeAwkXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let paths = awk_first_fields(&plan.file, plan.pattern.as_deref(), plan.field, stderr)?;
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn run_pipe_awk_producer(
    plan: &PipeAwkProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = awk_first_field_lines(&plan.file, plan.pattern.as_deref(), plan.field, stderr)?;
    match plan.mode {
        GrepFilePipeMode::Lines => {
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::WcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::Wc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::Head { limit } => {
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Tail { limit } => {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        GrepFilePipeMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        GrepFilePipeMode::SortUniqWc { mode } => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", count_wc_unique_byte_lines(&lines, mode))?;
        }
        GrepFilePipeMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::SortWc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortXargsEcho => {
            lines.sort_unstable();
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::SortXargsEchoBatches { size } => {
            lines.sort_unstable();
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::SortXargsWcLines => {
            lines.sort_unstable();
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::SortXargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, true, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsEcho => {
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::XargsEchoBatches { size } => {
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::XargsWcLines => {
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::XargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, false, mode, stdout, stderr);
        }
    }
    Ok(0)
}

fn run_pipe_awk_grep_producer(
    plan: &PipeAwkGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = filter_byte_lines_by_literal(
        awk_first_field_lines(&plan.file, plan.pattern.as_deref(), plan.field, stderr)?,
        &plan.downstream_pattern,
    );
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_awk_sort_uniq_producer(
    plan: &PipeAwkSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = sort_unique_byte_lines(awk_first_field_lines(
        &plan.file,
        plan.pattern.as_deref(),
        plan.field,
        stderr,
    )?);
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn awk_first_field_lines(
    file: &str,
    pattern: Option<&str>,
    field: usize,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    if file.is_empty() {
        let stdin = io::stdin();
        return awk_first_field_lines_reader(stdin.lock(), pattern, field);
    }

    let file_handle = match fs::File::open(file) {
        Ok(file) => file,
        Err(e) => {
            writeln!(stderr, "awk: {file}: {e}")?;
            return Ok(Vec::new());
        }
    };
    awk_first_field_lines_reader(BufReader::new(file_handle), pattern, field)
}

fn awk_first_field_lines_reader<R: BufRead>(
    reader: R,
    pattern: Option<&str>,
    field: usize,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if pattern.map_or(true, |pattern| line.contains(pattern)) {
            let token = line
                .split_whitespace()
                .nth(field.saturating_sub(1))
                .unwrap_or("");
            let mut out = token.as_bytes().to_vec();
            out.push(b'\n');
            lines.push(out);
        }
    }
    Ok(lines)
}

fn awk_first_fields(
    file: &str,
    pattern: Option<&str>,
    field: usize,
    stderr: &mut dyn Write,
) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    for line in awk_first_field_lines(file, pattern, field, stderr)? {
        tokens.extend(xargs_tokens_from_bytes(&line));
    }
    Ok(tokens)
}

fn run_awk_first_field(
    plan: &AwkFirstFieldPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = if plan.stdin {
        let stdin = io::stdin();
        awk_first_field_lines_reader(stdin.lock(), plan.pattern.as_deref(), plan.field)?
    } else {
        awk_first_field_lines(&plan.file, plan.pattern.as_deref(), plan.field, stderr)?
    };
    for line in lines {
        stdout.write_all(&line)?;
    }
    Ok(0)
}

fn run_pipe_echo_wc_lines(plan: &PipeEchoWcLinesPlan, stdout: &mut dyn Write) -> Result<i32> {
    let mut generated = Vec::new();
    write_echo(&plan.echo, &mut generated)?;
    let count = match plan.mode {
        WcCountMode::Lines => u64::from(plan.newline),
        WcCountMode::Bytes => generated.len() as u64,
        WcCountMode::Words => count_words_in_byte_chunks([generated.as_slice()]),
    };
    writeln!(stdout, "{count:8}")?;
    Ok(0)
}

fn run_pipe_echo_head(plan: &PipeEchoHeadPlan, stdout: &mut dyn Write) -> Result<i32> {
    if plan.limit > 0 {
        write_echo(&plan.echo, stdout)?;
    }
    Ok(0)
}

fn run_pipe_echo_tail(plan: &PipeEchoTailPlan, stdout: &mut dyn Write) -> Result<i32> {
    if plan.limit > 0 {
        write_echo(&plan.echo, stdout)?;
    }
    Ok(0)
}

fn run_pipe_echo_tr(plan: &PipeEchoTrPlan, stdout: &mut dyn Write) -> Result<i32> {
    let mut generated = Vec::new();
    write_echo(&plan.echo, &mut generated)?;
    transform_bytes(&generated, &plan.tr.mode, stdout)?;
    Ok(0)
}

fn run_pipe_echo_xargs_echo(plan: &PipeEchoXargsEchoPlan, stdout: &mut dyn Write) -> Result<i32> {
    let mut generated = Vec::new();
    write_echo(&plan.echo, &mut generated)?;
    match plan.mode {
        XargsEchoMode::OneLine => write_xargs_echo_byte_tokens(&generated, stdout)?,
        XargsEchoMode::Batch { size } => {
            let tokens = xargs_tokens_from_bytes(&generated);
            write_xargs_echo_batched_tokens(tokens.iter().map(String::as_str), size, stdout)?;
        }
    }
    Ok(0)
}

fn run_pipe_echo_xargs_wc_lines(
    plan: &PipeEchoXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut generated = Vec::new();
    write_echo(&plan.echo, &mut generated)?;
    let paths = xargs_tokens_from_bytes(&generated);
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn run_pipe_echo_awk_producer(
    plan: &PipeEchoAwkProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut generated = Vec::new();
    write_echo(&plan.echo, &mut generated)?;
    let lines = awk_first_field_lines_reader(
        io::Cursor::new(generated),
        plan.pattern.as_deref(),
        plan.field,
    )?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_printf_wc_lines(plan: &PipePrintfWcLinesPlan, stdout: &mut dyn Write) -> Result<i32> {
    let lines = printf_args_to_lines(&plan.args);
    writeln!(
        stdout,
        "{:8}",
        count_wc_byte_lines(&lines, plan.mode, false)
    )?;
    Ok(0)
}

fn run_pipe_printf_head(plan: &PipePrintfHeadPlan, stdout: &mut dyn Write) -> Result<i32> {
    for arg in plan
        .args
        .iter()
        .take(usize::try_from(plan.limit).unwrap_or(usize::MAX))
    {
        writeln!(stdout, "{arg}")?;
    }
    Ok(0)
}

fn run_pipe_printf_tail(plan: &PipePrintfTailPlan, stdout: &mut dyn Write) -> Result<i32> {
    let limit = usize::try_from(plan.limit).unwrap_or(usize::MAX);
    let start = plan.args.len().saturating_sub(limit);
    for arg in plan.args.iter().skip(start) {
        writeln!(stdout, "{arg}")?;
    }
    Ok(0)
}

fn run_pipe_printf_grep(plan: &PipePrintfGrepPlan, stdout: &mut dyn Write) -> Result<i32> {
    let needle = plan.pattern.as_bytes();
    let mut matched = false;
    for arg in &plan.args {
        if arg
            .as_bytes()
            .windows(needle.len())
            .any(|window| window == needle)
        {
            writeln!(stdout, "{arg}")?;
            matched = true;
        }
    }
    Ok(if matched { 0 } else { 1 })
}

fn run_pipe_printf_tr(plan: &PipePrintfTrPlan, stdout: &mut dyn Write) -> Result<i32> {
    let mut generated = Vec::new();
    write_printf(plan.printf.format, &plan.printf.args, &mut generated)?;
    transform_bytes(&generated, &plan.tr.mode, stdout)?;
    Ok(0)
}

fn printf_args_to_lines(args: &[String]) -> Vec<Vec<u8>> {
    args.iter()
        .map(|arg| {
            let mut line = arg.as_bytes().to_vec();
            line.push(b'\n');
            line
        })
        .collect()
}

fn awk_first_fields_from_byte_lines(
    source: Vec<Vec<u8>>,
    pattern: Option<&str>,
    field: usize,
) -> Result<Vec<Vec<u8>>> {
    let mut lines = Vec::new();
    for line in source {
        let text = String::from_utf8(line)?;
        if pattern.map_or(true, |pattern| text.contains(pattern)) {
            let token = text
                .split_whitespace()
                .nth(field.saturating_sub(1))
                .unwrap_or("");
            let mut out = token.as_bytes().to_vec();
            out.push(b'\n');
            lines.push(out);
        }
    }
    Ok(lines)
}

fn run_pipe_printf_awk_producer(
    plan: &PipePrintfAwkProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = awk_first_fields_from_byte_lines(
        printf_args_to_lines(&plan.args),
        plan.pattern.as_deref(),
        plan.field,
    )?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_printf_producer(
    plan: &PipePrintfProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = printf_args_to_lines(&plan.args);
    match plan.mode {
        GrepFilePipeMode::Lines => {
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        GrepFilePipeMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        GrepFilePipeMode::SortUniqWc { mode } => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", count_wc_unique_byte_lines(&lines, mode))?;
        }
        GrepFilePipeMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::SortWc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortXargsEcho => {
            lines.sort_unstable();
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::SortXargsEchoBatches { size } => {
            lines.sort_unstable();
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::SortXargsWcLines => {
            lines.sort_unstable();
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::SortXargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, true, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, false, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsEchoBatches { size } => {
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::WcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::Wc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::Head { .. }
        | GrepFilePipeMode::Tail { .. }
        | GrepFilePipeMode::XargsEcho
        | GrepFilePipeMode::XargsWcLines => {}
    }
    Ok(0)
}

fn run_pipe_printf_literal_producer(
    plan: &PipePrintfLiteralProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = byte_lines_from_slice(&plan.printf.bytes);
    let grep_exit_for_empty = plan.pattern.is_some();
    if let Some(pattern) = &plan.pattern {
        lines = filter_byte_lines_by_literal(lines, pattern);
        ensure_byte_lines_end_with_newline(&mut lines);
    }
    run_head_line_producer(lines, plan.mode, stdout, stderr, grep_exit_for_empty, true)
}

fn run_pipe_printf_grep_producer(
    plan: &PipePrintfGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    run_filtered_byte_line_producer(
        filter_byte_lines_by_literal(printf_args_to_lines(&plan.args), &plan.pattern),
        plan.mode,
        stdout,
        stderr,
    )
}

fn run_pipe_printf_xargs_echo(
    plan: &PipePrintfXargsEchoPlan,
    stdout: &mut dyn Write,
) -> Result<i32> {
    match plan.mode {
        XargsEchoMode::OneLine => write_xargs_echo_paths(&plan.args, stdout)?,
        XargsEchoMode::Batch { size } => {
            write_xargs_echo_batched_lines(&printf_args_to_lines(&plan.args), size, stdout)?;
        }
    }
    Ok(0)
}

fn run_pipe_printf_xargs_wc_lines(
    plan: &PipePrintfXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    write_xargs_wc_paths(&plan.args, stdout, stderr)
}

fn run_pipe_seq_wc_lines(plan: &PipeSeqWcLinesPlan, stdout: &mut dyn Write) -> Result<i32> {
    let lines = seq_to_lines(&plan.seq);
    writeln!(
        stdout,
        "{:8}",
        count_wc_byte_lines(&lines, plan.mode, false)
    )?;
    Ok(0)
}

fn run_pipe_seq_head(plan: &PipeSeqHeadPlan, stdout: &mut dyn Write) -> Result<i32> {
    write_seq(&plan.seq, stdout, plan.limit)?;
    Ok(0)
}

fn run_pipe_seq_tail(plan: &PipeSeqTailPlan, stdout: &mut dyn Write) -> Result<i32> {
    write_seq_tail(&plan.seq, stdout, plan.limit)?;
    Ok(0)
}

fn seq_to_lines(seq: &SeqPlan) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut current = seq.first as i128;
    let step = seq.step as i128;
    let last = seq.last as i128;
    let mut remaining = seq_count(seq);
    while remaining > 0 {
        let mut line = current.to_string().into_bytes();
        line.push(b'\n');
        lines.push(line);
        current += step;
        remaining -= 1;
        if (step > 0 && current > last) || (step < 0 && current < last) {
            break;
        }
    }
    lines
}

fn filter_byte_lines_by_literal(lines: Vec<Vec<u8>>, pattern: &str) -> Vec<Vec<u8>> {
    let needle = pattern.as_bytes();
    lines
        .into_iter()
        .filter(|line| line.windows(needle.len()).any(|window| window == needle))
        .collect()
}

fn sort_unique_byte_lines(mut lines: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    ensure_byte_lines_end_with_newline(&mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn run_pipe_printf_sort_uniq_producer(
    plan: &PipePrintfSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = sort_unique_byte_lines(printf_args_to_lines(&plan.args));
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_printf_grep_sort_uniq_producer(
    plan: &PipePrintfGrepSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = sort_unique_byte_lines(filter_byte_lines_by_literal(
        printf_args_to_lines(&plan.args),
        &plan.pattern,
    ));
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_seq_grep_producer(
    plan: &PipeSeqGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    run_filtered_byte_line_producer(
        filter_byte_lines_by_literal(seq_to_lines(&plan.seq), &plan.pattern),
        plan.mode,
        stdout,
        stderr,
    )
}

fn run_filtered_byte_line_producer(
    mut lines: Vec<Vec<u8>>,
    mode: GrepFilePipeMode,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let matched = !lines.is_empty();
    match mode {
        GrepFilePipeMode::Lines => {
            for line in lines {
                stdout.write_all(&line)?;
            }
            return Ok(if matched { 0 } else { 1 });
        }
        GrepFilePipeMode::WcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::Wc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::Head { limit } => {
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Tail { limit } => {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        GrepFilePipeMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        GrepFilePipeMode::SortUniqWc { mode } => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", count_wc_unique_byte_lines(&lines, mode))?;
        }
        GrepFilePipeMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::SortWc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortXargsEcho => {
            lines.sort_unstable();
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::SortXargsEchoBatches { size } => {
            lines.sort_unstable();
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::SortXargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, true, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsEcho => {
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::XargsEchoBatches { size } => {
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::XargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, false, mode, stdout, stderr);
        }
        GrepFilePipeMode::SortXargsWcLines => {
            lines.sort_unstable();
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
        GrepFilePipeMode::XargsWcLines => {
            let mut paths = Vec::new();
            for line in &lines {
                paths.extend(xargs_tokens_from_bytes(line));
            }
            return write_xargs_wc_paths(&paths, stdout, stderr);
        }
    }
    Ok(0)
}

fn run_pipe_seq_producer(
    plan: &PipeSeqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let mut lines = seq_to_lines(&plan.seq);
    match plan.mode {
        GrepFilePipeMode::Lines => {
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::Sort => {
            lines.sort_unstable();
            for line in lines {
                stdout.write_all(&line)?;
            }
        }
        GrepFilePipeMode::SortUniq => {
            lines.sort_unstable();
            write_unique_byte_lines(&lines, stdout)?;
        }
        GrepFilePipeMode::SortUniqWcLines => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", unique_byte_line_count(&lines))?;
        }
        GrepFilePipeMode::SortUniqWc { mode } => {
            lines.sort_unstable();
            writeln!(stdout, "{:8}", count_wc_unique_byte_lines(&lines, mode))?;
        }
        GrepFilePipeMode::SortWcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::SortWc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::SortHead { limit } => {
            lines.sort_unstable();
            for line in lines
                .iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
            {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortTail { limit } => {
            lines.sort_unstable();
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let start = lines.len().saturating_sub(limit);
            for line in lines.iter().skip(start) {
                stdout.write_all(line)?;
            }
        }
        GrepFilePipeMode::SortXargsEcho => {
            lines.sort_unstable();
            write_xargs_echo_line_tokens(&lines, stdout)?;
        }
        GrepFilePipeMode::SortXargsEchoBatches { size } => {
            lines.sort_unstable();
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::SortXargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, true, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsWcOutput { mode } => {
            return run_xargs_wc_output_from_line_tokens(lines, false, mode, stdout, stderr);
        }
        GrepFilePipeMode::XargsEchoBatches { size } => {
            write_xargs_echo_batched_lines(&lines, size, stdout)?;
        }
        GrepFilePipeMode::WcLines => {
            writeln!(stdout, "{:8}", lines.len())?;
        }
        GrepFilePipeMode::Wc { mode } => {
            writeln!(stdout, "{:8}", count_wc_byte_lines(&lines, mode, false))?;
        }
        GrepFilePipeMode::Head { .. }
        | GrepFilePipeMode::Tail { .. }
        | GrepFilePipeMode::SortXargsWcLines
        | GrepFilePipeMode::XargsEcho
        | GrepFilePipeMode::XargsWcLines => {}
    }
    Ok(0)
}

fn run_pipe_seq_sort_uniq_producer(
    plan: &PipeSeqSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = sort_unique_byte_lines(seq_to_lines(&plan.seq));
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_seq_grep_sort_uniq_producer(
    plan: &PipeSeqGrepSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let lines = sort_unique_byte_lines(filter_byte_lines_by_literal(
        seq_to_lines(&plan.seq),
        &plan.pattern,
    ));
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_seq_xargs_echo(plan: &PipeSeqXargsEchoPlan, stdout: &mut dyn Write) -> Result<i32> {
    let mut first = true;
    let mut batch = Vec::new();
    let mut current = plan.seq.first as i128;
    let step = plan.seq.step as i128;
    let last = plan.seq.last as i128;
    let mut remaining = seq_count(&plan.seq);
    while remaining > 0 {
        match plan.mode {
            XargsEchoMode::OneLine => {
                if !first {
                    write!(stdout, " ")?;
                }
                write!(stdout, "{current}")?;
                first = false;
            }
            XargsEchoMode::Batch { size } => {
                batch.push(current.to_string());
                if batch.len() == size {
                    writeln!(stdout, "{}", batch.join(" "))?;
                    batch.clear();
                }
            }
        }
        current += step;
        remaining -= 1;
        if (step > 0 && current > last) || (step < 0 && current < last) {
            break;
        }
    }
    if plan.mode == XargsEchoMode::OneLine && !first {
        writeln!(stdout)?;
    }
    if !batch.is_empty() {
        writeln!(stdout, "{}", batch.join(" "))?;
    }
    Ok(0)
}

fn run_pipe_yes_head(plan: &PipeYesHeadPlan, stdout: &mut dyn Write) -> Result<i32> {
    for _ in 0..plan.limit {
        writeln!(stdout, "{}", plan.value)?;
    }
    Ok(0)
}

fn write_xargs_echo_paths(paths: &[String], stdout: &mut dyn Write) -> Result<()> {
    let mut first = true;
    for path in paths {
        write_xargs_echo_path_tokens(path, stdout, &mut first)?;
    }
    if !first {
        writeln!(stdout)?;
    }
    Ok(())
}

fn write_xargs_echo_byte_tokens(input: &[u8], stdout: &mut dyn Write) -> Result<()> {
    let text = String::from_utf8_lossy(input);
    let mut first = true;
    write_xargs_echo_path_tokens(&text, stdout, &mut first)?;
    if !first {
        writeln!(stdout)?;
    }
    Ok(())
}

fn write_xargs_echo_path_tokens(
    path: &str,
    stdout: &mut dyn Write,
    first: &mut bool,
) -> Result<()> {
    for token in path.split_whitespace() {
        if !*first {
            write!(stdout, " ")?;
        }
        write!(stdout, "{token}")?;
        *first = false;
    }
    Ok(())
}

fn run_pipe_find_xargs_echo(
    plan: &PipeFindXargsEchoPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut first = true;
    write_find_xargs_echo_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stdout,
        stderr,
        &mut first,
    )?;
    if !first {
        writeln!(stdout)?;
    }
    Ok(0)
}

fn write_find_xargs_echo_paths(
    path: PathBuf,
    name_glob: &str,
    max_depth: Option<usize>,
    depth: usize,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    first: &mut bool,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        if find_name_matches(&path, name_glob) {
            write_xargs_echo_path_tokens(&path.display().to_string(), stdout, first)?;
        }
    } else if meta.file_type().is_dir() && find_should_descend(max_depth, depth) {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            write_find_xargs_echo_paths(
                entry.path(),
                name_glob,
                max_depth,
                depth + 1,
                stdout,
                stderr,
                first,
            )?;
        }
    }
    Ok(())
}

fn run_pipe_find_xargs_wc_lines(
    plan: &PipeFindXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut total = 0u64;
    let mut files = 0u64;
    let mut exit = 0;
    visit_find_wc(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stdout,
        stderr,
        &mut total,
        &mut files,
        &mut exit,
    )?;
    if files > 1 {
        writeln!(stdout, "{total:8} total")?;
    }
    Ok(exit)
}

fn run_pipe_find_xargs_wc_producer(
    plan: &PipeFindXargsWcProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    if let Some(pattern) = &plan.pattern {
        paths.retain(|path| path.contains(pattern));
    }
    if plan.sort_paths {
        paths.sort();
    }
    if plan.uniq_paths {
        paths.dedup();
    }
    let (lines, _xargs_exit) = collect_xargs_wc_path_lines(&paths, stderr)?;
    write_xargs_wc_output_lines(lines, plan.mode, stdout)?;
    Ok(0)
}

fn run_pipe_find_grep_xargs_echo(
    plan: &PipeFindGrepXargsEchoPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let paths = collect_find_grep_paths(source, &plan.pattern, stderr)?;
    write_xargs_echo_paths(&paths, stdout)?;
    Ok(0)
}

fn run_pipe_find_grep_xargs_wc_lines(
    plan: &PipeFindGrepXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let paths = collect_find_grep_paths(source, &plan.pattern, stderr)?;
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn collect_find_grep_lines(
    source: &FindPipeSource,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    Ok(collect_find_grep_paths(source, pattern, stderr)?
        .into_iter()
        .map(|path| {
            let mut line = path.into_bytes();
            line.push(b'\n');
            line
        })
        .collect())
}

fn run_pipe_find_grep_producer(
    plan: &PipeFindGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let lines = collect_find_grep_lines(source, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_find_grep_sort_xargs_echo(
    plan: &PipeFindGrepSortXargsEchoPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = collect_find_grep_paths(source, &plan.pattern, stderr)?;
    paths.sort();
    write_xargs_echo_paths(&paths, stdout)?;
    Ok(0)
}

fn run_pipe_find_grep_sort_xargs_wc_lines(
    plan: &PipeFindGrepSortXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = collect_find_grep_paths(source, &plan.pattern, stderr)?;
    paths.sort();
    write_xargs_wc_paths(&paths, stdout, stderr)
}

fn collect_find_grep_paths(
    source: &FindPipeSource,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.retain(|path| path.contains(pattern));
    Ok(paths)
}

fn visit_find_wc(
    path: PathBuf,
    name_glob: &str,
    max_depth: Option<usize>,
    depth: usize,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    total: &mut u64,
    files: &mut u64,
    exit: &mut i32,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            *exit = 1;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        if find_name_matches(&path, name_glob) {
            let file = path.display().to_string();
            match count_newlines(&file) {
                Ok(lines) => {
                    *total = total.saturating_add(lines);
                    *files += 1;
                    writeln!(stdout, "{lines:8} {file}")?;
                }
                Err(e) => {
                    writeln!(stderr, "wc: {file}: {e}")?;
                    *exit = 1;
                }
            }
        }
    } else if meta.file_type().is_dir() && find_should_descend(max_depth, depth) {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                *exit = 1;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            visit_find_wc(
                entry.path(),
                name_glob,
                max_depth,
                depth + 1,
                stdout,
                stderr,
                total,
                files,
                exit,
            )?;
        }
    }
    Ok(())
}

fn run_pipe_find_wc_lines(
    plan: &PipeFindWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut count = 0u64;
    visit_find_count(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut count,
    )?;
    writeln!(stdout, "{count:8}")?;
    Ok(0)
}

fn visit_find_count(
    path: PathBuf,
    name_glob: &str,
    max_depth: Option<usize>,
    depth: usize,
    stderr: &mut dyn Write,
    count: &mut u64,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        if find_name_matches(&path, name_glob) {
            *count = count.saturating_add(1);
        }
    } else if meta.file_type().is_dir() && find_should_descend(max_depth, depth) {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            visit_find_count(entry.path(), name_glob, max_depth, depth + 1, stderr, count)?;
        }
    }
    Ok(())
}

fn run_pipe_find_head(
    plan: &PipeFindHeadPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut remaining = plan.limit;
    let mut exit = 0;
    visit_find_head(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stdout,
        stderr,
        &mut remaining,
        &mut exit,
    )?;
    let _ = exit;
    Ok(0)
}

fn visit_find_head(
    path: PathBuf,
    name_glob: &str,
    max_depth: Option<usize>,
    depth: usize,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    remaining: &mut u64,
    exit: &mut i32,
) -> Result<()> {
    if *remaining == 0 {
        return Ok(());
    }
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            *exit = 1;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        if find_name_matches(&path, name_glob) {
            writeln!(stdout, "{}", path.display())?;
            *remaining = remaining.saturating_sub(1);
        }
    } else if meta.file_type().is_dir() && find_should_descend(max_depth, depth) {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                *exit = 1;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            visit_find_head(
                entry.path(),
                name_glob,
                max_depth,
                depth + 1,
                stdout,
                stderr,
                remaining,
                exit,
            )?;
            if *remaining == 0 {
                break;
            }
        }
    }
    Ok(())
}

fn write_tail_paths(paths: &[String], limit: u64, stdout: &mut dyn Write) -> Result<()> {
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    let start = paths.len().saturating_sub(limit);
    for path in paths.iter().skip(start) {
        writeln!(stdout, "{path}")?;
    }
    Ok(())
}

fn find_name_matches(path: &Path, name_glob: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| glob_match(name_glob, name))
}

fn run_pipe_find_tail(
    plan: &PipeFindTailPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let limit = usize::try_from(plan.limit).unwrap_or(usize::MAX);
    if limit == 0 {
        return Ok(0);
    }
    let mut paths = VecDeque::new();
    collect_find_tail_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        limit,
        &mut paths,
    )?;
    for path in paths {
        writeln!(stdout, "{path}")?;
    }
    Ok(0)
}

fn collect_find_tail_paths(
    path: PathBuf,
    name_glob: &str,
    max_depth: Option<usize>,
    depth: usize,
    stderr: &mut dyn Write,
    limit: usize,
    paths: &mut VecDeque<String>,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        if find_name_matches(&path, name_glob) {
            if paths.len() == limit {
                paths.pop_front();
            }
            paths.push_back(path.display().to_string());
        }
    } else if meta.file_type().is_dir() && find_should_descend(max_depth, depth) {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            collect_find_tail_paths(
                entry.path(),
                name_glob,
                max_depth,
                depth + 1,
                stderr,
                limit,
                paths,
            )?;
        }
    }
    Ok(())
}

fn run_pipe_find_sort(
    plan: &PipeFindSortPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    for path in paths {
        writeln!(stdout, "{path}")?;
    }
    Ok(0)
}

fn run_pipe_find_sort_uniq(
    plan: &PipeFindSortUniqPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    paths.dedup();
    for path in paths {
        writeln!(stdout, "{path}")?;
    }
    Ok(0)
}

fn run_pipe_find_sort_uniq_wc_lines(
    plan: &PipeFindSortUniqWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    paths.dedup();
    writeln!(stdout, "{:8}", paths.len())?;
    Ok(0)
}

fn collect_find_sort_uniq_lines(
    source: &FindPipeSource,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    paths.dedup();
    Ok(paths
        .into_iter()
        .map(|path| {
            let mut line = path.into_bytes();
            line.push(b'\n');
            line
        })
        .collect())
}

fn collect_find_sort_uniq_grep_lines(
    source: &FindPipeSource,
    pattern: &str,
    stderr: &mut dyn Write,
) -> Result<Vec<Vec<u8>>> {
    let mut lines =
        filter_byte_lines_by_literal(collect_find_sort_uniq_lines(source, stderr)?, pattern);
    ensure_byte_lines_end_with_newline(&mut lines);
    Ok(lines)
}

fn run_pipe_find_sort_uniq_producer(
    plan: &PipeFindSortUniqProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let lines = collect_find_sort_uniq_lines(source, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, false, true)
}

fn run_pipe_find_sort_uniq_grep_producer(
    plan: &PipeFindSortUniqGrepProducerPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let lines = collect_find_sort_uniq_grep_lines(source, &plan.pattern, stderr)?;
    run_head_line_producer(lines, plan.mode, stdout, stderr, true, false)
}

fn run_pipe_find_sort_xargs_echo(
    plan: &PipeFindSortXargsEchoPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    write_xargs_echo_paths(&paths, stdout)?;
    Ok(0)
}

fn run_pipe_find_sort_xargs_wc_lines(
    plan: &PipeFindSortXargsWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    let mut total = 0u64;
    let mut exit = 0;
    for file in &paths {
        match count_newlines(file) {
            Ok(lines) => {
                total = total.saturating_add(lines);
                writeln!(stdout, "{lines:8} {file}")?;
            }
            Err(e) => {
                writeln!(stderr, "wc: {file}: {e}")?;
                exit = 1;
            }
        }
    }
    if paths.len() > 1 {
        writeln!(stdout, "{total:8} total")?;
    }
    Ok(exit)
}

fn run_pipe_find_sort_wc_lines(
    plan: &PipeFindSortWcLinesPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    writeln!(stdout, "{:8}", paths.len())?;
    Ok(0)
}

fn run_pipe_find_sort_head(
    plan: &PipeFindSortHeadPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    for path in paths
        .iter()
        .take(usize::try_from(plan.limit).unwrap_or(usize::MAX))
    {
        writeln!(stdout, "{path}")?;
    }
    Ok(0)
}

fn run_pipe_find_sort_tail(
    plan: &PipeFindSortTailPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let source = &plan.source;
    let mut paths = Vec::new();
    collect_find_named_paths(
        PathBuf::from(&source.root),
        &source.name_glob,
        source.max_depth,
        0,
        stderr,
        &mut paths,
    )?;
    paths.sort();
    write_tail_paths(&paths, plan.limit, stdout)?;
    Ok(0)
}

fn collect_find_named_paths(
    path: PathBuf,
    name_glob: &str,
    max_depth: Option<usize>,
    depth: usize,
    stderr: &mut dyn Write,
    paths: &mut Vec<String>,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            return Ok(());
        }
    };
    if meta.file_type().is_file() {
        if find_name_matches(&path, name_glob) {
            paths.push(path.display().to_string());
        }
    } else if meta.file_type().is_dir() && find_should_descend(max_depth, depth) {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                return Ok(());
            }
        };
        for entry in entries.flatten() {
            collect_find_named_paths(entry.path(), name_glob, max_depth, depth + 1, stderr, paths)?;
        }
    }
    Ok(())
}

fn find_should_descend(max_depth: Option<usize>, depth: usize) -> bool {
    max_depth.is_none_or(|limit| depth < limit)
}

fn run_find(plan: &FindPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let mut exit = 0;
    visit_find(PathBuf::from(&plan.root), plan, stdout, stderr, &mut exit)?;
    Ok(exit)
}

fn visit_find(
    path: PathBuf,
    plan: &FindPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    exit: &mut i32,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            *exit = 1;
            return Ok(());
        }
    };

    if find_path_matches(&path, &meta, plan) {
        writeln!(stdout, "{}", path.display())?;
    }

    if meta.file_type().is_dir() {
        let mut children = Vec::new();
        match fs::read_dir(&path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => children.push(entry.path()),
                        Err(e) => {
                            writeln!(stderr, "find: {}: {e}", path.display())?;
                            *exit = 1;
                        }
                    }
                }
            }
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                *exit = 1;
            }
        }
        children.sort();
        for child in children {
            visit_find(child, plan, stdout, stderr, exit)?;
        }
    }

    Ok(())
}

fn find_path_matches(path: &Path, meta: &fs::Metadata, plan: &FindPlan) -> bool {
    let type_ok = match plan.type_filter {
        Some(FindType::File) => meta.file_type().is_file(),
        Some(FindType::Dir) => meta.file_type().is_dir(),
        None => true,
    };
    let name_ok = match &plan.name_pattern {
        Some(pattern) => path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| glob_match(pattern, name)),
        None => true,
    };
    type_ok && name_ok
}

fn run_sed_print(
    plan: &SedPrintPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let file = fs::File::open(&plan.file).with_context(|| format!("reading {}", plan.file))?;
    let reader = BufReader::new(file);
    for (idx, line) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let line = line?;
        if line_no >= plan.start_line && line_no <= plan.end_line {
            writeln!(stdout, "{line}")?;
        }
        if line_no > plan.end_line {
            break;
        }
    }
    Ok(0)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn run_wc_all(plan: &WcAllPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    if plan.files.is_empty() {
        let stdin = io::stdin();
        match count_reader_wc_all(stdin.lock()) {
            Ok(counts) => {
                write_wc_all_counts(stdout, counts, None)?;
                return Ok(0);
            }
            Err(e) => {
                writeln!(stderr, "wc: stdin: {e}")?;
                return Ok(1);
            }
        }
    }

    let mut total = WcCounts::default();
    let mut exit = 0;
    for file in &plan.files {
        match count_file_wc_all(file) {
            Ok(counts) => {
                total.add(counts);
                write_wc_all_counts(stdout, counts, Some(file))?;
            }
            Err(e) => {
                writeln!(stderr, "wc: {file}: {e}")?;
                exit = 1;
            }
        }
    }
    if plan.files.len() > 1 {
        write_wc_all_counts(stdout, total, Some("total"))?;
    }
    Ok(exit)
}

// @spec apps/cap/tech-design/logic/expand-high-volume-native-command-coverage.md#changes
fn run_wc_lines(plan: &WcLinesPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    if plan.files.is_empty() {
        let stdin = io::stdin();
        match count_reader_wc(stdin.lock(), plan.mode) {
            Ok(count) => {
                writeln!(stdout, "{count:8}")?;
                return Ok(0);
            }
            Err(e) => {
                writeln!(stderr, "wc: stdin: {e}")?;
                return Ok(1);
            }
        }
    }

    let mut total = 0u64;
    let mut exit = 0;
    for file in &plan.files {
        match count_file_wc(file, plan.mode) {
            Ok(count) => {
                total = total.saturating_add(count);
                writeln!(stdout, "{count:8} {file}")?;
            }
            Err(e) => {
                writeln!(stderr, "wc: {file}: {e}")?;
                exit = 1;
            }
        }
    }
    if plan.files.len() > 1 {
        writeln!(stdout, "{total:8} total")?;
    }
    Ok(exit)
}

#[derive(Clone, Copy, Debug, Default)]
struct WcCounts {
    lines: u64,
    words: u64,
    bytes: u64,
}

impl WcCounts {
    fn add(&mut self, other: Self) {
        self.lines = self.lines.saturating_add(other.lines);
        self.words = self.words.saturating_add(other.words);
        self.bytes = self.bytes.saturating_add(other.bytes);
    }
}

fn write_wc_all_counts(
    stdout: &mut dyn Write,
    counts: WcCounts,
    label: Option<&str>,
) -> Result<()> {
    write!(
        stdout,
        "{:8}{:8}{:8}",
        counts.lines, counts.words, counts.bytes
    )?;
    if let Some(label) = label {
        write!(stdout, " {label}")?;
    }
    writeln!(stdout)?;
    Ok(())
}

fn count_file_wc_all(file: &str) -> Result<WcCounts> {
    let file = fs::File::open(file)?;
    count_reader_wc_all(file)
}

fn count_reader_wc_all<R: Read>(reader: R) -> Result<WcCounts> {
    let mut reader = BufReader::new(reader);
    let mut buf = [0u8; 8192];
    let mut counts = WcCounts::default();
    let mut in_word = false;
    loop {
        let read = std::io::Read::read(&mut reader, &mut buf)?;
        if read == 0 {
            break;
        }
        counts.bytes = counts.bytes.saturating_add(read as u64);
        for byte in &buf[..read] {
            if *byte == b'\n' {
                counts.lines = counts.lines.saturating_add(1);
            }
            if byte.is_ascii_whitespace() {
                in_word = false;
            } else if !in_word {
                counts.words = counts.words.saturating_add(1);
                in_word = true;
            }
        }
    }
    Ok(counts)
}

fn count_file_wc(file: &str, mode: WcCountMode) -> Result<u64> {
    let file = fs::File::open(file)?;
    if mode == WcCountMode::Bytes {
        return Ok(file.metadata()?.len());
    }
    count_reader_wc(file, mode)
}

fn count_reader_wc<R: Read>(reader: R, mode: WcCountMode) -> Result<u64> {
    let mut reader = BufReader::new(reader);
    let mut buf = [0u8; 8192];
    let mut count = 0u64;
    let mut in_word = false;
    loop {
        let read = std::io::Read::read(&mut reader, &mut buf)?;
        if read == 0 {
            break;
        }
        match mode {
            WcCountMode::Lines => {
                count += buf[..read].iter().filter(|byte| **byte == b'\n').count() as u64;
            }
            WcCountMode::Bytes => {
                count = count.saturating_add(read as u64);
            }
            WcCountMode::Words => {
                for byte in &buf[..read] {
                    if byte.is_ascii_whitespace() {
                        in_word = false;
                    } else if !in_word {
                        count = count.saturating_add(1);
                        in_word = true;
                    }
                }
            }
        }
    }
    Ok(count)
}

fn count_newlines(file: &str) -> Result<u64> {
    let mut reader = BufReader::new(fs::File::open(file)?);
    let mut buf = [0u8; 8192];
    let mut lines = 0u64;
    loop {
        let read = std::io::Read::read(&mut reader, &mut buf)?;
        if read == 0 {
            break;
        }
        lines += buf[..read].iter().filter(|byte| **byte == b'\n').count() as u64;
    }
    Ok(lines)
}

fn basename_value(path: &str, suffix: Option<&str>) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let bytes = path.as_bytes();
    let mut end = bytes.len();
    while end > 1 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && bytes[start - 1] != b'/' {
        start -= 1;
    }
    if start == end && bytes.first() == Some(&b'/') {
        return "/".to_string();
    }
    let mut value = path[start..end].to_string();
    if let Some(suffix) = suffix {
        if !suffix.is_empty() && value.len() > suffix.len() && value.ends_with(suffix) {
            value.truncate(value.len() - suffix.len());
        }
    }
    value
}

fn dirname_value(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let bytes = path.as_bytes();
    let mut end = bytes.len();
    while end > 1 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    while end > 0 && bytes[end - 1] != b'/' {
        end -= 1;
    }
    if end == 0 {
        return ".".to_string();
    }
    while end > 1 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    path[..end].to_string()
}

fn touch_now(path: &Path) -> io::Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains NUL byte"))?;
    let rc = unsafe { libc::utimes(c_path.as_ptr(), std::ptr::null()) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn parse_sed_print_script(script: &str) -> Option<(usize, usize)> {
    let body = script.strip_suffix('p')?;
    if body.is_empty() {
        return None;
    }
    let (start, end) = if let Some((start, end)) = body.split_once(',') {
        (start.parse().ok()?, end.parse().ok()?)
    } else {
        let line = body.parse().ok()?;
        (line, line)
    };
    if start == 0 || end < start {
        return None;
    }
    Some((start, end))
}

fn glob_match(pattern: &str, text: &str) -> bool {
    fn inner(p: &[u8], t: &[u8]) -> bool {
        match p.split_first() {
            None => t.is_empty(),
            Some((&b'*', rest)) => inner(rest, t) || (!t.is_empty() && inner(p, &t[1..])),
            Some((&b'?', rest)) => !t.is_empty() && inner(rest, &t[1..]),
            Some((&pc, rest)) => t.first().is_some_and(|tc| *tc == pc) && inner(rest, &t[1..]),
        }
    }
    inner(pattern.as_bytes(), text.as_bytes())
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn render_argv(command: &[String]) -> String {
    match command.split_first() {
        Some((program, args)) => render_command(program, args),
        None => String::new(),
    }
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn render_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| shell_quote_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote_arg(s: &str) -> String {
    let safe = !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | ':'));
    if safe {
        s.to_string()
    } else {
        shell_single_quote(s)
    }
}

fn shell_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

fn command_on_path(program: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|dir| dir.join(program).is_file())
}

fn basename(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

fn has_shell_control_syntax(command: &str) -> bool {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        Single,
        Double,
    }

    let mut chars = command.chars();
    let mut state = State::Normal;
    while let Some(ch) = chars.next() {
        match state {
            State::Normal => match ch {
                '\'' => state = State::Single,
                '"' => state = State::Double,
                '\\' => {
                    if chars.next().is_none() {
                        return true;
                    }
                }
                '\n' | '\r' | '|' | '&' | ';' | '<' | '>' | '`' | '$' | '*' | '?' | '[' | ']'
                | '{' | '}' | '~' | '(' | ')' => return true,
                _ => {}
            },
            State::Single => {
                if ch == '\'' {
                    state = State::Normal;
                }
            }
            State::Double => match ch {
                '"' => state = State::Normal,
                '\\' => {
                    if chars.next().is_none() {
                        return true;
                    }
                }
                '`' | '$' => return true,
                _ => {}
            },
        }
    }

    !matches!(state, State::Normal)
}

fn split_simple_shell_words(command: &str) -> Option<Vec<String>> {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        Single,
        Double,
    }

    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars();
    let mut state = State::Normal;
    let mut in_token = false;

    while let Some(ch) = chars.next() {
        match state {
            State::Normal => match ch {
                '\'' => {
                    in_token = true;
                    state = State::Single;
                }
                '"' => {
                    in_token = true;
                    state = State::Double;
                }
                '\\' => {
                    in_token = true;
                    current.push(chars.next()?);
                }
                c if c.is_whitespace() => {
                    if in_token {
                        words.push(std::mem::take(&mut current));
                        in_token = false;
                    }
                }
                c => {
                    in_token = true;
                    current.push(c);
                }
            },
            State::Single => match ch {
                '\'' => state = State::Normal,
                c => current.push(c),
            },
            State::Double => match ch {
                '"' => state = State::Normal,
                '\\' => current.push(chars.next()?),
                c => current.push(c),
            },
        }
    }

    match state {
        State::Normal => {
            if in_token {
                words.push(current);
            }
            Some(words)
        }
        State::Single | State::Double => None,
    }
}

fn words_need_shell(words: &[String]) -> bool {
    let first = basename(&words[0]);
    first_word_needs_shell(first) || is_var_assignment(&words[0])
}

fn first_word_needs_shell(first: &str) -> bool {
    matches!(
        first,
        "alias"
            | "bg"
            | "break"
            | "cd"
            | "continue"
            | "eval"
            | "exec"
            | "export"
            | "fc"
            | "fg"
            | "jobs"
            | "read"
            | "readonly"
            | "return"
            | "set"
            | "shift"
            | "source"
            | "times"
            | "trap"
            | "type"
            | "typeset"
            | "ulimit"
            | "umask"
            | "unalias"
            | "unset"
            | "."
    )
}

fn is_var_assignment(tok: &str) -> bool {
    let Some(eq) = tok.find('=') else {
        return false;
    };
    let name = &tok[..eq];
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_plain_literal_pattern(pattern: &str) -> bool {
    !pattern.is_empty()
        && !pattern.chars().any(|ch| {
            matches!(
                ch,
                '.' | '[' | ']' | '\\' | '*' | '^' | '$' | '+' | '?' | '{' | '}' | '(' | ')' | '|'
            )
        })
}

fn exit_code_from_i32(code: i32) -> ExitCode {
    if (0..=255).contains(&code) {
        ExitCode::from(code as u8)
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    fn plan_without_tools(args: &[&str]) -> CommandPlan {
        plan_with_tool_resolver(&s(args), None, |_| false)
    }

    #[test]
    fn shell_string_simple_commands_use_cap_planner() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("file.txt"), "").unwrap();
        assert!(matches!(
            plan_shell(&format!("find {} -type f", tmp.path().display()), None),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Find(_),
                ..
            })
        ));
    }

    #[test]
    fn shell_string_pipes_keep_bash_semantics() {
        let CommandPlan::External(plan) = plan_shell("find . -type d | xargs wc -l", None) else {
            panic!("expected bash fallback");
        };
        assert_eq!(plan.program, "bash");
        assert_eq!(plan.args, vec!["-c", "find . -type d | xargs wc -l"]);
        assert_eq!(
            plan.reason,
            "shell command string requires bash semantics; running under bash -c"
        );
    }

    #[test]
    fn supported_pipe_shapes_plan_native() {
        let tmp = tempdir().unwrap();
        let grep_root = tmp.path().join("grep");
        fs::create_dir(&grep_root).unwrap();
        fs::write(grep_root.join("a.txt"), "NEEDLE\n").unwrap();
        let grep_file = tmp.path().join("grep-file.txt");
        fs::write(&grep_file, "z NEEDLE\na\nz NEEDLE\n").unwrap();
        let sed_file = tmp.path().join("sed.txt");
        fs::write(&sed_file, "a NEEDLE\nb\n").unwrap();
        let sort_file = tmp.path().join("sort.txt");
        fs::write(&sort_file, "z\na\nz\n").unwrap();
        let wc_a = tmp.path().join("wc-a.txt");
        let wc_b = tmp.path().join("wc-b.txt");
        fs::write(&wc_a, "one\n").unwrap();
        fs::write(&wc_b, "one\ntwo\n").unwrap();
        let xargs_wc_file = tmp.path().join("xargs-wc-paths.txt");
        fs::write(
            &xargs_wc_file,
            format!("{}\n{}\n", wc_b.display(), wc_a.display()),
        )
        .unwrap();
        let awk_xargs_wc_file = tmp.path().join("awk-xargs-wc-paths.txt");
        fs::write(
            &awk_xargs_wc_file,
            format!("{} NEEDLE\n{} NEEDLE\n", wc_b.display(), wc_a.display()),
        )
        .unwrap();
        let cut_file = tmp.path().join("cut.csv");
        fs::write(&cut_file, "alpha,beta\nplain\ngamma,delta\n").unwrap();
        let find_root = tmp.path().join("find");
        fs::create_dir(&find_root).unwrap();
        fs::write(find_root.join("a.txt"), "one\n").unwrap();
        fs::write(find_root.join("a.rs"), "one\n").unwrap();
        let list_dir = tmp.path().join("list");
        fs::create_dir(&list_dir).unwrap();
        fs::write(list_dir.join("entry"), "").unwrap();
        fs::write(list_dir.join(".hidden-entry"), "").unwrap();
        let missing_path = tmp.path().join("missing");
        let mkdir_pipe_dir = tmp.path().join("mkdir-pipe-created");
        let touch_pipe_file = tmp.path().join("touch-pipe-created.txt");

        for command in [
            "true | wc -l".to_string(),
            "false | wc -l".to_string(),
            "false | grep NEEDLE".to_string(),
            "false | grep NEEDLE | wc -l".to_string(),
            "true | xargs echo".to_string(),
            format!("mkdir -p {} | wc -l", mkdir_pipe_dir.display()),
            format!("mkdir -p {} | xargs echo", mkdir_pipe_dir.display()),
            format!("touch {} | wc -l", touch_pipe_file.display()),
            format!("touch {} | sort | xargs echo", touch_pipe_file.display()),
            format!("test -f {} | wc -l", grep_file.display()),
            format!("test ! -e {} | xargs echo", missing_path.display()),
            format!("[ -d {} ] | sort | xargs echo", list_dir.display()),
            format!("test -d {} | grep NEEDLE | wc -l", grep_file.display()),
            "wc -l | wc -l".to_string(),
            "wc -c | head -n 1".to_string(),
            "wc -w | grep 3 | wc -l".to_string(),
            "wc -l | sort | xargs echo".to_string(),
            format!("wc -l {} | xargs echo", wc_a.display()),
            format!("wc -c {} {} | wc -l", wc_a.display(), wc_b.display()),
            format!("wc -l {} {} | grep total | wc -l", wc_a.display(), wc_b.display()),
            format!("wc -w {} | sort | xargs echo", wc_b.display()),
            "printf 'alpha\\nbeta\\n' | wc -l".to_string(),
            "printf 'zeta\\nalpha\\n' | sort | xargs echo".to_string(),
            "printf 'alpha\\nbeta\\n' | grep beta | wc -l".to_string(),
            "printf 'alpha\\nbeta' | grep beta | wc -l".to_string(),
            format!("du -sk {} | wc -l", find_root.display()),
            format!("du -sk {} | xargs echo", find_root.display()),
            format!("du -sk {} | grep find | wc -l", find_root.display()),
            "echo alpha beta | wc -l".to_string(),
            "echo alpha beta | wc -c".to_string(),
            "echo alpha beta | wc -w".to_string(),
            "echo -n alpha beta | head -n 1".to_string(),
            "echo -n alpha beta | tail -n 1".to_string(),
            "echo 'alpha beta' 'gamma delta' | awk '{ print $1 }'".to_string(),
            "echo 'alpha beta' 'gamma delta' | awk '{ print $2 }'".to_string(),
            "echo 'alpha beta' 'gamma delta' | awk '{ print $1 }' | xargs".to_string(),
            "awk '{ print $1 }' | wc -l".to_string(),
            "awk '{ print $2 }' | wc -l".to_string(),
            "awk '/NEEDLE/ { print $1 }' | wc -l".to_string(),
            "awk '/NEEDLE/ { print $2 }' | wc -l".to_string(),
            "grep NEEDLE | wc -l".to_string(),
            "grep NEEDLE | head -n 1".to_string(),
            "grep NEEDLE | xargs echo".to_string(),
            "xargs echo | grep NEEDLE".to_string(),
            "xargs echo | grep NEEDLE | wc -l".to_string(),
            "xargs echo | grep NEEDLE | head -n 1".to_string(),
            "printf '%s\\n' alpha beta gamma | wc -l".to_string(),
            "printf '%s\\n' alpha beta gamma | wc -c".to_string(),
            "printf '%s\\n' alpha beta gamma | wc -w".to_string(),
            "printf '%s\\n' alpha beta gamma | head -n 2".to_string(),
            "printf '%s\\n' alpha beta gamma | tail -n 2".to_string(),
            "printf '%s\\n' 'alpha beta' 'gamma delta' | awk '{ print $1 }' | wc -l"
                .to_string(),
            "printf '%s\\n' 'alpha beta' 'gamma delta' | awk '{ print $2 }' | wc -l"
                .to_string(),
            "printf '%s\\n' 'gamma two' 'alpha one' 'alpha three' | awk '{ print $1 }' | sort | uniq"
                .to_string(),
            "printf '%s\\n' 'gamma two' 'alpha one' 'alpha three' | awk '{ print $2 }' | sort | uniq"
                .to_string(),
            "printf '%s\\n' alpha NEEDLE gamma | grep NEEDLE".to_string(),
            "printf '%s\\n' alpha NEEDLE gamma NEEDLE | grep NEEDLE | wc -l".to_string(),
            "printf '%s\\n' alpha NEEDLE gamma NEEDLE | grep NEEDLE | wc -w".to_string(),
            "printf '%s\\n' alpha NEEDLE1 NEEDLE2 gamma | grep NEEDLE | head -n 1".to_string(),
            "printf '%s\\n' alpha NEEDLE1 NEEDLE2 gamma | grep NEEDLE | tail -n 1".to_string(),
            "printf '%s\\n' zeta NEEDLE2 alpha NEEDLE1 | grep NEEDLE | sort".to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq".to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | wc -l"
                .to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | wc -c"
                .to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | head -n 1"
                .to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | tail -n 1"
                .to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | sort | xargs echo"
                .to_string(),
            format!(
                "printf '%s\\n' {} {} {} | grep wc | sort | uniq | xargs wc -l",
                wc_b.display(),
                wc_a.display(),
                wc_b.display()
            ),
            "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | wc -l".to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | head -n 1".to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | tail -n 1".to_string(),
            "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | xargs echo".to_string(),
            "printf '%s\\n' alpha NEEDLE1 NEEDLE2 gamma | grep NEEDLE | xargs echo".to_string(),
            "printf '%s\\n' alpha beta gamma | tr a-z A-Z".to_string(),
            "printf '%s\\n' gamma alpha beta | sort".to_string(),
            "printf '%s\\n' gamma alpha alpha | sort | uniq".to_string(),
            "printf '%s\\n' gamma alpha alpha | sort | uniq | wc -l".to_string(),
            "printf '%s\\n' gamma alpha alpha | sort | uniq | head -n 1".to_string(),
            "printf '%s\\n' gamma alpha alpha | sort | uniq | tail -n 1".to_string(),
            "printf '%s\\n' gamma alpha alpha | sort | uniq | sort | xargs echo".to_string(),
            format!(
                "printf '%s\\n' {} {} {} | sort | uniq | xargs wc -l",
                wc_b.display(),
                wc_a.display(),
                wc_b.display()
            ),
            "printf '%s\\n' gamma alpha beta | sort | wc -l".to_string(),
            "printf '%s\\n' gamma alpha beta | sort | head -n 1".to_string(),
            "printf '%s\\n' gamma alpha beta | sort | tail -n 1".to_string(),
            "printf '%s\\n' gamma alpha beta | sort | xargs echo".to_string(),
            "printf '%s\\n' gamma alpha beta | sort | xargs -n1 echo".to_string(),
            "printf '%s\\n' gamma alpha beta delta | sort | xargs -n2 echo".to_string(),
            format!(
                "printf '%s\\n' {} {} | sort | xargs wc -l",
                wc_b.display(),
                wc_a.display()
            ),
            "seq 1 10 | wc -l".to_string(),
            "seq 1 10 | head -n 3".to_string(),
            "seq 1 10 | tail -n 3".to_string(),
            "seq 1 10 | sort".to_string(),
            "seq 1 10 | sort | uniq".to_string(),
            "seq 1 10 | sort | uniq | wc -l".to_string(),
            "seq 1 10 | sort | uniq | head -n 3".to_string(),
            "seq 1 10 | sort | uniq | tail -n 3".to_string(),
            "seq 1 10 | sort | uniq | sort | xargs echo".to_string(),
            "seq 1 10 | sort | wc -l".to_string(),
            "seq 1 10 | sort | head -n 3".to_string(),
            "seq 1 10 | sort | tail -n 3".to_string(),
            "seq 1 10 | sort | xargs echo".to_string(),
            "seq 1 20 | grep 1".to_string(),
            "seq 1 20 | grep 1 | wc -l".to_string(),
            "seq 1 20 | grep 1 | head -n 3".to_string(),
            "seq 1 20 | grep 1 | tail -n 3".to_string(),
            "seq 1 20 | grep 1 | sort".to_string(),
            "seq 1 20 | grep 1 | sort | uniq".to_string(),
            "seq 1 20 | grep 1 | sort | uniq | wc -l".to_string(),
            "seq 1 20 | grep 1 | sort | uniq | head -n 3".to_string(),
            "seq 1 20 | grep 1 | sort | uniq | tail -n 3".to_string(),
            "seq 1 20 | grep 1 | sort | uniq | sort | xargs echo".to_string(),
            "seq 1 20 | grep 1 | sort | wc -l".to_string(),
            "seq 1 20 | grep 1 | sort | head -n 3".to_string(),
            "seq 1 20 | grep 1 | sort | tail -n 3".to_string(),
            "seq 1 20 | grep 1 | sort | xargs echo".to_string(),
            "seq 1 20 | grep 1 | xargs echo".to_string(),
            "yes | head -n 3".to_string(),
            "yes READY | head -n 3".to_string(),
            "which sh | wc -l".to_string(),
            "which sh | head -n 1".to_string(),
            "which sh | tail -n 1".to_string(),
            "which sh | grep / | wc -l".to_string(),
            "which sh | grep / | xargs echo".to_string(),
            "which sh | xargs echo".to_string(),
            "which sh | sort | wc -l".to_string(),
            "which sh | sort | xargs echo".to_string(),
            "which -a sh | wc -l".to_string(),
            "which -a sh | head -n 1".to_string(),
            "which -a sh | tail -n 1".to_string(),
            "which -a sh | grep / | wc -l".to_string(),
            "which -a sh | grep / | xargs echo".to_string(),
            "which -a sh | xargs echo".to_string(),
            "which -a sh | sort | wc -l".to_string(),
            "which -a sh | sort | xargs echo".to_string(),
            "id | wc -l".to_string(),
            "id | grep uid | xargs echo".to_string(),
            "id | sort | xargs echo".to_string(),
            "command -v sh | wc -l".to_string(),
            "command -v sh | head -n 1".to_string(),
            "command -v sh | tail -n 1".to_string(),
            "command -v sh | grep / | wc -l".to_string(),
            "command -v sh | grep / | head -n 1".to_string(),
            "command -v sh | xargs echo".to_string(),
            "command -v sh | sort | wc -l".to_string(),
            "command -v sh | sort | xargs echo".to_string(),
            "printenv PATH | wc -l".to_string(),
            "printenv PATH | head -n 1".to_string(),
            "printenv PATH | tail -n 1".to_string(),
            "printenv PATH | grep /".to_string(),
            "printenv PATH | grep / | wc -l".to_string(),
            "printenv PATH | grep / | head -n 1".to_string(),
            "printenv PATH | grep / | sort".to_string(),
            "printenv PATH | grep / | xargs echo".to_string(),
            "printenv PATH | sort".to_string(),
            "printenv PATH | xargs echo".to_string(),
            "printenv PATH | sort | xargs echo".to_string(),
            "hostname | wc -l".to_string(),
            "hostname | head -n 1".to_string(),
            "hostname | tail -n 1".to_string(),
            "hostname | grep host".to_string(),
            "hostname | grep host | wc -l".to_string(),
            "hostname | grep host | xargs echo".to_string(),
            "hostname | sort".to_string(),
            "hostname | xargs echo".to_string(),
            "hostname | sort | xargs echo".to_string(),
            "pwd | wc -l".to_string(),
            "pwd | head -n 1".to_string(),
            "pwd | tail -n 1".to_string(),
            "pwd | sort".to_string(),
            "pwd | grep /".to_string(),
            "pwd | grep / | wc -l".to_string(),
            "pwd | xargs echo".to_string(),
            "pwd | sort | xargs echo".to_string(),
            "basename /tmp/cap/example.txt .txt | wc -l".to_string(),
            "basename /tmp/cap/example.txt .txt | grep example | xargs echo".to_string(),
            "dirname /tmp/cap/example.txt | sort | tail -n 1".to_string(),
            "whoami | wc -l".to_string(),
            "id -u | head -n 1".to_string(),
            "id -un | xargs echo".to_string(),
            "id -G | wc -w".to_string(),
            "id -Gn | sort | xargs echo".to_string(),
            "uname -m | sort".to_string(),
            "uname -p | xargs echo".to_string(),
            "uname -a | xargs echo".to_string(),
            format!("sed -n 1,2p {} | wc -l", sed_file.display()),
            format!("sed -n 1,2p {} | head -n 1", sed_file.display()),
            format!("sed -n 1,2p {} | tail -n 1", sed_file.display()),
            format!("sed -n 1,2p {} | sort", sed_file.display()),
            format!("sed -n 1,2p {} | sort | uniq", sed_file.display()),
            format!("sed -n 1,2p {} | sort | uniq | wc -l", sed_file.display()),
            format!("sed -n 1,2p {} | sort | wc -l", sed_file.display()),
            format!("sed -n 1,2p {} | sort | head -n 1", sed_file.display()),
            format!("sed -n 1,2p {} | sort | tail -n 1", sed_file.display()),
            format!("sed -n 1,2p {} | sort | xargs echo", sed_file.display()),
            format!(
                "sed -n 1,2p {} | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("sed -n 1,2p {} | xargs echo", sed_file.display()),
            format!("sed -n 1,2p {} | xargs wc -l", xargs_wc_file.display()),
            format!("sed -n 1,2p {} | grep NEEDLE", sed_file.display()),
            format!("sed -n 1,2p {} | grep NEEDLE | wc -l", sed_file.display()),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | head -n 1",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | tail -n 1",
                sed_file.display()
            ),
            format!("sed -n 1,2p {} | grep NEEDLE | sort", sed_file.display()),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | sort | uniq",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | sort | wc -l",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | sort | head -n 1",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | sort | tail -n 1",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep NEEDLE | xargs echo",
                sed_file.display()
            ),
            format!(
                "sed -n 1,2p {} | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            "cut -d, -f1 | wc -l".to_string(),
            "cut -d, -f1 | grep a | xargs echo".to_string(),
            "echo Alpha | tr '[:lower:]' '[:upper:]'".to_string(),
            "printf '%s\\n' Alpha | tr '[:upper:]' '[:lower:]'".to_string(),
            format!("cat {} | sed -n 1,2p", sed_file.display()),
            format!("cat {} | sed -n 1,2p | wc -l", sed_file.display()),
            format!("cat {} | sed -n 1,2p | head -n 1", sed_file.display()),
            format!("cat {} | sed -n 1,2p | tail -n 1", sed_file.display()),
            format!("cat {} | sed -n 1,2p | sort", sed_file.display()),
            format!("cat {} | sed -n 1,2p | sort | uniq", sed_file.display()),
            format!(
                "cat {} | sed -n 1,2p | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | grep NEEDLE",
                sed_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | grep NEEDLE | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | grep NEEDLE | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | grep NEEDLE | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | sed -n 1,2p | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cut -d, -f1 {} | wc -l", cut_file.display()),
            format!("cut -d, -f1 {} | head -n 1", cut_file.display()),
            format!("cut -d, -f1 {} | tail -n 1", cut_file.display()),
            format!("cut -d, -f1 {} | sort", cut_file.display()),
            format!("cut -d, -f1 {} | sort | uniq", cut_file.display()),
            format!("cut -d, -f1 {} | sort | uniq | wc -l", cut_file.display()),
            format!("cut -d, -f1 {} | sort | wc -l", cut_file.display()),
            format!("cut -d, -f1 {} | sort | head -n 1", cut_file.display()),
            format!("cut -d, -f1 {} | sort | tail -n 1", cut_file.display()),
            format!("cut -d, -f1 {} | sort | xargs echo", cut_file.display()),
            format!(
                "cut -d, -f1 {} | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cut -d, -f1 {} | xargs echo", cut_file.display()),
            format!("cut -d, -f1 {} | xargs wc -l", xargs_wc_file.display()),
            format!("cut -d, -f1 {} | grep a", cut_file.display()),
            format!("cut -d, -f1 {} | grep a | wc -l", cut_file.display()),
            format!("cut -d, -f1 {} | grep a | head -n 1", cut_file.display()),
            format!("cut -d, -f1 {} | grep a | tail -n 1", cut_file.display()),
            format!("cut -d, -f1 {} | grep a | sort", cut_file.display()),
            format!("cut -d, -f1 {} | grep a | sort | uniq", cut_file.display()),
            format!(
                "cut -d, -f1 {} | grep a | sort | uniq | wc -l",
                cut_file.display()
            ),
            format!("cut -d, -f1 {} | grep a | sort | wc -l", cut_file.display()),
            format!(
                "cut -d, -f1 {} | grep a | sort | head -n 1",
                cut_file.display()
            ),
            format!(
                "cut -d, -f1 {} | grep a | sort | tail -n 1",
                cut_file.display()
            ),
            format!(
                "cut -d, -f1 {} | grep a | sort | xargs echo",
                cut_file.display()
            ),
            format!(
                "cut -d, -f1 {} | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cut -d, -f1 {} | grep a | xargs echo", cut_file.display()),
            format!(
                "cut -d, -f1 {} | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            "echo alpha beta | tr a-z A-Z".to_string(),
            "echo alpha beta | xargs echo".to_string(),
            "echo alpha beta | xargs -n 1 echo".to_string(),
            "echo alpha beta gamma delta | xargs -n 2 echo".to_string(),
            format!("echo {} {} | xargs wc -l", wc_a.display(), wc_b.display()),
            "xargs | wc -l".to_string(),
            "xargs echo | wc -l".to_string(),
            "xargs -n 1 echo | wc -l".to_string(),
            "xargs -n 2 echo | wc -l".to_string(),
            "xargs -n1 echo | grep NEEDLE | head -n 1".to_string(),
            "xargs -n2 echo | grep NEEDLE | head -n 1".to_string(),
            "xargs echo | head -n 1".to_string(),
            "printf '%s\\n' alpha beta gamma | xargs echo".to_string(),
            "printf '%s\\n' alpha beta gamma | xargs -n 1 echo".to_string(),
            "printf '%s\\n' alpha beta gamma delta | xargs -n 2 echo".to_string(),
            format!(
                "printf '%s\\n' {} {} | xargs wc -l",
                wc_a.display(),
                wc_b.display()
            ),
            "seq 1 10 | xargs echo".to_string(),
            "seq 1 10 | wc -c".to_string(),
            "seq 1 10 | wc -w".to_string(),
            format!("head -n 2 {} | wc -l", sort_file.display()),
            format!("head -n 2 {} | wc -c", sort_file.display()),
            format!("head -n 2 {} | wc -w", sort_file.display()),
            "head -n 2 | wc -l".to_string(),
            format!("head -n 2 {} | head -n 1", sort_file.display()),
            format!("head -n 2 {} | tail -n 1", sort_file.display()),
            format!("head -n 2 {} | sort", sort_file.display()),
            format!("head -n 3 {} | sort | uniq", sort_file.display()),
            format!("head -n 3 {} | sort | uniq | wc -l", sort_file.display()),
            format!("head -n 2 {} | sort | wc -l", sort_file.display()),
            format!("head -n 2 {} | sort | head -n 1", sort_file.display()),
            format!("head -n 2 {} | sort | tail -n 1", sort_file.display()),
            format!("head -n 2 {} | sort | xargs echo", sort_file.display()),
            format!("head -n 2 {} | sort | xargs wc -l", xargs_wc_file.display()),
            format!("head -n 2 {} | xargs echo", sort_file.display()),
            format!("head -n 2 {} | xargs wc -l", xargs_wc_file.display()),
            format!("head -n 2 {} | grep z", sort_file.display()),
            format!("head -n 2 {} | grep z | wc -l", sort_file.display()),
            format!("head -n 2 {} | grep z | head -n 1", sort_file.display()),
            format!("head -n 2 {} | grep z | tail -n 1", sort_file.display()),
            format!("head -n 2 {} | grep z | sort", sort_file.display()),
            format!("head -n 3 {} | grep z | sort | uniq", sort_file.display()),
            format!(
                "head -n 3 {} | grep z | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!(
                "head -n 3 {} | grep z | sort | uniq | wc -w",
                sort_file.display()
            ),
            format!("head -n 2 {} | grep z | sort | wc -l", sort_file.display()),
            format!(
                "head -n 2 {} | grep z | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "head -n 2 {} | grep z | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "head -n 2 {} | grep z | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "head -n 2 {} | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("head -n 2 {} | grep z | xargs echo", sort_file.display()),
            format!(
                "head -n 2 {} | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("tail -n 2 {} | wc -l", sort_file.display()),
            "tail -n 2 | wc -l".to_string(),
            format!("tail -n 2 {} | head -n 1", sort_file.display()),
            format!("tail -n 2 {} | tail -n 1", sort_file.display()),
            format!("tail -n 0 {} | wc -l", sort_file.display()),
            format!("tail -n 2 {} | sort", sort_file.display()),
            format!("tail -n 3 {} | sort | uniq", sort_file.display()),
            format!("tail -n 3 {} | sort | uniq | wc -l", sort_file.display()),
            format!("tail -n 2 {} | sort | wc -l", sort_file.display()),
            format!("tail -n 2 {} | sort | head -n 1", sort_file.display()),
            format!("tail -n 2 {} | sort | tail -n 1", sort_file.display()),
            format!("tail -n 2 {} | sort | xargs echo", sort_file.display()),
            format!("tail -n 2 {} | sort | xargs wc -l", xargs_wc_file.display()),
            format!("tail -n 2 {} | xargs echo", sort_file.display()),
            format!("tail -n 2 {} | xargs wc -l", xargs_wc_file.display()),
            format!("tail -n 2 {} | grep z", sort_file.display()),
            format!("tail -n 2 {} | grep z | wc -l", sort_file.display()),
            format!("tail -n 2 {} | grep z | head -n 1", sort_file.display()),
            format!("tail -n 2 {} | grep z | tail -n 1", sort_file.display()),
            format!("tail -n 2 {} | grep z | sort", sort_file.display()),
            format!("tail -n 3 {} | grep z | sort | uniq", sort_file.display()),
            format!(
                "tail -n 3 {} | grep z | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!("tail -n 2 {} | grep z | sort | wc -l", sort_file.display()),
            format!(
                "tail -n 2 {} | grep z | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "tail -n 2 {} | grep z | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "tail -n 2 {} | grep z | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "tail -n 2 {} | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("tail -n 2 {} | grep z | xargs echo", sort_file.display()),
            format!(
                "tail -n 2 {} | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("ls -1 {} | wc -l", list_dir.display()),
            format!("ls -1 {} | head -n 1", list_dir.display()),
            format!("ls -1 {} | tail -n 1", list_dir.display()),
            format!("ls -1 {} | sort", list_dir.display()),
            format!("ls -1 {} | sort | uniq", list_dir.display()),
            format!("ls -1 {} | sort | uniq | wc -l", list_dir.display()),
            format!("ls -1 {} | sort | uniq | head -n 1", list_dir.display()),
            format!("ls -1 {} | sort | uniq | tail -n 1", list_dir.display()),
            format!("ls -1 {} | sort | uniq | sort", list_dir.display()),
            format!("ls -1 {} | sort | uniq | sort | uniq", list_dir.display()),
            format!(
                "ls -1 {} | sort | uniq | sort | uniq | wc -l",
                list_dir.display()
            ),
            format!("ls -1 {} | sort | uniq | sort | wc -l", list_dir.display()),
            format!(
                "ls -1 {} | sort | uniq | sort | head -n 1",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | sort | tail -n 1",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | sort | xargs echo",
                list_dir.display()
            ),
            format!("ls -1 {} | sort | uniq | xargs echo", list_dir.display()),
            format!("ls -1 {} | sort | uniq | grep entry", list_dir.display()),
            format!(
                "ls -1 {} | sort | uniq | grep entry | wc -l",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | head -n 1",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | tail -n 1",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | sort",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | sort | uniq",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | sort | uniq | wc -l",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | sort | wc -l",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | sort | head -n 1",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | sort | tail -n 1",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | sort | xargs echo",
                list_dir.display()
            ),
            format!(
                "ls -1 {} | sort | uniq | grep entry | xargs echo",
                list_dir.display()
            ),
            format!("ls -1 {} | sort | wc -l", list_dir.display()),
            format!("ls -1 {} | sort | head -n 1", list_dir.display()),
            format!("ls -1 {} | sort | tail -n 1", list_dir.display()),
            format!("ls -1 {} | sort | xargs echo", list_dir.display()),
            format!("ls -1 {} | grep entry", list_dir.display()),
            format!("ls -1 {} | grep entry | wc -l", list_dir.display()),
            format!("ls -1 {} | grep entry | head -n 1", list_dir.display()),
            format!("ls -1 {} | grep entry | tail -n 1", list_dir.display()),
            format!("ls -1 {} | grep entry | sort", list_dir.display()),
            format!(
                "ls -1 {} | grep entry | sort | uniq | wc -l",
                list_dir.display()
            ),
            format!("ls -1 {} | grep entry | xargs echo", list_dir.display()),
            format!(
                "ls -1 {} | grep entry | sort | xargs echo",
                list_dir.display()
            ),
            format!("ls -1 {} | xargs echo", list_dir.display()),
            format!("ls -a {} | wc -l", list_dir.display()),
            format!("ls -a {} | grep hidden | wc -l", list_dir.display()),
            format!("ls -a {} | sort | tail -n 1", list_dir.display()),
            format!("ls -a {} | xargs echo", list_dir.display()),
            format!("ls -a {} | sort | xargs echo", list_dir.display()),
            format!("ls -A {} | wc -l", list_dir.display()),
            format!("ls -A {} | grep hidden | wc -l", list_dir.display()),
            format!("ls -A {} | sort | tail -n 1", list_dir.display()),
            format!("ls -A {} | xargs echo", list_dir.display()),
            format!("ls -A {} | sort | xargs echo", list_dir.display()),
            format!("sort {} | uniq", sort_file.display()),
            format!("sort {} | uniq | wc -l", sort_file.display()),
            format!("sort {} | uniq | wc -c", sort_file.display()),
            format!("sort {} | uniq | wc -w", sort_file.display()),
            format!("sort {} | uniq | head -n 1", sort_file.display()),
            format!("sort {} | uniq | tail -n 1", sort_file.display()),
            format!("sort {} | uniq | sort", sort_file.display()),
            format!("sort {} | uniq | sort | uniq", sort_file.display()),
            format!("sort {} | uniq | sort | uniq | wc -l", sort_file.display()),
            format!("sort {} | uniq | sort | wc -l", sort_file.display()),
            format!("sort {} | uniq | sort | head -n 1", sort_file.display()),
            format!("sort {} | uniq | sort | tail -n 1", sort_file.display()),
            format!("sort {} | uniq | sort | xargs echo", sort_file.display()),
            format!(
                "sort {} | uniq | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("sort {} | uniq | xargs echo", sort_file.display()),
            format!("sort {} | uniq | xargs wc -l", xargs_wc_file.display()),
            format!("sort {} | uniq | grep same", sort_file.display()),
            format!("sort {} | uniq | grep same | wc -l", sort_file.display()),
            format!(
                "sort {} | uniq | grep same | head -n 1",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep same | tail -n 1",
                sort_file.display()
            ),
            format!("sort {} | uniq | grep same | sort", sort_file.display()),
            format!(
                "sort {} | uniq | grep same | sort | uniq",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep same | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep same | sort | wc -l",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep same | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep same | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep same | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "sort {} | uniq | grep same | xargs echo",
                sort_file.display()
            ),
            format!(
                "sort {} | uniq | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("sort {} | grep same", sort_file.display()),
            format!("sort {} | grep same | wc -l", sort_file.display()),
            format!("sort {} | grep same | head -n 1", sort_file.display()),
            format!("sort {} | grep same | tail -n 1", sort_file.display()),
            format!("sort {} | grep same | sort", sort_file.display()),
            format!("sort {} | grep same | sort | uniq", sort_file.display()),
            format!(
                "sort {} | grep same | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!(
                "sort {} | grep same | sort | wc -l",
                sort_file.display()
            ),
            format!(
                "sort {} | grep same | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "sort {} | grep same | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "sort {} | grep same | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "sort {} | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("sort {} | grep same | xargs echo", sort_file.display()),
            format!(
                "sort {} | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("sort {} | head -n 1", sort_file.display()),
            format!("sort {} | tail -n 1", sort_file.display()),
            format!("sort {} | wc -l", sort_file.display()),
            format!("sort {} | wc -c", sort_file.display()),
            format!("sort {} | wc -w", sort_file.display()),
            format!("cat {} | wc -l", sed_file.display()),
            format!("cat {} | wc -c", sed_file.display()),
            format!("cat {} | wc -w", sed_file.display()),
            format!("cat {} | head", sed_file.display()),
            format!("cat {} | head -n 1", sed_file.display()),
            format!("cat {} | head -1", sed_file.display()),
            format!("cat {} | head -n 2 | wc -l", sed_file.display()),
            format!("cat {} | head -n 2 | sort", sed_file.display()),
            format!("cat {} | head -n 2 | sort | uniq | wc -l", sed_file.display()),
            format!(
                "cat {} | head -n 2 | grep NEEDLE | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | head -n 2 | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | tail", sed_file.display()),
            format!("cat {} | tail -n 1", sed_file.display()),
            format!("cat {} | tail -1", sed_file.display()),
            format!("cat {} | tail -n 2 | wc -l", sed_file.display()),
            format!("cat {} | tail -n 2 | sort", sed_file.display()),
            format!("cat {} | tail -n 2 | sort | uniq | wc -l", sed_file.display()),
            format!(
                "cat {} | tail -n 2 | grep NEEDLE | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | tail -n 2 | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | grep NEEDLE", sed_file.display()),
            format!("cat {} | grep NEEDLE | wc -l", sed_file.display()),
            format!("cat {} | grep NEEDLE | head -n 1", sed_file.display()),
            format!("cat {} | grep NEEDLE | tail -n 1", sed_file.display()),
            format!("cat {} | grep e | sort", sort_file.display()),
            format!("cat {} | grep e | sort | uniq", sort_file.display()),
            format!(
                "cat {} | grep e | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | grep e | sort | uniq | head -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | grep e | sort | uniq | tail -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | grep e | sort | uniq | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | grep wc | sort | uniq | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | grep wc | sort | uniq | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | grep e | sort | wc -l", sort_file.display()),
            format!("cat {} | grep e | sort | head -n 1", sort_file.display()),
            format!("cat {} | grep e | sort | tail -n 1", sort_file.display()),
            format!("cat {} | cut -d, -f1", cut_file.display()),
            format!("cat {} | cut -d, -f1 | wc -l", cut_file.display()),
            format!("cat {} | cut -d, -f1 | head -n 1", cut_file.display()),
            format!("cat {} | cut -d, -f1 | tail -n 1", cut_file.display()),
            format!("cat {} | cut -d, -f1 | sort", cut_file.display()),
            format!("cat {} | cut -d, -f1 | sort | uniq", cut_file.display()),
            format!(
                "cat {} | cut -d, -f1 | sort | uniq | wc -l",
                cut_file.display()
            ),
            format!("cat {} | cut -d, -f1 | sort | wc -l", cut_file.display()),
            format!(
                "cat {} | cut -d, -f1 | sort | head -n 1",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | sort | tail -n 1",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | sort | xargs echo",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | cut -d, -f1 | xargs echo", cut_file.display()),
            format!(
                "cat {} | cut -d, -f1 | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | cut -d, -f1 | grep a", cut_file.display()),
            format!("cat {} | cut -d, -f1 | grep a | wc -l", cut_file.display()),
            format!(
                "cat {} | cut -d, -f1 | grep a | head -n 1",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep a | tail -n 1",
                cut_file.display()
            ),
            format!("cat {} | cut -d, -f1 | grep a | sort", cut_file.display()),
            format!(
                "cat {} | cut -d, -f1 | grep a | sort | uniq",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep a | sort | uniq | wc -l",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep a | sort | wc -l",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep a | sort | head -n 1",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep a | sort | tail -n 1",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep a | sort | xargs echo",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep a | xargs echo",
                cut_file.display()
            ),
            format!(
                "cat {} | cut -d, -f1 | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | tr a-z A-Z", sed_file.display()),
            format!("cat {} | tr a-z A-Z | wc -l", sed_file.display()),
            format!("cat {} | tr a-z A-Z | head -n 1", sed_file.display()),
            format!("cat {} | tr a-z A-Z | tail -n 1", sed_file.display()),
            format!("cat {} | tr a-z A-Z | sort", sed_file.display()),
            format!("cat {} | tr a-z A-Z | sort | uniq", sed_file.display()),
            format!(
                "cat {} | tr a-z A-Z | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!("cat {} | tr a-z A-Z | sort | wc -l", sed_file.display()),
            format!("cat {} | tr a-z A-Z | sort | head -n 1", sed_file.display()),
            format!("cat {} | tr a-z A-Z | sort | tail -n 1", sed_file.display()),
            format!(
                "cat {} | tr a-z A-Z | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z a-z | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | tr a-z A-Z | xargs echo", sed_file.display()),
            format!("cat {} | tr a-z a-z | xargs wc -l", xargs_wc_file.display()),
            format!("cat {} | tr a-z A-Z | grep NEEDLE", sed_file.display()),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | head -n 1",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | tail -n 1",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | sort",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | sort | uniq",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | sort | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | sort | head -n 1",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | sort | tail -n 1",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z a-z | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | tr a-z A-Z | grep NEEDLE | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | tr a-z a-z | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | uniq", sort_file.display()),
            format!("cat {} | uniq | wc -l", sort_file.display()),
            format!("cat {} | uniq | head -n 1", sort_file.display()),
            format!("cat {} | uniq | tail -n 1", sort_file.display()),
            format!("cat {} | uniq | sort", sort_file.display()),
            format!("cat {} | uniq | sort | uniq", sort_file.display()),
            format!("cat {} | uniq | sort | uniq | wc -l", sort_file.display()),
            format!("cat {} | uniq | sort | wc -l", sort_file.display()),
            format!("cat {} | uniq | sort | head -n 1", sort_file.display()),
            format!("cat {} | uniq | sort | tail -n 1", sort_file.display()),
            format!("cat {} | uniq | sort | xargs echo", sort_file.display()),
            format!(
                "cat {} | uniq | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | uniq | xargs echo", sort_file.display()),
            format!("cat {} | uniq | xargs wc -l", xargs_wc_file.display()),
            format!("cat {} | uniq | grep same", sort_file.display()),
            format!("cat {} | uniq | grep same | wc -l", sort_file.display()),
            format!("cat {} | uniq | grep same | head -n 1", sort_file.display()),
            format!("cat {} | uniq | grep same | tail -n 1", sort_file.display()),
            format!("cat {} | uniq | grep same | sort", sort_file.display()),
            format!(
                "cat {} | uniq | grep same | sort | uniq",
                sort_file.display()
            ),
            format!(
                "cat {} | uniq | grep same | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | uniq | grep same | sort | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | uniq | grep same | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | uniq | grep same | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | uniq | grep same | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | uniq | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | uniq | grep same | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | uniq | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("uniq {} | wc -l", sort_file.display()),
            format!("uniq {} | head -n 1", sort_file.display()),
            format!("uniq {} | tail -n 1", sort_file.display()),
            format!("uniq {} | sort", sort_file.display()),
            format!("uniq {} | sort | uniq", sort_file.display()),
            format!("uniq {} | sort | uniq | wc -l", sort_file.display()),
            format!("uniq {} | sort | wc -l", sort_file.display()),
            format!("uniq {} | sort | head -n 1", sort_file.display()),
            format!("uniq {} | sort | tail -n 1", sort_file.display()),
            format!("uniq {} | sort | xargs echo", sort_file.display()),
            format!("uniq {} | sort | xargs wc -l", xargs_wc_file.display()),
            format!("uniq {} | xargs echo", sort_file.display()),
            format!("uniq {} | xargs wc -l", xargs_wc_file.display()),
            format!("uniq {} | grep same", sort_file.display()),
            format!("uniq {} | grep same | wc -l", sort_file.display()),
            format!("uniq {} | grep same | head -n 1", sort_file.display()),
            format!("uniq {} | grep same | tail -n 1", sort_file.display()),
            format!("uniq {} | grep same | sort", sort_file.display()),
            format!("uniq {} | grep same | sort | uniq", sort_file.display()),
            format!(
                "uniq {} | grep same | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!("uniq {} | grep same | sort | wc -l", sort_file.display()),
            format!(
                "uniq {} | grep same | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "uniq {} | grep same | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "uniq {} | grep same | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "uniq {} | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("uniq {} | grep same | xargs echo", sort_file.display()),
            format!(
                "uniq {} | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | sort", sort_file.display()),
            format!("cat {} | sort | uniq", sort_file.display()),
            format!("cat {} | sort | uniq | wc -l", sort_file.display()),
            format!("cat {} | sort | uniq | head -n 1", sort_file.display()),
            format!("cat {} | sort | uniq | tail -n 1", sort_file.display()),
            format!("cat {} | sort | uniq | sort", sort_file.display()),
            format!("cat {} | sort | uniq | sort | uniq", sort_file.display()),
            format!(
                "cat {} | sort | uniq | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!("cat {} | sort | uniq | sort | wc -l", sort_file.display()),
            format!(
                "cat {} | sort | uniq | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | sort | uniq | xargs echo", sort_file.display()),
            format!(
                "cat {} | sort | uniq | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | sort | uniq | grep same", sort_file.display()),
            format!(
                "cat {} | sort | uniq | grep same | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | head -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | tail -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | sort",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | sort | uniq",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | sort | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep same | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | uniq | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | sort | grep same", sort_file.display()),
            format!(
                "cat {} | sort | grep same | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | head -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | tail -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | sort",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | sort | uniq",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | sort | uniq | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | sort | wc -l",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | sort | head -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | sort | tail -n 1",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep same | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep wc | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | sort | grep same | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | sort | grep wc | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("cat {} | sort | wc -l", sort_file.display()),
            format!("cat {} | sort | wc -c", sort_file.display()),
            format!("cat {} | sort | wc -w", sort_file.display()),
            format!("cat {} | sort | head -n 1", sort_file.display()),
            format!("cat {} | sort | tail -n 1", sort_file.display()),
            format!("cat {} | xargs echo", sort_file.display()),
            format!("sort {} | xargs echo", sort_file.display()),
            format!("cat {} | sort | xargs echo", sort_file.display()),
            format!("cat {} | xargs wc -l", xargs_wc_file.display()),
            format!("cat {} | xargs wc -l | sort", xargs_wc_file.display()),
            format!("sort {} | xargs wc -l", xargs_wc_file.display()),
            format!(
                "sort {} | xargs wc -l | sort | tail -n 1",
                xargs_wc_file.display()
            ),
            format!("cat {} | sort | xargs wc -l", xargs_wc_file.display()),
            format!(
                "cat {} | sort | xargs wc -l | sort",
                xargs_wc_file.display()
            ),
            format!("cat {} | grep same | xargs echo", sort_file.display()),
            format!("cat {} | grep wc-a | xargs wc -l", xargs_wc_file.display()),
            format!(
                "cat {} | grep same | sort | xargs echo",
                sort_file.display()
            ),
            format!(
                "cat {} | grep wc-a | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("grep -R NEEDLE {} | head -n 1", grep_root.display()),
            format!("grep -R NEEDLE {} | tail -n 1", grep_root.display()),
            format!("grep -R NEEDLE {} | sort", grep_root.display()),
            format!("grep -R NEEDLE {} | sort | uniq", grep_root.display()),
            format!(
                "grep -R NEEDLE {} | sort | uniq | wc -l",
                grep_root.display()
            ),
            format!(
                "grep -R NEEDLE {} | sort | uniq | head -n 1",
                grep_root.display()
            ),
            format!(
                "grep -R NEEDLE {} | sort | uniq | tail -n 1",
                grep_root.display()
            ),
            format!(
                "grep -R NEEDLE {} | sort | uniq | sort | xargs echo",
                grep_root.display()
            ),
            format!("grep -R NEEDLE {} | sort | wc -l", grep_root.display()),
            format!("grep -R NEEDLE {} | sort | head -n 1", grep_root.display()),
            format!("grep -R NEEDLE {} | sort | tail -n 1", grep_root.display()),
            format!("grep -R NEEDLE {} | wc -l", grep_root.display()),
            format!("grep NEEDLE {} | wc -l", grep_file.display()),
            format!("grep NEEDLE {} | head -n 1", grep_file.display()),
            format!("grep NEEDLE {} | tail -n 1", grep_file.display()),
            format!("grep NEEDLE {} | sort", grep_file.display()),
            format!("grep NEEDLE {} | sort | uniq", grep_file.display()),
            format!("grep NEEDLE {} | sort | uniq | wc -l", grep_file.display()),
            format!("grep NEEDLE {} | sort | uniq | head -n 1", grep_file.display()),
            format!("grep NEEDLE {} | sort | uniq | tail -n 1", grep_file.display()),
            format!(
                "grep NEEDLE {} | sort | uniq | sort | xargs echo",
                grep_file.display()
            ),
            format!(
                "grep wc {} | sort | uniq | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "grep wc {} | sort | uniq | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("grep NEEDLE {} | sort | wc -l", grep_file.display()),
            format!("grep NEEDLE {} | sort | head -n 1", grep_file.display()),
            format!("grep NEEDLE {} | sort | tail -n 1", grep_file.display()),
            format!("grep NEEDLE {} | xargs echo", grep_file.display()),
            format!("grep count- {} | xargs wc -l", xargs_wc_file.display()),
            format!(
                "grep count- {} | xargs wc -l | sort",
                xargs_wc_file.display()
            ),
            format!("grep NEEDLE {} | sort | xargs echo", grep_file.display()),
            format!(
                "grep count- {} | sort | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "grep count- {} | sort | xargs wc -l | sort | tail -n 1",
                xargs_wc_file.display()
            ),
            format!("grep NEEDLE {} | cut -d ' ' -f1", grep_file.display()),
            format!(
                "grep NEEDLE {} | cut -d ' ' -f1 | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | cut -d ' ' -f1 | sort",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | cut -d ' ' -f1 | sort | uniq | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | cut -d ' ' -f1 | grep NEEDLE | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | cut -d ' ' -f1 | xargs echo",
                grep_file.display()
            ),
            format!(
                "grep count- {} | cut -d ' ' -f1 | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("grep NEEDLE {} | awk '{{ print $1 }}'", grep_file.display()),
            format!(
                "grep NEEDLE {} | awk '/NEEDLE/ {{ print $1 }}'",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | awk '{{ print $1 }}' | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | awk '{{ print $2 }}' | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | awk '{{print$1}}' | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | awk '{{ print $1 }}' | sort",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | awk '{{ print $1 }}' | sort | uniq | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | awk '{{ print $1 }}' | grep NEEDLE | wc -l",
                grep_file.display()
            ),
            format!(
                "grep NEEDLE {} | awk '{{ print $1 }}' | xargs echo",
                grep_file.display()
            ),
            format!(
                "grep count- {} | awk '{{ print $1 }}' | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!("awk '{{ print $1 }}' {} | wc -l", sed_file.display()),
            format!("awk '{{ print $2 }}' {} | wc -l", sed_file.display()),
            format!("awk '{{print $1}}' {} | wc -l", sed_file.display()),
            format!("awk '{{print $2}}' {} | wc -l", sed_file.display()),
            format!(
                "awk '{{ print $1 }}' {} | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "awk '{{ print $1 }}' {} | xargs echo",
                sed_file.display()
            ),
            format!(
                "awk '{{ print $1 }}' {} | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "awk '{{ print $1 }}' {} | xargs wc -l | sort",
                xargs_wc_file.display()
            ),
            format!("awk '{{ print $1 }}' {} | grep line", sed_file.display()),
            format!(
                "awk '{{ print $1 }}' {} | grep line | wc -l",
                sed_file.display()
            ),
            format!(
                "awk '{{ print $1 }}' {} | grep line | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "awk '{{ print $1 }}' {} | grep count- | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "awk '{{ print $1 }}' {} | grep count- | xargs wc -l | sort",
                awk_xargs_wc_file.display()
            ),
            format!("cat {} | awk '{{ print $1 }}'", sed_file.display()),
            format!("cat {} | awk '{{ print $2 }}'", sed_file.display()),
            format!("cat {} | awk '{{print$1}}'", sed_file.display()),
            format!(
                "cat {} | awk '{{ print $1 }}' | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '{{ print $1 }}' | xargs wc -l",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | awk '{{ print $1 }}' | xargs wc -l | sort",
                xargs_wc_file.display()
            ),
            format!(
                "cat {} | awk '{{ print $1 }}' | grep line | tail -n 1",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '{{ print $1 }}' | grep count- | sort | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "cat {} | awk '{{ print $1 }}' | grep count- | sort | xargs wc -l | sort | tail -n 1",
                awk_xargs_wc_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | wc -l",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | head -n 1",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | tail -n 1",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | head -n 1",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | tail -n 1",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | xargs echo",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | sort | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | wc -l",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | head -n 1",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | tail -n 1",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | xargs echo",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | xargs wc -l | sort",
                awk_xargs_wc_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs wc -l | sort | tail -n 1",
                awk_xargs_wc_file.display()
            ),
            format!("cat {} | awk '/NEEDLE/ {{ print $1 }}'", sed_file.display()),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | head -n 1",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | tail -n 1",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | uniq",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | uniq | wc -l",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | uniq | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | xargs wc -l | sort",
                awk_xargs_wc_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | xargs echo",
                sed_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | xargs wc -l",
                awk_xargs_wc_file.display()
            ),
            format!(
                "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | xargs wc -l | sort | tail -n 1",
                awk_xargs_wc_file.display()
            ),
            format!("find {} -type f | xargs wc -l", find_root.display()),
            format!("find {} -type f | xargs wc -l | wc -l", find_root.display()),
            format!("find {} -type f | xargs wc -l | head -n 1", find_root.display()),
            format!("find {} -type f | xargs wc -l | tail -n 1", find_root.display()),
            format!("find {} -type f | xargs wc -l | sort", find_root.display()),
            format!(
                "find {} -type f | xargs wc -l | sort | uniq",
                find_root.display()
            ),
            format!(
                "find {} -type f | xargs wc -l | sort | uniq | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | xargs wc -l | sort | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | xargs wc -l | sort | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | xargs wc -l | sort | tail -n 1",
                find_root.display()
            ),
            format!("find {} -type f | xargs echo", find_root.display()),
            format!("find {} -type f | xargs", find_root.display()),
            format!("find {} -type f | wc -l", find_root.display()),
            format!("find {} -type f | head -n 1", find_root.display()),
            format!("find {} -type f | tail -n 1", find_root.display()),
            format!("find {} -type f | sort", find_root.display()),
            format!("find {} -type f | sort | uniq", find_root.display()),
            format!("find {} -type f | sort | uniq | wc -l", find_root.display()),
            format!(
                "find {} -type f | sort | uniq | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | tail -n 1",
                find_root.display()
            ),
            format!("find {} -type f | sort | uniq | sort", find_root.display()),
            format!(
                "find {} -type f | sort | uniq | sort | uniq",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | sort | uniq | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | sort | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | sort | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | sort | tail -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | sort | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | sort | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | xargs wc -l | sort",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | tail -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort | uniq",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort | uniq | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort | tail -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | sort | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | uniq | grep entry | xargs wc -l",
                find_root.display()
            ),
            format!("find {} -type f | sort | wc -l", find_root.display()),
            format!("find {} -type f | sort | xargs echo", find_root.display()),
            format!("find {} -type f | sort | xargs wc -l", find_root.display()),
            format!(
                "find {} -type f | sort | xargs wc -l | sort",
                find_root.display()
            ),
            format!(
                "find {} -type f | sort | xargs wc -l | sort | tail -n 1",
                find_root.display()
            ),
            format!("find {} -type f | sort | head -n 1", find_root.display()),
            format!("find {} -type f | sort | tail -n 1", find_root.display()),
            format!(
                "find {} -maxdepth 1 -type f | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -maxdepth 1 -type f | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -maxdepth 1 -type f | grep entry | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -maxdepth 1 -type f | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -maxdepth 2 -type f | sort | tail -n 1",
                find_root.display()
            ),
            format!(
                "find {} -maxdepth 2 -type f -name '*.rs' | grep entry | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | xargs wc -l | sort",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | xargs echo",
                find_root.display()
            ),
            format!("find {} -type f -name '*.rs' | xargs", find_root.display()),
            format!(
                "find {} -type f -name '*.rs' | grep entry | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | xargs wc -l | sort",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | tail -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | sort",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | sort | uniq | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | sort | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | sort | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | sort | xargs wc -l | sort | tail -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | grep entry | sort | uniq | xargs wc -l | sort | tail -n 1",
                find_root.display()
            ),
            format!("find {} -type f -name '*.rs' | wc -l", find_root.display()),
            format!(
                "find {} -type f -name '*.rs' | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | tail -n 1",
                find_root.display()
            ),
            format!("find {} -type f -name '*.rs' | sort", find_root.display()),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | tail -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | sort",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | grep entry",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | grep entry | sort | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | uniq | grep entry | sort | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | xargs echo",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | xargs wc -l",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | head -n 1",
                find_root.display()
            ),
            format!(
                "find {} -type f -name '*.rs' | sort | tail -n 1",
                find_root.display()
            ),
        ] {
            assert!(
                matches!(plan_shell(&command, None), CommandPlan::Native(_)),
                "expected native pipe plan for {command}"
            );
        }
    }

    #[test]
    fn grep_uses_rg_replacement_when_rg_exists() {
        let tmp = tempdir().unwrap();
        let CommandPlan::External(plan) = plan_with_tool_resolver(
            &s(&["grep", "-R", "TODO", tmp.path().to_str().unwrap()]),
            None,
            |tool| tool == "rg",
        ) else {
            panic!("expected replacement plan");
        };
        assert_eq!(plan.implementation, ExternalImplementation::Replacement);
    }

    #[test]
    fn shell_free_safe_subsets_plan_native() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("a.txt");
        fs::write(&file, "one\ntwo\nthree\n").unwrap();
        let list_dir = tmp.path().join("list");
        fs::create_dir(&list_dir).unwrap();
        fs::write(list_dir.join("entry"), "").unwrap();
        let find_dir = tmp.path().join("find");
        fs::create_dir(&find_dir).unwrap();
        fs::write(find_dir.join("file.txt"), "").unwrap();
        fs::write(find_dir.join("file.rs"), "").unwrap();
        let sort_file = tmp.path().join("sort.txt");
        fs::write(&sort_file, "z\na\n").unwrap();
        let cut_file = tmp.path().join("cut.csv");
        fs::write(&cut_file, "alpha,beta\nplain\ngamma,delta\n").unwrap();
        let cut_tab_file = tmp.path().join("cut.tsv");
        fs::write(&cut_tab_file, "alpha\tbeta\nplain\ngamma\tdelta\n").unwrap();
        let sed_file = tmp.path().join("sed.txt");
        fs::write(&sed_file, "one\ntwo\nthree\n").unwrap();
        let wc_dir = tmp.path().join("wc");
        fs::create_dir(&wc_dir).unwrap();
        let mut wc_files = Vec::new();
        for idx in 0..2 {
            let file = wc_dir.join(format!("wc-{idx:04}.txt"));
            fs::write(&file, "one\ntwo\n").unwrap();
            wc_files.push(file);
        }

        assert!(matches!(
            plan_without_tools(&["ls", list_dir.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Ls(_),
                ..
            })
        ));

        assert!(matches!(
            plan_without_tools(&["sort", sort_file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Sort(_),
                ..
            })
        ));

        assert!(matches!(
            plan_without_tools(&["cat", file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Cat(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&["find", find_dir.to_str().unwrap(), "-type", "f"]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Find(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&["sed", "-n", "1,2p", sed_file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::SedPrint(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&["grep", "two", file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::GrepFile(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&["awk", "{ print $1 }", sed_file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::AwkFirstField(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&["awk", "{ print $2 }", sed_file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::AwkFirstField(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&["awk", "{print$1}", sed_file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::AwkFirstField(_),
                ..
            })
        ));
        let mkdir_path = tmp.path().join("new-dir").to_string_lossy().to_string();
        let touch_path = tmp.path().join("new-file").to_string_lossy().to_string();
        let missing_path = tmp.path().join("missing").to_string_lossy().to_string();
        for args in [
            vec!["true"],
            vec!["false"],
            vec!["pwd"],
            vec!["echo", "alpha", "beta"],
            vec!["echo", "-n", "alpha"],
            vec!["printf", "alpha\\nbeta\\n"],
            vec!["printf", "%s", "alpha", "beta"],
            vec!["printf", "%s\\n", "alpha", "beta"],
            vec!["seq", "1", "3"],
            vec!["seq", "3", "-1", "1"],
            vec!["whoami"],
            vec!["id"],
            vec!["id", "-u"],
            vec!["id", "-un"],
            vec!["id", "-g"],
            vec!["id", "-gn"],
            vec!["id", "-G"],
            vec!["id", "-Gn"],
            vec!["uname"],
            vec!["uname", "-a"],
            vec!["uname", "-m"],
            vec!["uname", "-p"],
            vec!["hostname"],
            vec!["test", "-f", file.to_str().unwrap()],
            vec!["test", "alpha", "=", "alpha"],
            vec!["test", "5", "-gt", "3"],
            vec!["test", "!", "-e", missing_path.as_str()],
            vec!["[", "-d", tmp.path().to_str().unwrap(), "]"],
            vec!["basename", "/tmp/cap/example.txt", ".txt"],
            vec!["dirname", "/tmp/cap/example.txt"],
            vec!["head"],
            vec!["head", "-n", "1"],
            vec!["head", "-c", "3"],
            vec!["head", "-n", "1", file.to_str().unwrap()],
            vec!["tail"],
            vec!["tail", "-n", "1"],
            vec!["tail", "-c", "3"],
            vec!["tail", "-c", "3", file.to_str().unwrap()],
            vec!["sort"],
            vec!["uniq"],
            vec!["cut", "-d,", "-f1"],
            vec!["cut", "-d,", "-f1", cut_file.to_str().unwrap()],
            vec!["cut", "-f", "2", cut_tab_file.to_str().unwrap()],
            vec!["tr", "a-z", "A-Z"],
            vec!["tr", "[:lower:]", "[:upper:]"],
            vec!["tr", "-d", "0-9"],
            vec!["tr", "-d", "[:digit:]"],
            vec!["wc"],
            vec!["wc", "-l"],
            vec!["wc", "-c"],
            vec!["wc", "-w"],
            vec!["awk", "{ print $1 }"],
            vec!["awk", "{ print $2 }"],
            vec!["awk", "/NEEDLE/ { print $1 }"],
            vec!["awk", "/NEEDLE/ { print $2 }"],
            vec!["awk", "/NEEDLE/ { c++ } END { print c }"],
            vec!["mkdir", "-p", mkdir_path.as_str()],
            vec!["touch", touch_path.as_str()],
            vec!["xargs"],
            vec!["xargs", "echo"],
            vec!["xargs", "-n", "2"],
            vec!["xargs", "-n2", "echo"],
            vec!["xargs", "wc", "-l"],
            vec!["which", "sh"],
            vec!["which", "-a", "sh"],
            vec!["command", "-v", "sh"],
            vec!["env"],
            vec!["printenv"],
            vec!["printenv", "PATH"],
        ] {
            assert!(
                matches!(plan_without_tools(&args), CommandPlan::Native(_)),
                "expected native for {args:?}"
            );
        }

        let mut wc_args = vec!["wc".to_string(), "-l".to_string()];
        wc_args.extend(
            wc_files
                .iter()
                .map(|file| file.to_string_lossy().to_string()),
        );
        assert!(matches!(
            plan(&wc_args, None),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::WcLines(_),
                ..
            })
        ));
        let mut wc_all_args = vec!["wc".to_string()];
        wc_all_args.extend(
            wc_files
                .iter()
                .map(|file| file.to_string_lossy().to_string()),
        );
        assert!(matches!(
            plan(&wc_all_args, None),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::WcAll(_),
                ..
            })
        ));
    }

    #[test]
    fn unsupported_options_keep_original_path() {
        let tmp = tempdir().unwrap();
        let wc_file = tmp.path().join("wc-small.txt");
        fs::write(&wc_file, "one\ntwo\n").unwrap();

        for args in [
            vec!["wc", "-m", wc_file.to_str().unwrap()],
            vec!["head", "-q", wc_file.to_str().unwrap()],
            vec!["head", "-n", "0", wc_file.to_str().unwrap()],
            vec!["head", "-c", "0", wc_file.to_str().unwrap()],
            vec!["touch", "-a", wc_file.to_str().unwrap()],
            vec!["echo", "-e", "one"],
            vec!["echo", "-n", "-e", "one"],
            vec!["printf", "%q\\n", "one"],
            vec!["seq", "1", "0", "3"],
            vec!["whoami", "--help"],
            vec!["id", "-P"],
            vec!["hostname", "-s"],
            vec!["hostname", "new-name"],
            vec!["grep", "t.o", wc_file.to_str().unwrap()],
            vec!["test", "a", "-a", "b"],
            vec!["test", "a", "-nt", "b"],
            vec!["[", "-f", wc_file.to_str().unwrap()],
            vec!["cut", "-c", "1", wc_file.to_str().unwrap()],
            vec!["cut", "-d", "::", "-f", "1", wc_file.to_str().unwrap()],
            vec!["cut", "-d,", "-f1,2", wc_file.to_str().unwrap()],
            vec!["tr", "[:space:]", " "],
            vec!["tr", "a-z", "A"],
            vec!["tr", "-s", "a"],
            vec!["which", "-s", "sh"],
            vec!["command", "-p", "-v", "sh"],
            vec!["command", "-v"],
            vec!["env", "-i"],
            vec!["env", "FOO=bar"],
            vec!["printenv", "-0"],
            vec!["printenv", "HOME", "PATH"],
        ] {
            let CommandPlan::External(plan) = plan_without_tools(&args) else {
                panic!("expected original fallback for {args:?}");
            };
            assert_eq!(plan.implementation, ExternalImplementation::Original);
        }

        let CommandPlan::External(plan) =
            plan_shell(&format!("cat {} | grep N.*D", wc_file.display()), None)
        else {
            panic!("expected regex-looking cat|grep fallback");
        };
        assert_eq!(plan.program, "bash");

        let CommandPlan::External(plan) = plan_shell("echo -n -e one | wc -l", None) else {
            panic!("expected option-sensitive echo pipe fallback");
        };
        assert_eq!(plan.program, "bash");

        let CommandPlan::External(plan) = plan_shell("yes -bad | head -n 3", None) else {
            panic!("expected option-looking yes pipe fallback");
        };
        assert_eq!(plan.program, "bash");

        let CommandPlan::External(plan) = plan_shell("yes READY SET | head -n 3", None) else {
            panic!("expected multi-operand yes pipe fallback");
        };
        assert_eq!(plan.program, "bash");

        let CommandPlan::External(plan) =
            plan_shell(&format!("sort {} | uniq -c", wc_file.display()), None)
        else {
            panic!("expected unsupported sort|uniq option fallback");
        };
        assert_eq!(plan.program, "bash");

        let CommandPlan::External(plan) =
            plan_shell(&format!("cat {} | cut -c 1", wc_file.display()), None)
        else {
            panic!("expected unsupported cat|cut option fallback");
        };
        assert_eq!(plan.program, "bash");

        for command in [
            "env | wc -l",
            "env | head -n 1",
            "env | tail -n 1",
            "env | grep PATH",
            "env | sort",
            "printenv | wc -l",
        ] {
            let CommandPlan::External(plan) = plan_shell(command, None) else {
                panic!("expected full-environment pipe fallback for {command}");
            };
            assert_eq!(plan.program, "bash");
        }

        let list_dir = tmp.path().join("list-dir");
        fs::create_dir(&list_dir).unwrap();
        fs::write(list_dir.join("entry-a.txt"), "").unwrap();
        fs::write(list_dir.join("entry-b.txt"), "").unwrap();

        for command in [
            format!("ls -1 {} | grep entry | xargs wc -l", list_dir.display()),
            format!(
                "ls -1 {} | grep entry | sort | xargs wc -l",
                list_dir.display()
            ),
        ] {
            let CommandPlan::External(plan) = plan_shell(&command, None) else {
                panic!("expected cwd-sensitive ls|grep|xargs wc fallback for {command}");
            };
            assert_eq!(plan.program, "bash");
        }

        let CommandPlan::External(plan) = plan_shell(
            &format!("cat {} | tr '[:space:]' ' '", wc_file.display()),
            None,
        ) else {
            panic!("expected unsupported cat|tr class fallback");
        };
        assert_eq!(plan.program, "bash");

        let plan = plan_shell(
            &format!(
                "find {} -type f -name '*.rs' | sort | uniq",
                tmp.path().display()
            ),
            None,
        );
        assert!(matches!(plan, CommandPlan::Native(_)));

        let CommandPlan::External(plan) = plan_shell(
            &format!(
                "find {} -type f -name '*.rs' | sort | uniq -c",
                tmp.path().display()
            ),
            None,
        ) else {
            panic!("expected unsupported find|sort|uniq option fallback");
        };
        assert_eq!(plan.program, "bash");

        let CommandPlan::External(plan) = plan_shell(
            &format!("sort {} | head -n 3", tmp.path().join("missing").display()),
            None,
        ) else {
            panic!("expected missing sort file pipe fallback");
        };
        assert_eq!(plan.program, "bash");

        let CommandPlan::External(plan) =
            plan_shell(&format!("[ -f {} ]", wc_file.display()), None)
        else {
            panic!("expected bracket shell string fallback");
        };
        assert_eq!(plan.program, "bash");
    }

    #[test]
    fn recursive_grep_can_use_replacement_when_rg_exists() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("file.txt"), "TODO\n").unwrap();

        let CommandPlan::External(plan) = plan_with_tool_resolver(
            &s(&["grep", "-R", "TODO", tmp.path().to_str().unwrap()]),
            None,
            |tool| tool == "rg",
        ) else {
            panic!("expected grep replacement");
        };
        assert_eq!(plan.implementation, ExternalImplementation::Replacement);
    }
}
// CODEGEN-END
