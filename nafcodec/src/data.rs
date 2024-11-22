//! Common data types for this crate.

// --- MaskUnit ----------------------------------------------------------------

use std::borrow::Cow;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::fmt;

/// A single masked unit with associated status decoded from the mask block.
#[derive(Debug, Clone, PartialEq)]
pub enum MaskUnit {
    Masked(u64),
    Unmasked(u64),
}

// --- Record ------------------------------------------------------------------

/// A single sequence record from a Nucleotide Archive Format file.
///
/// ## Quality
///
/// If set, the quality string length should be equal to the sequence
/// string length, and to the record length. Since the data is compressed
/// as raw text, it could contain other sort of annotation, such as RNA
/// secondary structure in dot-bracket notation, or protein secondary
/// structure.
///
#[derive(Debug, Clone, Default)]
pub struct Record<'a> {
    /// The record identifier (accession number).
    pub id: Option<Cow<'a, str>>,
    /// The record comment (description).
    pub comment: Option<Cow<'a, str>>,
    /// The record sequence.
    pub sequence: Option<Cow<'a, [u8]>>,
    /// The record quality string.
    pub quality: Option<Cow<'a, str>>,
    /// The record sequence length.
    pub length: Option<u64>,
}


// TODO: implement FASTA/FASTQ IO for Record

// --- FormatVersion -----------------------------------------------------------

/// The supported format versions inside NAF archives.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FormatVersion {
    #[default]
    V1 = 1,
    V2 = 2,
}

// --- SequenceType ------------------------------------------------------------

/// The type of sequence stored in a Nucleotide Archive Format file.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SequenceType {
    #[default]
    /// DNA Sequence - ATCG(N-)
    Dna = 0,
    /// RNA Sequence - AUCG(N-)
    Rna = 1,
    /// Protein Sequence - single character amino acid
    Protein = 2,
    /// Test Sequence - arbitrary string
    Text = 3,
}

impl SequenceType {
    /// Check whether the sequence type is a nucleotide type.
    #[inline]
    #[must_use]
    pub const fn is_nucleotide(&self) -> bool {
        match self {
            Self::Dna | Self::Rna => true,
            Self::Protein | Self::Text => false,
        }
    }
}

// --- Flag --------------------------------------------------------------------

/// A single flag inside header flags.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flag {
    /// A flag indicating sequence qualities are stored in the archive.
    Quality = 0x1,
    /// A flag indicating sequences are stored in the archive.
    Sequence = 0x2,
    /// A flag indicating sequence masks are stored in the archive.
    Mask = 0x4,
    /// A flag indicating sequence lengths are stored in the archive.
    Length = 0x8,
    /// A flag indicating sequence comments are stored in the archive.
    Comment = 0x10,
    /// A flag indicating sequence identifiers are stored in the archive.
    Id = 0x20,
    /// A flag indicating the archive has a title.
    Title = 0x40,
    /// A flag reserved for future extension of the format.
    Extended = 0x80,
}

impl Flag {
    /// Get all individual flags.
    #[must_use]
    pub const fn values() -> &'static [Self] {
        &[
            Flag::Quality,
            Flag::Sequence,
            Flag::Mask,
            Flag::Length,
            Flag::Comment,
            Flag::Id,
            Flag::Title,
            Flag::Extended,
        ]
    }

    /// View the flag as a single byte mask.
    #[must_use]
    pub const fn as_byte(&self) -> u8 {
        *self as u8
    }
}

impl BitOr<Flag> for Flag {
    type Output = Flags;
    fn bitor(self, rhs: Flag) -> Self::Output {
        Flags((self as u8).bitor(rhs as u8))
    }
}

// --- Flags -------------------------------------------------------------------

/// The flags for optional content blocks inside a NAF archive.
#[derive(Debug, Clone, Copy)]
pub struct Flags(u8);

impl Flags {
    /// Create new `Flags` with all flags unset.
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Check if the given flag is set.
    #[must_use]
    pub const fn test(&self, flag: Flag) -> bool {
        (self.0 & flag as u8) != 0
    }

    /// Set the given flag.
    pub fn set(&mut self, flag: Flag) {
        self.0 |= flag as u8;
    }

    /// Unset the given flag.
    pub fn unset(&mut self, flag: Flag) {
        self.0 &= !(flag as u8);
    }

    /// View the flags as a single byte.
    #[must_use]
    pub const fn as_byte(&self) -> u8 {
        self.0
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Flag> for Flags {
    fn from(value: Flag) -> Self {
        Self(value.as_byte())
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
        flags.0
    }
}

impl BitOr<Flag> for Flags {
    type Output = Flags;
    fn bitor(self, rhs: Flag) -> Self::Output {
        Flags(self.0.bitor(rhs as u8))
    }
}

impl BitOrAssign<Flag> for Flags {
    fn bitor_assign(&mut self, rhs: Flag) {
        self.0 = self.0 | rhs as u8;
    }
}

/// The header section of a Nucleotide Archive Format file.
///
/// Headers are the only mandatory section of NAF files, and contain
/// metadata about the stored sequences, as well as some metadata for
/// the formatting the records during decompression.
///
#[derive(Debug, Clone)]
pub struct Header {
    pub(crate) format_version: FormatVersion,
    pub(crate) sequence_type: SequenceType,
    pub(crate) flags: Flags,
    pub(crate) name_separator: char,
    pub(crate) line_length: u64,
    pub(crate) number_of_sequences: u64,
}

impl Header {
    /// Get the flags of the archive header.
    #[must_use]
    pub const fn flags(&self) -> Flags {
        self.flags
    }

    /// Get the default line length stored in the archive.
    #[must_use]
    pub const fn line_length(&self) -> u64 {
        self.line_length
    }

    /// Get the name separator used in the archive.
    #[must_use]
    pub const fn name_separator(&self) -> char {
        self.name_separator
    }

    /// Get the number of sequences stored in the archive.
    #[must_use]
    pub const fn number_of_sequences(&self) -> u64 {
        self.number_of_sequences
    }

    /// Get the type of sequences stored in the archive.
    #[must_use]
    pub const fn sequence_type(&self) -> SequenceType {
        self.sequence_type
    }

    /// Get the archive format version.
    #[must_use]
    pub const fn format_version(&self) -> FormatVersion {
        self.format_version
    }
}

impl Default for Header {
    fn default() -> Self {
        Header {
            format_version: FormatVersion::V1,
            sequence_type: SequenceType::Dna,
            flags: Flags::default(),
            name_separator: ' ',
            line_length: 60,
            number_of_sequences: 0,
        }
    }
}

//---- Size ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Size {
    block: String,
    pub original: u64,
    pub compressed: u64
}

impl Size {
    pub fn new(block: String, original: u64, compressed: Option<u64>) -> Self {
        let compressed_real = if let Some(real_size) = compressed {real_size} else { original };
        Size{block, original, compressed: compressed_real}
    }
}

#[allow(clippy::cast_precision_loss)]
impl fmt::Display for Size {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        if self.original == self.compressed {
            write!(f, "{}: {}",self.block,self.original)
        } else {
            write!(f, "{}: {} / {} ({:.3}%)",
                self.block,
                self.compressed,
                self.original,
                (self.compressed as f32 * 100.0)/(self.original as f32))
        }
    }
}
