use std::{
    convert::{TryFrom, TryInto},
    path::Path,
};

use igo::Morpheme as IgoMorpheme;
use igo::Tagger;

#[derive(Clone)]
pub struct Parser {
    parser: Tagger,
}

impl Parser {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let tagger = Tagger::new(Path::new(path))?;
        Ok(Parser { parser: tagger })
    }

    pub fn parse<'text, 'dict>(&'dict self, text: &'text str) -> Vec<Morpheme<'dict, 'text>> {
        self.parser
            .parse(text)
            .into_iter()
            .map(Morpheme::from)
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Morpheme<'dict, 'input> {
    pub surface: &'input str,
    pub basic: &'dict str,
    pub word_class: WordClass<'dict>,
    pub conjungation: Conjungation,
    pub origin: Option<Origin>,
    pub reading: &'dict str,
    pub lexeme: &'dict str,
    pub start: usize,
}

impl<'dict, 'input> From<IgoMorpheme<'dict, 'input>> for Morpheme<'dict, 'input> {
    fn from(igo_morph: IgoMorpheme<'dict, 'input>) -> Morpheme<'dict, 'input> {
        let features: &Vec<_> = &igo_morph.feature.split(',').collect();

        let word_class: WordClass = features.try_into().unwrap();

        let conjungation_form: ConjungationForm = features.try_into().unwrap();
        let conjungation_kind: ConjungationKind = features.try_into().unwrap();

        let conjungation = Conjungation {
            kind: conjungation_kind,
            form: conjungation_form,
        };

        let origin = if features.len() > 12 {
            Origin::from_string(features[12])
        } else {
            None
        };

        let basic = str_or_empty(features, 6);
        let lexeme = str_or_empty(features, 10);
        let reading = str_or_empty(features, 9);

        Morpheme {
            start: igo_morph.start,
            surface: igo_morph.surface,
            basic,
            lexeme,
            reading,
            word_class,
            origin,
            conjungation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Conjungation {
    pub kind: ConjungationKind,
    pub form: ConjungationForm,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ConjungationKind {
    None,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ConjungationForm {
    None,
    Plain,
    Imperative,
    Negative,
    Attributive,
    Continuous,
    Conditional,
    Stem,
    Realis,
    Kugohou,
}

impl<'a> TryFrom<&Vec<&'a str>> for ConjungationKind {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        Ok(match value[4] {
            _ => Self::None,
            //_ => return Err(format!("conjungation kind not found {}", value[4])),
        })
    }
}

impl<'a> TryFrom<&Vec<&'a str>> for ConjungationForm {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        let t = split_type(value[5]).0;
        Ok(match t {
            "*" => ConjungationForm::None,
            "?????????" => ConjungationForm::Plain,
            "?????????" => ConjungationForm::Imperative,
            "?????????" => ConjungationForm::Negative,
            "?????????" => ConjungationForm::Attributive,
            "?????????" => ConjungationForm::Continuous,
            "?????????" => ConjungationForm::Conditional,
            "??????" => ConjungationForm::Stem,
            "?????????" => ConjungationForm::Realis,
            "???????????????" => ConjungationForm::None, // wtf is this?
            "?????????" => ConjungationForm::Kugohou,
            _ => return Err(format!("conjungation form not found {}", t)),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum WordClass<'a> {
    Particle(ParticleType),
    Verb(VerbType<'a>),
    Adjective(AdjectiveType),
    Adverb,
    Noun(NounType),
    Pronoun,
    Interjection,
    Symbol,
    Conjungtion,
    Suffix,
    Prefix,
    PreNoun,
    Space,
}

impl<'a> Default for WordClass<'a> {
    fn default() -> Self {
        Self::Adverb
    }
}

impl<'a> WordClass<'a> {
    /// Returns `true` if the word_class is [`Particle`].
    pub fn is_particle(&self) -> bool {
        matches!(self, Self::Particle(..))
    }

    /// Returns `true` if the word_class is [`Verb`].
    pub fn is_verb(&self) -> bool {
        matches!(self, Self::Verb(..))
    }

    /// Returns `true` if the word_class is [`Adjective`].
    pub fn is_adjective(&self) -> bool {
        matches!(self, Self::Adjective(..))
    }

    /// Returns `true` if the word_class is [`Noun`].
    pub fn is_noun(&self) -> bool {
        matches!(self, Self::Noun(..))
    }

    /// Returns `true` if the word_class is [`Pronoun`].
    pub fn is_pronoun(&self) -> bool {
        matches!(self, Self::Pronoun)
    }

    /// Returns `true` if the word_class is [`Interjection`].
    pub fn is_interjection(&self) -> bool {
        matches!(self, Self::Interjection)
    }

    /// Returns `true` if the word_class is [`Symbol`].
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol)
    }

    /// Returns `true` if the word_class is [`Conjungtion`].
    pub fn is_conjungtion(&self) -> bool {
        matches!(self, Self::Conjungtion)
    }

    /// Returns `true` if the word_class is [`Suffix`].
    pub fn is_suffix(&self) -> bool {
        matches!(self, Self::Suffix)
    }

    /// Returns `true` if the word_class is [`Prefix`].
    pub fn is_prefix(&self) -> bool {
        matches!(self, Self::Prefix)
    }

    /// Returns `true` if the word_class is [`PreNoun`].
    pub fn is_pre_noun(&self) -> bool {
        matches!(self, Self::PreNoun)
    }

    /// Returns `true` if the word_class is [`Space`].
    pub fn is_space(&self) -> bool {
        matches!(self, Self::Space)
    }

    /// Returns `true` if the word_class is [`Adverb`].
    pub fn is_adverb(&self) -> bool {
        matches!(self, Self::Adverb)
    }
}

impl<'a> TryFrom<&Vec<&'a str>> for WordClass<'a> {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        Ok(match value[0] {
            "??????" => WordClass::Particle(ParticleType::try_from(value)?),
            "?????????" | "?????????" => WordClass::Adjective(AdjectiveType::try_from(value)?),
            "?????????" | "??????" => WordClass::Verb(VerbType::try_from(value)?),
            "?????????" => WordClass::Pronoun,
            "?????????" => WordClass::Interjection,
            "????????????" | "??????" => WordClass::Symbol,
            "?????????" => WordClass::Conjungtion,
            "?????????" => WordClass::Suffix,
            "?????????" => WordClass::Prefix,
            "??????" => WordClass::Adverb,
            "??????" => WordClass::Space,
            "??????" => WordClass::Noun(NounType::try_from(value)?),
            "?????????" => WordClass::PreNoun,
            _ => return Err(format!("wc not found {}", value[0])),
        })
    }
}

//
// ------ Noun
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum NounType {
    Common,
    Proper,
    Numeral,
    Suffix,
    Jodoushi,
}

impl TryFrom<&Vec<&str>> for NounType {
    type Error = String;
    fn try_from(value: &Vec<&str>) -> Result<Self, Self::Error> {
        Ok(match value[1] {
            "????????????" => Self::Common,
            "????????????" => Self::Proper,
            "??????" | "???" => Self::Numeral,
            "??????" => Self::Suffix,
            "???????????????" => Self::Jodoushi,
            _ => return Err(format!("Nountype not found {}", value[1])),
        })
    }
}

//
// ------ Particle
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ParticleType {
    Connecting,
    SentenceEnding,
    CaseMaking,
    Conjungtion,
    Adverbial,
    Nominalizing,
}

impl TryFrom<&Vec<&str>> for ParticleType {
    type Error = String;
    fn try_from(value: &Vec<&str>) -> Result<Self, Self::Error> {
        Ok(match value[1] {
            "?????????" => Self::Connecting,
            "?????????" => Self::SentenceEnding,
            "?????????" => Self::CaseMaking,
            "????????????" => Self::Conjungtion,
            "?????????" => Self::Adverbial,
            "????????????" => Self::Nominalizing,
            _ => return Err(format!("particle not found {}", value[1])),
        })
    }
}

//
// ------ Adjective
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum AdjectiveType {
    I,
    Na,
}

impl TryFrom<&Vec<&str>> for AdjectiveType {
    type Error = String;
    fn try_from(value: &Vec<&str>) -> Result<Self, Self::Error> {
        Ok(match value[0] {
            "?????????" => Self::I,
            "?????????" => Self::Na,
            _ => return Err(format!("adjective not found {}", value[0])),
        })
    }
}

//
// ------ Verb
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum VerbType<'a> {
    Auxilary(&'a str),
    Godan(SyllableRow),
    Ichidan(SyllableRow),
    IchidanEruConjungation,
    IrrWrittenLang,
    Suru,
    Kuru,
    IrregRu,
    IrregNu,
}

impl<'a> TryFrom<&Vec<&'a str>> for VerbType<'a> {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        let verb_type = value[4];

        Ok(match value[0] {
            "?????????" => Self::Auxilary(split_type(verb_type).1),
            "??????" => Self::parse_general(verb_type)?,
            _ => return Err(format!("Verb not found {}", value[0])),
        })
    }
}

impl<'a> VerbType<'a> {
    fn parse_general(verb_type: &'a str) -> Result<Self, String> {
        let verb_type = split_type(verb_type);
        Ok(match verb_type.0 {
            "??????" | "???????????????" | "????????????" | "???????????????" | "?????????" => {
                VerbType::Godan(SyllableRow::try_from(verb_type.1)?)
            }
            "??????" | "?????????" | "?????????" => {
                VerbType::Ichidan(SyllableRow::try_from(verb_type.1)?)
            }
            "????????????" | "??????????????????" => VerbType::Suru,
            "????????????" => VerbType::Kuru,
            "????????????" | "??????????????????" => VerbType::IrregRu,
            "????????????" | "??????????????????" => VerbType::IrregNu,
            "??????????????????" => VerbType::IrrWrittenLang,
            "???????????????" => VerbType::IrrWrittenLang,
            _ => return Err(format!("Verbtype {} not found", verb_type.0)),
        })
    }
}

//
// ------ Other
//

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum SyllableRow {
    G,
    K,
    M,
    A,
    R,
    S,
    Z,
    T,
    D,
    B,
    H,
    P,
    N,
    Wa,
    Y,
}

impl<'a> TryFrom<&'a str> for SyllableRow {
    type Error = String;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(match value {
            "??????" => SyllableRow::G,
            "??????" => SyllableRow::K,
            "??????" => SyllableRow::M,
            "??????" => SyllableRow::A,
            "??????" => SyllableRow::R,
            "??????" => SyllableRow::S,
            "??????" => SyllableRow::Z,
            "??????" => SyllableRow::T,
            "??????" => SyllableRow::D,
            "??????" => SyllableRow::H,
            "??????" => SyllableRow::B,
            "??????" => SyllableRow::P,
            "??????" => SyllableRow::N,
            "?????????" => SyllableRow::Wa,
            "??????" => SyllableRow::Y,
            _ => return Err(format!("Syllable ending not found {}", value)),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Origin {
    China,
    Japan,
}

impl Origin {
    fn from_string(s: &str) -> Option<Self> {
        match s {
            "???" => Some(Self::China),
            "???" => Some(Self::Japan),
            _ => None,
        }
    }
}

//
// Helper
//

/// Splits a type definition and gets both sides
fn split_type<'a>(inp: &'a str) -> (&'a str, &'a str) {
    let mut s = if inp.contains("-") {
        inp.split("-")
    } else {
        inp.split("???")
    };
    (s.next().unwrap_or(""), s.next().unwrap_or(""))
}

fn str_or_empty<'a>(vec: &Vec<&'a str>, pos: usize) -> &'a str {
    if vec.len() > pos {
        vec[pos]
    } else {
        ""
    }
}
